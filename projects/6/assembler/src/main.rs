use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};

const MAX_ADDRESS: usize = 0x7FFF;
const STARTING_VARIABLE_ADDRESS: usize = 16;
static SYMBOL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z_.$:]+[A-Za-z_.$:0-9]*$").unwrap());
static ADDRESS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9]+$").unwrap());

#[derive(Clone, Debug, PartialEq)]
enum InstructionType {
    LInstruction,
    AInstruction,
    CInstruction,
}

#[derive(Debug)]
enum ParserError {
    EmptyAddress,
    InvalidNumber(std::num::ParseIntError),
    InvalidFormat,
    InvalidSymbol,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::EmptyAddress => write!(f, "Empty address provided"),
            ParserError::InvalidNumber(n) => write!(f, "Unable to parse number: {}", n),
            ParserError::InvalidFormat => write!(f, "Invalid instruction format"),
            ParserError::InvalidSymbol => write!(f, "Invalid symbol format"),
        }
    }
}

impl From<std::num::ParseIntError> for ParserError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParserError::InvalidNumber(err)
    }
}

struct Parser<'a> {
    input_filepath: &'a str,
    output_file: File,
    program_counter: &'a mut usize,
    symbols: &'a mut HashMap<String, usize>,
    symbol_counter: &'a mut usize,
}

impl Parser<'_> {
    fn clean_line<'a>(&self, line: &'a str) -> &'a str {
        line.split_once("//").map(|(before, _)| before).unwrap_or(line).trim()
    }

    fn first_pass(&mut self, lines: &Vec<String>) {
        for line in lines {
            // Take instructions before inline comments if any
            let current_instruction = self.clean_line(line);

            // Check if instruction is whitespace or comment
            if current_instruction.is_empty() {
                continue;
            }

            // Determine instruction type
            let instruction_type: InstructionType = self.instruction_type(current_instruction);

            // Parse labels
            if instruction_type == InstructionType::LInstruction {
                let _ = self.parse_label_symbol(current_instruction);
            } else {
                *self.program_counter += 1;
            }
        }
    }

    fn second_pass(&mut self, lines: &Vec<String>) {
        // Second pass
        for line in lines {
            let current_instruction = self.clean_line(line);

            if current_instruction.is_empty() {
                continue;
            }

            // Determine instruction type and parse C and A instructions
            let result = match self.instruction_type(current_instruction) {
                InstructionType::LInstruction => continue,
                InstructionType::CInstruction => {
                    self.parse_c_instruction(current_instruction)
                }
                InstructionType::AInstruction => {
                    self.parse_a_instruction(current_instruction)
                }
            };

            if let Err(e) = write!(self.output_file, "{}\n", result.unwrap()) {
                panic!("Error parsing instruction '{}': {}", line, e);
            }
        }
    }

    fn parse(&mut self) {
        let lines = self.read_lines().ok().expect("read file");

        self.first_pass(&lines);
        self.second_pass(&lines);
    }

    fn parse_c_instruction(&mut self, current_instruction: &str) -> Result<String, ParserError> {
        let comp_bin_str = Code.comp_to_binary(self.comp(current_instruction)?);
        let dest_str = Code.dest_to_binary(self.dest(current_instruction).unwrap_or("null"));
        let jump_str = Code.jump_to_binary(self.jump(current_instruction).unwrap_or("null"));
        let bin_str = format!("111{}{}{}", comp_bin_str, dest_str, jump_str);

        Ok(bin_str)
    }

    fn parse_a_instruction(&mut self, current_instruction: &str) -> Result<String, ParserError> {
        let addr_str = self
            .addr(current_instruction)
            .ok_or(ParserError::EmptyAddress)?;

        if addr_str.is_empty() {
            return Err(ParserError::EmptyAddress);
        }

        if SYMBOL_RE.is_match(&addr_str) {
            return Ok(self.parse_symbol(&addr_str)?);
        } else if ADDRESS_RE.is_match(&addr_str) {
            let bin_str = format!("{:016b}", addr_str.parse::<usize>()? & MAX_ADDRESS);
            return Ok(bin_str);
        }

        Err(ParserError::InvalidFormat)
    }

    fn parse_label_symbol(&mut self, current_instruction: &str) -> Result<String, ParserError> {
        // Remove parenthesis
        let label_str = &current_instruction[1..current_instruction.len() - 1];

        if SYMBOL_RE.is_match(&label_str) {
            self.symbols
                .insert(label_str.to_string(), *self.program_counter);
            return Ok(label_str.to_string());
        }

        Err(ParserError::InvalidFormat)
    }

    fn parse_symbol(&mut self, addr_str: &str) -> Result<String, ParserError> {
        // If symbol is not in the symbol table, add it to symbol table as variable
        if !self.symbols.contains_key(addr_str) {
            self.symbols
                .insert(addr_str.to_string(), *self.symbol_counter);
            *self.symbol_counter += 1;
        }

        let Some(symbol_val) = self.symbols.get(addr_str) else {
            return Err(ParserError::InvalidSymbol);
        };
        let bin_str = format!("{:016b}", symbol_val & MAX_ADDRESS);

        Ok(bin_str)
    }

    fn addr<'a>(&self, instruction: &'a str) -> Option<&'a str> {
        Some(&instruction[1..])
    }

    fn comp<'a>(&self, instruction: &'a str) -> Result<&'a str, ParserError> {
        // Handle DEST=COMP
        if let Some((_, after)) = instruction.rsplit_once('=') {
            // Handle DEST=COMP;JUMP
            if let Some((middle, _)) = after.rsplit_once(';') {
                return Ok(middle);
            } else {
                return Ok(after);
            }
        // Handle COMP;JUMP
        } else if let Some((before, _)) = instruction.rsplit_once(';') {
            return Ok(before);
        };

        Err(ParserError::InvalidFormat)
    }

    fn dest<'a>(&self, instruction: &'a str) -> Option<&'a str> {
        let Some((dest_str, _)) = instruction.rsplit_once('=') else {
            return None;
        };

        Some(dest_str)
    }

    fn jump<'a>(&self, instruction: &'a str) -> Option<&'a str> {
        let Some((_, jump_str)) = instruction.rsplit_once(';') else {
            return None;
        };

        Some(jump_str)
    }

    fn instruction_type(&self, line: &str) -> InstructionType {
        if line.starts_with("@") {
            return InstructionType::AInstruction;
        } else if line.starts_with("(") && line.ends_with(")") {
            return InstructionType::LInstruction;
        }

        InstructionType::CInstruction
    }

    fn read_lines(&self) -> io::Result<Vec<String>> {
        let file = File::open(self.input_filepath)?;
        let buffer = io::BufReader::new(file);

        buffer.lines().collect()
    }
}

