//! Module which contains Z80 registers implementation
use crate::{utils::*, z80::Prefix};
use core::fmt;

// Flag register bits

pub const FLAG_CARRY: u8 = 0b00000001;
pub const FLAG_SUB: u8 = 0b00000010;
pub const FLAG_PV: u8 = 0b00000100;
pub const FLAG_F3: u8 = 0b00001000;
pub const FLAG_HALF_CARRY: u8 = 0b00010000;
pub const FLAG_F5: u8 = 0b00100000;
pub const FLAG_ZERO: u8 = 0b01000000;
pub const FLAG_SIGN: u8 = 0b10000000;

/// Struct for handling F register flags
#[derive(Clone, Copy)]
pub enum Flag {
    Carry,
    Sub,
    ParityOveflow,
    F3,
    HalfCarry,
    F5,
    Zero,
    Sign,
}
impl Flag {
    /// Returns current flag mask
    pub fn mask(self) -> u8 {
        match self {
            Flag::Carry => FLAG_CARRY,
            Flag::Sub => FLAG_SUB,
            Flag::ParityOveflow => FLAG_PV,
            Flag::F3 => FLAG_F3,
            Flag::HalfCarry => FLAG_HALF_CARRY,
            Flag::F5 => FLAG_F5,
            Flag::Zero => FLAG_ZERO,
            Flag::Sign => FLAG_SIGN,
        }
    }
}

/// Conditions
#[derive(Clone, Copy)]
pub enum Condition {
    NonZero,
    Zero,
    NonCarry,
    Carry,
    ParityOdd,
    ParityEven,
    SignPositive,
    SignNegative,
}

impl Condition {
    /// Returns condition encoded in 3-bit value
    pub fn from_u3(code: U3) -> Condition {
        match code {
            U3::N0 => Condition::NonZero,
            U3::N1 => Condition::Zero,
            U3::N2 => Condition::NonCarry,
            U3::N3 => Condition::Carry,
            U3::N4 => Condition::ParityOdd,
            U3::N5 => Condition::ParityEven,
            U3::N6 => Condition::SignPositive,
            U3::N7 => Condition::SignNegative,
        }
    }
}

/// 8-bit register names
#[derive(Clone,Copy)]
#[rustfmt::skip]
#[allow(clippy::upper_case_acronyms)]
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

    /// Modificates 8-bit register with prefix
    pub fn with_prefix(self, pref: Prefix) -> Self {
        match self {
            reg @ RegName8::H | reg @ RegName8::L => match pref {
                Prefix::DD => match reg {
                    RegName8::H => RegName8::IXH,
                    RegName8::L => RegName8::IXL,
                    _ => reg,
                },
                Prefix::FD => match reg {
                    RegName8::H => RegName8::IYH,
                    RegName8::L => RegName8::IYL,
                    _ => reg,
                },
                _ => reg,
            },
            _ => self,
        }
    }
}

/// 16-bit register names
#[derive(Clone,Copy)]
#[rustfmt::skip]
#[allow(clippy::upper_case_acronyms)]
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

    // Modificates 16-bit register with prefix
    pub fn with_prefix(self, pref: Prefix) -> Self {
        match self {
            RegName16::HL => match pref {
                Prefix::DD => RegName16::IX,
                Prefix::FD => RegName16::IY,
                _ => self,
            },
            _ => self,
        }
    }
}

/// Z80 registers structure
#[rustfmt::skip]
#[derive(Default)]
pub struct Regs {
    // program counter
    pc: u16,
    // stack pointer
    sp: u16,
    // index register X [Ho - Lo]
    ixh: u8, ixl: u8,
    // index register Y [Ho - Lo]
    iyh: u8, iyl: u8,
    // Memory refresh register
    r: u8,
    // Interrupt Page Adress register
    i: u8,
    // interrupt flip-flops
    iff1: bool, iff2: bool,
    // general purpose regs: [A, F, B, C, D, E, H, L]
    a: u8, f: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,
    // general purpose alternative regs: [A', F', B', C', D', E', H', L']
    a_alt: u8, f_alt: u8,
    b_alt: u8, c_alt: u8,
    d_alt: u8, e_alt: u8,
    h_alt: u8, l_alt: u8,
}

impl Regs {
    // general operations, name of reg as param --------------------------------------------------

    /// Returns value of 8-bit register
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

    /// Changes value of 8-bit register
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

    /// Returns value of 16-bit register
    pub fn get_reg_16(&self, index: RegName16) -> u16 {
        match index {
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
        }
    }

    /// Changes value of 16-bit register
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

