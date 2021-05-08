//! Z80 CPU module

use crate::{
    utils::*,
    z80::{opcodes::*, *},
};

/// Z80 Processor struct
pub struct Z80 {
    /// Contains Z80 registers data
    pub regs: Regs,
    /// active if Z80 waiting for interrupt
    pub halted: bool,
    /// enabled if interrupt check will be skipped nex time
    pub skip_interrupt: bool,
    /// type of interrupt
    pub int_mode: IntMode,
    active_prefix: Prefix,
}

impl Default for Z80 {
    fn default() -> Self {
        Self::new()
    }
}

impl Z80 {
    /// Returns new cpu instance
    pub fn new() -> Z80 {
        Z80 {
            regs: Regs::new(),
            halted: false,
            skip_interrupt: false,
            int_mode: IntMode::IM0,
            active_prefix: Prefix::None,
        }
    }

    /// Reads byte from memory and increments PC
    #[inline]
    pub fn fetch_byte(&mut self, bus: &mut dyn Z80Bus, clk: Clocks) -> u8 {
        let addr = self.regs.get_pc();
        self.regs.inc_pc(1);
        bus.read(addr, clk)
    }

    /// Reads word from memory and increments PC twice
    #[inline]
    pub fn fetch_word(&mut self, bus: &mut dyn Z80Bus, clk: Clocks) -> u16 {
        let (hi_addr, lo_addr);
        lo_addr = self.regs.get_pc();
        let lo = bus.read(lo_addr, clk);
        hi_addr = self.regs.inc_pc(1);
        let hi = bus.read(hi_addr, clk);
        self.regs.inc_pc(1);
        make_word(hi, lo)
    }

    /// Checks is cpu halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Returns current interrupt mode
    pub fn get_im(&self) -> IntMode {
        self.int_mode
    }

    /// Changes interrupt mode
    pub fn set_im(&mut self, value: u8) {
        assert!(value < 3);
        self.int_mode = match value {
            0 => IntMode::IM0,
            1 => IntMode::IM1,
            2 => IntMode::IM2,
            _ => unreachable!(),
        }
    }

    /// Main emulation step function
    /// return `false` if execution can be continued or true if last event must be executed
    /// instantly
    pub fn emulate(&mut self, bus: &mut dyn Z80Bus) -> bool {
        // check interrupts
        if !self.skip_interrupt {
            // at first check nmi
            if bus.nmi_active() {
                // send to bus halt end message
                if self.halted {
                    bus.halt(false);
                    self.halted = false;
                    self.regs.inc_pc(1);
                }
                // push pc and set pc to 0x0066 ( pleace PC on bus ?)
                bus.wait_loop(self.regs.get_pc(), Clocks(5));
                // reset iff1
                self.regs.set_iff1(false);
                // 3 x 2 clocks consumed
                execute_push_16(self, bus, RegName16::PC, Clocks(3));
                self.regs.set_pc(0x0066);
                self.regs.inc_r(1);
            // 5 + 3 + 3 = 11 clocks
            } else if bus.int_active() && self.regs.get_iff1() {
                // send to bus halt end message
                if self.halted {
                    bus.halt(false);
                    self.halted = false;
                    self.regs.inc_pc(1);
                }
                self.regs.inc_r(1);
                // clear flip-flops
                self.regs.set_iff1(false);
                self.regs.set_iff2(false);
                match self.int_mode {
                    // execute instruction on the bus
                    IntMode::IM0 => {
                        // for zx spectrum same as IM1
                        execute_push_16(self, bus, RegName16::PC, Clocks(3));
                        self.regs.set_pc(0x0038);
                        bus.wait_internal(Clocks(7));
                    }
                    // push pc and jump to 0x0038
                    IntMode::IM1 => {
                        execute_push_16(self, bus, RegName16::PC, Clocks(3));
                        self.regs.set_pc(0x0038);
                        bus.wait_internal(Clocks(7));
                        // 3 + 3 + 7 = 13 clocks
                    }
                    // jump using interrupt vector
                    IntMode::IM2 => {
                        execute_push_16(self, bus, RegName16::PC, Clocks(3));
                        // build interrupt vector
                        let addr = (((self.regs.get_i() as u16) << 8) & 0xFF00)
                            | ((bus.read_interrupt() as u16) & 0x00FF);
                        let addr = bus.read_word(addr, Clocks(3));
                        self.regs.set_pc(addr);
                        bus.wait_internal(Clocks(7));
                        // 3 + 3 + 3 + 3 + 7 = 19 clocks
                    }
                }
            }
        } else {
            // allow interrupts again
            self.skip_interrupt = false;
        };
        // get first byte
        let byte1 = if self.active_prefix != Prefix::None {
            let tmp = self.active_prefix.to_byte().unwrap();
            self.active_prefix = Prefix::None;
            tmp
        } else {
            self.regs.inc_r(1);
            self.fetch_byte(bus, Clocks(4))
        };
        let prefix_hi = Prefix::from_byte(byte1);
        // if prefix finded
        if prefix_hi != Prefix::None {
            match prefix_hi {
                // may double-prefixed
                prefix_single @ Prefix::DD | prefix_single @ Prefix::FD => {
                    // next byte, prefix or opcode
                    let byte2 = self.fetch_byte(bus, Clocks(4));
                    self.regs.inc_r(1);
                    let prefix_lo = Prefix::from_byte(byte2);
                    // if second prefix finded
                    match prefix_lo {
                        Prefix::DD | Prefix::ED | Prefix::FD => {
                            self.active_prefix = prefix_lo;
                            self.skip_interrupt = true;
                        }
                        Prefix::CB => {
                            // FDCB, DDCB prefixed
                            execute_bits(self, bus, prefix_single);
                        }
                        Prefix::None => {
                            // use second byte as opcode
                            let opcode = Opcode::from_byte(byte2);
                            execute_normal(self, bus, opcode, prefix_single);
                        }
                    };
                }
                // CB-prefixed
                Prefix::CB => {
                    // opcode will be read in function
                    execute_bits(self, bus, Prefix::None);
                }
                // ED-prefixed
                Prefix::ED => {
                    // next byte, prefix or opcode
                    let byte2 = self.fetch_byte(bus, Clocks(4));
                    self.regs.inc_r(1);
                    let opcode = Opcode::from_byte(byte2);
                    execute_extended(self, bus, opcode);
                }
                _ => unreachable!(),
            };
        } else {
            // Non-prefixed
            let opcode = Opcode::from_byte(byte1);
            execute_normal(self, bus, opcode, Prefix::None);
        };
        // check events on bus
        // if some events found, then signal that emulator must process events before
        // next cpu step
        bus.pc_callback(self.regs.get_pc());
        // return true if events happened
        bus.instant_event()
    }
}
