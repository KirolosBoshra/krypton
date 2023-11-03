mod generator;
mod parser;
mod tokenize;
use std::{fs::File, io::Read, io::Write, process::Command};

use generator::Generator;
use parser::Parser;
use tokenize::Tokenizer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 4 {
        eprintln!("Usages:\n./krypton <file_path> -o <output_path>");
        std::process::exit(1);
    }
    let mut file_name = "main.kr";
    let mut output_name = "out";

    args.iter()
        .enumerate()
        .for_each(|(i, arg)| match arg.as_str() {
            "-o" => output_name = &args[i + 1],
            _ => {
                if arg.contains(".kr") {
                    file_name = &arg
                }
            }
        });

    let mut input = String::new();
    let mut file = File::open(file_name).expect(&format!("Can't open file {file_name}"));
    file.read_to_string(&mut input).expect("can't read file");

    let tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize();

    let mut ast = Parser::new(&tokens);
    println!("Parse Trees: {:?}", ast.parse_tokens());

    let mut generator = Generator::new(&ast.parse_tokens());
    let asm_cont = generator.generate_linux_64();

    let mut file = File::create(format!("{output_name}.s")).expect("Failed to create file");
    file.write_all(asm_cont.as_bytes())
        .expect("Failed to write to file assembly file");

    let nasm = Command::new("nasm")
        .args(&[
            "-f",
            "elf64",
            "-g",
            "-F",
            "dwarf",
            "-o",
            &format!("{output_name}.o"),
            &format!("{output_name}.s"),
        ])
        .spawn();
    if nasm.is_ok() {
        Command::new("ld")
            .args(&["-o", &output_name, &format!("{output_name}.o")])
            .spawn()
            .expect("Cannot run ld");
    } else {
        eprintln!("Cannot run nasm")
    }
}
