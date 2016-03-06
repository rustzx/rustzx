use cpu::*;
use utils::*;
/// Z80 processor System bus
/// Implement it for communication with CPU.
// TODO: display debug info only when some cfg flag activated
// TODO: code reorganization, refactoring
// TODO: maybe devide Z80 into set of traits from base "chip" ?
#[allow(unused_variables)]
pub trait Z80Bus {
    /// Required method for read byte from bus
    fn read(&self, addr: u16) -> u8;
    /// Required method for write byte to bus
    fn write(&mut self, addr: u16, data: u8);
    // Method for reading from io port
    fn read_io(&mut self, addr: u16) -> u8 {
        0
    }
    // Required method for writing to io port
    fn write_io(&mut self, addr: u16, data: u8) {

    }
    /// provided metod to write word, LSB first
    fn write_word(&mut self, addr: u16, data: u16) {
        let (h, l) = split_word(data);
        self.write(addr, l);
        self.write(addr.wrapping_add(1), h);
    }
    /// provided method to read word
    fn read_word(&mut self, addr: u16) -> u16 {
        let l = self.read(addr);
        let h = self.read(addr.wrapping_add(1));
        make_word(h, l)
    }
    /// method, invoked by Z80 in case of RETI instruction
    fn reti_signal(&mut self) {}
}


/// Opcode, devided in parts
/// ```text
/// xxyyyzzz
/// xxppqzzz
/// ```
/// Used for splitting opcode byte into parts
#[derive(Clone, Copy)]
struct Opcode {
    pub byte: u8,
    pub x: U2,
    pub y: U3,
    pub z: U3,
    pub p: U2,
    pub q: U1,
}
impl Opcode {
    /// split opcode into parts
    fn from_byte(data: u8) -> Opcode {
        Opcode {
            byte: data,
            x: U2::from_byte(data, 6),
            y: U3::from_byte(data, 3),
            z: U3::from_byte(data, 0),
            p: U2::from_byte(data, 4),
            q: U1::from_byte(data, 3),
        }
    }
}

/// Execution result of the instruction
enum ExecResult {
    /// contains clocks count, which was elapsed to execute instruction
    Executed(u8),
    NonInstuction,
    /// emulator fail
    Fail,
}

/// Operand for 8-bit LD instructions
enum LoadOperand8 {
    Indirect(u16),
    Reg(RegName8),
}

/// Operand for 8-bit Rotate instructions
enum RotOperand8  {
    Indirect(u16),
    Reg(RegName8),
}

/// Operand for 8-bit ALU instructions
enum AluOperand8 {
    Indirect(u16),
    Reg(RegName8),
    Const(u8),
}

/// Interrupt mode
enum IntMode {
    IM0,
    IM1,
    IM2,
}

// direction of address cahange in block functions
enum BlockDir {
    Inc,
    Dec,
}

/// Z80 Processor struct
pub struct Z80 {
    /// CPU Regs struct
    regs: Regs,
    /// cycles, which were uncounted from previous emulation
    uncounted_cycles: u64,
    halted: bool,
    skip_interrupt: bool,
    int_mode: IntMode,
}

impl Z80 {
    /// new cpu instance
    pub fn new() -> Z80 {
        Z80 {
            regs: Regs::new(),
            uncounted_cycles: 0,
            halted: false,
            skip_interrupt: false,
            int_mode: IntMode::IM0,
        }
    }

    // Returns true if z80 is halted at the moment
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// read byte from rom and, pc += 1
    fn rom_next_byte(&mut self, bus: &Z80Bus) -> u8 {
        let addr = self.regs.get_pc();
        self.regs.inc_pc(1);
        bus.read(addr)
    }

    /// read word from rom and, pc += 2
    fn rom_next_word(&mut self, bus: &Z80Bus) -> u16 {
        let (hi, lo);
        lo = self.regs.get_pc();
        hi = self.regs.inc_pc(1);
        self.regs.inc_pc(1);
        make_word(bus.read(hi), bus.read(lo))
    }


