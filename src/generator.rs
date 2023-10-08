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

pub struct Generator {
    tree: Vec<Tree>,
    assembly_out: String,
    start_section: String,
    text_section: String,
    vars: Vec<Var>,
    lb_count: usize,
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
            lb_count: 0,
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
                if self.vars.iter().any(|var| var.name == ident.to_string()) {
                    panic!(
                        "{} is already declared use {} = ({:?}) instead",
                        ident, ident, expr
                    );
                }
                self.vars.push(Var::new(ident.to_string(), self.stack));
                self.stack += 1;
                program += &self.handle_vars(&ident, &expr);
            }

            Tree::Assign(ident, expr) => {
                program += &self.handle_vars(&ident, &expr);
            }

            Tree::If {
                expr,
                body,
                els,
                els_ifs,
            } => {
                program += &self.gen_if_stmt(expr, body);
                els_ifs.iter().for_each(|block| match block {
                    Tree::ElsIf { expr, body } => {
                        self.lb_count += 1;
                        program += &self.gen_if_stmt(expr, body);
                    }
                    _ => (),
                });
                if !els.is_empty() {
                    self.lb_count += 1;
                    program += &format!("\tjmp .LB{}\n", self.lb_count);
                    program += &format!(".LB{}:\n", self.lb_count);
                    els.iter().for_each(|stmt| {
                        program += &self.gen_linux_64_program(stmt);
                    });
                }
                self.lb_count += 1;
            }

            Tree::While(expr, body) => {
                self.lb_count += 1;
                program += &format!("\tjmp .LB{}\n", self.lb_count);
                program += &format!(".LB{}:\n", self.lb_count);

                self.begin_scope();
                body.iter().for_each(|stmt| {
                    program += &self.gen_linux_64_program(stmt);
                });
                self.end_scope();

                self.lb_count += 1;
                program += &format!(".LB{}:\n", self.lb_count);
                program += &self.gen_cmp_exp(expr);
                let ex = expr.clone();
                match *ex {
                    Tree::CmpOp(_, c, _) => match c {
                        Token::EquEqu => {
                            program += &format!("\tje .LB{}\n", self.lb_count - 1);
                        }
                        Token::NotEqu => {
                            program += &format!("\tjne .LB{}\n", self.lb_count - 1);
                        }
                        Token::Greater => {
                            program += &format!("\tjg .LB{}\n", self.lb_count - 1);
                        }
                        Token::GreatEqu => {
                            program += &format!("\tjge .LB{}\n", self.lb_count - 1);
                        }
                        Token::Less => {
                            program += &format!("\tjl .LB{}\n", self.lb_count - 1);
                        }
                        Token::LessEqu => {
                            program += &format!("\tjle .LB{}\n", self.lb_count - 1);
                        }
                        _ => (),
                    },
                    _ => (),
                }
                self.lb_count += 1;
                program += &format!(".LB{}:\n", self.lb_count);
            }

            Tree::Exit(expr) => {
                program += &self.gen_expr(&expr, "rax");
                program += &self.push("rax");
                program += "\tmov rax, 60\n";
                program += &self.pop("rdi");
                program += "\tsyscall\n";
            }
            _ => (),
        }
        program
    }

    fn gen_expr(&mut self, tree: &Tree, reg: &str) -> String {
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
            Tree::BinOp(..) => self.gen_bin_exp(tree),
            Tree::CmpOp(..) => self.gen_cmp_exp(tree),
            _ => panic!("Unexpected expr"),
        }
    }

    fn gen_bin_op(&mut self, left: &Tree, right: &Tree, op: &str) -> String {
        let mut buffer = String::new();
        buffer += &self.gen_expr(left, "rax");
        buffer += &self.gen_expr(right, "rbx");
        match op {
            "div" => {
                buffer += "\txor rdx, rdx\n";
                buffer += &format!("\t{} rbx\n", op);
            }
            _ => buffer += &format!("\t{} rax, rbx\n", op),
        }
        buffer
    }

    fn gen_bin_exp(&mut self, tree: &Tree) -> String {
        match tree {
            Tree::BinOp(left, op, right) => match op {
                Token::Plus => self.gen_bin_op(left, right, "add"),
                Token::Minus => self.gen_bin_op(left, right, "sub"),
                Token::Multiply => self.gen_bin_op(left, right, "imul"),
                Token::Divide => self.gen_bin_op(left, right, "div"),
                _ => panic!("invalid Token"),
            },
            _ => panic!("Expected BinOp Tree"),
        }
    }

    fn gen_cmp_op(&mut self, left: &Tree, right: &Tree) -> String {
        let mut buffer = String::new();
        buffer += &self.gen_expr(left, "rax");
        buffer += &self.gen_expr(right, "rbx");
        buffer += "\tcmp rax, rbx\n";
        buffer
    }

    fn gen_cmp_exp(&mut self, tree: &Tree) -> String {
        match tree {
            Tree::CmpOp(left, _, right) => self.gen_cmp_op(&left, &right),
            _ => panic!("Expected CMP OP"),
        }
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

    fn gen_if_stmt(&mut self, expr: &Tree, body: &Vec<Tree>) -> String {
        let mut buffer = String::new();
        let mut body_cont = String::new();
        self.begin_scope();
        body.iter().for_each(|stmt| {
            body_cont += &self.gen_linux_64_program(stmt);
        });
        self.end_scope();

        buffer += &self.gen_cmp_exp(expr);
        let ex = expr.clone();
        match ex {
            Tree::CmpOp(_, c, _) => match c {
                Token::EquEqu => {
                    buffer += &format!("\tjne .LB{}\n", self.lb_count);
                }
                Token::NotEqu => {
                    buffer += &format!("\tje .LB{}\n", self.lb_count);
                }
                Token::Greater | Token::GreatEqu => {
                    buffer += &format!("\tjle .LB{}\n", self.lb_count);
                }
                Token::Less | Token::LessEqu => {
                    buffer += &format!("\tjg .LB{}\n", self.lb_count);
                }
                _ => (),
            },
            _ => (),
        }
        buffer += &body_cont;
        buffer += &format!(".LB{}:\n", self.lb_count);
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
