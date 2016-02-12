use cpu::registers::{RegName8, RegName16, Regs, Flag};
use cpu::tables;
use utils::*;
use cpu::decoders::{ConditionDecoder, RegNameDecoder};

/// Z80 processor System bus
/// Implement it for communication with CPU.
pub trait Z80Bus {
    /// Required method for read byte from bus
    fn read(&self, addr: u16) -> u8;
    /// Required method for write byte to bus
    fn write(&mut self, addr: u16, data: u8);
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
}


/// Instruction prefix type
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prefix {
    CB,
    DD,
    ED,
    FD,
}
impl Prefix {
    /// get prefix option from byte value
    fn from_byte(data: u8) -> Option<Prefix> {
        match data {
            0xCB => Some(Prefix::CB),
            0xDD => Some(Prefix::DD),
            0xED => Some(Prefix::ED),
            0xFD => Some(Prefix::FD),
            _ => None,
        }
    }
}

/// Opcode, devided in parts
/// ```text
/// xxyyyzzz
/// xxppqzzz
/// ```
/// Used for splitting opcode byte into parts and assemble them back
#[derive(Clone, Copy)]
struct Opcode {
    x: u8,
    y: u8,
    z: u8,
    p: u8,
    q: u8,
}
impl Opcode {
    /// split opcode into parts
    fn from_byte(data: u8) -> Opcode {
        Opcode {
            x: (data >> 6) & 0b11,
            y: (data >> 3) & 0b111,
            z: data & 0b111,
            p: ((data >> 3) & 0b111) / 2,
            q: ((data >> 3) & 0b111) % 2,
        }
    }
    /// merge parts
    fn to_byte(self) -> u8 {
        (self.x << 6) | (self.y << 3) | (self.z)
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

/// Operand for some instructions
enum Operand8 {
    Indirect(u16),
    Reg(RegName8),
}

/// Modificate 8-bit register with prefix
fn reg_with_prefix_8(reg: RegName8, pref: Prefix) -> RegName8 {
    match reg {
        reg @ RegName8::H | reg @ RegName8::L => {
            match pref {
                Prefix::DD => {
                    match reg {
                        RegName8::H => RegName8::IXH,
                        RegName8::L => RegName8::IXL,
                        _ => reg,
                    }
                }
                Prefix::FD => {
                    match reg {
                        RegName8::H => RegName8::IYH,
                        RegName8::L => RegName8::IYL,
                        _ => reg,
                    }
                }
                _ => reg,
            }
        }
        _ => reg,
    }
}

/// Modificate 16-bit register with prefix
fn reg_with_prefix_16(reg: RegName16, pref: Prefix) -> RegName16 {
    match reg {
        RegName16::HL => {
            match pref {
                Prefix::DD => RegName16::IX,
                Prefix::FD => RegName16::IY,
                _ => reg,
            }
        }
        _ => reg,
    }
}

/// Z80 Processor struct
pub struct Z80 {
    /// CPU Regs struct
    regs: Regs,
    /// cycles, which were uncounted from previous emulation
    uncounted_cycles: u64,
}

impl Z80 {
    /// new cpu instance
    pub fn new() -> Z80 {
        Z80 {
            regs: Regs::new(),
            uncounted_cycles: 0,
        }
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
    fn execute_normal(&mut self,
                      bus: &mut Z80Bus,
                      opcode: Opcode,
                      prefix: Option<Prefix>)
                      -> ExecResult {
        let mut clocks = 0;
        // 2 first bits of opcode
        match opcode.x {
            // ---------------------------------
            // [0x00yyyzzz] instruction section
            // ---------------------------------
            // [0x00yyy000] instruction group (NOP, EX, DJNZ, JR)
            0 if opcode.z == 0b000 => {
                match opcode.y {
                    // NOP
                    // [0x00000000] = 0x00
                    0 => {}
                    // EX AF, AF'
                    // [0x00001000] = 0x08
                    1 => {
                        self.regs.swap_af_alt();
                    }
                    // DJNZ offset;   13/8 clocks
                    // [0x00010000] = 0x10
                    2 => {
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
                    // [0x00011000] = 0x18
                    3 => {
                        let offset = self.rom_next_byte(bus) as i8;
                        self.regs.shift_pc(offset);
                    }
                    // JR condition[y-4] displacement;
                    // NZ [0x00100000], Z [0x00101000] NC [0x00110000] C [0x00111000]
                    4...7 => {
                        // 0x20, 0x28, 0x30, 0x38
                        let offset = self.rom_next_byte(bus) as i8;
                        let condition = ConditionDecoder::condition(opcode.y - 4);
                        if self.regs.eval_condition(condition) {
                            self.regs.shift_pc(offset);
                            clocks += 12;
                        } else {
                            clocks += 7;
                        };
                    }
                    _ => unreachable!(), // y only can be in range 0...7
                };
            }
            // [0x00ppq001] instruction group (LD, ADD)
            0 if opcode.z == 1 => {
                match opcode.q {
                    // LD rp[p], nn
                    // [0x00pp0001] : 0x01, 0x11, 0x21, 0x31
                    0 => {
                        let mut reg = RegNameDecoder::reg_16_with_sp(opcode.p);
                        // mod by prefix
                        if let Some(prefix) = prefix {
                            reg = reg_with_prefix_16(reg, prefix);
                        };
                        let data = self.rom_next_word(bus);
                        self.regs.set_reg_16(reg, data);
                    }
                    // ADD HL/IX/IY, ss ; ss - 16 bit with sp set
                    // [0x00pp1001] : 0x09; 0x19; 0x29; 0x39
                    1 => {
                        let mut reg_operand = RegNameDecoder::reg_16_with_sp(opcode.p);
                        let mut reg_acc = RegName16::HL;
                        if let Some(prefix) = prefix {
                            reg_operand = reg_with_prefix_16(reg_operand, prefix);
                            reg_acc = reg_with_prefix_16(reg_acc, prefix);
                        };
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
                    _ => unreachable!(), // q is a bit (0 or 1)
                };
            }
            // [0x00ppq010] instruction group (LD INDIRECT)
            0 if opcode.z == 2 => {
                match opcode.q {
                    // LD (BC), A
                    // [0x00000010] : 0x02
                    0 if opcode.p == 0 => {
                        bus.write(self.regs.get_reg_16(RegName16::BC),
                                  self.regs.get_reg_8(RegName8::A));
                    }
                    // LD (DE), A
                    // [0x00010010] : 0x12
                    0 if opcode.p == 1 => {
                        bus.write(self.regs.get_reg_16(RegName16::DE),
                                  self.regs.get_reg_8(RegName8::A));
                    }
                    // LD (nn), HL/IX/IY
                    // [0x00100010] : 0x22
                    0 if opcode.p == 2 => {
                        let addr = self.rom_next_word(bus);
                        let reg = if let Some(prefix) = prefix {
                            reg_with_prefix_16(RegName16::HL, prefix)
                        } else {
                            RegName16::HL
                        };
                        bus.write_word(addr, self.regs.get_reg_16(reg));
                    }
                    // LD (nn), A
                    // [0x00110010] : 0x32
                    0 if opcode.p == 3 => {
                        let addr = self.rom_next_word(bus);
                        bus.write(addr, self.regs.get_reg_8(RegName8::A));
                    }
                    // LD A, (BC)
                    // [0x00001010] : 0x0A
                    1 if opcode.p == 0 => {
                        let addr = self.regs.get_reg_16(RegName16::BC);
                        self.regs.set_reg_8(RegName8::A, bus.read(addr));
                    }
                    // LD A, (DE)
                    // [0x00011010] : 0x1A
                    1 if opcode.p == 1 => {
                        let addr = self.regs.get_reg_16(RegName16::BC);
                        self.regs.set_reg_8(RegName8::A, bus.read(addr));
                    }
                    // LD HL/IX/IY, (nn)
                    // [0x00101010] : 0x2A
                    1 if opcode.p == 2 => {
                        let addr = self.rom_next_word(bus);
                        let reg = if let Some(prefix) = prefix {
                            reg_with_prefix_16(RegName16::HL, prefix)
                        } else {
                            RegName16::HL
                        };
                        self.regs.set_reg_16(reg, bus.read_word(addr));
                    }
                    // LD A, (nn)
                    // [0x00111010] : 0x3A
                    1 if opcode.p == 3 => {
                        let addr = self.rom_next_word(bus);
                        self.regs.set_reg_8(RegName8::A, bus.read(addr));
                    }
                    _ => unreachable!(), // q is a bit, p in range 0...3
                };
            }
            // [0x00ppq011] instruction group (INC, DEC)
            0 if opcode.z == 3 => {
                // get register by rp[pp]
                let mut reg = RegNameDecoder::reg_16_with_sp(opcode.p);
                if let Some(prefix) = prefix {
                    reg = reg_with_prefix_16(reg, prefix);
                };
                match opcode.q {
                    // INC BC/DE/HL/IX/IY/SP
                    // [0x00pp0011] : 0x03, 0x13, 0x23, 0x33
                    0 => {
                        self.regs.inc_reg_16(reg, 1);
                    }
                    // DEC BC/DE/HL/IX/IY/SP
                    // [0x00pp1011] : 0x03, 0x13, 0x23, 0x33
                    1 => {
                        self.regs.dec_reg_16(reg, 1);
                    }
                    _ => unreachable!(), // q is a bit
                };
            }
            // [0x00yyy100], [0x00yyy101] instruction group (INC, DEC) 8 bit
            0 if (opcode.z == 4) || (opcode.z == 5) => {
                // TODO: make use Operand8
                let mut addr = 0xFFFF;
                let mut reg = RegName8::A;
                let data;
                let result;
                // ------------
                //   get data
                // ------------
                if opcode.y == 6 {
                    // INC (HL)/(IX + d)/(IY + d), DEC (HL)/(IX + d)/(IY + d) ; INDIRECT
                    // INC [0x00110100], DEC [0x00110101] : 0x34, 0x35
                    if let Some(prefix) = prefix {
                        // we have INC/DEC (IX/IY + d)
                        let d = self.rom_next_byte(bus) as i8;
                        addr = self.regs.get_reg_16(reg_with_prefix_16(RegName16::HL, prefix));
                        // displacement with d
                        addr = word_displacement(addr, d)
                    } else {
                        // we have IND/DEC (HL)
                        addr = self.regs.get_reg_16(RegName16::HL);
                    }
                    // read data
                    data = bus.read(addr);
                } else {
                    // INC r[y], DEC y[y] ; IX and IY also used
                    // INC [0x00yyy100] : 0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x3C
                    // DEC [0x00yyy101] : 0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x3D
                    reg = RegNameDecoder::reg_8(opcode.y);
                    if let Some(prefix) = prefix {
                        // INC IXH/IXL/IYH/IYL (undocumented)
                        reg = reg_with_prefix_8(reg, prefix);
                    };
                    data = self.regs.get_reg_8(reg);
                }
                // ------------
                //   execute
                // ------------
                if opcode.z == 4 {
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
                if opcode.y == 6 {
                    bus.write(addr, result);
                } else {
                    self.regs.set_reg_8(reg, result);
                }
            }
            // [0x00yyy110] instruction group (LD 8 bit) :
            // 0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E
            // TODO: make use Operand8
            0 if opcode.z == 6 => {
                // <PREFIX>[0x110110]
                if opcode.y == 6 {
                    // indirect, LD (HL/IX+d/IY+d), nn
                    // Get address
                    let addr = if let Some(prefix) = prefix {
                        let d = self.rom_next_byte(bus) as i8;
                        word_displacement(self.regs.get_reg_16(reg_with_prefix_16(RegName16::HL,
                                                                                  prefix)),
                                          d)
                    } else {
                        // we have IND/DEC (HL)
                        self.regs.get_reg_16(RegName16::HL)
                    };
                    let data = self.rom_next_byte(bus);
                    bus.write(addr, data);
                } else {
                    // LD REG/IXL/IXH/IYH,IYL, nn
                    let mut reg = RegNameDecoder::reg_8(opcode.y);
                    if let Some(prefix) = prefix {
                        reg = reg_with_prefix_8(reg, prefix);
                    };
                    let data = self.rom_next_byte(bus);
                    self.regs.set_reg_8(reg, data);
                }
            }
            // [0x00yyy111] instruction group
            0 if opcode.z == 7 => {
                match opcode.y {
                    // RLCA ; Rotate left; msb will become lsb; carry = msb
                    // [0x00000111] : 0x07
                    0 => {
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
                    // [0x00001111] : 0x0F
                    1 => {
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
                    // [0x00010111]: 0x17
                    2 => {
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
                    // [0x00011111] : 0x1F
                    3 => {
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
                    // DAA [0x00100111] [link to the algorithm in header]
                    4 => {

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
                    // [0x00101111] : 0x2F
                    5 => {
                        let data = !self.regs.get_acc();
                        self.regs.set_flag(Flag::HalfCarry, true);
                        self.regs.set_flag(Flag::Sub, true);
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_acc(data);
                    }
                    // SCF  Set carry flag
                    // [0x00110111] : 0x37
                    6 => {
                        let data = self.regs.get_acc();
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        self.regs.set_flag(Flag::HalfCarry, false);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, true);
                    }
                    // CCF Invert carry flag
                    // [0x00111111] : 0x3F
                    7 => {
                        let data = self.regs.get_acc();
                        self.regs.set_flag(Flag::F3, data & 0b1000 != 0); // 3 bit
                        self.regs.set_flag(Flag::F5, data & 0b100000 != 0); // 5 bit
                        let carry = self.regs.get_flag(Flag::Carry);
                        self.regs.set_flag(Flag::HalfCarry, carry);
                        self.regs.set_flag(Flag::Sub, false);
                        self.regs.set_flag(Flag::Carry, !carry);
                    }
                    _ => unreachable!(),
                }
            }
            // ---------------------------------
            // [0x01yyyzzz] instruction section
            // ---------------------------------
            // LD r[y], r[z]
            // [0x01yyyzzz]: 0x40...0x7F
            1 if !((opcode.z == 6) && (opcode.y == 6)) => {
                // LD r[y], r[z] without indirection
                if (opcode.z != 6) && (opcode.y != 6) {
                    let mut from = RegNameDecoder::reg_8(opcode.z);
                    let mut to = RegNameDecoder::reg_8(opcode.y);
                    if let Some(prefix) = prefix {
                        from = reg_with_prefix_8(from, prefix);
                        to = reg_with_prefix_8(to, prefix);
                    };
                    let tmp = self.regs.get_reg_8(from);
                    self.regs.set_reg_8(to, tmp);
                } else {
                    // LD (HL/IX+d/IY+d), r ; LD r, (HL/IX+d/IY+d)
                    // 0x01110zzz; 0x01yyy110
                    let from = if opcode.z == 6 {
                        // if prefixed - add displacement to index reg
                        if let Some(prefix) = prefix {
                            let d = self.rom_next_byte(bus) as i8;
                            Operand8::Indirect(word_displacement(self.regs.get_reg_16(
                                               reg_with_prefix_16(RegName16::HL, prefix)), d))
                        } else {
                            Operand8::Indirect(self.regs.get_reg_16(RegName16::HL))
                        }
                    } else {
                        // H/L is not affected by prefix if already indirection
                        Operand8::Reg(RegNameDecoder::reg_8(opcode.z))
                    };
                    let to = if opcode.y == 6 {
                        // if prefixed - add displacement to index reg
                        if let Some(prefix) = prefix {
                            let d = self.rom_next_byte(bus) as i8;
                            Operand8::Indirect(word_displacement(self.regs.get_reg_16(
                                               reg_with_prefix_16(RegName16::HL, prefix)), d))
                        } else {
                            Operand8::Indirect(self.regs.get_reg_16(RegName16::HL))
                        }
                    } else {
                        // H/L is not affected by prefix if already indirection
                        Operand8::Reg(RegNameDecoder::reg_8(opcode.y))
                    };
                    let data = match from {
                        Operand8::Indirect(addr) => bus.read(addr),
                        Operand8::Reg(reg) => self.regs.get_reg_8(reg),
                    };
                    match to {
                        Operand8::Indirect(addr) => {
                            bus.write(addr, data);
                        },
                        Operand8::Reg(reg) => {
                            self.regs.set_reg_8(reg, data);
                        },
                    };
                }
            }
            // HALT
            // [0x01110110] : 0x76
            1 if (opcode.z == 6) && (opcode.y == 6) => {
                unimplemented!();
            }
            _ => panic!("Opcode {:#X} unimplented", opcode.to_byte()),
        };
        if let Some(_) = prefix {
            clocks += tables::CLOCKS_DD_FD[opcode.to_byte() as usize];
        } else {
            clocks += tables::CLOCKS_NORMAL[opcode.to_byte() as usize];
        };

        // DEBUG
        // print!("Opcode {:#X} executed in {} clocks",
        //        opcode.to_byte(),
        //        clocks);
        // self.regs.print();
        // DEBUG

        ExecResult::Executed(clocks)
    }


    /// emulation cycle, returns cycle count
    pub fn emulate(&mut self, bus: &mut Z80Bus) -> u64 {

        // cycle_counter initial value
        let mut cycle_counter = 0_u64; // self.uncounted_cycles;
        // just for debug at the moment
        let mut cycle_counter_2 = 0_u64; // self.uncounted_cycles;
        loop {
            // loop
            // Figure out instruction execution group:
            let byte1 = self.rom_next_byte(bus);
            // if prefix finded
            if let Some(prefix_hi) = Prefix::from_byte(byte1) {
                // next byte, prefix or opcode
                let byte2 = self.rom_next_byte(bus);
                match prefix_hi {
                    // may double-prefixed
                    prefix_single @ Prefix::DD | prefix_single @ Prefix::FD => {
                        // if second prefix finded
                        if let Some(prefix_lo) = Prefix::from_byte(byte2) {
                            match prefix_lo {
                                Prefix::DD | Prefix::ED | Prefix::FD => {
                                    // NONI (pc--),(No Operation No Interrupts).
                                    // Its interpretation is 'perform a no-operation (wait
                                    // 4 T-states) and do not allow interrupts to occur
                                    // immediately after this instruction)
                                    unimplemented!();
                                }
                                Prefix::CB if prefix_hi == Prefix::DD => {
                                    // DDCB prefixed
                                    unimplemented!();
                                }
                                Prefix::CB => {
                                    // FDCB prefixed
                                    unimplemented!();
                                }
                            }
                        } else {
                            match prefix_single {
                                Prefix::DD => {
                                    let opcode = Opcode::from_byte(byte2);
                                    if let ExecResult::Executed(exec_cycles) =
                                           self.execute_normal(bus, opcode, Some(Prefix::DD)) {
                                        cycle_counter += exec_cycles as u64;
                                    };
                                }
                                _ => {
                                    let opcode = Opcode::from_byte(byte2);
                                    if let ExecResult::Executed(exec_cycles) =
                                           self.execute_normal(bus, opcode, Some(Prefix::FD)) {
                                        cycle_counter += exec_cycles as u64;
                                    };
                                }
                            }
                        }
                    }
                    // CB-prefixed
                    Prefix::CB => {
                        unimplemented!();
                    }
                    // ED-prefixed
                    Prefix::ED => {
                        unimplemented!();
                    }
                };
            } else {
                // Non-prefixed
                let opcode = Opcode::from_byte(byte1);
                if let ExecResult::Executed(exec_cycles) = self.execute_normal(bus, opcode, None) {
                    cycle_counter += exec_cycles as u64;
                };
            };
            // contains clocks count, which was elapsed to execute instruction,
            // debug
            cycle_counter_2 += 1;
            if cycle_counter_2 > 100 {
                break;
            }
        } //loop
        cycle_counter
    }
}
