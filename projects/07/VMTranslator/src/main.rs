mod parser;

fn main() {
    let parser: parser::Parser =
        parser::Parser::new("../StackArithmetic/SimpleAdd/SimpleAdd.vm".to_string());

    parser.parse().ok();
}