    /// normal execution group, can be modified with prefixes DD, FD, providing
    /// DD OPCODE [NN], FD OPCODE [NN] instruction group
    ///
    /// Opcode matching organised based on
    /// [document](http://www.z80.info/decoding.htm) by Cristian Dinu
    ///
    /// DAA algorithm
    /// [link](http://www.worldofspectrum.org/faq/reference/z80reference.htm#DAA)
    fn execute_normal(&mut self, bus: &mut Z80Bus, opcode: Opcode, prefix: Prefix) -> ExecResult {
        let mut clocks = 0;
        // 2 first bits of opcode
        match opcode.x {
            // ---------------------------------
            // [0b00yyyzzz] instruction section
            // ---------------------------------
            // [0b00yyy000] instruction group (NOP, EX, DJNZ, JR)
            U2::N0 if opcode.z == U3::N0 => {
                match opcode.y {
                    // NOP
                    // [0b00000000] = 0x00
                    U3::N0 => {}
                    // EX AF, AF'
                    // [0b00001000] = 0x08
                    U3::N1 => {
                        self.regs.swap_af_alt();
                    }
                    // DJNZ offset;   13/8 clocks
                    // [0b00010000] = 0x10
                    U3::N2 => {
                        let offset = self.rom_next_byte(bus) as i8;
                        // preform jump
                        if self.regs.dec_reg_8(RegName8::B, 1) != 0 {
                            self.regs.shift_pc(offset);
                            clocks += 13;
                        } else {
                            clocks += 8;
                        };
                        // pc already pointing to next instruction
                    }
                    // JR offset
                    // [0b00011000] = 0x18
                    U3::N3 => {
                        let offset = self.rom_next_byte(bus) as i8;
                        self.regs.shift_pc(offset);
                    }
                    // JR condition[y-4] displacement;
                    // NZ [0b00100000], Z [0b00101000] NC [0b00110000] C [0b00111000]
                    U3::N4 | U3::N5 | U3::N6 | U3::N7 => {
                        // 0x20, 0x28, 0x30, 0x38
                        let offset = self.rom_next_byte(bus) as i8;
                        // y in range 4..7, non-wrapped sub allowed
                        let cnd = Condition::from_u3(U3::from_byte(opcode.y.as_byte() - 4, 0));
                        if self.regs.eval_condition(cnd) {
                            self.regs.shift_pc(offset);
                            clocks += 12;
                        } else {
                            clocks += 7;
                        };
                    }
                };
            }
            // [0b00ppq001] instruction group (LD, ADD)
            U2::N0 if opcode.z == U3::N1 => {
                match opcode.q {
                    // LD rp[p], nn
                    // [0b00pp0001] : 0x01, 0x11, 0x21, 0x31
                    U1::N0 => {
                        let reg = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                        let data = self.rom_next_word(bus);
                        self.regs.set_reg_16(reg, data);
                    }
                    // ADD HL/IX/IY, ss ; ss - 16 bit with sp set
                    // [0b00pp1001] : 0x09; 0x19; 0x29; 0x39
                    U1::N1 => {
                        let reg_operand = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                        let reg_acc = RegName16::HL.with_prefix(prefix);
                        let acc = self.regs.get_reg_16(reg_acc);
                        let operand = self.regs.get_reg_16(reg_operand);
                        // calc half_carry
                        let half_carry = half_carry_16(acc, operand);
                        let (acc, carry) = acc.overflowing_add(operand);
                        // check flags!
                        self.regs.set_flag(Flag::Carry, carry); //set carry
                        self.regs.set_flag(Flag::Sub, false); // is addition
                        self.regs.set_flag(Flag::HalfCarry, half_carry); // half carry
                        self.regs.set_flag(Flag::F3, acc & 0b100000000000 != 0); // 3 bit of hi
                        self.regs.set_flag(Flag::F5, acc & 0b10000000000000 != 0); // 5 bit of hi
                        // set register!
                        self.regs.set_reg_16(reg_acc, acc);
                    }
                };
            }
            // [0b00ppq010] instruction group (LD INDIRECT)
            U2::N0 if opcode.z == U3::N2 => {
                match opcode.q {
                    // LD (BC), A
                    // [0b00000010] : 0x02
                    U1::N0 if opcode.p == U2::N0 => {
                        bus.write(self.regs.get_reg_16(RegName16::BC),
                                  self.regs.get_reg_8(RegName8::A));
                    }
                    // LD (DE), A
                    // [0b00010010] : 0x12
                    U1::N0 if opcode.p == U2::N1 => {
                        bus.write(self.regs.get_reg_16(RegName16::DE),
                                  self.regs.get_reg_8(RegName8::A));
                    }
                    // LD (nn), HL/IX/IY
                    // [0b00100010] : 0x22
                    U1::N0 if opcode.p == U2::N2 => {
                        let addr = self.rom_next_word(bus);
                        let reg = RegName16::HL.with_prefix(prefix);
                        bus.write_word(addr, self.regs.get_reg_16(reg));
                    }
                    // LD (nn), A
                    // [0b00110010] : 0x32
                    U1::N0 => {
                        let addr = self.rom_next_word(bus);
                        bus.write(addr, self.regs.get_reg_8(RegName8::A));
                    }
                    // LD A, (BC)
                    // [0b00001010] : 0x0A
                    U1::N1 if opcode.p == U2::N0 => {
                        let addr = self.regs.get_reg_16(RegName16::BC);
                        self.regs.set_reg_8(RegName8::A, bus.read(addr));
                    }
                    // LD A, (DE)
                    // [0b00011010] : 0x1A
                    U1::N1 if opcode.p == U2::N1 => {
                        let addr = self.regs.get_reg_16(RegName16::BC);
                        self.regs.set_reg_8(RegName8::A, bus.read(addr));
                    }
                    // LD HL/IX/IY, (nn)
                    // [0b00101010] : 0x2A
                    U1::N1 if opcode.p == U2::N2 => {
                        let addr = self.rom_next_word(bus);
                        let reg = RegName16::HL.with_prefix(prefix);
                        self.regs.set_reg_16(reg, bus.read_word(addr));
                    }
                    // LD A, (nn)
                    // [0b00111010] : 0x3A
                    U1::N1 => {
                        let addr = self.rom_next_word(bus);
                        self.regs.set_reg_8(RegName8::A, bus.read(addr));
                    }
                };
            }
            // [0b00ppq011] instruction group (INC, DEC)
            U2::N0 if opcode.z == U3::N3 => {
                // get register by rp[pp]
                let reg = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                match opcode.q {
                    // INC BC/DE/HL/IX/IY/SP
                    // [0b00pp0011] : 0x03, 0x13, 0x23, 0x33
                    U1::N0 => {
                        self.regs.inc_reg_16(reg, 1);
                    }
                    // DEC BC/DE/HL/IX/IY/SP
                    // [0b00pp1011] : 0x03, 0x13, 0x23, 0x33
                    U1::N1 => {
                        self.regs.dec_reg_16(reg, 1);
                    }
                };
            }
            // [0b00yyy100], [0b00yyy101] instruction group (INC, DEC) 8 bit
            U2::N0 if (opcode.z == U3::N4) || (opcode.z == U3::N5) => {
                let operand;
                let data;
                let result;
                // ------------
                //   get data
                // ------------
                if let Some(mut reg) = RegName8::from_u3(opcode.y) {
                    // INC r[y], DEC y[y] ; IX and IY also used
                    // INC [0b00yyy100] : 0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x3C
                    // DEC [0b00yyy101] : 0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x3D
                    reg = reg.with_prefix(prefix);
                    data = self.regs.get_reg_8(reg);
                    operand = LoadOperand8::Reg(reg);
                } else {
                    // INC (HL)/(IX + d)/(IY + d), DEC (HL)/(IX + d)/(IY + d) ; INDIRECT
                    // INC [0b00110100], DEC [0b00110101] : 0x34, 0x35
                    let addr = if prefix == Prefix::None {
                        // we have IND/DEC (HL)
                        self.regs.get_reg_16(RegName16::HL)
                    } else {
                        // we have INC/DEC (IX/IY + d)
                        let d = self.rom_next_byte(bus) as i8;
                        word_displacement(self.regs.get_reg_16(RegName16::HL.with_prefix(prefix)),
                                          d)
                    };
                    // read data
                    data = bus.read(addr);
                    operand = LoadOperand8::Indirect(addr);
                };
                // ------------
                //   execute
                // ------------
                if opcode.z == U3::N4 {
                    // INC
                    result = data.wrapping_add(1);
                    self.regs.set_flag(Flag::Sub, false);
                    self.regs.set_flag(Flag::ParityOveflow, data == 0x7F);
                    self.regs.set_flag(Flag::HalfCarry, half_carry_8(data, 1));
                } else {
                    // DEC
                    result = data.wrapping_sub(1);
                    self.regs.set_flag(Flag::Sub, true);
                    self.regs.set_flag(Flag::ParityOveflow, data == 0x80);
                    self.regs.set_flag(Flag::HalfCarry, half_borrow_8(data, 1));
                }
                self.regs.set_flag(Flag::Zero, result == 0);
                self.regs.set_flag(Flag::Sign, result & 0x80 != 0); // last bit check
                self.regs.set_flag(Flag::F3, result & 0b1000 != 0); // 3 bit
                self.regs.set_flag(Flag::F5, result & 0b100000 != 0); // 5 bit
                // ------------
                //  write data
                // ------------
                match operand {
                    LoadOperand8::Indirect(addr) => {
                        bus.write(addr, result);
                    }
                    LoadOperand8::Reg(reg) => {
                        self.regs.set_reg_8(reg, result);
                    }
                };
            }
            // [0b00yyy110] instruction group (LD R, N 8 bit) :
            // 0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E
            U2::N0 if opcode.z == U3::N6 => {
                let operand = if let Some(reg) = RegName8::from_u3(opcode.y) {
                    // Direct LD R, N
                    LoadOperand8::Reg(reg.with_prefix(prefix))
                } else {
                    // INDIRECT LD (HL/IX+d/IY+d), N <PREFIX>[0b00110110] : 0x36
                    if prefix == Prefix::None {
                        // LD (HL)
                        LoadOperand8::Indirect(self.regs.get_reg_16(RegName16::HL))
                    } else {
                        // LD (IX+d/ IY+d)
                        let d = self.rom_next_byte(bus) as i8;
                        LoadOperand8::Indirect(word_displacement(
                            self.regs.get_reg_16(RegName16::HL.with_prefix(prefix)),d))
                    }
                };
                // Read const operand
                let data = self.rom_next_byte(bus);
                // write to bus or reg
                match operand {
                    LoadOperand8::Indirect(addr) => {
                        bus.write(addr, data);
                    }
                    LoadOperand8::Reg(reg) => {
                        self.regs.set_reg_8(reg, data);
                    }
                };
            }
            // [0b00yyy111] instruction group (Assorted)
            U2::N0 => {
                match opcode.y {
                    // RLCA ; Rotate left; msb will become lsb; carry = msb
                    // [0b00000111] : 0x07
                    U3::N0 => {
                        let mut data = self.regs.get_acc();
                        let carry = (data & 0x80) != 0;
                        data = data.wrapping_shl(1);
                        if carry {
                            data |= 1;
                        } else {
                            data &= 0xFE;
                        };
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, carry);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(data);
                    }
                    // RRCA ; Rotate right; lsb will become msb; carry = lsb
                    // [0b00001111] : 0x0F
                    U3::N1 => {
                        let mut data = self.regs.get_acc();
                        let carry = (data & 0x01) != 0;
                        data = data.wrapping_shr(1);
                        if carry {
                            data |= 0x80;
                        } else {
                            data &= 0x7F;
                        };
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, carry);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(data);
                    }
                    // RLA Rotate left trough carry
                    // [0b00010111]: 0x17
                    U3::N2 => {
                        let mut data = self.regs.get_acc();
                        let carry = (data & 0x80) != 0;
                        data = data.wrapping_shl(1);
                        if self.regs.get_flag(Flag::Carry) {
                            data |= 1;
                        } else {
                            data &= 0xFE;
                        };
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, carry);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(data);
                    }
                    // RRA Rotate right trough carry
                    // [0b00011111] : 0x1F
                    U3::N3 => {
                        let before = self.regs.get_acc();
                        let mut data = self.regs.get_acc();
                        let carry = (data & 0x01) != 0;
                        data = data.wrapping_shr(1);
                        if self.regs.get_flag(Flag::Carry) {
                            data |= 0x80;
                        } else {
                            data &= 0x7F;
                        };
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, carry);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(data);
                        println!("RRA BEGORE {:#0b} AFTER {:#0b} CARRY {}",
                                 before,
                                 data,
                                 carry);
                    }
                    // DAA [0b00100111] [link to the algorithm in header]
                    U3::N4 => {
                        let acc = self.regs.get_acc();
                        let mut correction;
                        if (acc > 0x99) || self.regs.get_flag(Flag::Carry) {
                            correction = 0x60_u8;
                            self.regs.set_flag(Flag::Carry, true);
                        } else {
                            correction = 0x00_u8;
                            self.regs.set_flag(Flag::Carry, false);
                        };
                        if ((acc & 0x0F) > 0x09) || self.regs.get_flag(Flag::HalfCarry) {
                            correction |= 0x06;
                        }
                        let acc_new = if !self.regs.get_flag(Flag::Sub) {
                            self.regs.set_flag(Flag::HalfCarry, half_carry_8(acc, correction));
                            acc.wrapping_add(correction)
                        } else {
                            self.regs.set_flag(Flag::HalfCarry, half_borrow_8(acc, correction));
                            acc.wrapping_sub(correction)
                        };
                        self.regs.set_flag(Flag::Sign, acc_new & 0x80 != 0); // Sign
                        self.regs.set_flag(Flag::Zero, acc_new == 0); // Zero
                        self.regs.set_flag(Flag::ParityOveflow,
                                           tables::PARITY_BIT[acc_new as usize] != 0);
                        self.regs.set_flag(Flag::F3, acc_new & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, acc_new & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(acc_new);
                    }
                    // CPL Invert (Complement)
                    // [0b00101111] : 0x2F
                    U3::N5 => {
                        let data = !self.regs.get_acc();
                        self.regs.set_flag(Flag::HalfCarry, true);
                        self.regs.set_flag(Flag::Sub, true);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(data);
                    }
                    // SCF  Set carry flag
                    // [0b00110111] : 0x37
                    U3::N6 => {
                        let data = self.regs.get_acc();
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, true);
                    }
                    // CCF Invert carry flag
                    // [0b00111111] : 0x3F
                    U3::N7 => {
                        let data = self.regs.get_acc();
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        let carry = self.regs.get_flag(Flag::Carry);
                        self.regs.set_flag(Flag::HalfCarry, carry);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, !carry);
                    }
                }
            }
            // HALT
            // [0b01110110] : 0x76
            U2::N1 if (opcode.z == U3::N6) && (opcode.y == U3::N6) => {
                self.halted = true;
            }
            // ---------------------------------
            // [0b01yyyzzz] instruction section
            // ---------------------------------
            // LD r[y], r[z]
            // [0b01yyyzzz]: 0x40...0x7F
            U2::N1 => {
                // LD r[y], r[z] without indirection
                if (opcode.z != U3::N6) && (opcode.y != U3::N6) {
                    let from = RegName8::from_u3(opcode.z).unwrap().with_prefix(prefix);
                    let to = RegName8::from_u3(opcode.y).unwrap().with_prefix(prefix);
                    let tmp = self.regs.get_reg_8(from);
                    self.regs.set_reg_8(to, tmp);
                } else {
                    // LD (HL/IX+d/IY+d), r ; LD r, (HL/IX+d/IY+d)
                    // 0x01110zzz; 0x01yyy110
                    let from = if let Some(reg) = RegName8::from_u3(opcode.z) {
                        // H/L is not affected by prefix if already indirection
                        LoadOperand8::Reg(reg)
                    } else {
                        if prefix == Prefix::None {
                            LoadOperand8::Indirect(self.regs.get_reg_16(RegName16::HL))
                        } else {
                            let d = self.rom_next_byte(bus) as i8;
                            LoadOperand8::Indirect(word_displacement(self.regs.get_reg_16(
                                RegName16::HL.with_prefix(prefix)), d))
                        }
                    };
                    let to = if let Some(reg) = RegName8::from_u3(opcode.y) {
                        // H/L is not affected by prefix if already indirection
                        LoadOperand8::Reg(reg)
                    } else {
                        if prefix == Prefix::None {
                            LoadOperand8::Indirect(self.regs.get_reg_16(RegName16::HL))
                        } else {
                            let d = self.rom_next_byte(bus) as i8;
                            LoadOperand8::Indirect(word_displacement(self.regs.get_reg_16(
                                RegName16::HL.with_prefix(prefix)), d))
                        }
                    };
                    let data = match from {
                        LoadOperand8::Indirect(addr) => bus.read(addr),
                        LoadOperand8::Reg(reg) => self.regs.get_reg_8(reg),
                    };
                    match to {
                        LoadOperand8::Indirect(addr) => {
                            bus.write(addr, data);
                        }
                        LoadOperand8::Reg(reg) => {
                            self.regs.set_reg_8(reg, data);
                        }
                    };
                }
            }
            // ---------------------------------
            // [0b10yyyzzz] instruction section
            // ---------------------------------
            // alu[y], operand[z-based]; 0x80...0xBF
            U2::N2 => {
                let operand = if let Some(reg) = RegName8::from_u3(opcode.z) {
                    // alu[y] reg
                    self.regs.get_reg_8(reg.with_prefix(prefix))
                } else {
                    // alu[y] (HL/IX+d/IY+d)
                    if prefix == Prefix::None {
                        bus.read(self.regs.get_reg_16(RegName16::HL))
                    } else {
                        let d = self.rom_next_byte(bus) as i8;
                        let addr = self.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                        bus.read(word_displacement(addr, d))
                    }
                };
                self.execute_alu_8(opcode.y, operand);
            }
            // ---------------------------------
            // [0b11yyyzzz] instruction section
            // ---------------------------------
            // RET cc[y]
            // [0b11yyy000] : C0; C8; D0; D8; E0; E8; F0; F8;
            U2::N3 if opcode.z == U3::N0 => {
                if self.regs.eval_condition(Condition::from_u3(opcode.y)) {
                    // write value from stack to pc
                    self.execute_pop_16(bus, RegName16::PC);
                    clocks += 11;
                } else {
                    clocks += 5;
                };
            }
            // [0b11ppq001] instruction group
            U2::N3 if opcode.z == U3::N1 => {
                match opcode.q {
                    // POP (AF/BC/DE/HL/IX/IY) ; pop 16 bit register featuring A
                    // [0b11pp0001]: C1; D1; E1; F1;
                    U1::N0 => {
                        self.execute_pop_16(bus,
                                            RegName16::from_u2_af(opcode.p).with_prefix(prefix));
                    }
                    // [0b11pp1001] instruction group (assorted)
                    U1::N1 => {
                        match opcode.p {
                            // RET ; return
                            // [0b11001001] : C9;
                            U2::N0 => {
                                self.execute_pop_16(bus, RegName16::PC);
                            }
                            // EXX
                            // [0b11011001] : D9;
                            U2::N1 => {
                                self.regs.exx();
                            }
                            // JP HL/IX/IY
                            // [0b11101001] : E9
                            U2::N2 => {
                                let addr = self.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                                self.regs.set_pc(addr);
                            }
                            // LD SP, HL/IX/IY
                            // [0b11111001] : F9
                            U2::N3 => {
                                let data = self.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                                self.regs.set_sp(data);
                            }
                        }
                    }
                };
            }
            // JP cc[y], nn [timings is set to 10 anyway as showed in Z80 instruction!]
            // [0b11yyy010]: C2,CA,D2,DA,E2,EA,F2,FA
            // NOTE: Maybe timings incorrect
            U2::N3 if opcode.z == U3::N2 => {
                let addr = self.rom_next_word(bus);
                if self.regs.eval_condition(Condition::from_u3(opcode.y)) {
                    self.regs.set_pc(addr);
                };
            }
            // [0b11yyy011] instruction group (assorted)
            U2::N3 if opcode.z == U3::N3 => {
                match opcode.y {
                    // JP nn
                    // [0b11000011]: C3
                    U3::N0 => {
                        let addr = self.rom_next_word(bus);
                        self.regs.set_pc(addr);
                    }
                    // CB prefix
                    U3::N1 => {
                        panic!("CB prefix passed as non-prefixed instruction");
                    }
                    // OUT (n), A
                    // [0b11010011] : D3
                    U3::N2 => {
                        let data = self.rom_next_byte(bus);
                        let acc = self.regs.get_acc();
                        // write Acc to port A*256 + operand
                        bus.write_io(((acc as u16) << 8) | data as u16, acc);
                    }
                    // IN A, (n)
                    // [0b11011011] : DB
                    U3::N3 => {
                        let data = self.rom_next_byte(bus);
                        let acc = self.regs.get_acc();
                        // read from port A*256 + operand to Acc
                        self.regs.set_acc(bus.read_io(((acc as u16) << 8) | data as u16));
                    }
                    // EX (SP), HL/IX/IY
                    // [0b11100011] : E3
                    U3::N4 => {
                        let reg = RegName16::HL.with_prefix(prefix);
                        let addr = self.regs.get_sp();
                        let tmp = bus.read_word(addr);
                        bus.write_word(addr, self.regs.get_reg_16(reg));
                        self.regs.set_reg_16(reg, tmp);
                    }
                    // EX DE, HL
                    // [0b11101011]
                    U3::N5 => {
                        let de = self.regs.get_reg_16(RegName16::DE);
                        let hl = self.regs.get_reg_16(RegName16::HL);
                        self.regs.set_reg_16(RegName16::DE, hl);
                        self.regs.set_reg_16(RegName16::HL, de);
                    }
                    // DI
                    // [0b11110011] : F3
                    U3::N6 => {
                        // skip interrupt check and reset flip-flops
                        self.skip_interrupt = true;
                        self.regs.set_iff1(false);
                        self.regs.set_iff2(false);
                    }
                    // EI
                    // [0b11111011] : FB
                    U3::N7 => {
                        // skip interrupt check and set flip-flops
                        self.skip_interrupt = true;
                        self.regs.set_iff1(true);
                        self.regs.set_iff2(true);
                    }
                }
            }
            // CALL cc[y], nn
            // [0b11ccc100] : C4; CC; D4; DC; E4; EC; F4; FC
            U2::N3 if opcode.z == U3::N4 => {
                let addr = self.rom_next_word(bus);
                if self.regs.eval_condition(Condition::from_u3(opcode.y)) {
                    self.execute_push_16(bus, RegName16::PC);
                    self.regs.set_reg_16(RegName16::PC, addr);
                    clocks += 5;
                } else {
                    clocks += 3;
                };
            }
            //  [0b11ppq101] opcodes group : PUSH rp2[p], CALL nn
            U2::N3 if opcode.z == U3::N5 => {
                match opcode.q {
                    // PUSH rp2[p]
                    // [0b11pp0101] : C5; D5; E5; F5;
                    U1::N0 => {
                        self.execute_push_16(bus,
                                             RegName16::from_u2_af(opcode.p).with_prefix(prefix));
                    }
                    U1::N1 => {
                        match opcode.p {
                            // CALL nn
                            // [0b11001101] : CD
                            U2::N0 => {
                                let addr = self.rom_next_word(bus);
                                self.execute_push_16(bus, RegName16::PC);
                                self.regs.set_reg_16(RegName16::PC, addr);
                            }
                            // [0b11011101] : DD
                            U2::N1 => {
                                panic!("DD prefix passed as non-prefixed instruction");
                            }
                            // [0b11101101] : ED
                            U2::N2 => {
                                panic!("ED prefix passed as non-prefixed instruction");
                            }
                            // [0b11111101] : FD
                            U2::N3 => {
                                panic!("FD prefix passed as non-prefixed instruction");
                            }
                        }
                    }
                }
            }
            // alu[y] NN
            // [0b11yyy110] : C6; CE; D6; DE; E6; EE; F6; FE
            U2::N3 if opcode.z == U3::N6 => {
                let operand = self.rom_next_byte(bus);
                self.execute_alu_8(opcode.y, operand);
            }
            // RST y*8
            // [0b11yyy111]
            U2::N3 => {
                self.execute_push_16(bus, RegName16::PC);
                // CALL y*8
                self.regs.set_reg_16(RegName16::PC, (opcode.y.as_byte() as u16) << 3);
            }
        };
        if prefix == Prefix::None {
            clocks += tables::CLOCKS_NORMAL[opcode.byte as usize];
            self.regs.inc_r(1); // single inc
        } else {
            clocks += tables::CLOCKS_DD_FD[opcode.byte as usize];
            self.regs.inc_r(2); // DD or FD prefix double inc R reg
        };

        // NOTE: DEBUG
        print!("Opcode {:#X} executed in {} clocks", opcode.byte, clocks);
        print!("{}", self.regs);
        // DEBUG

        ExecResult::Executed(clocks)
    }

    /// Instruction group which operatis with bits
    /// Includes rotations, setting, reseting, testing.
    /// covers CB, DDCB and FDCB execution group
    /// `prefix` param stands for first byte in double-prefixed instructions
    fn execute_bits(&mut self, bus: &mut Z80Bus, opcode: Opcode, prefix: Prefix) -> ExecResult {
        // at first = check prefix. if exists - swap opcode and displacement.
        // this must be happened because DDCB/FDCB instructions looks like
        // DD CB displacement opcode
        let displacement;
        let opcode = if prefix != Prefix::None {
            displacement = opcode.byte as i8;
            Opcode::from_byte(self.rom_next_byte(bus))
        } else {
            displacement = 0i8;
            opcode
        };

        let mut clocks = 0;
        // determinate data to rotate
        // (HL) selected if z is 6 in non-prefixed or if opcode is prefixed
        let operand = if (opcode.z == U3::N6) | (prefix != Prefix::None) {
            // of non prefixed, reg will become HL, else prefix corrects if to IX or IY
            let reg = RegName16::HL.with_prefix(prefix);
            // displacement will be equal zero if prefix isn't set, so next code is ok
            let addr = word_displacement(self.regs.get_reg_16(reg), displacement);
            RotOperand8::Indirect(addr)
        } else {
            // opcode.z will never be 6 at this moment, so unwrap
            RotOperand8::Reg(RegName8::from_u3(opcode.z).unwrap())
        };
        // if opcode is prefixed and z != 6 then we must copy
        // result to register (B, C, D, E, F, H, L, A), selected by z
        let copy_reg = if (opcode.z != U3::N6) & (prefix != Prefix::None) {
            Some(RegName8::from_u3(opcode.z).unwrap())
        } else {
            None
        };

        // valiable to store result of next computations,
        // used in DDCB, FDCB opcodes for result store
        let result;
        // parse opcode
        match opcode.x {
            // Rotate group. 0x00...0x3F
            U2::N0 => {
                result = self.execute_rot(bus, opcode.y, operand);
            }
            // Bit test, set, reset group
            U2::N1 | U2::N2 | U2::N3 => {
                // get bit number and data byte
                let bit_number = opcode.y.as_byte();
                let data = match operand {
                    RotOperand8::Indirect(addr) => {
                        bus.read(addr)
                    }
                    RotOperand8::Reg(reg) => {
                        self.regs.get_reg_8(reg)
                    }
                };
                match opcode.x {
                    // BIT y, r[z]
                    // [0b01yyyzzz] : 0x40...0x7F
                    U2::N1 => {
                        let bit_is_set = data & (0x01 << bit_number) == 0;
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::HalfCarry, true);
                        self.regs.set_flag(Flag::Zero, bit_is_set);
                        self.regs.set_flag(Flag::ParityOveflow, bit_is_set);
                        self.regs.set_flag(Flag::Sign, bit_is_set && (bit_number == 7));
                        if let RotOperand8::Indirect(addr) = operand {
                            // copy of high address byte 3 and 5 bits
                            self.regs.set_flag(Flag::F3, addr & 0x0800 != 0);
                            self.regs.set_flag(Flag::F5, addr & 0x2000 != 0);
                        } else {
                            // wierd rules
                            self.regs.set_flag(Flag::F3, bit_is_set && (bit_number == 3));
                            self.regs.set_flag(Flag::F5, bit_is_set && (bit_number == 5));
                        };
                        result = 0; // mask compiler error
                    }
                    // RES y, r[z]
                    // [0b10yyyzzz] : 0x80...0xBF
                    U2::N2 => {
                        result = data & (!(0x01 << bit_number));
                        match operand {
                            RotOperand8::Indirect(addr) => {
                                bus.write(addr, result);
                            }
                            RotOperand8::Reg(reg) => {
                                self.regs.set_reg_8(reg, result);
                            }
                        };
                    }
                    // SET y, r[z]
                    // [0b01yyyzzz] : 0xC0...0xFF
                    U2::N3 => {
                        result = data | (0x01 << bit_number);
                        match operand {
                            RotOperand8::Indirect(addr) => {
                                bus.write(addr, result);
                            }
                            RotOperand8::Reg(reg) => {
                                self.regs.set_reg_8(reg, result);
                            }
                        };
                    }
                    _ => unreachable!()
                }
            }
        };
        // if result must be copied
        if let Some(reg) = copy_reg {
            // if operation is not BIT
            if opcode.x != U2::N1 {
                self.regs.set_reg_8(reg, result);
            };
        };
        if prefix == Prefix::None {
            clocks += tables::CLOCKS_CB[opcode.byte as usize];
        } else {
            clocks += tables::CLOCKS_DDCB_FDCB[opcode.byte as usize];
        };
        self.regs.inc_r(2); // DDCB,FDCB or CB prefix double inc R reg (yes, wierd enough)

        // NOTE: DEBUG
        print!("Opcode {:#X} executed in {} clocks", opcode.byte, clocks);
        print!("{}", self.regs);
        // DEBUG

        ExecResult::Executed(clocks)
    }

    /// Extended instruction group (ED-prefixed)
    /// Operations are assorted.
    fn execute_extended(&mut self, bus: &mut Z80Bus, opcode: Opcode) -> ExecResult {
        let mut clocks = 0;

        // LD A, R; LD R, A accessing R after inc
        self.regs.inc_r(2);

        match opcode.x {
            U2::N0 | U2::N3 => {
                // Nothing. Just nothung. Invalid opcodes.
                // But timings in table still exsist, all ok.
            }
            // ---------------------------------
            // [0b01yyyzzz] instruction section
            // ---------------------------------
            // Assorted operations
            U2::N1 => {
                match opcode.z {
                    // IN
                    // [0b01yyy000] : 40 48 50 58 60 68 70 78
                    U3::N0 => {
                        // option, if y == 6 then reg = None
                        let reg = RegName8::from_u3(opcode.y);
                        // put BC on bus (this how Z80 acts on real HW) and get io data
                        let data = bus.read_io(self.regs.get_bc());
                        if let Some(reg) = reg {
                            self.regs.set_reg_8(reg, data);
                        };
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0);
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0);
                        self.regs.set_flag(Flag::Zero, data == 0);
                        self.regs.set_flag(Flag::Sign, data & 0x80 != 0);
                        self.regs.set_flag(Flag::ParityOveflow,
                                           tables::PARITY_BIT[data as usize] != 0);
                    }
                    // OUT
                    // [0b01yyy001] : 41 49 51 59 61 69 71 79
                    U3::N1 => {
                        let data = if let Some(reg) = RegName8::from_u3(opcode.y) {
                            self.regs.get_reg_8(reg)
                        } else {
                            0
                        };
                        bus.write_io(self.regs.get_bc(), data);
                    }
                    // SBC, ADC
                    U3::N2 => {
                        let prev_carry = bool_to_u8(self.regs.get_flag(Flag::Carry)) as u16;
                        let operand = self.regs.get_reg_16(RegName16::from_u2_sp(opcode.p));
                        let hl =  self.regs.get_hl();
                        let (carry, sub, pv, half_carry);
                        let result;
                        match opcode.q {
                            // SBC HL, rp[p]
                            U1::N0 => {
                                let (r_tmp, c1) = hl.overflowing_sub(operand);
                                let (r, c2) = r_tmp.overflowing_sub(prev_carry);
                                carry = c1 | c2;
                                result = r;
                                sub = true;
                                pv = check_sub_overflow_16(hl as i16, operand as i16) |
                                     check_sub_overflow_16(r_tmp as i16, prev_carry as i16);
                                half_carry = half_borrow_16(hl, operand) |
                                             half_borrow_16(r_tmp, prev_carry);
                            }
                            // ADC HL, rp[p]
                            U1::N1 => {
                                let (r_tmp, c1) = hl.overflowing_add(operand);
                                let (r, c2) = r_tmp.overflowing_add(prev_carry);
                                carry = c1 | c2;
                                result = r;
                                sub = false;
                                pv = check_add_overflow_16(hl as i16, operand as i16) |
                                     check_add_overflow_16(r_tmp as i16, prev_carry as i16);
                                half_carry = half_carry_16(hl, operand) |
                                             half_carry_16(r_tmp, prev_carry);
                            }
                        }
                        // set f3, f5, z, s
                        self.regs.set_flag(Flag::Carry, carry);
                        self.regs.set_flag(Flag::Sub, sub);
                        self.regs.set_flag(Flag::ParityOveflow, pv);
                        self.regs.set_flag(Flag::F3, result & 0b1000 != 0);
                        self.regs.set_flag(Flag::F5, result & 0b100000 != 0);
                        self.regs.set_flag(Flag::HalfCarry, half_carry);
                        self.regs.set_flag(Flag::Zero, result == 0);
                        self.regs.set_flag(Flag::Sign, result & 0x8000 != 0);
                        self.regs.set_hl(result);
                    }
                    // LD
                    U3::N3 => {
                        let addr = self.rom_next_word(bus);
                        let reg = RegName16::from_u2_sp(opcode.p);
                        match opcode.q {
                            // LD (nn), rp[p]
                            U1::N0 => {
                                bus.write_word(addr, self.regs.get_reg_16(reg));
                            }
                            // LD rp[p], (nn)
                            U1::N1 => {
                                self.regs.set_reg_16(reg, bus.read_word(addr));
                            }
                        }
                    }
                    // NEG (A = 0 - A)
                    U3::N4 => {
                        let acc = self.regs.get_acc();
                        let result = 0u8.wrapping_sub(acc);
                        self.regs.set_acc(result);
                        self.regs.set_flag(Flag::Sign, result & 0x80 != 0);
                        self.regs.set_flag(Flag::Zero, result == 0);
                        self.regs.set_flag(Flag::HalfCarry, half_borrow_8(0, acc));
                        self.regs.set_flag(Flag::ParityOveflow, acc == 0x80);
                        self.regs.set_flag(Flag::Sub, true);
                        self.regs.set_flag(Flag::Carry, acc != 0x00);
                        self.regs.set_flag(Flag::F3, result & 0b1000 != 0);
                        self.regs.set_flag(Flag::F5, result & 0b100000 != 0);
                    }
                    // RETN, RETI
                    U3::N5 => {
                        // RETN and even RETI copy iff2 into iff1
                        let iff2 = self.regs.get_iff2();
                        self.regs.set_iff1(iff2);
                        // restore PC
                        self.execute_pop_16(bus, RegName16::PC);
                        if opcode.y == U3::N1 {
                            bus.reti_signal();
                        }
                    }
                    // IM im[y]
                    U3::N6 => {
                        self.int_mode =  match opcode.y {
                            U3::N0 | U3::N1 | U3::N4 | U3::N5 => {
                                IntMode::IM0
                            }
                            U3::N2 | U3::N6 => {
                                IntMode::IM1
                            }
                            U3::N3 | U3::N7 => {
                                IntMode::IM2
                            }
                        };
                    }
                    // Assorted - LD,Rotates, Nop
                    U3::N7 => {
                        match opcode.y {
                            // LD I, A
                            U3::N0 => {
                                let acc = self.regs.get_acc();
                                self.regs.set_i(acc);
                            }
                            // LD R, A
                            U3::N1 => {
                                let acc = self.regs.get_acc();
                                self.regs.set_r(acc);
                            }
                            // LD A, I
                            U3::N2 => {
                                let i = self.regs.get_i();
                                self.regs.set_acc(i);
                            }
                            // LD A, R
                            U3::N3 => {
                                let r = self.regs.get_r();
                                self.regs.set_acc(r);
                            }
                            // RRD
                            U3::N4 => {
                                let mut acc = self.regs.get_acc();
                                let mut mem = bus.read(self.regs.get_hl());
                                // low nimble
                                let mem_lo = mem & 0x0F;
                                // mem_hi to mem_lo and clear hi nimble
                                mem = (mem >> 4) & 0x0F;
                                // acc_lo to mem_hi
                                mem = mem | ((acc << 4) & 0xF0);
                                acc = (acc & 0xF0) | mem_lo;
                                self.regs.set_acc(acc);
                                bus.write(self.regs.get_hl(), mem);
                                self.regs.set_flag(Flag::Sign, acc & 0x80 != 0);
                                self.regs.set_flag(Flag::Zero, acc == 0);
                                self.regs.set_flag(Flag::HalfCarry, false);
                                self.regs.set_flag(Flag::ParityOveflow,
                                                   tables::PARITY_BIT[acc as usize] != 0);
                                self.regs.set_flag(Flag::Sub, false);
                                self.regs.set_flag(Flag::F3, acc & 0b1000 != 0);
                                self.regs.set_flag(Flag::F5, acc & 0b100000 != 0);

                            }
                            // RLD
                            U3::N5 => {
                                let mut acc = self.regs.get_acc();
                                let mut mem = bus.read(self.regs.get_hl());
                                // low nimble
                                let acc_lo = acc & 0x0F;
                                // mem_hi to acc_lo
                                acc = (acc & 0xF0) | ((mem >> 4) & 0x0F);
                                // mem_lo to mem_hi and tmp to mem_lo
                                mem = ((mem << 4) & 0xF0) | acc_lo;
                                self.regs.set_acc(acc);
                                bus.write(self.regs.get_hl(), mem);
                                self.regs.set_flag(Flag::Sign, acc & 0x80 != 0);
                                self.regs.set_flag(Flag::Zero, acc == 0);
                                self.regs.set_flag(Flag::HalfCarry, false);
                                self.regs.set_flag(Flag::ParityOveflow,
                                                   tables::PARITY_BIT[acc as usize] != 0);
                                self.regs.set_flag(Flag::Sub, false);
                                self.regs.set_flag(Flag::F3, acc & 0b1000 != 0);
                                self.regs.set_flag(Flag::F5, acc & 0b100000 != 0);
                            }
                            // NOP
                            U3::N6 | U3::N7 => {
                                // No operation
                            }
                        }
                    }
                }
            }
            // ---------------------------------
            // [0b10yyyzzz] instruction section
            // ---------------------------------
            // Block instructions
            U2::N2 => {
                match opcode.z {
                    // LD Block group
                    U3::N0 => {
                        match opcode.y {
                            // LDI
                            U3::N4 => self.execute_ldi_ldd(bus, BlockDir::Inc),
                            // LDD
                            U3::N5 => self.execute_ldi_ldd(bus, BlockDir::Dec),
                            // LDIR
                            U3::N6 => {
                                self.execute_ldi_ldd(bus, BlockDir::Inc);
                                if self.regs.get_reg_16(RegName16::BC) != 0 {
                                    self.regs.dec_pc(2);
                                    clocks += 21;
                                } else {
                                    clocks += 16;
                                };
                            }
                            // LDDR
                            U3::N7 => {
                                self.execute_ldi_ldd(bus, BlockDir::Dec);
                                if self.regs.get_reg_16(RegName16::BC) != 0 {
                                    self.regs.dec_pc(2);
                                    clocks += 21;
                                } else {
                                    clocks += 16;
                                };
                            }
                            // No operation
                            _ => {},
                        }
                    }
                    // CP Block group
                    U3::N1 => {
                        match opcode.y {
                            // CPI
                            U3::N4 => {
                                self.execute_cpi_cpd(bus, BlockDir::Inc);
                            }
                            // CPD
                            U3::N5 => {
                                self.execute_cpi_cpd(bus, BlockDir::Dec);
                            }
                            // CPIR
                            U3::N6 => {
                                let result = self.execute_cpi_cpd(bus, BlockDir::Inc);
                                if (self.regs.get_reg_16(RegName16::BC) != 0) & (!result) {
                                    self.regs.dec_pc(2);
                                    clocks += 21;
                                } else {
                                    clocks += 16;
                                };
                            }
                            // CPDR
                            U3::N7 => {
                                let result = self.execute_cpi_cpd(bus, BlockDir::Dec);
                                if (self.regs.get_reg_16(RegName16::BC) != 0) & (!result) {
                                    self.regs.dec_pc(2);
                                    clocks += 21;
                                } else {
                                    clocks += 16;
                                };
                            }
                            // No operation
                            _ => {},
                        }
                    }
                    // IN Block group
                    U3::N2 => {
                        match opcode.y {
                            // INI
                            U3::N4 => self.execute_ini_ind(bus, BlockDir::Inc),
                            // IND
                            U3::N5 => self.execute_ini_ind(bus, BlockDir::Dec),
                            // INIR
                            U3::N6 => {
                                self.execute_ini_ind(bus, BlockDir::Inc);
                                if self.regs.get_reg_8(RegName8::B) != 0 {
                                    self.regs.dec_pc(2);
                                    clocks += 21
                                } else {
                                    clocks += 16;
                                };
                            }
                            // INDR
                            U3::N7 => {
                                self.execute_ini_ind(bus, BlockDir::Dec);
                                if self.regs.get_reg_8(RegName8::B) != 0 {
                                    self.regs.dec_pc(2);
                                    clocks += 21;
                                } else {
                                    clocks += 16;
                                };
                            }
                            // No operation
                            _ => {},
                        }
                    }
                    // Out Block group
                    U3::N3 => {
                        match opcode.y {
                            // OUTI
                            U3::N4 => self.execute_outi_outd(bus, BlockDir::Inc),
                            // OUTD
                            U3::N5 => self.execute_outi_outd(bus, BlockDir::Dec),
                            // OTIR
                            U3::N6 => {
                                self.execute_outi_outd(bus, BlockDir::Inc);
                                if self.regs.get_reg_8(RegName8::B) != 0 {
                                    self.regs.dec_pc(2);
                                    clocks += 21
                                } else {
                                    clocks += 16;
                                };
                            }
                            // OTDR
                            U3::N7 => {
                                self.execute_outi_outd(bus, BlockDir::Dec);
                                if self.regs.get_reg_8(RegName8::B) != 0 {
                                    self.regs.dec_pc(2);
                                    clocks += 21;
                                } else {
                                    clocks += 16;
                                };
                            }
                            // No operation
                            _ => {},
                        }
                    }
                    // No operation
                    _ => {},
                }
            }
        }

        clocks += tables::CLOCKS_ED[opcode.byte as usize];

        // NOTE: DEBUG
        print!("Opcode {:#X} executed in {} clocks", opcode.byte, clocks);
        print!("{}", self.regs);
        // DEBUG

        ExecResult::Executed(clocks)
    }

    /// ldi or ldd instruction
    fn execute_ldi_ldd(&mut self, bus: &mut Z80Bus, dir: BlockDir) {
        // read (HL)
        let src = bus.read(self.regs.get_hl());
        // write (HL) to (DE)
        bus.write(self.regs.get_de(), src);
        // inc or dec HL and DE
        match dir {
            BlockDir::Inc => {
                self.regs.inc_reg_16(RegName16::HL, 1);
                self.regs.inc_reg_16(RegName16::DE, 1);
            }
            BlockDir::Dec => {
                self.regs.dec_reg_16(RegName16::HL, 1);
                self.regs.dec_reg_16(RegName16::DE, 1);
            }
        }
        // dec BC
        let bc = self.regs.dec_reg_16(RegName16::BC, 1);
        // flags
        self.regs.set_flag(Flag::Sub, false);
        self.regs.set_flag(Flag::HalfCarry, false);
        self.regs.set_flag(Flag::ParityOveflow, bc != 0);
        let src_plus_a = src.wrapping_add(self.regs.get_acc());
        self.regs.set_flag(Flag::F3, (src_plus_a & 0b1000) != 0);
        self.regs.set_flag(Flag::F5, (src_plus_a & 0b10) != 0);
    }

    /// cpi or cpd instruction
    fn execute_cpi_cpd(&mut self, bus: &mut Z80Bus, dir: BlockDir) -> bool {
        // read (HL)
        let src = bus.read(self.regs.get_hl());
        // move pointer
        match dir {
            BlockDir::Inc => self.regs.inc_reg_16(RegName16::HL, 1),
            BlockDir::Dec => self.regs.dec_reg_16(RegName16::HL, 1),
        };
        // dec bc
        let bc = self.regs.dec_reg_16(RegName16::BC, 1);
        let acc = self.regs.get_acc();
        // variable to store CP (HL) subtract result
        let tmp = acc.wrapping_sub(src);
        // flags
        self.regs.set_flag(Flag::Sub, true);
        self.regs.set_flag(Flag::ParityOveflow, bc != 0);
        self.regs.set_flag(Flag::Zero, tmp == 0);
        self.regs.set_flag(Flag::Sign, (tmp & 0x80) != 0);
        let half_borrow = half_borrow_8(acc, src);
        self.regs.set_flag(Flag::HalfCarry, half_borrow);
        let tmp2 = if half_borrow {
            tmp.wrapping_sub(1)
        } else {
            tmp
        };
        self.regs.set_flag(Flag::F3, (tmp2 & 0b1000) != 0);
        self.regs.set_flag(Flag::F5, (tmp2 & 0b10) != 0);
        tmp == 0
    }

    /// ini or ind instruction
    fn execute_ini_ind(&mut self, bus: &mut Z80Bus, dir: BlockDir) {
        // get input data
        let src = bus.read_io(self.regs.get_bc());
        // write to memory
        bus.write(self.regs.get_hl(), src);
        // dec b
        let b = self.regs.dec_reg_8(RegName8::B, 1);
        // move pointer
        match dir {
            BlockDir::Inc => self.regs.inc_reg_16(RegName16::HL, 1),
            BlockDir::Dec => self.regs.dec_reg_16(RegName16::HL, 1),
        };
        // flags
        self.regs.set_flag(Flag::Zero, b == 0);
        self.regs.set_flag(Flag::Sign, (b & 0x80) != 0);
        self.regs.set_flag(Flag::F3, (b & 0b1000) != 0);
        self.regs.set_flag(Flag::F5, (b & 0b10) != 0);
        self.regs.set_flag(Flag::Sub, (src & 0x80) != 0);
        let c = self.regs.get_reg_8(RegName8::C);
        let cc = match dir {
            BlockDir::Inc => c.wrapping_add(1),
            BlockDir::Dec => c.wrapping_sub(1),
        };
        let (_, carry) = cc.overflowing_add(src);
        self.regs.set_flag(Flag::Carry, carry);
        self.regs.set_flag(Flag::HalfCarry, carry);
        // and now most hard. P/V flag :D
        // at first, build "Temp1"
        let temp1_operands = (bool_to_u8(bit(c, 1)) << 3) |
                             (bool_to_u8(bit(c, 0)) << 2) |
                             (bool_to_u8(bit(src, 1)) << 1) |
                             (bool_to_u8(bit(src, 0)));
        // obtain temp1
        let temp1 = match dir {
            BlockDir::Inc => tables::IO_INC_TEMP1[temp1_operands as usize] != 0,
            BlockDir::Dec => tables::IO_DEC_TEMP1[temp1_operands as usize] != 0,
        };
        // TODO: rewrite as table, described in Z80 Undocumended documented
        let temp2 = if (b & 0x0F) == 0 {
            (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 4) | (bit(b, 6) & (!bit(b, 5))))
        } else {
            (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 0) | (bit(b, 2) & (!bit(b, 1))))
        };
        self.regs.set_flag(Flag::ParityOveflow, temp1 ^ temp2 ^ bit(c, 2) ^ bit(src, 2));
        // Oh, this was pretty hard.
        // TODO: place pv falag detection in separate function
    }

    /// outi or outd instruction
    fn execute_outi_outd(&mut self, bus: &mut Z80Bus, dir: BlockDir) {
        // get input data
        let src = bus.read(self.regs.get_hl());
        // acсording to the official docs, b decrements before moving it to the addres bus
        // dec b
        let b = self.regs.dec_reg_8(RegName8::B, 1);
        bus.write_io(self.regs.get_bc(), src);
        // move pointer
        match dir {
            BlockDir::Inc => self.regs.inc_reg_16(RegName16::HL, 1),
            BlockDir::Dec => self.regs.dec_reg_16(RegName16::HL, 1),
        };
        // flags
        self.regs.set_flag(Flag::Zero, b == 0);
        self.regs.set_flag(Flag::Sign, (b & 0x80) != 0);
        self.regs.set_flag(Flag::F3, (b & 0b1000) != 0);
        self.regs.set_flag(Flag::F5, (b & 0b10) != 0);
        self.regs.set_flag(Flag::Sub, (src & 0x80) != 0);
        let c = self.regs.get_reg_8(RegName8::C);
        let cc = match dir {
            BlockDir::Inc => c.wrapping_add(1),
            BlockDir::Dec => c.wrapping_sub(1),
        };
        let (_, carry) = cc.overflowing_add(src);
        self.regs.set_flag(Flag::Carry, carry);
        self.regs.set_flag(Flag::HalfCarry, carry);
        // and now most hard. P/V flag :D
        // at first, build "Temp1"
        let temp1_operands = (bool_to_u8(bit(c, 1)) << 3) |
                             (bool_to_u8(bit(c, 0)) << 2) |
                             (bool_to_u8(bit(src, 1)) << 1) |
                             (bool_to_u8(bit(src, 0)));
        // obtain temp1
        let temp1 = match dir {
            BlockDir::Inc => tables::IO_INC_TEMP1[temp1_operands as usize] != 0,
            BlockDir::Dec => tables::IO_DEC_TEMP1[temp1_operands as usize] != 0,
        };
        // TODO: rewrite as table, described in Z80 Undocumended documented
        let temp2 = if (b & 0x0F) == 0 {
            (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 4) | (bit(b, 6) & (!bit(b, 5))))
        } else {
            (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 0) | (bit(b, 2) & (!bit(b, 1))))
        };
        self.regs.set_flag(Flag::ParityOveflow, temp1 ^ temp2 ^ bit(c, 2) ^ bit(src, 2));
        // Oh, this was pretty hard.
        // TODO: place pv falag detection in separate function
    }

    /// Rotate operations (RLC, RRC, RL, RR, SLA, SRA, SLL, SRL)
    /// returns result (can be useful with DDCB/FDCB instructions)
    fn execute_rot(&mut self, bus: &mut Z80Bus, rot_code: U3, operand: RotOperand8) -> u8 {
        // get byte which will be rotated
        let mut data = match operand {
            RotOperand8::Indirect(addr) => bus.read(addr),
            RotOperand8::Reg(reg) => self.regs.get_reg_8(reg),
        };
        let (sign, zero, f5, f3, half_carry, pv, sub, carry);
        match rot_code {
            // RLC
            U3::N0 => {
                // get msb
                carry = (data & 0x80) != 0;
                // shift left and clear lowerest bit
                data = (data << 1) & 0xFE;
                // set lsb if msb was set
                if carry {
                    data |= 0x01;
                };
            }
            // RRC
            U3::N1 => {
                // get lsb
                carry = (data & 0x01) != 0;
                // shift left and clear highest bit
                data = (data >> 1) & 0x7F;
                // set lsb if msb was set
                if carry {
                    data |= 0x80;
                };
            }
            // RL
            U3::N2 => {
                // get msb
                carry = (data & 0x80) != 0;
                // shift left and clear lowerest bit
                data = (data << 1) & 0xFE;
                // set lsb if msb was set
                if self.regs.get_flag(Flag::Carry) {
                    data |= 0x01;
                };
            }
            // RR
            U3::N3 => {
                // get lsb
                carry = (data & 0x01) != 0;
                // shift left and clear highest bit
                data = (data >> 1) & 0x7F;
                // set lsb if msb was set
                if self.regs.get_flag(Flag::Carry) {
                    data |= 0x80;
                };
            }
            // SLA
            U3::N4 => {
                // get msb
                carry = (data & 0x80) != 0;
                // shift left and clear lowerest bit
                data = (data << 1) & 0xFE;
            }
            // SRA
            U3::N5 => {
                // get lsb
                carry = (data & 0x01) != 0;
                // shift left and leave highest bit unchanged
                data = ((data >> 1) & 0x7F) | (data & 0x80);
            }
            // SLL
            U3::N6 => {
                // get msb
                carry = (data & 0x80) != 0;
                // shift left and set lowerest bit
                data = (data << 1) | 0x01;
            }
            // SRL
            U3::N7 => {
                // get lsb
                carry = (data & 0x01) != 0;
                // shift left and leave highest bit unchanged
                data = (data >> 1) & 0x7F;
            }
        };
        zero = data == 0;
        sign = (data & 0x80) != 0;
        half_carry = true;
        pv = tables::PARITY_BIT[data as usize] != 0;
        sub = false;
        f3 = data & 0b1000 != 0;
        f5 = data & 0b100000 != 0;
        // write result
        match operand {
            RotOperand8::Indirect(addr) => {
                bus.write(addr, data);
            }
            RotOperand8::Reg(reg) => {
                self.regs.set_reg_8(reg, data);
            }
        };
        self.regs.set_flag(Flag::Carry, carry);
        self.regs.set_flag(Flag::Sub, sub);
        self.regs.set_flag(Flag::ParityOveflow, pv);
        self.regs.set_flag(Flag::F3, f3);
        self.regs.set_flag(Flag::HalfCarry, half_carry);
        self.regs.set_flag(Flag::F5, f5);
        self.regs.set_flag(Flag::Zero, zero);
        self.regs.set_flag(Flag::Sign, sign);
        data
    }

    /// 8-bit ALU operations
    fn execute_alu_8(&mut self, alu_code: U3, operand: u8) {
        let acc = self.regs.get_acc(); // old acc
        let result;
        // all flags are changing after alu
        let (sign, zero, f5, f3, half_carry, pv, sub, carry);
        match alu_code {
            // ADD A, Operand
            U3::N0 => {
                let (r, c) = acc.overflowing_add(operand);
                result = r;
                carry = c;
                sub = false;
                pv = check_add_overflow_8(acc as i8, operand as i8);
                half_carry = half_carry_8(acc, operand);
            }
            // ADC A, Operand
            U3::N1 => {
                let prev_carry = bool_to_u8(self.regs.get_flag(Flag::Carry));
                let (r_tmp, c1) = acc.overflowing_add(operand);
                let (r, c2) = r_tmp.overflowing_add(prev_carry);
                result = r;
                carry = c1 | c2;
                sub = false;
                pv = check_add_overflow_8(acc as i8, operand as i8) |
                     check_add_overflow_8(r_tmp as i8, prev_carry as i8);
                half_carry = half_carry_8(acc, operand) | half_carry_8(r_tmp, prev_carry);
            }
            // SUB A, Operand
            U3::N2 | U3::N7 => {
                let (r, c) = acc.overflowing_sub(operand);
                result = r;
                carry = c;
                sub = true;
                pv = check_sub_overflow_8(acc as i8, operand as i8);
                half_carry = half_borrow_8(acc, operand);
            }
            // SBC A, Operand; CP A, Operand
            U3::N3 => {
                let prev_carry = bool_to_u8(self.regs.get_flag(Flag::Carry));
                let (r_tmp, c1) = acc.overflowing_sub(operand);
                let (r, c2) = r_tmp.overflowing_sub(prev_carry);
                result = r;
                carry = c1 | c2;
                sub = true;
                pv = check_sub_overflow_8(acc as i8, operand as i8) |
                     check_sub_overflow_8(r_tmp as i8, prev_carry as i8);
                half_carry = half_borrow_8(acc, operand) | half_borrow_8(r_tmp, prev_carry);
            }
            // AND A, Operand
            U3::N4 => {
                result = acc & operand;
                carry = false;
                sub = false;
                pv = tables::PARITY_BIT[result as usize] != 0;
                half_carry = true;
            }
            // XOR A, Operand
            U3::N5 => {
                result = acc ^ operand;
                carry = false;
                sub = false;
                pv = tables::PARITY_BIT[result as usize] != 0;
                half_carry = false;
            }
            // OR A, Operand
            U3::N6 => {
                result = acc | operand;
                carry = false;
                sub = false;
                pv = tables::PARITY_BIT[result as usize] != 0;
                half_carry = false;
            }
        };
        // CP, f3 and f5 from acc, else from result
        if alu_code == U3::N7 {
            f3 = acc & 0b1000 != 0;
            f5 = acc & 0b100000 != 0;
            // if CP, don't write result
        } else {
            f3 = result & 0b1000 != 0;
            f5 = result & 0b100000 != 0;
            self.regs.set_acc(result);
        };
        zero = result == 0;
        sign = (result & 0x80) != 0;
        self.regs.set_flag(Flag::Carry, carry);
        self.regs.set_flag(Flag::Sub, sub);
        self.regs.set_flag(Flag::ParityOveflow, pv);
        self.regs.set_flag(Flag::F3, f3);
        self.regs.set_flag(Flag::HalfCarry, half_carry);
        self.regs.set_flag(Flag::F5, f5);
        self.regs.set_flag(Flag::Zero, zero);
        self.regs.set_flag(Flag::Sign, sign);
    }

    // push 16 bit value to stack
    fn execute_push_16(&mut self, bus: &mut Z80Bus, reg: RegName16) {
        let data = self.regs.get_reg_16(reg);
        bus.write_word(self.regs.dec_sp(2), data);
    }
    // pop 16 bit value from stack
    fn execute_pop_16(&mut self, bus: &mut Z80Bus, reg: RegName16) {
        let data = bus.read_word(self.regs.get_sp());
        self.regs.inc_sp(2);
        self.regs.set_reg_16(reg, data);
    }


    /// returns clocks, needed for executing NOP instruction
    fn execute_internal_nop(&mut self) -> u8 {
        self.regs.inc_r(1); // inc R register for emulating memory refresh
        tables::CLOCKS_NORMAL[0x00] // return clocks, needed for NOP
    }


    /// No operation, no interrupts
    fn execute_internal_noni(&mut self) -> u8 {
        self.skip_interrupt = true;
        self.execute_internal_nop()
    }

    /// emulation cycle, returns cycle count
    pub fn emulate(&mut self, bus: &mut Z80Bus) -> u64 {

        // cycle_counter initial value
        let mut cycle_counter = 0_u64; // self.uncounted_cycles;
        // check interrupts
        if !self.skip_interrupt {
            // TODO: implement interrupts
        } else {
            self.skip_interrupt = false;
        }
        if self.halted {
            // execute nop NOP
            return self.execute_internal_nop() as u64;
        };
        // Figure out instruction execution group:
        let byte1 = self.rom_next_byte(bus);
        let prefix_hi = Prefix::from_byte(byte1);
        // if prefix finded
        if prefix_hi != Prefix::None {
            // next byte, prefix or opcode
            let byte2 = self.rom_next_byte(bus);
            match prefix_hi {
                // may double-prefixed
                prefix_single @ Prefix::DD | prefix_single @ Prefix::FD => {
                    let prefix_lo = Prefix::from_byte(byte2);
                    // if second prefix finded
                    match prefix_lo {
                        Prefix::DD | Prefix::ED | Prefix::FD => {
                            // move back, read second prefix again on next cycle and do `noni`
                            self.regs.dec_pc(1);
                            self.execute_internal_noni();
                        }
                        Prefix::CB if prefix_hi == Prefix::DD => {
                            // DDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let ExecResult::Executed(exec_cycles) =
                                self.execute_bits(bus, opcode, Prefix::DD) {
                                cycle_counter += exec_cycles as u64;
                            };
                        }
                        Prefix::CB => {
                            // FDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let ExecResult::Executed(exec_cycles) =
                                self.execute_bits(bus, opcode, Prefix::FD) {
                                cycle_counter += exec_cycles as u64;
                            };
                        }
                        Prefix::None => {
                            // use secon byte as opcode
                            let opcode = Opcode::from_byte(byte2);
                            match prefix_single {
                                // DD prefixed
                                Prefix::DD => {
                                    if let ExecResult::Executed(exec_cycles) =
                                           self.execute_normal(bus, opcode, Prefix::DD) {
                                        cycle_counter += exec_cycles as u64;
                                    };
                                }
                                // FD prefixed
                                _ => {
                                    if let ExecResult::Executed(exec_cycles) =
                                           self.execute_normal(bus, opcode, Prefix::FD) {
                                        cycle_counter += exec_cycles as u64;
                                    };
                                }
                            };
                        }
                    };
                }
                // CB-prefixed
                Prefix::CB => {
                    // NOTE: DEBUG
                    println!("Prefix: CB");
                    let opcode = Opcode::from_byte(byte2);
                    if let ExecResult::Executed(exec_cycles) = self.execute_bits(bus, opcode,
                                                                                 Prefix::None) {
                        cycle_counter += exec_cycles as u64;
                    };
                }
                // ED-prefixed
                Prefix::ED => {
                    // NOTE: DEBUG
                    println!("Prefix: ED");
                    let opcode = Opcode::from_byte(byte2);
                    if let ExecResult::Executed(exec_cycles) = self.execute_extended(bus, opcode) {
                        cycle_counter += exec_cycles as u64;
                    };
                }
                _ => unreachable!(),
            };
        } else {
            // Non-prefixed
            let opcode = Opcode::from_byte(byte1);
            if let ExecResult::Executed(exec_cycles) = self.execute_normal(bus,
                                                                           opcode,
                                                                           Prefix::None) {
                cycle_counter += exec_cycles as u64;
            };
        };

        cycle_counter
    }
}
