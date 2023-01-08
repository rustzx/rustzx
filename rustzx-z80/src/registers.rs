//! Module which contains Z80 registers implementation
use crate::{
    opcode::Prefix,
    smallnum::{U2, U3},
};

pub const FLAG_CARRY: u8 = 0b00000001;
pub const FLAG_SUB: u8 = 0b00000010;
pub const FLAG_PV: u8 = 0b00000100;
pub const FLAG_F3: u8 = 0b00001000;
pub const FLAG_HALF_CARRY: u8 = 0b00010000;
pub const FLAG_F5: u8 = 0b00100000;
pub const FLAG_ZERO: u8 = 0b01000000;
pub const FLAG_SIGN: u8 = 0b10000000;

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

    /// Returns 8-bit register name for prefixed opcode version
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
    MemPtr
}
impl RegName16 {
    /// Returns 16 bit general purpose register name from its code. featuring AF
    pub fn from_u2_af(byte: U2) -> RegName16 {
        match byte {
            U2::N0 => RegName16::BC,
            U2::N1 => RegName16::DE,
            U2::N2 => RegName16::HL,
            U2::N3 => RegName16::AF,
        }
    }

    /// Returns 16 bit general purpose register name from its code. featuring SP
    pub fn from_u2_sp(byte: U2) -> RegName16 {
        match byte {
            U2::N0 => RegName16::BC,
            U2::N1 => RegName16::DE,
            U2::N2 => RegName16::HL,
            U2::N3 => RegName16::SP,
        }
    }

    // Returns 16-bit register name for prefixed opcode
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

/// Z80 registers
#[rustfmt::skip]
#[derive(Default)]
pub struct Regs {
    pc: u16,
    sp: u16,
    mem_ptr: u16,
    ixh: u8, ixl: u8,
    iyh: u8, iyl: u8,
    r: u8,
    i: u8,
    iff1: bool, iff2: bool,
    a: u8, f: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,
    a_alt: u8, f_alt: u8,
    b_alt: u8, c_alt: u8,
    d_alt: u8, e_alt: u8,
    h_alt: u8, l_alt: u8,
}

impl Regs {
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

    pub fn get_reg_16(&self, index: RegName16) -> u16 {
        match index {
            RegName16::PC => self.pc,
            RegName16::SP => self.sp,
            RegName16::MemPtr => self.mem_ptr,
            _ => {
                let word_bytes_le = match index {
                    RegName16::AF => [self.f, self.a],
                    RegName16::BC => [self.c, self.b],
                    RegName16::DE => [self.e, self.d],
                    RegName16::HL => [self.l, self.h],
                    RegName16::IX => [self.ixl, self.ixh],
                    RegName16::IY => [self.iyl, self.iyh],
                    _ => unreachable!(),
                };
                u16::from_le_bytes(word_bytes_le)
            }
        }
    }

