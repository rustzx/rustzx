use utils::{make_word, split_word};

/// Conditions
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

/// Z80 registers structure
pub struct Regs {
    /// program counter
    pc: u16,
    /// stack pointer
    sp: u16,
    /// index register X [Ho - Lo]
    ixh: u8,
    ixl: u8,
    /// index register Y [Ho - Lo]
    iyh: u8,
    iyl: u8,
    /// Memory refresh register
    r: u8,
    /// Interrupt Page Adress register
    i: u8,
    /// general purpose regs: [A, F, B, C, D, E, H, L]
    gp: [u8; 8],
    /// Alternative general purpose regs
    gp_alt: [u8; 8],
}

impl Regs {
    /// Constructs new Regs struct
    pub fn new() -> Regs {
        Regs {
            pc: 0,
            sp: 0,
            ixh: 0,
            ixl: 0,
            iyh: 0,
            iyl: 0,
            r: 0,
            i: 0,
            gp: [0_u8; 8],
            gp_alt: [0_u8; 8],
        }
    }

    /// returns value of 8-bit register
    pub fn get_reg_8(&self, index: RegName8) -> u8 {
        match index {
            RegName8::A => self.gp[0],
            RegName8::F => self.gp[1],
            RegName8::B => self.gp[2],
            RegName8::C => self.gp[3],
            RegName8::D => self.gp[4],
            RegName8::E => self.gp[5],
            RegName8::H => self.gp[6],
            RegName8::L => self.gp[7],
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
        let value_ref = match index {
            RegName8::A => &mut self.gp[0],
            RegName8::F => &mut self.gp[1],
            RegName8::B => &mut self.gp[2],
            RegName8::C => &mut self.gp[3],
            RegName8::D => &mut self.gp[4],
            RegName8::E => &mut self.gp[5],
            RegName8::H => &mut self.gp[6],
            RegName8::L => &mut self.gp[7],
            RegName8::IXH => &mut self.ixh,
            RegName8::IXL => &mut self.ixl,
            RegName8::IYH => &mut self.iyh,
            RegName8::IYL => &mut self.iyl,
            RegName8::I => &mut self.i,
            RegName8::R => &mut self.r,
        };
        *value_ref = value;
        value
    }

    /// returns value of 16-bit register
    pub fn get_reg_16(&self, index: RegName16) -> u16 {
        match index {
            RegName16::PC => self.pc,
            RegName16::SP => self.sp,
            RegName16::AF => make_word(self.gp[0],self.gp[1]),
            RegName16::BC => make_word(self.gp[2],self.gp[3]),
            RegName16::DE => make_word(self.gp[4],self.gp[5]),
            RegName16::HL => make_word(self.gp[6],self.gp[7]),
            RegName16::IX => make_word(self.ixh,self.ixl),
            RegName16::IY => make_word(self.iyh,self.iyl),
        }
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
            },
            RegName16::IY => {
                self.iyh = h;
                self.iyl = l;
            },
            index @ _ => {
                let shift = match index {
                    RegName16::AF => 0,
                    RegName16::BC => 2,
                    RegName16::DE => 4,
                    RegName16::HL => 6,
                    _ => unreachable!("Ureachable code!"),
                };
                self.gp[shift] = h;
                self.gp[shift + 1] = l;
            },
        };
        value
    }

    /// inc reg
    pub fn inc_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let k = self.get_reg_8(reg);
        self.set_reg_8(reg, k + value)
    }
    /// inc reg
    pub fn inc_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let k = self.get_reg_16(reg);
        self.set_reg_16(reg, k + value)
    }
    /// dec reg
    pub fn dec_reg_8(&mut self, reg: RegName8, value: u8) -> u8 {
        let k = self.get_reg_8(reg);
        self.set_reg_8(reg, k - value)
    }
    /// dec reg
    pub fn dec_reg_16(&mut self, reg: RegName16, value: u16) -> u16 {
        let k = self.get_reg_16(reg);
        self.set_reg_16(reg, k - value)
    }
    /// move pc relatively
    pub fn shift_pc(&mut self, displacement: i8) -> u16 {
        let mut k: u16 = self.get_reg_16(RegName16::PC);
        k = if displacement >= 0 {
            k + displacement as u16
        } else {
            k - displacement.abs() as u16
        };
        self.set_reg_16(RegName16::PC, k)
    }

    // swap AF with its alternative
    pub fn swap_af_alt(&mut self) {
        let (a, f) = (self.gp[0], self.gp[1]);
        self.gp[0] = self.gp_alt[0];
        self.gp[1] = self.gp_alt[1];
        self.gp_alt[0] = a;
        self.gp_alt[1] = f;
    }

    /// evalute condition on flags register
    pub fn eval_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Cary => self.gp[1] & 0b00000001 != 0,
            Condition::NonCary => self.gp[1] & 0b00000001 == 0,
            Condition::Zero => self.gp[1] & 0b01000000 != 0,
            Condition::NonZero => self.gp[1] & 0b01000000 == 0,
            Condition::SignNegative => self.gp[1] & 0b10000000 != 0,
            Condition::SignPositive => self.gp[1] & 0b10000000 == 0,
            Condition::ParityEven => self.gp[1] & 0b00000100 != 0,
            Condition::ParityOdd => self.gp[1] & 0b00000100 == 0,
        }
    }


}
