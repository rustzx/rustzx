# rustzx
ZX Spectrum emulator which I writing in rust.   
Project is just for fun and learning architecture of CPU's  
Licensed under MIT License.

## Current progress
I writing Z80 CPU emulation part at the moment  
### instruction groups
__Implemented__  
- NOP
- INC
- DEC
- DJNZ  (Jump if B register is non-zero)
- JR  (Relative jumps)
- DAA (Decimal Adjust)
- CPL (Complement, NOT operation)
- SCF (Set carry flag)
- CCF (Invert carry flag)
- HALT
- ADD    
- SUB
- AND
- OR
- XOR
- EX (Exchange)
- EXX (BC, DE, HL Block exchange)
- POP
- PUSH
- RET (RET, Conditional RET)
- RST
- JP (JUMP, conditional JUMP)
- DI (Disable interrupts)
- EI (Enable interrupts)
- CALL (CALL, conditional CALL)

__Partialy implemented__  
- ADC
    - [x] 8 bit
    - [ ] 16 bit
- SBC
    - [x] 8 bit
    - [ ] 16 bit    
- CP
    - [x] 8 bit
    - [ ] block instructions       
- LD
	- [x] LD A, (BC/DE)
	- [x] LD A, (NN)
	- [x] LD BC/DE/HL/IX/IY, NN
	- [x] LD HL/IX/IY, (NN)
	- [x] LD (BC/DE), A
	- [x] LD (NN), A
	- [x] LD (NN), HL/IX/IY
    - [x] LD r[y], n ; where r - 8 bit register, n - const
    - [x] LD (HL/IX+d/IY+d), n
    - [x] LD reg1, reg2
    - [x] LD (HL/IX+d/IY+d), reg
    - [x] LD reg, (HL/IX+d/IY+d)
    - [X] LD SP, HL/IX/IY
	- [ ] LD between A and I or R
    - [ ] LD BC/DE/SP (NN)
    - [ ] LD (NN), BC/DE/SP
    - [ ] block instructions
- ROTATE
    - [x] RRA
    - [x] RLA
    - [x] RRCA
    - [x] RLCA
    - [ ] Other instructions of group
- IN
    - [x] IN A, (n)
    - [ ] IN (C)
    - [ ] IN r, (C)
    - [ ] block instructions
- OUT
    - [x] OUT (n), A
    - [ ] OUT (C), 0
    - [ ] OUT (C), r
    - [ ] block instructions
