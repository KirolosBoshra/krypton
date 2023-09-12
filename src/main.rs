mod tokenize;
mod parser;
use std::fs::File;
use std::io::Write;

use tokenize::{Token, Tokenizer};

use crate::parser::Parser;


fn generate_asm_win(tokens: &[Token]) -> String {
    let mut iter = tokens.iter().peekable();
    let mut asm = String::new();
    let mut main_block = String::from("main:\n");
    let mut text_block = String::from("segment .text\n");
    let other_block = String::from("\n");

    asm += "bits 64\ndefault rel\n";
    text_block += "global main\n";

    while let Some(op) = iter.next() {
        match op {
            Token::Exit => {
                text_block += "\textern ExitProcess\n";
                main_block += "\tcall\tExitProcess\n";
            }
            _ => (),
        }
    }
    
    asm += &text_block;
    asm += &main_block;
    asm += &other_block;

    asm
}

fn main() {
    let input = "let x = 5 \nexit() let a7a = x let aa = \"'a7a{}'\" \n let y = exit (2) exit 20";
    let tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize();

    println!("{}", input);
    println!("Tokens: {:?}", tokens);

    let file_name = "main.asm";
    let asm_cont = generate_asm_win(&tokens);

    let mut file = File::create(file_name).expect("Failed to create file");
    file.write_all(asm_cont.as_bytes()).expect("Failed to write to file");

    let mut ast = Parser::new(&tokens);
    println!("Parse Trees: {:?}", ast.parse_tokens());
}
