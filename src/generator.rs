use crate::{parser::Tree, tokenize::Token};

#[derive(Debug, Clone)]
struct Var {
    name: String,
    stack_loc: usize,
}
impl Var {
    pub fn new(name: String, stack_loc: usize) -> Self {
        Self { name, stack_loc }
    }
}

static SYSCALL_REGS: [&str; 6] = ["rdi", "rsi", "rdx", "r10", "r8", "r9"];

pub struct Generator {
    tree: Vec<Tree>,
    assembly_out: String,
    start_section: String,
    text_section: String,
    vars: Vec<Var>,
    stack: usize,
    scopes: Vec<usize>,
}

impl Generator {
    pub fn new(tree: &Vec<Tree>) -> Self {
        Self {
            tree: tree.to_vec(),
            vars: vec![],
            stack: 0,
            scopes: vec![],
            assembly_out: String::new(),
            start_section: String::new(),
            text_section: String::new(),
        }
    }

    pub fn generate_linux_64(&mut self) -> &String {
        let tree_clone = self.tree.clone();
        let mut iter = tree_clone.iter().peekable();
        self.text_section += "section .text\n\tglobal _start\n";
        self.start_section += "_start:\n";
        let mut program = String::new();

        while let Some(tree) = iter.peek() {
            program += &self.gen_linux_64_program(tree);
            iter.next();
        }

        self.start_section += &program;

        println!("{:?}", self.vars);
        println!("stack: {}", self.stack);
        self.assembly_out += &self.text_section;
        self.assembly_out += &self.start_section;
        &self.assembly_out
    }
    fn gen_linux_64_program(&mut self, tree: &Tree) -> String {
        let mut program = String::new();
        match tree {
            Tree::Let(ident, expr) => {
                program += &format!("\t;; Let {} = {:?} ;;\n", ident, expr);
                if self.vars.iter().any(|var| var.name == ident.to_string()) {
                    panic!(
                        "{} is already declared use {} = {:?} instead",
                        ident, ident, expr
                    );
                }
                self.vars.push(Var::new(ident.to_string(), self.stack));
                self.stack += 1;
                program += &self.handle_vars(&ident, &expr);
            }

            Tree::Assign(ident, expr) => {
                program += &format!("\t;; {} = {:?} ;;\n", ident, expr);
                program += &self.handle_vars(&ident, &expr);
            }

            Tree::Inc(var) => {
                program += &format!("\t;; {}++ ;;\n", var);
                program += &format!("\tinc QWORD [rsp + {}]\n", self.find_var(var).stack_loc * 8)
            }

            Tree::Dec(var) => {
                program += &format!("\t;; {}-- ;;\n", var);
                program += &format!("\tdec QWORD [rsp + {}]\n", self.find_var(var).stack_loc * 8)
            }

            Tree::If {
                expr,
                body,
                last_case,
                next_case,
                els,
                els_ifs,
            } => {
                program += &format!("\t;; If({:?}) ;;\n", expr);
                program += &self.gen_if_cmp(expr, next_case);
                program += &self.create_scope(body);
                if next_case == last_case {
                    program += &format!(".LB{}:\n", next_case);
                } else {
                    program += &format!("\tjmp .LB{}\n", last_case);
                    els_ifs.iter().for_each(|stmt| {
                        program += &self.gen_elsif_stmt(stmt, last_case);
                    });
                    if !els.is_empty() {
                        program += "\t;; Els ;;\n";
                        program += &format!(".LB{}:\n", last_case - 1);
                        program += &self.create_scope(els);
                        program += &format!(".LB{}:\n", last_case);
                        program += "\t;; End Els ;;\n";
                    }
                }
                program += "\t;; End If ;;\n";
            }

            Tree::While {
                expr,
                start,
                body,
                end,
            } => {
                program += &format!("\t;; While({:?}) ;;\n", expr);
                program += &format!("\tjmp .LB{}\n", end);
                program += &format!(".LB{}:\n", start);
                program += &self.create_scope(body);
                program += &format!(".LB{}:\n", end);
                program += &self.gen_cmp_exp(expr);
                program += "\ttest al, 1\n";
                program += &format!("\tjnz .LB{}\n", start);
                program += "\t;; End While ;;\n";
            }

            // Tree::For { var, expr, body } => {}
            Tree::SysCall(args) => {
                program += &format!("\t;; Syscall({:?}) ;;\n", args);
                for i in 1..args.len() {
                    program += &self.gen_expr(&args[i], SYSCALL_REGS[i - 1]);
                }
                program += &self.gen_expr(&args[0], "rax");
                program += "\tsyscall\n";
            }

            Tree::Exit(expr) => {
                program += &format!("\t;; Exit({:?}) ;;\n", expr);
                program += &self.gen_expr(&expr, "rax");
                program += &self.push("rax");
                program += "\tmov rax, 60\n";
                program += &self.pop("rdi");
                program += "\tsyscall\n";
                program += "\t;; End Exit ;;\n";
            }
            _ => (),
        }
        program
    }

