//! Module which contains structs for Z80 registers implementation/using

use utils::*;
use std::fmt;
use cpu::{Condition, Flag, Prefix, U2, U3};

/// 8-bit registers names
#[derive(Clone,Copy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum RegName8 {
    A, F,
    B, C,
    D, E,
    H, L,
    IXH, IXL,
    IYH, IYL,
    I, R,
}
impl RegName8 {
    /// Returns 8 bit general purpose register name from code.
    /// # Failures
    /// Returns None if code equals `0b110` (Indirect)
    pub fn from_u3(byte: U3) -> Option<Self> {
        match byte {
            U3::N0 => Some(RegName8::B),
            U3::N1 => Some(RegName8::C),
            U3::N2 => Some(RegName8::D),
            U3::N3 => Some(RegName8::E),
            U3::N4 => Some(RegName8::H),
            U3::N5 => Some(RegName8::L),
            U3::N6 => None,
            U3::N7 => Some(RegName8::A),
        }
    }
    /// Modificate 8-bit register with prefix
    pub fn with_prefix(self, pref: Prefix) -> Self {
        match self {
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
            _ => self,
        }
    }
}
/// 16-bit registers names
#[derive(Clone,Copy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum RegName16 {
    PC, SP,
    AF, BC,
    DE, HL,
    IX, IY,
}
impl RegName16 {
    /// Returns 16 bit general purpose register name from code. featuring AF
    pub fn from_u2_af(byte: U2) -> RegName16 {
        match byte {
            U2::N0 => RegName16::BC,
            U2::N1 => RegName16::DE,
            U2::N2 => RegName16::HL,
            U2::N3 => RegName16::AF,
        }
    }
    /// Returns 16 bit general purpose register name from code. featuring SP
    pub fn from_u2_sp(byte: U2) -> RegName16 {
        match byte {
            U2::N0 => RegName16::BC,
            U2::N1 => RegName16::DE,
            U2::N2 => RegName16::HL,
            U2::N3 => RegName16::SP,
        }
    }
    // Modificate 16-bit register with prefix
    pub fn with_prefix(self, pref: Prefix) -> Self {
        match self  {
            RegName16::HL => {
                match pref {
                    Prefix::DD => RegName16::IX,
                    Prefix::FD => RegName16::IY,
                    _ => self,
                }
            }
            _ => self,
        }
    }
}

/// Z80 registers structure
#[cfg_attr(rustfmt, rustfmt_skip)]
pub struct Regs {
    /// program counter
    pc: u16,
    /// stack pointer
    sp: u16,
    /// index register X [Ho - Lo]
    ixh: u8, ixl: u8,
    /// index register Y [Ho - Lo]
    iyh: u8, iyl: u8,
    /// Memory refresh register
    r: u8,
    /// Interrupt Page Adress register
    i: u8,
    /// interrupt flip-flops
    iff1: bool, iff2: bool,
    /// general purpose regs: [A, F, B, C, D, E, H, L]
    a: u8, f: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,

    /// general purpose alternative regs: [A', F', B', C', D', E', H', L']
    a_alt: u8, f_alt: u8,
    b_alt: u8, c_alt: u8,
    d_alt: u8, e_alt: u8,
    h_alt: u8, l_alt: u8,
}

impl Regs {
    /// Constructs new Regs struct
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn new() -> Regs {
        Regs {
            pc: 0, sp: 0,
            ixh: 0, ixl: 0,
            iyh: 0, iyl: 0,
            r: 0, i: 0,
            iff1: false, iff2: false,
            a: 0, f: 0,
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            a_alt: 0, f_alt: 0,
            b_alt: 0, c_alt: 0,
            d_alt: 0, e_alt: 0,
            h_alt: 0, l_alt: 0,
        }
    }

    /// returns value of 8-bit register
    pub fn get_reg_8(&self, index: RegName8) -> u8 {
        match index {
            RegName8::A => self.a,
            RegName8::F => self.f,
            RegName8::B => self.b,
            RegName8::C => self.c,
            RegName8::D => self.d,
            RegName8::E => self.e,
            RegName8::H => self.h,
            RegName8::L => self.l,
            RegName8::IXH => self.ixh,
            RegName8::IXL => self.ixl,
            RegName8::IYH => self.iyh,
            RegName8::IYL => self.iyl,
            RegName8::I => self.i,
            RegName8::R => self.r,
        }
    }

    /// changes value of 8-bit register
    pub fn set_reg_8(&mut self, index: RegName8, value: u8) -> u8 {
        match index {
            RegName8::A => self.a = value,
            RegName8::F => self.f = value,
            RegName8::B => self.b = value,
            RegName8::C => self.c = value,
            RegName8::D => self.d = value,
            RegName8::E => self.e = value,
            RegName8::H => self.h = value,
            RegName8::L => self.l = value,
            RegName8::IXH => self.ixh = value,
            RegName8::IXL => self.ixl = value,
            RegName8::IYH => self.iyh = value,
            RegName8::IYL => self.iyl = value,
            RegName8::I => self.i = value,
            RegName8::R => self.r = value,
        };
        value
    }