    pub fn set_reg_16(&mut self, index: RegName16, value: u16) -> u16 {
        let [l, h] = value.to_le_bytes();
        match index {
            RegName16::PC => self.pc = value,
            RegName16::SP => self.sp = value,
            RegName16::MemPtr => self.mem_ptr = value,
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

    pub fn inc_reg_8(&mut self, reg: RegName8) -> u8 {
        let data = self.get_reg_8(reg).wrapping_add(1);
        self.set_reg_8(reg, data)
    }

    pub fn inc_reg_16(&mut self, reg: RegName16) -> u16 {
        let data = self.get_reg_16(reg).wrapping_add(1);
        self.set_reg_16(reg, data)
    }

    pub fn dec_reg_8(&mut self, reg: RegName8) -> u8 {
        let data = self.get_reg_8(reg).wrapping_sub(1);
        self.set_reg_8(reg, data)
    }

    pub fn dec_reg_16(&mut self, reg: RegName16) -> u16 {
        let data = self.get_reg_16(reg).wrapping_sub(1);
        self.set_reg_16(reg, data)
    }

    pub(crate) fn get_reg_16_with_displacement(&self, reg: RegName16, displacement: i8) -> u16 {
        let word = self.get_reg_16(reg);
        (word as i32).wrapping_add(displacement as i32) as u16
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u16) -> u16 {
        self.pc = value;
        self.pc
    }

    pub fn inc_pc(&mut self) -> u16 {
        self.pc = self.pc.wrapping_add(1);
        self.pc
    }

    pub fn set_mem_ptr(&mut self, value: u16) -> u16 {
        self.mem_ptr = value;
        self.mem_ptr
    }

    pub fn get_mem_ptr(&self) -> u16 {
        self.mem_ptr
    }

    pub fn dec_pc(&mut self) -> u16 {
        self.pc = self.pc.wrapping_sub(1);
        self.pc
    }

    /// Displaces program counter with signed value
    pub fn shift_pc(&mut self, displacement: i8) -> u16 {
        self.pc = self.get_reg_16_with_displacement(RegName16::PC, displacement);
        self.pc
    }

    pub fn get_af(&self) -> u16 {
        u16::from_le_bytes([self.f, self.a])
    }

    pub fn get_bc(&self) -> u16 {
        u16::from_le_bytes([self.c, self.b])
    }

    pub fn get_ix(&self) -> u16 {
        u16::from_le_bytes([self.ixl, self.ixh])
    }

    pub fn get_iy(&self) -> u16 {
        u16::from_le_bytes([self.iyl, self.iyh])
    }

    pub fn set_af(&mut self, value: u16) -> u16 {
        let [f, a] = value.to_le_bytes();
        self.a = a;
        self.f = f;
        value
    }

    pub fn set_bc(&mut self, value: u16) -> u16 {
        let [c, b] = value.to_le_bytes();
        self.b = b;
        self.c = c;
        value
    }

    pub fn get_hl(&self) -> u16 {
        u16::from_le_bytes([self.l, self.h])
    }

    pub fn set_hl(&mut self, value: u16) -> u16 {
        let [l, h] = value.to_le_bytes();
        self.h = h;
        self.l = l;
        value
    }

    pub fn get_de(&self) -> u16 {
        u16::from_le_bytes([self.e, self.d])
    }

    pub fn set_de(&mut self, value: u16) -> u16 {
        let [e, d] = value.to_le_bytes();
        self.d = d;
        self.e = e;
        value
    }

    pub fn set_ix(&mut self, value: u16) -> u16 {
        let [ixl, ixh] = value.to_le_bytes();
        self.ixh = ixh;
        self.ixl = ixl;
        value
    }

    pub fn set_iy(&mut self, value: u16) -> u16 {
        let [iyl, iyh] = value.to_le_bytes();
        self.iyh = iyh;
        self.iyl = iyl;
        value
    }

    pub fn inc_sp(&mut self) -> u16 {
        self.sp = self.sp.wrapping_add(1);
        self.sp
    }

    pub fn dec_sp(&mut self) -> u16 {
        self.sp = self.sp.wrapping_sub(1);
        self.sp
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u16) -> u16 {
        self.sp = value;
        self.sp
    }

    pub fn get_ir(&self) -> u16 {
        ((self.i as u16) << 8) | (self.r as u16)
    }

    pub fn get_acc(&self) -> u8 {
        self.a
    }

    pub fn get_acc_alt(&self) -> u8 {
        self.a_alt
    }

    pub fn set_acc(&mut self, value: u8) -> u8 {
        self.a = value;
        self.a
    }

    pub fn set_flags(&mut self, value: u8) -> u8 {
        self.f = value;
        self.f
    }

    pub fn get_flags(&self) -> u8 {
        self.f
    }

    pub fn get_flags_alt(&self) -> u8 {
        self.f_alt
    }

    pub fn get_i(&self) -> u8 {
        self.i
    }

    pub fn set_i(&mut self, value: u8) -> u8 {
        self.i = value;
        self.i
    }

    pub fn get_r(&self) -> u8 {
        self.r
    }

    pub fn set_r(&mut self, value: u8) -> u8 {
        self.r = value;
        self.r
    }

    /// Special function for incrementing only lower 7 bits of `R` register
    pub fn inc_r(&mut self) -> u8 {
        let r = self.r.wrapping_add(1) & 0x7F | self.r & 0x80;
        self.r = r;
        r
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }

    pub fn get_c(&self) -> u8 {
        self.c
    }

    pub fn get_d(&self) -> u8 {
        self.d
    }

    pub fn get_e(&self) -> u8 {
        self.e
    }

    pub fn get_h(&self) -> u8 {
        self.h
    }

    pub fn get_l(&self) -> u8 {
        self.l
    }

    pub fn get_b_alt(&self) -> u8 {
        self.b_alt
    }

    pub fn get_c_alt(&self) -> u8 {
        self.c_alt
    }

    pub fn get_d_alt(&self) -> u8 {
        self.d_alt
    }

    pub fn get_e_alt(&self) -> u8 {
        self.e_alt
    }

    pub fn get_h_alt(&self) -> u8 {
        self.h
    }

    pub fn get_l_alt(&self) -> u8 {
        self.l
    }

    pub fn get_iff1(&self) -> bool {
        self.iff1
    }

    pub fn get_iff2(&self) -> bool {
        self.iff2
    }

    pub fn set_iff1(&mut self, value: bool) -> bool {
        self.iff1 = value;
        value
    }

    pub fn set_iff2(&mut self, value: bool) -> bool {
        self.iff2 = value;
        value
    }

    pub fn swap_af_alt(&mut self) {
        core::mem::swap(&mut self.a, &mut self.a_alt);
        core::mem::swap(&mut self.f, &mut self.f_alt);
    }

    pub fn exx(&mut self) {
        core::mem::swap(&mut self.b, &mut self.b_alt);
        core::mem::swap(&mut self.c, &mut self.c_alt);
        core::mem::swap(&mut self.d, &mut self.d_alt);
        core::mem::swap(&mut self.e, &mut self.e_alt);
        core::mem::swap(&mut self.h, &mut self.h_alt);
        core::mem::swap(&mut self.l, &mut self.l_alt);
    }
}
