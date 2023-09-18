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
                    let vars_clone = self.vars.clone();
                    let iter = vars_clone.iter().find(|var| var.name == ident.to_string());
                    match iter {
                        Some(var) => {
                            let mut expr_loc = String::new();
                            start += self.gen_expr(expr).as_str();
                            expr_loc += "QWORD [rsp + ";
                            expr_loc += ((self.stack - self.stack - 1) * 8)
                                .abs()
                                .to_string()
                                .as_str();
                            expr_loc += "]";
                            start += self.push(expr_loc.as_str()).as_str();
                            start += "\tmov QWORD [rsp + ";
                            start += ((self.stack - var.stack_loc - 1) * 8).to_string().as_str();
                            start += "], rax\n";
                            start += self.pop("rax").as_str();
                        }
                        _ => {
                            let var = Var::new(ident.to_string(), self.stack);
                            self.vars.push(var);
                            start += self.gen_expr(expr).as_str();
                        }
                    }
                    println!("{:?}", self.vars);
                }

                Tree::Assign(ident, expr) => {
                    let vars_clone = self.vars.clone();
                    let iter = vars_clone.iter().find(|var| var.name == ident.to_string());
                    match iter {
                        Some(var) => {
                            let mut expr_loc = String::new();
                            start += self.gen_expr(expr).as_str();
                            expr_loc += "QWORD [rsp + ";
                            expr_loc += ((self.stack - self.stack - 1) * 8).to_string().as_str();
                            expr_loc += "]";
                            start += self.push(expr_loc.as_str()).as_str();
                            start += "\tmov QWORD [rsp + ";
                            start += ((self.stack - var.stack_loc - 1) * 8).to_string().as_str();
                            start += "], rax\n";
                            start += self.pop("rax").as_str();
                        }
                        _ => panic!("Var not declared"),
                    }
                    println!("{:?}", self.vars);
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
        asm += &section_text;
        asm += &start.as_str();
        asm
    }
    fn gen_expr(&mut self, tree: &Tree) -> String {
        match tree {
            Tree::Number(num) => {
                let mut buffer = String::new();
                buffer += "\tmov rax, ";
                buffer += num.to_string().as_str();
                buffer += "\n";
                buffer += self.push("rax").as_str();
                buffer
            }
            Tree::Ident(var) => {
                let mut var_loc = String::new();
                let mut buffer = String::new();
                let stack_loc;
                let vars = self.vars.iter().find(|vars| vars.name == var.to_string());
                match vars {
                    Some(vars) => {
                        stack_loc = vars.stack_loc;
                    }
                    _ => {
                        panic!("Var not declared");
                    }
                }
                var_loc += "QWORD [rsp + ";
                var_loc += ((self.stack - stack_loc - 1) * 8).to_string().as_str();
                var_loc += "]";
                buffer += self.push(&var_loc).as_str();
                buffer
            }
            Tree::BinOp(..) => self.gen_bin_exp(tree),
            _ => panic!("WTF"),
        }
    }
    fn gen_bin_exp(&mut self, tree: &Tree) -> String {
        match tree {
            Tree::BinOp(left, op, right) => match op {
                Token::Plus => {
                    let mut buffer = String::new();
                    buffer += self.gen_expr(right).as_str();
                    buffer += self.gen_expr(left).as_str();
                    buffer += self.pop("rax").as_str();
                    buffer += self.pop("rbx").as_str();
                    buffer += "\tadd rax, rbx\n";
                    buffer += self.push("rax").as_str();
                    buffer
                }

                Token::Minus => {
                    let mut buffer = String::new();
                    buffer += self.gen_expr(right).as_str();
                    buffer += self.gen_expr(left).as_str();
                    buffer += self.pop("rax").as_str();
                    buffer += self.pop("rbx").as_str();
                    buffer += "\tsub rax, rbx\n";
                    buffer += self.push("rax").as_str();
                    buffer
                }

                Token::Multiply => {
                    let mut buffer = String::new();
                    buffer += self.gen_expr(right).as_str();
                    buffer += self.gen_expr(left).as_str();
                    buffer += self.pop("rax").as_str();
                    buffer += self.pop("rbx").as_str();
                    buffer += "\tmul rbx\n";
                    buffer += self.push("rax").as_str();
                    buffer
                }

                Token::Divide => {
                    let mut buffer = String::new();
                    buffer += self.gen_expr(right).as_str();
                    buffer += self.gen_expr(left).as_str();
                    buffer += self.pop("rax").as_str();
                    buffer += self.pop("rbx").as_str();
                    buffer += "\tdiv rbx\n";
                    buffer += self.push("rax").as_str();
                    buffer
                }
                _ => panic!("invalid Token"),
            },
            _ => panic!("invalid"),
        }
    }
    fn push(&mut self, buf: &str) -> String {
        self.stack += 1;
        let mut buffer = String::from("\tpush ");
        buffer += buf;
        buffer += "\n";
        buffer
    }
    fn pop(&mut self, buf: &str) -> String {
        self.stack -= 1;
        let mut buffer = String::from("\tpop ");
        buffer += buf;
        buffer += "\n";
        buffer
    }
}