    fn gen_elsif_stmt(&mut self, stmt: &Tree, last_case: &usize) -> String {
        let mut buffer = String::new();
        match stmt {
            Tree::ElsIf {
                curr_case,
                expr,
                body,
                next_case,
            } => {
                buffer += &format!("\t;; ElsIf({:?}) ;;\n", expr);
                buffer += &format!(".LB{}:\n", curr_case);
                buffer += &self.gen_if_cmp(expr, next_case);
                self.begin_scope();
                body.iter().for_each(|stmt| {
                    buffer += &self.gen_linux_64_program(stmt);
                });
                self.end_scope();
                if next_case != last_case {
                    buffer += &format!("\tjmp .LB{}\n", last_case);
                } else {
                    buffer += &format!(".LB{}:\n", last_case);
                }
                buffer += "\t;; End ElsIf ;;\n";
            }
            _ => (),
        }
        buffer
    }

    fn gen_expr(&mut self, tree: &Tree, reg: &str) -> String {
        //[TODO] STATIC TYPES & [?]Pointers
        match tree {
            Tree::Number(num) => {
                format!("\tmov {}, {}\n", reg, num)
            }
            Tree::Ident(var) => {
                format!(
                    "\tmov {}, QWORD [rsp + {}]\n",
                    reg,
                    (self.find_var(var).stack_loc * 8)
                )
            }
            Tree::BinOp(..) => self.gen_bin_exp(tree, reg),
            Tree::CmpOp(..) => self.gen_cmp_exp(tree),
            Tree::Empty() => String::new(),
            _ => panic!("Unexpected expr"),
        }
    }

    fn gen_bin_op(&mut self, left: &Tree, right: &Tree, op: &str, reg: &str) -> String {
        let mut buffer = String::new();
        buffer += &format!("\t;; BinOp({:?} {op} {:?}) ;;\n", left, right);
        let mut lreg = "rax";
        let mut rreg = "rbx";
        buffer += &self.gen_expr(left, "rax");
        match left {
            Tree::BinOp(..) | Tree::CmpOp(..) => {
                buffer += "\tmov rcx, rax\n";
                lreg = "rcx";
                match right {
                    Tree::BinOp(..) | Tree::CmpOp(..) => {
                        rreg = "rax";
                    }
                    _ => (),
                }
            }
            _ => match right {
                Tree::BinOp(..) | Tree::CmpOp(..) => {
                    buffer += "\tmov rcx, rax\n";
                    lreg = "rcx";
                }
                _ => (),
            },
        }
        buffer += &self.gen_expr(right, "rbx");
        match op {
            "div" => {
                if lreg != "rax" {
                    lreg = "rax";
                    buffer += &format!("\tmov rax, {lreg}\n");
                }
                if rreg != "rbx" {
                    rreg = "rbx";
                    buffer += &format!("\tmov rbx, {rreg}\n");
                }
                buffer += "\txor rdx, rdx\n";
                buffer += &format!("\t{} {rreg}\n", op);
            }
            _ => buffer += &format!("\t{} {lreg}, {rreg}\n", op),
        }
        if lreg != reg {
            buffer += &format!("\tmov {reg}, {lreg}\n");
        }
        buffer += &format!("\t;; End BinOp({:?} {op} {:?}) ;;\n", left, right);
        buffer
    }