struct Code;

impl Code {
    fn comp_to_binary(&self, comp_str: &str) -> &str {
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
            _ => "0101010",
        }
    }

    fn dest_to_binary(&self, dest_str: &str) -> &str {
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

    fn jump_to_binary(&self, jump_str: &str) -> &str {
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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: assembler -- <input filename> <output filename>");
        return;
    }

    let mut default_symbols = HashMap::<String, usize>::from([
        ("SP".to_string(), 0x0000),
        ("LCL".to_string(), 0x0001),
        ("ARG".to_string(), 0x0002),
        ("THIS".to_string(), 0x0003),
        ("THAT".to_string(), 0x0004),
        ("SCREEN".to_string(), 0x4000),
        ("KBD".to_string(), 0x6000),
    ]);
    let mut symbol_counter: usize = STARTING_VARIABLE_ADDRESS;

    // Default registers R0..R15
    for i in 0..=15 {
        let formatted_symbol = format!("R{:}", i);
        default_symbols.insert(formatted_symbol, i);
    }

    let output_file = File::create(&args[2]);

    let mut parser = Parser {
        input_filepath: &args[1],
        output_file: output_file.expect("output file"),
        program_counter: &mut (0 as usize),
        symbols: &mut default_symbols,
        symbol_counter: &mut symbol_counter,
    };

    // Call parser to translate .asm file into binary
    parser.parse();
}
