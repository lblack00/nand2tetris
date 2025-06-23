use crate::parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

enum ArithmeticTranslation {
    Add,
    Neg,
    Sub,
    And,
    Or,
    Not,
}

fn dedent(translation: String) -> String {
    translation
        .lines()
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n")
}

impl ArithmeticTranslation {
    fn two_arg_f(&self, operation: &str) -> String {
        dedent(format!(
            // SP--
            // D=RAM[SP]
            // SP--
            // RAM[SP] = RAM[SP + 1] <op> RAM[SP]
            // SP++
            "@SP
            M=M-1
            A=M
            D=M
            @SP
            M=M-1
            A=M
            M=M{}D
            @SP
            M=M+1",
            operation
        ))
    }

    fn one_arg_f(&self, operation: &str) -> String {
        dedent(format!(
            // SP--
            // RAM[SP]=<op>RAM[SP]
            // SP++
            "@SP
            M=M-1
            A=M
            M={}M
            @SP
            M=M+1",
            operation
        ))
    }

    fn value(&self) -> String {
        match *self {
            ArithmeticTranslation::Add => self.two_arg_f("+"),
            ArithmeticTranslation::Neg => self.one_arg_f("-"),
            ArithmeticTranslation::Sub => self.two_arg_f("-"),
            ArithmeticTranslation::And => self.two_arg_f("&"),
            ArithmeticTranslation::Or => self.two_arg_f("|"),
            ArithmeticTranslation::Not => self.one_arg_f("!"),
        }
    }
}

enum LogicalTranslation {
    Equal,
    LessThan,
    GreaterThan,
}

impl LogicalTranslation {
    fn generate_bool_compare_asm(&self, keyword: &str, jump: &str, cnt: usize) -> String {
        format!(
            // SP--
            // D = RAM[SP + 1]
            // SP--
            // if D - RAM[SP] == 0 then 1 else 0
            // SP++
            "@SP
            M=M-1
            A=M
            D=M
            @SP
            M=M-1
            A=M
            D=M-D
            @{1}_{0}
            D;{2}
            @NOT_{1}_{0}
            0;JMP
            
            ({1}_{0})
            @SP
            A=M
            M=-1
            @SP
            M=M+1
            @END_{1}_{0}
            0;JMP
            (NOT_{1}_{0})
            @SP
            A=M
            M=0
            @SP
            M=M+1
            (END_{1}_{0})
            ",
            cnt, keyword, jump
        )
    }

    fn value(&self, command_counts: &HashMap<String, usize>) -> String {
        match *self {
            LogicalTranslation::Equal => {
                let eq_cnt: usize = *command_counts.get("eq").expect("eq key");
                let eq_asm: String = self.generate_bool_compare_asm("EQUAL", "JEQ", eq_cnt);

                dedent(eq_asm)
            }
            LogicalTranslation::LessThan => {
                let lt_cnt: usize = *command_counts.get("lt").expect("lt key");
                let lt_asm: String = self.generate_bool_compare_asm("LT", "JLT", lt_cnt);

                dedent(lt_asm)
            }
            LogicalTranslation::GreaterThan => {
                let gt_cnt: usize = *command_counts.get("gt").expect("gt key");
                let gt_asm: String = self.generate_bool_compare_asm("GT", "JGT", gt_cnt);

                dedent(gt_asm)
            }
        }
    }
}

pub struct CodeWriter {
    output_file: File,
    command_counts: HashMap<String, usize>,
}

impl CodeWriter {
    pub fn new(output_filename: String) -> Self {
        let output_file = File::create(output_filename);
        Self {
            output_file: output_file.expect("output file"),
            command_counts: HashMap::<String, usize>::from([
                ("eq".to_string(), 0),
                ("lt".to_string(), 0),
                ("gt".to_string(), 0),
            ]),
        }
    }