    fn gen_bin_exp(&mut self, tree: &Tree, lreg: &str) -> String {
        match tree {
            Tree::BinOp(left, op, right) => match op {
                Token::Plus => self.gen_bin_op(left, right, "add", lreg),
                Token::Minus => self.gen_bin_op(left, right, "sub", lreg),
                Token::Multiply => self.gen_bin_op(left, right, "imul", lreg),
                Token::Divide => self.gen_bin_op(left, right, "div", lreg),
                _ => panic!("invalid Token"),
            },
            _ => panic!("Expected BinOp Tree"),
        }
    }

    fn gen_cmp_op(&mut self, left: &Tree, op: &str, right: &Tree) -> String {
        let mut buffer = String::new();
        buffer += &self.gen_expr(left, "rax");
        match right {
            Tree::BinOp(..) | Tree::CmpOp(..) => {
                buffer += "\tmov rcx, rax\n";
                buffer += &self.gen_expr(right, "rbx");
                buffer += "\tmov rax, rcx\n";
            }
            _ => buffer += &self.gen_expr(right, "rbx"),
        }
        buffer += "\tcmp rax, rbx\n";
        buffer += &format!("\t{} al\n", op);
        buffer += &format!("\tmovzx rax, al\n");
        buffer
    }

    fn gen_cmp_exp(&mut self, tree: &Tree) -> String {
        //[TODO] Adding the rest of CmpOp
        match tree {
            Tree::CmpOp(left, op, right) => {
                let op_str = match op {
                    Token::EquEqu => "sete",
                    Token::NotEqu => "setne",
                    Token::Greater => "setg",
                    Token::GreatEqu => "setge",
                    Token::Less => "setl",
                    Token::LessEqu => "setle",
                    _ => "",
                };
                self.gen_cmp_op(&left, op_str, &right)
            }
            _ => panic!("Expected CMP OP"),
        }
    }

    fn gen_if_cmp(&mut self, expr: &Tree, next_case: &usize) -> String {
        let mut buffer = String::new();
        buffer += &self.gen_cmp_exp(&expr);
        buffer += "\ttest al, 1\n";
        buffer += &format!("\tjz .LB{}\n", next_case);
        buffer
    }

    fn handle_vars(&mut self, ident: &String, expr: &Tree) -> String {
        let mut buffer = String::new();
        match *expr {
            Tree::Number(num) => {
                let stack_loc = (self.find_var(ident).stack_loc * 8).to_string();
                buffer += &format!("\tmov QWORD [rsp + {}], {}\n", stack_loc, num);
            }
            _ => {
                buffer += &self.gen_expr(expr, "rax");
                let stack_loc = (self.find_var(ident).stack_loc * 8).to_string();
                buffer.push_str(&format!("\tmov QWORD [rsp + {}], rax\n", stack_loc));
            }
        }
        buffer
    }

    fn begin_scope(&mut self) {
        self.scopes.push(self.vars.len())
    }
    fn end_scope(&mut self) {
        let pop_count = self.vars.len() - self.scopes.last().unwrap();
        // not used for now
        // let mut buffer = String::new();
        // buffer += &format!("\tadd rsp, {}\n", pop_count * 8);
        self.stack -= pop_count;
        for _ in 0..pop_count {
            self.vars.pop();
        }
        self.scopes.pop();
        // buffer
    }
    fn create_scope(&mut self, body: &Vec<Tree>) -> String {
        let mut buffer = String::new();
        self.begin_scope();
        body.iter().for_each(|stmt| {
            buffer += &self.gen_linux_64_program(stmt);
        });
        self.end_scope();
        buffer
    }
    fn find_var(&self, ident: &String) -> &Var {
        self.vars
            .iter()
            .find(|vars| vars.name == ident.to_string())
            .expect(&format!("{} not declared", ident))
    }

    fn push(&mut self, buf: &str) -> String {
        self.stack += 1;
        format!("\tpush {}\n", buf)
    }

    fn pop(&mut self, buf: &str) -> String {
        self.stack -= 1;
        format!("\tpop {}\n", buf)
    }
}