    /// Increments register 8 bit
    pub fn inc_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let data = self.get_reg_8(reg).wrapping_add(value);
        self.set_reg_8(reg, data)
    }

    /// Increments register 16 bit
    pub fn inc_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let data = self.get_reg_16(reg).wrapping_add(value);
        self.set_reg_16(reg, data)
    }

    /// Decrements register 8 bit
    pub fn dec_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let data = self.get_reg_8(reg).wrapping_sub(value);
        self.set_reg_8(reg, data)
    }

    /// Decrements register 16 bit
    pub fn dec_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let data = self.get_reg_16(reg).wrapping_sub(value);
        self.set_reg_16(reg, data)
    }

    // 16-bit individual ------------------------------------------------------------------------

    /// Returns program counter
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    /// Changes program counter
    pub fn set_pc(&mut self, value: u16) -> u16 {
        self.pc = value;
        self.pc
    }

    /// Increments program counter
    pub fn inc_pc(&mut self, value: u16) -> u16 {
        self.pc = self.pc.wrapping_add(value);
        self.pc
    }

    /// Decrements program counter
    pub fn dec_pc(&mut self, value: u16) -> u16 {
        self.pc = self.pc.wrapping_sub(value);
        self.pc
    }

    /// Shifts program counter relatively with signed displacement
    pub fn shift_pc(&mut self, displacement: i8) -> u16 {
        self.pc = word_displacement(self.pc, displacement);
        self.pc
    }

    /// Returns af
    pub fn get_af(&self) -> u16 {
        make_word(self.a, self.f)
    }

    /// Returns bc
    pub fn get_bc(&self) -> u16 {
        make_word(self.b, self.c)
    }

    /// Returns ix
    pub fn get_ix(&self) -> u16 {
        make_word(self.ixh, self.ixl)
    }

    /// Returns iy
    pub fn get_iy(&self) -> u16 {
        make_word(self.iyh, self.iyl)
    }

    /// Changes AF
    pub fn set_af(&mut self, value: u16) -> u16 {
        let (a, f) = split_word(value);
        self.a = a;
        self.f = f;
        value
    }

    /// Changes BC
    pub fn set_bc(&mut self, value: u16) -> u16 {
        let (b, c) = split_word(value);
        self.b = b;
        self.c = c;
        value
    }

    /// Returns HL
    pub fn get_hl(&self) -> u16 {
        make_word(self.h, self.l)
    }

    /// Changes HL
    pub fn set_hl(&mut self, value: u16) -> u16 {
        let (h, l) = split_word(value);
        self.h = h;
        self.l = l;
        value
    }

    /// Returns DE
    pub fn get_de(&self) -> u16 {
        make_word(self.d, self.e)
    }

    /// Changes DE
    pub fn set_de(&mut self, value: u16) -> u16 {
        let (d, e) = split_word(value);
        self.d = d;
        self.e = e;
        value
    }

    /// Changes IX
    pub fn set_ix(&mut self, value: u16) -> u16 {
        let (ixh, ixl) = split_word(value);
        self.ixh = ixh;
        self.ixl = ixl;
        value
    }

    /// Changes IY
    pub fn set_iy(&mut self, value: u16) -> u16 {
        let (iyh, iyl) = split_word(value);
        self.iyh = iyh;
        self.iyl = iyl;
        value
    }

    /// Increments stack pointer
    pub fn inc_sp(&mut self, value: u16) -> u16 {
        self.sp = self.sp.wrapping_add(value);
        self.sp
    }

    /// Decrements stack pointer
    pub fn dec_sp(&mut self, value: u16) -> u16 {
        self.sp = self.sp.wrapping_sub(value);
        self.sp
    }

    /// Returns stack pointer
    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    /// Changes stack pointer
    pub fn set_sp(&mut self, value: u16) -> u16 {
        self.sp = value;
        self.sp
    }

    /// Returns internal IR Pair
    pub fn get_ir(&self) -> u16 {
        ((self.i as u16) << 8) | (self.r as u16)
    }

    // 8-bit individual --------------------------------------------------------------------------

    /// Returns accumulator
    pub fn get_acc(&self) -> u8 {
        self.a
    }

    /// Changes accumulator
    pub fn set_acc(&mut self, value: u8) -> u8 {
        self.a = value;
        self.a
    }

    /// Changes flags register
    pub fn set_flags(&mut self, value: u8) -> u8 {
        self.f = value;
        self.f
    }

    /// Returns F
    pub fn get_flags(&self) -> u8 {
        self.f
    }

    /// Returns I
    pub fn get_i(&self) -> u8 {
        self.i
    }

    /// Changes I
    pub fn set_i(&mut self, value: u8) -> u8 {
        self.i = value;
        self.i
    }

    /// Returns R
    pub fn get_r(&self) -> u8 {
        self.r
    }

    /// Changes R
    pub fn set_r(&mut self, value: u8) -> u8 {
        self.r = value;
        self.r
    }

    /// Special function for incrementing only lower 7 bits of `R` register
    pub fn inc_r(&mut self, value: u8) -> u8 {
        let r = self.r.wrapping_add(value) & 0x7F | self.r & 0x80;
        self.r = r;
        r
    }

    /// Returns B
    pub fn get_b(&self) -> u8 {
        self.b
    }

    /// Returns C
    pub fn get_c(&self) -> u8 {
        self.c
    }

    /// Returns H
    pub fn get_h(&self) -> u8 {
        self.h
    }

    /// Returns L
    pub fn get_l(&self) -> u8 {
        self.l
    }

    // flip-flops --------------------------------------------------------------------------------

    /// Returns iff1
    pub fn get_iff1(&self) -> bool {
        self.iff1
    }

    /// Returns iff2
    pub fn get_iff2(&self) -> bool {
        self.iff2
    }

    /// Changes iff1
    pub fn set_iff1(&mut self, value: bool) -> bool {
        self.iff1 = value;
        value
    }

    /// Changes iff2
    pub fn set_iff2(&mut self, value: bool) -> bool {
        self.iff2 = value;
        value
    }

    // swap operations ---------------------------------------------------------------------------

    // Swaps AF with its alternative
    pub fn swap_af_alt(&mut self) {
        let (a, f) = (self.a, self.f);
        self.a = self.a_alt;
        self.f = self.f_alt;
        self.a_alt = a;
        self.f_alt = f;
    }

    /// Swaps BC, DE, HL with alternatives
    #[rustfmt::skip]
    #[allow(clippy::many_single_char_names)]
    pub fn exx(&mut self) {
        let (b, c) = (self.b, self.c);
        let (d, e) = (self.d, self.e);
        let (h, l) = (self.h, self.l);
        self.b = self.b_alt; self.c = self.c_alt;
        self.d = self.d_alt; self.e = self.e_alt;
        self.h = self.h_alt; self.l = self.l_alt;
        self.b_alt = b; self.c_alt = c;
        self.d_alt = d; self.e_alt = e;
        self.h_alt = h; self.l_alt = l;
    }

    // flags operaions ---------------------------------------------------------------------------

    /// Evals condition on flags register
    pub fn eval_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Carry => (self.f & FLAG_CARRY) != 0,
            Condition::NonCarry => (self.f & FLAG_CARRY) == 0,
            Condition::Zero => (self.f & FLAG_ZERO) != 0,
            Condition::NonZero => (self.f & FLAG_ZERO) == 0,
            Condition::SignNegative => (self.f & FLAG_SIGN) != 0,
            Condition::SignPositive => (self.f & FLAG_SIGN) == 0,
            Condition::ParityEven => (self.f & FLAG_PV) != 0,
            Condition::ParityOdd => (self.f & FLAG_PV) == 0,
        }
    }

    /// Returns selected flag
    pub fn get_flag(&self, flag: Flag) -> bool {
        self.f & flag.mask() != 0
    }

    /// Changes selected flag
    pub fn set_flag(&mut self, flag: Flag, value: bool) -> bool {
        if value {
            self.f |= flag.mask(); // set bit
        } else {
            self.f &= !flag.mask(); // reset bit
        }
        value
    }
}

impl fmt::Display for Regs {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Regs:
                     \tpc: {:02X}; sp: {:02X}; i: {:02X}; r: {:02X}
                     \tix: {:02X}{:02X}; iy: {:02X}{:02X}
                     \taf: {:02X}{:02X}; bc: {:02X}{:02X}; de: {:02X}{:02X}; hl: {:02X}{:02X}
                     \t[ALT] af: {:02X}{:02X}; bc: {:02X}{:02X}; de: {:02X}{:02X}; hl: {:02X}{:02X}
                     \t flip-flops: {} {}",
                self.pc, self.sp, self.i, self.r,
                self.ixh, self.ixl, self.iyh, self.iyl,
                self.a, self.f, self.b, self.c,
                self.d, self.e, self.h, self.l,
                self.a_alt, self.f_alt, self.b_alt, self.c_alt,
                self.d_alt, self.e_alt, self.h_alt, self.l_alt,
                self.iff1, self.iff2)
    }
}