    /// returns value of 16-bit register
    pub fn get_reg_16(&self, index: RegName16) -> u16 {
        let value = match index {
            RegName16::PC => self.pc,
            RegName16::SP => self.sp,
            _ => {
                let (h, l) = match index {
                    RegName16::AF => (self.a, self.f),
                    RegName16::BC => (self.b, self.c),
                    RegName16::DE => (self.d, self.e),
                    RegName16::HL => (self.h, self.l),
                    RegName16::IX => (self.ixh, self.ixl),
                    RegName16::IY => (self.iyh, self.iyl),
                    _ => unreachable!(),
                };
                make_word(h, l)
            }
        };
        value
    }

    /// changes value of 16-bit register
    pub fn set_reg_16(&mut self, index: RegName16, value: u16) -> u16 {
        let (h, l) = split_word(value);
        match index {
            RegName16::PC => self.pc = value,
            RegName16::SP => self.sp = value,
            RegName16::IX => {
                self.ixh = h;
                self.ixl = l;
            }
            RegName16::IY => {
                self.iyh = h;
                self.iyl = l;
            }
            RegName16::AF => {
                self.a = h;
                self.f = l;
            }
            RegName16::BC => {
                self.b = h;
                self.c = l;
            }
            RegName16::DE => {
                self.d = h;
                self.e = l;
            }
            RegName16::HL => {
                self.h = h;
                self.l = l;
            }
        };
        value
    }

    /// return program counter
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    /// increments program counter
    pub fn inc_pc(&mut self, value: u16) -> u16 {
        self.pc = self.pc.wrapping_add(value);
        self.pc
    }

    /// decrements program counter
    pub fn dec_pc(&mut self, value: u16) -> u16 {
        self.pc = self.pc.wrapping_sub(value);
        self.pc
    }

    /// get accumulator
    pub fn get_acc(&self) -> u8 {
        self.a
    }

    /// set accumulator
    pub fn set_acc(&mut self, value: u8) -> u8 {
        self.a = value;
        self.a
    }

    /// inc register 8 bit
    pub fn inc_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let data = self.get_reg_8(reg).wrapping_add(value);
        self.set_reg_8(reg, data)
    }
    /// inc register 16 bit
    pub fn inc_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let data =  self.get_reg_16(reg).wrapping_add(value);
        self.set_reg_16(reg, data)
    }
    /// dec register 8 bit
    pub fn dec_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let data = self.get_reg_8(reg).wrapping_sub(value);
        self.set_reg_8(reg, data)
    }
    /// dec register 16 bit
    pub fn dec_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let data = self.get_reg_16(reg).wrapping_sub(value);
        self.set_reg_16(reg, data)
    }


    /// Shift program counter relatively with signed displacement
    pub fn shift_pc(&mut self, displacement: i8) -> u16 {
        self.pc = word_displacement(self.sp, displacement);
        self.pc
    }

    /// special function for incrementing only lower 7 bits of `R` register
    pub fn inc_r(&mut self, value: u8) -> u8 {
        let r = self.r.wrapping_add(value) & 0x7F | self.r & 0x80;
        self.r = r;
        r
    }

    // swap AF with its alternative
    pub fn swap_af_alt(&mut self) {
        let (a, f) = (self.a, self.f);
        self.a = self.a_alt;
        self.f = self.f_alt;
        self.a_alt = a;
        self.f_alt = f;
    }

    /// evalute condition on flags register
    pub fn eval_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Cary => self.f & 0b00000001 != 0,
            Condition::NonCary => self.f & 0b00000001 == 0,
            Condition::Zero => self.f & 0b01000000 != 0,
            Condition::NonZero => self.f & 0b01000000 == 0,
            Condition::SignNegative => self.f & 0b10000000 != 0,
            Condition::SignPositive => self.f & 0b10000000 == 0,
            Condition::ParityEven => self.f & 0b00000100 != 0,
            Condition::ParityOdd => self.f & 0b00000100 == 0,
        }
    }

    /// returns selected flag
    pub fn get_flag(&self, flag: Flag) -> bool {
        self.f & flag.mask() != 0
    }

    /// changes selected flag
    pub fn set_flag(&mut self, flag: Flag, value: bool) -> bool {
        if value {
            self.f |= flag.mask(); // set bit
        } else {
            self.f &= !flag.mask();
        }
        value
    }

    /// returns iff1
    pub fn get_iff1(&self) -> bool {
        self.iff1
    }
    /// returns iff2
    pub fn get_iff2(&self) -> bool {
        self.iff2
    }
    /// changes iff1
    pub fn set_iff1(&mut self, value: bool) -> bool {
        self.iff1 = value;
        value
    }
    /// changes iff2
    pub fn set_iff2(&mut self, value: bool) -> bool {
        self.iff2 = value;
        value
    }
}

impl fmt::Display for Regs {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Regs:
                     \tpc: {:02X}; sp: {:02X}; i: {:02X}; r: {:02X}
                     \tix: {:02X}{:02X}; iy: {:02X}{:02X}
                     \taf: {:02X}{:02X}; bc: {:02X}{:02X}; de: {:02X}{:02X}; hl: {:02X}{:02X}
                     \t[ALT] af: {:02X}{:02X}; bc: {:02X}{:02X}; de: {:02X}{:02X}; hl: {:02X}{:02X}",
                self.pc, self.sp, self.i, self.r,
                self.ixh, self.ixl, self.iyh, self.iyl,
                self.a, self.f, self.b, self.c,
                self.d, self.e, self.h, self.l,
                self.a_alt, self.f_alt, self.b_alt, self.c_alt,
                self.d_alt, self.e_alt, self.h_alt, self.l_alt)

    }
}
