use utils::*;
use super::*;
use super::opcodes::*;

/// Z80 Processor struct
pub struct Z80 {
    /// CPU Regs struct
    pub regs: Regs,
    pub halted: bool,
    pub skip_interrupt: bool,
    pub int_mode: IntMode,
    io_as_rom: bool,
    int_req: bool,
    nmi_req: bool,
}

impl Z80 {
    /// new cpu instance
    pub fn new() -> Z80 {
        Z80 {
            regs: Regs::new(),
            halted: false,
            skip_interrupt: false,
            int_mode: IntMode::IM0,
            io_as_rom: false,
            int_req: false,
            nmi_req: false,
        }
    }

    /// read byte from rom and, pc += 1
    #[inline]
    pub fn rom_next_byte(&mut self, bus: &mut Z80Bus) -> u8 {
        if self.io_as_rom {
            bus.read_interrupt()
        } else {
            let addr = self.regs.get_pc();
            self.regs.inc_pc(1);
            bus.read(addr)
        }
    }

    /// read word from rom and, pc += 2
    #[inline]
    pub fn rom_next_word(&mut self, bus: &mut Z80Bus) -> u16 {
        if self.io_as_rom {
            let (hi, lo);
            lo = bus.read_interrupt();
            hi = bus.read_interrupt();
            make_word(hi, lo)
        } else {
            let (hi, lo);
            lo = self.regs.get_pc();
            hi = self.regs.inc_pc(1);
            self.regs.inc_pc(1);
            make_word(bus.read(hi), bus.read(lo))
        }
    }

    pub fn request_interrupt(&mut self) {
        self.int_req = true;
    }

    pub fn request_nmi(&mut self) {
        self.nmi_req = true;
    }
    /// emulation cycle
    pub fn emulate(&mut self, bus: &mut Z80Bus) -> u64 {
        let mut clocks = 0u64;
        // NOTE: DEBUG
        //let mut last_pc = 0u16;
        // check interrupts
        if !self.skip_interrupt {
            // at first check nmi
            if self.nmi_req {
                // send to bus halt end message
                bus.halt(false);
                self.halted = false;
                // push pc and set pc to 0x0066
                clocks += 11;
                // reset iff1
                self.regs.set_iff1(false);
                execute_push_16(self, bus, RegName16::PC);
                self.regs.set_pc(0x0066);
                self.regs.inc_r(1);
                self.nmi_req = false;
            } else if self.int_req {
                // check flip-flop
                if self.regs.get_iff1() {
                    // then check int
                    // send to bus halt end message
                    bus.halt(false);
                    self.halted = false;
                    self.regs.inc_r(1);
                    // clear flip-flops
                    self.regs.set_iff1(false);
                    self.regs.set_iff2(false);
                    match self.int_mode {
                        // execute instruction on the bus
                        IntMode::IM0 => {
                            // instruction needs 2 more ticks
                            clocks += 2;
                            // disable nested interrupt check
                            self.skip_interrupt = true;
                            // set io as rom and execute instruction on it.
                            self.io_as_rom = true;
                            clocks +=  self.emulate(bus);
                            self.io_as_rom = false;
                        }
                        // push pc and jump to 0x0038
                        IntMode::IM1 => {
                            clocks += 13;
                            execute_push_16(self, bus, RegName16::PC);
                            self.regs.set_pc(0x0038);
                        }
                        // jump using interrupt vector
                        IntMode::IM2 => {
                            clocks += 19;
                            execute_push_16(self, bus, RegName16::PC);
                            // build interrupt vector
                            let addr = ((self.regs.get_i() as u16) << 8) + 
                                bus.read_interrupt() as u16;
                            self.regs.set_pc(addr);
                        }
                    }
                    self.int_req = false;
                }
            }
        } else {
            // allow interrupts again
            self.skip_interrupt = false;
        };
        // halt
        if self.halted {
            // execute nop NOP
            clocks += execute_internal_nop(self) as u64;
            bus.tell_clocks(clocks);
            return clocks;
        };
        //NOTE: debug
        // if last_pc == 0x12BC {
        //     println!("NOW WE IN {:#02X}", self.regs.get_pc());
        // }
        // // Figure out instruction execution group:
        // if self.regs.get_pc() == 0x12BC {
        //     println!("GOTCHA, HL : {:#02X}", self.regs.get_hl());
        //     last_pc = 0x12BC;
        // };
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
                            execute_internal_noni(self);
                        }
                        Prefix::CB if prefix_hi == Prefix::DD => {
                            // DDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let Clocks::Some(exec_clocks) =
                                execute_bits(self, bus, opcode, Prefix::DD) {
                                clocks += exec_clocks as u64;
                            };
                        }
                        Prefix::CB => {
                            // FDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let Clocks::Some(exec_clocks) =
                                execute_bits(self, bus, opcode, Prefix::FD) {
                                clocks += exec_clocks as u64;
                            };
                        }
                        Prefix::None => {
                            // use secon byte as opcode
                            let opcode = Opcode::from_byte(byte2);
                            match prefix_single {
                                // DD prefixed
                                Prefix::DD => {
                                    if let Clocks::Some(exec_clocks) =
                                           execute_normal(self, bus, opcode, Prefix::DD) {
                                        clocks += exec_clocks as u64;
                                    };
                                }
                                // FD prefixed
                                _ => {
                                    if let Clocks::Some(exec_clocks) =
                                           execute_normal(self, bus, opcode, Prefix::FD) {
                                        clocks += exec_clocks as u64;
                                    };
                                }
                            };
                        }
                    };
                }
                // CB-prefixed
                Prefix::CB => {
                    // NOTE: DEBUG
                    let opcode = Opcode::from_byte(byte2);
                    if let Clocks::Some(exec_clocks) = execute_bits(self, bus,
                                                                    opcode, Prefix::None) {
                        clocks += exec_clocks as u64;
                    };
                }
                // ED-prefixed
                Prefix::ED => {
                    // NOTE: DEBUG
                    let opcode = Opcode::from_byte(byte2);
                    if let Clocks::Some(exec_clocks) = execute_extended(self, bus, opcode) {
                        clocks += exec_clocks as u64;
                    };
                }
                _ => unreachable!(),
            };
        } else {
            // Non-prefixed
            let opcode = Opcode::from_byte(byte1);
            if let Clocks::Some(exec_clocks) = execute_normal(self, bus, opcode,
                                                              Prefix::None) {
                clocks += exec_clocks as u64;
            };
        };
        bus.tell_clocks(clocks);
        return clocks;
    }
}
