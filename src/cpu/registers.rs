//! Module which contains structs for Z80 registers implementation/using
// TODO: switch fro Wrapping<T> to WrappingOps for primitive types, make Reg fields pub (?)
use utils::{make_word, split_word};
use std::num::Wrapping;


// TODO: Move Conditions somewhere
/// Conditions
#[derive(Clone,Copy)]
pub enum Condition {
    NonZero,
    Zero,
    NonCary,
    Cary,
    ParityOdd,
    ParityEven,
    SignPositive,
    SignNegative,
}

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
    I,
    R,
}
/// 16-bit registers names
#[derive(Clone,Copy)]
pub enum RegName16 {
    PC,
    SP,
    AF,
    BC,
    DE,
    HL,
    IX,
    IY,
}

#[derive(Clone,Copy)]
// Flag bits by name access
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
    /// get flag mask
    fn mask(self) -> u8 {
        match self {
            Flag::Carry => 0b00000001,
            Flag::Sub => 0b00000010,
            Flag::ParityOveflow => 0b00000100,
            Flag::F3 => 0b00001000,
            Flag::HalfCarry => 0b00010000,
            Flag::F5 => 0b00100000,
            Flag::Zero => 0b01000000,
            Flag::Sign => 0b10000000,
        }
    }
}


/// Z80 registers structure
pub struct Regs {
    /// program counter
    pc: Wrapping<u16>,
    /// stack pointer
    sp: Wrapping<u16>,
    /// index register X [Ho - Lo]
    ixh: Wrapping<u8>,
    ixl: Wrapping<u8>,
    /// index register Y [Ho - Lo]
    iyh: Wrapping<u8>,
    iyl: Wrapping<u8>,
    /// Memory refresh register
    r: Wrapping<u8>,
    /// Interrupt Page Adress register
    i: Wrapping<u8>,

    /// general purpose regs: [A, F, B, C, D, E, H, L]
    a: Wrapping<u8>,
    f: Wrapping<u8>,
    b: Wrapping<u8>,
    c: Wrapping<u8>,
    d: Wrapping<u8>,
    e: Wrapping<u8>,
    h: Wrapping<u8>,
    l: Wrapping<u8>,

    /// general purpose alternative regs: [A', F', B', C', D', E', H', L']
    a_alt: Wrapping<u8>,
    f_alt: Wrapping<u8>,
    b_alt: Wrapping<u8>,
    c_alt: Wrapping<u8>,
    d_alt: Wrapping<u8>,
    e_alt: Wrapping<u8>,
    h_alt: Wrapping<u8>,
    l_alt: Wrapping<u8>,
}

impl Regs {
    /// Constructs new Regs struct
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn new() -> Regs {
        Regs {
            pc: Wrapping(0), sp: Wrapping(0),
            ixh: Wrapping(0), ixl: Wrapping(0),
            iyh: Wrapping(0), iyl: Wrapping(0),
            r: Wrapping(0), i: Wrapping(0),
            a: Wrapping(0), f: Wrapping(0),
            b: Wrapping(0), c: Wrapping(0),
            d: Wrapping(0), e: Wrapping(0),
            h: Wrapping(0), l: Wrapping(0),
            a_alt: Wrapping(0), f_alt: Wrapping(0),
            b_alt: Wrapping(0), c_alt: Wrapping(0),
            d_alt: Wrapping(0), e_alt: Wrapping(0),
            h_alt: Wrapping(0), l_alt: Wrapping(0),
        }
    }

    /// returns value of 8-bit register
    pub fn get_reg_8(&self, index: RegName8) -> u8 {
        let Wrapping(value) = match index {
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
        };
        value
    }

    /// changes value of 8-bit register
    pub fn set_reg_8(&mut self, index: RegName8, value: u8) -> u8 {
        let value = Wrapping(value);
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
        value.0
    }

