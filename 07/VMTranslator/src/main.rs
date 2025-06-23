use std::env;

mod code_writer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: cargo run -- <input filename> <output filename>");
        return;
    }

    let mut parser: parser::Parser = parser::Parser::new(
        args[1].clone(),
        code_writer::CodeWriter::new(args[2].clone()),
    );

    parser.parse().ok();
}
