use crate::{parser::Tree, tokenize::Token};

#[derive(Debug, Clone)]
struct Var {
    name: String,
    stack_loc: i32,
}
impl Var {
    pub fn new(name: String, stack_loc: i32) -> Self {
        Self { name, stack_loc }
    }
}

pub struct Generator {
    tree: Vec<Tree>,
    vars: Vec<Var>,
    stack: i32,
}

impl Generator {
    
    pub fn new(tree: &Vec<Tree>) -> Self {
        let vars = vec![];
        let stack = 0;
        Self {
            tree: tree.to_vec(),
            vars,
            stack,
        }
    }

    pub fn generate_linux_64(&mut self) -> String {
        let tree_clone = self.tree.clone();
        let mut iter = tree_clone.iter().peekable();
        let mut asm = String::new();
        let mut section_text = String::new();
        let mut start = String::new();
        section_text += "section .text\n\tglobal _start\n";
        start += "_start:\n";

        while let Some(op) = iter.next() {
            match op {
                Tree::Let(ident, expr) => {
                    if self.vars.iter().any(|var| var.name == ident.to_string()) {
                        panic!("{} is already declared use {} = (expr) instead", ident, ident);
                    }
                    self.vars.push(Var::new(ident.to_string(), self.stack));
                    self.handle_vars(ident, expr, &mut start);
                }

                Tree::Assign(ident, expr) => {
                    self.handle_vars(ident, expr, &mut start);
                }

                Tree::Exit(expr) => {
                    start += self.gen_expr(expr).as_str();
                    start += "\tmov rax, 60\n";
                    start += self.pop("rdi").as_str();
                    start += "\tsyscall\n";
                }
                _ => (),
            }
        }
        println!("{:?}", self.vars);
        asm += &section_text;
        asm += &start.as_str();
        asm
    }

    fn gen_expr(&mut self, tree: &Tree) -> String {
        match tree {
            Tree::Number(num) => {
                let mut buffer = format!("\tmov rax, {}\n", num);
                buffer += &self.push("rax");
                buffer
            },
            Tree::Ident(var) => {
                let var_loc = format!("QWORD [rsp + {}]", (self.stack - self.find_var(var).stack_loc - 1) * 8);
                let buffer = self.push(&var_loc);
                buffer
            },
            Tree::BinOp(..) => self.gen_bin_exp(tree),
            _ => panic!("Unexpected expr"),
        }
    }

    fn gen_bin_op(&mut self, left: &Tree, right: &Tree, op: &str) -> String {
        let mut buffer = String::new();
        buffer += self.gen_expr(right).as_str();
        buffer += self.gen_expr(left).as_str();
        buffer += self.pop("rax").as_str();
        buffer += self.pop("rbx").as_str();
        match op {
            "div" => buffer += &format!("\t{} rax\n", op),
            _ => buffer += &format!("\t{} rax, rbx\n", op)
        }
        buffer += self.push("rax").as_str();
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

    fn handle_vars(&mut self, ident: &String, expr: &Tree, start: &mut String) {
        match *expr {
            Tree::Number(num) => {
                self.stack += 1;
                let stack_loc = ((self.find_var(ident).stack_loc) * 8).to_string();
                start.push_str(&format!("\tmov QWORD [rsp + {}], {}\n", stack_loc, num));
            }
            _ => {
                let stack_loc = ((self.stack - (self.find_var(ident).stack_loc - 1)) * 8).to_string();
                start.push_str(&format!("{} \tmov QWORD [rsp + {}], rax\n", self.gen_expr(expr), stack_loc));
            }
        }
        
    }

    fn find_var(&self, ident: &String) -> &Var {
        self.vars.iter().find(|vars| vars.name == ident.to_string()).expect(&format!("{} not declared", ident))
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
