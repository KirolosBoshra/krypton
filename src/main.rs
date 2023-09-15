mod generator;
mod parser;
mod tokenize;
// use std::io::Write;
use std::{fs::File, io::Read, io::Write};

use generator::Generator;
use parser::Parser;
use tokenize::Tokenizer;

fn main() {
    let mut input = String::new();
    let mut file = File::open("main.kr").expect("Can't open file");
    file.read_to_string(&mut input).expect("can't read file");

    let tokenizer = Tokenizer::new(&input);
    let tokens = tokenizer.tokenize();

    println!("{}", input);
    println!("Tokens: {:?}", tokens);

    let mut ast = Parser::new(&tokens);
    println!("Parse Trees: {:?}", ast.parse_tokens());

    let generator = Generator::new(ast.parse_tokens());
    let file_name = "main.asm";
    let asm_cont = generator.generate_linux_64();

    let mut file = File::create(file_name).expect("Failed to create file");
    file.write_all(asm_cont.as_bytes())
        .expect("Failed to write to file");
}