    pub fn write_arithmetic_or_logical(&mut self, arg1: String) -> Result<(), std::io::Error> {
        if arg1 == "eq" || arg1 == "lt" || arg1 == "gt" {
            let cnt: usize = *self
                .command_counts
                .get(&arg1)
                .expect(format!("{} key", arg1).as_str());
            let asm_string: String = match arg1.as_str() {
                "eq" => LogicalTranslation::Equal.value(&self.command_counts),
                "lt" => LogicalTranslation::LessThan.value(&self.command_counts),
                "gt" => LogicalTranslation::GreaterThan.value(&self.command_counts),
                _ => unreachable!(),
            };

            writeln!(self.output_file, "{}", asm_string)?;
            self.command_counts.insert(arg1.to_string(), cnt + 1);
        } else {
            let asm_string: String = match arg1.as_str() {
                "add" => ArithmeticTranslation::Add.value(),
                "neg" => ArithmeticTranslation::Neg.value(),
                "sub" => ArithmeticTranslation::Sub.value(),
                "and" => ArithmeticTranslation::And.value(),
                "or" => ArithmeticTranslation::Or.value(),
                "not" => ArithmeticTranslation::Not.value(),
                _ => todo!(),
            };

            writeln!(self.output_file, "{}", asm_string)?;
        }

        Ok(())
    }

    fn get_address_symbol(&self, arg1: String, arg2: &str) -> String {
        let address_symbol: &str = match arg1.as_str() {
            "local" => "LCL",
            "this" => "THIS",
            "that" => "THAT",
            "argument" => "ARG",
            "static" => arg2,
            "pointer" => {
                if arg2 == "0" {
                    "THIS"
                } else {
                    "THAT"
                }
            }
            "temp" => match arg2 {
                "0" => "5",
                "1" => "6",
                "2" => "7",
                "3" => "8",
                "4" => "9",
                "5" => "10",
                "6" => "11",
                "7" => "12",
                &_ => unreachable!(),
            },
            _ => unreachable!(),
        };

        address_symbol.to_string()
    }

    fn get_push_asm(&self, arg1: String, arg2: String) -> String {
        if arg1 == "constant" || arg1 == "temp" || arg1 == "static" || arg1 == "pointer" {
            let this_or_that: &str = if arg2 == "0" { "THIS" } else { "THAT" };

            let address_asm = match arg1.as_str() {
                "constant" => format!(
                    "@{}
                    D=A",
                    arg2.as_str()
                ),
                "temp" => format!(
                    "@{}
                    D=M",
                    self.get_address_symbol(arg1, &arg2)
                ),
                "static" => format!(
                    "@{}
                    D=A
                    @16
                    A=D+A
                    D=M",
                    arg2.as_str()
                ),
                "pointer" => format!(
                    "@{}
                    D=M",
                    this_or_that
                ),
                _ => unreachable!(),
            };

            return format!(
                // D = i
                // RAM[SP] = D
                // SP++
                "{}
                @SP
                A=M
                M=D
                @SP
                M=M+1",
                address_asm
            );
        }

        let address_symbol: String = self.get_address_symbol(arg1, &arg2);

        format!(
            "@{0}
            D=M
            @{1}
            D=D+A
            A=D
            D=M
            @SP
            A=M
            M=D
            @SP
            M=M+1",
            address_symbol, arg2
        )
    }

    fn get_pop_asm(&self, arg1: String, arg2: String) -> String {
        if arg1 == "constant" || arg1 == "temp" || arg1 == "static" || arg1 == "pointer" {
            let this_or_that: &str = if arg2 == "0" { "THIS" } else { "THAT" };

            let address_asm = match arg1.as_str() {
                "constant" => arg2.as_str(),
                "temp" => &self.get_address_symbol(arg1, &arg2),
                "static" => &format!(
                    "{}",
                    (arg2.parse::<usize>().expect("static") + 16)
                        .to_string()
                        .as_str()
                ),
                "pointer" => &format!("{}", this_or_that),
                _ => unreachable!(),
            };

            return format!(
                // SP--
                // RAM[i] = RAM[SP]
                "@SP
                M=M-1
                A=M
                D=M
                @{}
                M=D",
                address_asm
            );
        }

        let address_symbol: String = self.get_address_symbol(arg1, &arg2);

        format!(
            "@{}
            D=A
            @{}
            D=D+M
            @R13
            M=D
            @SP
            M=M-1
            A=M
            D=M
            @R13
            A=M
            M=D",
            arg2, address_symbol
        )
    }

    pub fn write_push_pop(
        &mut self,
        arg1: String,
        arg2: String,
        instruction_type: parser::InstructionType,
    ) -> Result<(), std::io::Error> {
        if instruction_type == parser::InstructionType::Push {
            let push_asm: String = dedent(self.get_push_asm(arg1, arg2));
            writeln!(self.output_file, "{}", push_asm)?;
        } else if instruction_type == parser::InstructionType::Pop {
            let pop_asm: String = dedent(self.get_pop_asm(arg1, arg2));
            writeln!(self.output_file, "{}", pop_asm)?;
        }

        Ok(())
    }
}
