use crate::parser;
use std::io::Write;
use std::fs::File;

enum ArithmeticTranslation {
	Add,
	// Equal
	Neg,
	Sub
}

impl ArithmeticTranslation {
	fn dedent(&self, translation: &str) -> String {
		translation
			.lines()
			.map(str::trim_start)
			.collect::<Vec<_>>()
			.join("\n")
	}

	fn value(&self) -> String {
		match *self {
			ArithmeticTranslation::Add => self.dedent(
				// SP--
				// D=RAM[SP]
				// SP--
				// RAM[SP] = RAM[SP + 1] + RAM[SP]
				// SP++
				"@SP
				M=M-1
				A=M
				D=M
				@SP
				M=M-1
				A=M
				M=D+M
				@SP
				M=M+1"
			),
			ArithmeticTranslation::Neg => self.dedent(
				"@SP
				M=M-1
				A=M
				M=-M
				@SP
				M=M+1"
			),
			ArithmeticTranslation::Sub => self.dedent(
				"@SP
				M=M-1
				A=M
				D=M
				@SP
				M=M-1
				A=M
				M=M-D
				@SP
				M=M+1"
			)
			// ArithmeticTranslation::Equal => self.dedent(
				// SP--
				// D=RAM[SP]
				// SP--
				// if RAM[SP] == D (RAM[SP + 1])
				// then RAM[SP] = 1
				// else RAM[SP] = 0

				// "@SP
				// M=M-1
				// A=M
				// D=M
				// @SP
				// M=M-1
				// A=M
				// M=D-M
				// @EQUAL
				// M;JEQ
				// M=0
				// @SP
				// M=M+1
				// @NOT_EQUAL
				// 0;JEQ
				// (EQUAL)
				// @SP
				// A=M
				// M=1
				// @SP
				// M=M+1
				// (NOT_EQUAL)"
			// )
		}
	}
}

pub struct CodeWriter {
	output_file: File
}
// pub struct CodeWriter;

impl CodeWriter {
	pub fn new(output_filename: String) -> Self {
		let output_file = File::create(output_filename);
		Self { output_file: output_file.expect("output file") }
	}

	pub fn write_arithmetic(&mut self, arg1: String) {
		if arg1 == "add" {
			//SP--
			writeln!(self.output_file, "{}", ArithmeticTranslation::Add.value());
		} else if arg1 == "neg" {
			// writeln!(self.output_file, "")
			writeln!(self.output_file, "{}", ArithmeticTranslation::Neg.value());
		} else if arg1 == "sub" {
			writeln!(self.output_file, "{}", ArithmeticTranslation::Sub.value());
		}
	}

	pub fn write_push_pop(&mut self, arg1: String, arg2: String, instruction_type: parser::InstructionType) {
		if instruction_type == parser::InstructionType::Push {
			// D = i
			writeln!(self.output_file, "@{}", arg2);
			writeln!(self.output_file, "D=A");
			// RAM[SP] = D
			writeln!(self.output_file, "@SP");
			writeln!(self.output_file, "A=M");
			writeln!(self.output_file, "M=D");
			// SP++
			writeln!(self.output_file, "@SP");
			writeln!(self.output_file, "M=M+1");
		} else if instruction_type == parser::InstructionType::Pop {
			// SP--
			writeln!(self.output_file, "@SP");
			writeln!(self.output_file, "M=M-1");
			writeln!(self.output_file, "A=M");
			writeln!(self.output_file, "D=M");
			// RAM[i] = RAM[SP]
			writeln!(self.output_file, "@{}", arg2);
			writeln!(self.output_file, "M=D");
		}
	}
}