    /// returns value of 16-bit register
    pub fn get_reg_16(&self, index: RegName16) -> u16 {
        let Wrapping(value) = match index {
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
                    _ => unreachable!()
                };
                Wrapping(make_word(h.0, l.0))
            }
        };
        value
    }

    /// changes value of 16-bit register
    pub fn set_reg_16(&mut self, index: RegName16, value: u16) -> u16 {
        let (h, l) = split_word(value);
        let (h, l) = (Wrapping(h), Wrapping(l));
        match index {
            RegName16::PC => self.pc = Wrapping(value),
            RegName16::SP => self.sp = Wrapping(value),
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
        self.pc.0
    }

    /// increments program counter
    pub fn inc_pc(&mut self, value: u16) -> u16 {
        self.pc = self.pc + Wrapping(value);
        self.pc.0
    }

    /// get accumulator
    pub fn get_acc(&self) -> u8 {
        self.a.0
    }

    /// set accumulator
    pub fn set_acc(&mut self, value: u8) -> u8 {
        self.a = Wrapping(value);
        self.a.0
    }

    /// inc register 8 bit
    pub fn inc_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let Wrapping(new_value) = Wrapping(self.get_reg_8(reg)) + Wrapping(value);
        self.set_reg_8(reg, new_value)
    }
    /// inc register 16 bit
    pub fn inc_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let Wrapping(new_value) = Wrapping(self.get_reg_16(reg)) + Wrapping(value);
        self.set_reg_16(reg, new_value)
    }
    /// dec register 8 bit
    pub fn dec_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let Wrapping(new_value) = Wrapping(self.get_reg_8(reg)) - Wrapping(value);
        self.set_reg_8(reg, new_value)
    }
    /// dec register 16 bit
    pub fn dec_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let Wrapping(new_value) = Wrapping(self.get_reg_16(reg)) - Wrapping(value);
        self.set_reg_16(reg, new_value)
    }


    /// Shift program counter relatively with signed displacement
    pub fn shift_pc(&mut self, displacement: i8) -> u16 {
        // TODO: Rewrite with util function
        let mut k = Wrapping(self.get_reg_16(RegName16::PC));
        k = if displacement >= 0 {
            k + Wrapping(displacement as u16)
        } else {
            k - Wrapping(displacement.abs() as u16)
        };
        self.set_reg_16(RegName16::PC, k.0)
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
        let Wrapping(f) = self.f;
        match condition {
            Condition::Cary => f & 0b00000001 != 0,
            Condition::NonCary => f & 0b00000001 == 0,
            Condition::Zero => f & 0b01000000 != 0,
            Condition::NonZero => f & 0b01000000 == 0,
            Condition::SignNegative => f & 0b10000000 != 0,
            Condition::SignPositive => f & 0b10000000 == 0,
            Condition::ParityEven => f & 0b00000100 != 0,
            Condition::ParityOdd => f & 0b00000100 == 0,
        }
    }

    /// returns selected flag
    pub fn get_flag(&self, flag: Flag) -> bool {
        let Wrapping(f) = self.f;
        f & flag.mask() != 0
    }

    /// changes selected flag
    pub fn set_flag(&mut self, flag: Flag, value: bool) -> bool {
        let Wrapping(mut f) = self.f;
        f &= !flag.mask(); // clear bit
        if value {
            f |= flag.mask(); // set bit
        }
        self.f = Wrapping(f);
        value
    }

    // TODO: Rewrite as implementation of Debug trait
    /// prints full information
    pub fn print(&self) {
        println!("Regs:");
        println!("pc: {:02X}; sp: {:02X}; i: {:02X}; r: {:02X}",
            self.pc.0, self.sp.0, self.i.0, self.r.0);
        println!("ix: {:02X}{:02X}; iy: {:02X}{:02X}",
            self.ixh.0, self.ixl.0, self.iyh.0, self.iyl.0);
        println!("af: {:02X}{:02X}; bc: {:02X}{:02X}; de: {:02X}{:02X}; hl: {:02X}{:02X}",
            self.a.0, self.f.0, self.b.0, self.c.0,
            self.d.0, self.e.0, self.h.0, self.l.0);
        println!("[ALT] af: {:02X}{:02X}; bc: {:02X}{:02X}; de: {:02X}{:02X}; hl: {:02X}{:02X}",
            self.a_alt.0, self.f_alt.0, self.b_alt.0, self.c_alt.0,
            self.d_alt.0, self.e_alt.0, self.h_alt.0, self.l_alt.0);
    }
}
