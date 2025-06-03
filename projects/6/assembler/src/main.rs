use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Debug, PartialEq)]
enum InstructionType {
    LInstruction,
    AInstruction,
    CInstruction,
}

struct Parser<'a> {
    filepath: &'a str,
}

impl Parser<'_> {
    pub fn parse(&mut self) {
        if let Ok(lines) = self.read_lines() {
            // Consumes the iterator, returns an (Optional) String
            for line in lines.map_while(Result::ok) {
                let current_instruction = match line.split_once("//") {
                    Some((before, _)) => before.trim(),
                    None => line.trim(),
                };

                if current_instruction.is_empty() {
                    continue;
                }

                let instruction_type: InstructionType = self.instruction_type(current_instruction);

                if instruction_type == InstructionType::CInstruction {
                    let dest_str = self.dest(current_instruction);
                    let comp_str = self.comp(current_instruction);
                    let jump_str = self.jump(current_instruction);
                    println!("{:?} {:?} {:?}", dest_str, comp_str, jump_str);
                } else if instruction_type == InstructionType::AInstruction {
                    // let address = &line_without_comments[1..].parse::<usize>().unwrap() & 0x7FFF;
                    // let bin_address = format!("{:015b}", address);
                    // println!("{:?}", bin_address);
                }

                println!("{:?} {:?}", current_instruction, instruction_type);
            }
        }
    }

    fn comp(&self, instruction: &str) -> &str {
        let comp_str = if let Some((_, after)) = instruction.rsplit_once('=') {
            if let Some((middle, _)) = after.rsplit_once(';') {
                middle
            } else {
                after
            }
        } else if let Some((before, _)) = instruction.rsplit_once(';') {
            before
        } else {
            return "0101010";
        };

        match comp_str {
            "0" => "0101010",
            "1" => "0111111",
            "-1" => "0111010",
            "D" => "0001100",
            "A" => "0110000",
            "!D" => "0001101",
            "!A" => "0110001",
            "-D" => "0001111",
            "-A" => "0110011",
            "D+1" => "0011111",
            "A+1" => "0110111",
            "D-1" => "0001110",
            "A-1" => "0110010",
            "D+A" => "0000010",
            "D-A" => "0010011",
            "A-D" => "0000111",
            "D&A" => "0000000",
            "D|A" => "0010101",
            "M" => "1110000",
            "!M" => "1110001",
            "-M" => "1110011",
            "M+1" => "1110111",
            "M-1" => "1110010",
            "D+M" => "1000010",
            "D-M" => "1010011",
            "M-D" => "1000111",
            "D&M" => "1000000",
            "D|M" => "1010101",
            &_ => "0101010",
        }
    }

    fn dest(&self, instruction: &str) -> &str {
        let Some((dest_str, _)) = instruction.rsplit_once('=') else {
            return "000";
        };

        match dest_str {
            "null" => "000",
            "M" => "001",
            "D" => "010",
            "MD" => "011",
            "A" => "100",
            "AM" => "101",
            "AD" => "110",
            "AMD" => "111",
            &_ => "000",
        }
    }

    fn jump(&self, instruction: &str) -> &str {
        let Some((_, jump_str)) = instruction.rsplit_once(';') else {
            return "000";
        };

        match jump_str {
            "null" => "000",
            "JGT" => "001",
            "JEQ" => "010",
            "JGE" => "011",
            "JLT" => "100",
            "JNE" => "101",
            "JLE" => "110",
            "JMP" => "111",
            &_ => "000",
        }
    }

    fn instruction_type(&self, line: &str) -> InstructionType {
        if line.starts_with("@") {
            return InstructionType::AInstruction;
        } else if line.starts_with("(") && line.ends_with(")") {
            return InstructionType::LInstruction;
        }

        return InstructionType::CInstruction;
    }

    fn read_lines(&self) -> io::Result<io::Lines<io::BufReader<File>>> {
        let file = File::open(self.filepath)?;
        Ok(io::BufReader::new(file).lines())
    }
}

fn main() {
    let mut parser = Parser {
        filepath: "../add/Add.asm",
    };
    // Call parser to translate .asm file into binary
    parser.parse();
}
