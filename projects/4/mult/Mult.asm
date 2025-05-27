// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

@R0
D=M
@CHECKZERO
D;JLE
@R1
D=M
@CHECKZERO
D;JLE
@R2
M=D
@R0
D=M
D=D-1
@LOOP
D;JGT
@END
D;JLE

(LOOP)
	@temp
	M=D
	@R1
	D=M
	@R2
	M=D+M
	@temp
	D=M
	D=D-1
	@LOOP
	D;JGT
	@END
	D;JLE

(CHECKZERO)
	@R2
	M=0
	@END
	0;JMP

(END)
	@END
	0;JMP
