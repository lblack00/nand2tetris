use crate::code_writer;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};

#[derive(Debug, PartialEq)]
pub enum InstructionType {
    Arithmetic,
    Push,
    Pop,
}

pub struct Parser {
    input_filepath: String,
    writer: code_writer::CodeWriter,
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Parser: {}", self.input_filepath)
    }
}

impl Parser {
    pub fn new(input_filepath: String, writer: code_writer::CodeWriter) -> Self {
        Self {
            input_filepath: input_filepath,
            writer: writer,
        }
    }

    fn clean_line(&self, line: &str) -> String {
        line.split_once("//")
            .map(|(before, _)| before)
            .unwrap_or(&line)
            .trim()
            .to_string()
    }

    pub fn parse(&mut self) -> Result<(), std::io::Error> {
        let lines = self.read_lines()?;

        for line in lines {
            let line = line?;
            let current_instruction = self.clean_line(&line);

            if current_instruction.is_empty() {
                continue;
            }

            let instruction_type = self.command_type(&current_instruction);
            let arg1 = self.arg_n(&current_instruction, &instruction_type, true);
            let arg2 = self.arg_n(&current_instruction, &instruction_type, false);

            // println!("{:?} {:?} {:?} {:?}",
            //     current_instruction,
            //     instruction_type,
            //     arg1,
            //     arg2);
            if instruction_type == InstructionType::Arithmetic {
                self.writer.write_arithmetic_or_logical(arg1);
            } else {
                self.writer.write_push_pop(arg1, arg2, instruction_type);
            }
        }

        Ok(())
    }

    fn command_type(&self, instruction: &str) -> InstructionType {
        match instruction.split_whitespace().next() {
            Some("push") => InstructionType::Push,
            Some("pop") => InstructionType::Pop,
            _ => InstructionType::Arithmetic,
        }
    }

    fn arg_n(
        &self,
        instruction: &str,
        instruction_type: &InstructionType,
        first_arg: bool,
    ) -> String {
        if *instruction_type == InstructionType::Arithmetic {
            return if first_arg {
                instruction.to_string()
            } else {
                String::new()
            };
        }

        instruction
            .split_whitespace()
            .nth(if first_arg { 1 } else { 2 })
            .unwrap_or("")
            .to_string()
    }

    fn read_lines(&self) -> io::Result<Lines<BufReader<File>>> {
        let file = File::open(&self.input_filepath)?;

        Ok(BufReader::new(file).lines())
    }
}
