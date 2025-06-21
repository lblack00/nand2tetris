mod code_writer;
mod parser;

fn main() {
    let mut parser: parser::Parser = parser::Parser::new(
        // "../StackArithmetic/SimpleAdd/SimpleAdd.vm".to_string(),
        // code_writer::CodeWriter::new("../StackArithmetic/SimpleAdd/SimpleAdd.asm".to_string()),
        // "../StackArithmetic/StackTest/StackTest.vm".to_string(),
        // code_writer::CodeWriter::new("../StackArithmetic/StackTest/StackTest.asm".to_string()),
        "../MemoryAccess/BasicTest/BasicTest.vm".to_string(),
        code_writer::CodeWriter::new("../MemoryAccess/BasicTest/BasicTest.asm".to_string()),
        // "src/test.vm".to_string(),
        // code_writer::CodeWriter::new("src/test.asm".to_string()),
    );
    // parser::Parser::new("../MemoryAccess/BasicTest/BasicTest.vm".to_string());

    parser.parse().ok();
}
