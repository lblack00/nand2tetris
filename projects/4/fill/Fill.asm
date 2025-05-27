// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

@v
M=0

(LOOP)
	@256
	D=A
	@m
	M=D

	@SCREEN
	D=A
	@addr
	M=D

	@DRAW_COLS
	0;JMP

	@CHECK_INPUT
	0;JMP

(DRAW_ROW)
	@v
	D=M
	@addr
	A=M
	M=D
	@addr
	D=M
	@1
	D=D+A
	@addr
	M=D
	@n
	MD=M-1
	@DRAW_ROW
	D;JGT

(DRAW_COLS)
	@32
    D=A
	@n
	M=D
	@m
	MD=M-1
	@DRAW_ROW
	D;JGT

(CHECK_INPUT)
	@KBD
	D=M
	@SET_BLACK
	D;JNE
	@SET_WHITE
	D;JEQ

(SET_BLACK)
	@v
	M=-1
	@LOOP
	0;JMP

(SET_WHITE)
	@v
	M=0
	@LOOP
	0;JMP

(END)
