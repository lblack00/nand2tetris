mod parser;
mod code_writer;

fn main() {
    let mut parser: parser::Parser =
        parser::Parser::new(
            // "../StackArithmetic/StackTest/StackTest.vm".to_string(),
            // code_writer::CodeWriter::new("../StackArithmetic/StackTest/StackTest.asm".to_string())
            "src/test.vm".to_string(),
            code_writer::CodeWriter::new("src/test.new.asm".to_string())
        );
        // parser::Parser::new("../MemoryAccess/BasicTest/BasicTest.vm".to_string());

    parser.parse().ok();
}
