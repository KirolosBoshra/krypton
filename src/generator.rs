use crate::{parser::Tree, tokenize::Token};

pub struct Generator {
    tree: Vec<Tree>,
}

impl Generator {
    pub fn new(tree: Vec<Tree>) -> Self {
        Self { tree }
    }
    pub fn generate_linux_64(&self) -> String {
        let mut iter = self.tree.iter().peekable();
        let mut asm = String::new();
        let mut section_text = String::new();
        let mut start = String::new();

        section_text += "section .text\n\tglobal _start\n";
        start += "_start:\n";

        while let Some(op) = iter.next() {
            match op {
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
        asm += &start;
        asm
    }
    fn gen_expr(&self, tree: &Box<Tree>) -> String {
        match **tree {
            Tree::Number(num) => {
                let mut buffer = String::new();
                buffer += "\tmov rax, ";
                buffer += num.to_string().as_str();
                buffer += "\n";
                buffer += self.push("rax").as_str();
                buffer
            }
            Tree::BinOp(..) => self.gen_bin_exp(tree),
            _ => panic!("WTF"),
        }
    }
    fn gen_bin_exp(&self, tree: &Tree) -> String {
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
    fn push(&self, buf: &str) -> String {
        let mut buffer = String::from("\tpush ");
        buffer += buf;
        buffer += "\n";
        buffer
    }
    fn pop(&self, buf: &str) -> String {
        let mut buffer = String::from("\tpop ");
        buffer += buf;
        buffer += "\n";
        buffer
    }
}
