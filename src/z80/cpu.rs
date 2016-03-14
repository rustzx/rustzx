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
    /// cycles, which were uncounted from previous emulation
    cycles: u64,
    io_as_rom: bool,
}

impl Z80 {
    /// new cpu instance
    pub fn new() -> Z80 {
        Z80 {
            regs: Regs::new(),
            halted: false,
            skip_interrupt: false,
            int_mode: IntMode::IM0,
            cycles: 0,
            io_as_rom: false,
        }
    }

    /// read byte from rom and, pc += 1
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

    /// returns current cycles count
    pub fn get_cycles(&self) -> u64 {
        self.cycles
    }

    /// resets cycle counter
    pub fn reset_cycles(&mut self) {
        self.cycles = 0;
    }

    /// emulation cycle, returns cycle count
    pub fn emulate(&mut self, bus: &mut Z80Bus) -> Result<(), ()> {
        // check interrupts
        if !self.skip_interrupt {
            // at first check nmi
            if bus.nmi() {
                // send to bus halt end message
                bus.halt(false);
                // push pc and set pc to 0x0066
                self.cycles += 11;
                // reset iff1
                self.regs.set_iff1(false);
                execute_push_16(self, bus, RegName16::PC);
                self.regs.set_pc(0x0066);
                self.regs.inc_r(1);
                return Result::Ok(());
            } else if bus.int() {
                // check flip-flop
                if self.regs.get_iff1() {
                    // then check int
                    // send to bus halt end message
                    bus.halt(false);
                    self.regs.inc_r(1);
                    // clear flip-flops
                    self.regs.set_iff1(false);
                    self.regs.set_iff2(false);
                    match self.int_mode {
                        // execute instruction on the bus
                        IntMode::IM0 => {
                            // instruction needs 2 more ticks
                            self.cycles += 2;
                            // disable nested interrupt check
                            self.skip_interrupt = true;
                            // set io as rom and execute instruction on it.
                            self.io_as_rom = true;
                            let result =  self.emulate(bus);
                            self.io_as_rom = false;
                            return result;
                        }
                        // push pc and jump to 0x0038
                        IntMode::IM1 => {
                            self.cycles += 13;
                            execute_push_16(self, bus, RegName16::PC);
                            self.regs.set_pc(0x0038);
                            return Result::Ok(());
                        }
                        // jump using interrupt vector
                        IntMode::IM2 => {
                            self.cycles += 19;
                            execute_push_16(self, bus, RegName16::PC);
                            // build interrupt vector
                            let addr = make_word(bus.read_interrupt(), self.regs.get_i());
                            self.regs.set_pc(addr);
                            return Result::Ok(());
                        }
                    }
                }
            }
        } else {
            // allow interrupts again
            self.skip_interrupt = false;
        }
        // halt
        if self.halted {
            // execute nop NOP
            execute_internal_nop(self);
            return Result::Ok(());
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
                            execute_internal_noni(self);
                        }
                        Prefix::CB if prefix_hi == Prefix::DD => {
                            // DDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let Clocks::Some(exec_cycles) =
                                execute_bits(self, bus, opcode, Prefix::DD) {
                                self.cycles += exec_cycles as u64;
                            };
                        }
                        Prefix::CB => {
                            // FDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let Clocks::Some(exec_cycles) =
                                execute_bits(self, bus, opcode, Prefix::FD) {
                                self.cycles += exec_cycles as u64;
                            };
                        }
                        Prefix::None => {
                            // use secon byte as opcode
                            let opcode = Opcode::from_byte(byte2);
                            match prefix_single {
                                // DD prefixed
                                Prefix::DD => {
                                    if let Clocks::Some(exec_cycles) =
                                           execute_normal(self, bus, opcode, Prefix::DD) {
                                        self.cycles += exec_cycles as u64;
                                    };
                                }
                                // FD prefixed
                                _ => {
                                    if let Clocks::Some(exec_cycles) =
                                           execute_normal(self, bus, opcode, Prefix::FD) {
                                        self.cycles += exec_cycles as u64;
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
                    if let Clocks::Some(exec_cycles) = execute_bits(self, bus, opcode,
                                                                                 Prefix::None) {
                        self.cycles += exec_cycles as u64;
                    };
                }
                // ED-prefixed
                Prefix::ED => {
                    // NOTE: DEBUG
                    let opcode = Opcode::from_byte(byte2);
                    if let Clocks::Some(exec_cycles) = execute_extended(self, bus, opcode) {
                        self.cycles += exec_cycles as u64;
                    };
                }
                _ => unreachable!(),
            };
        } else {
            // Non-prefixed
            let opcode = Opcode::from_byte(byte1);
            if let Clocks::Some(exec_cycles) = execute_normal(self, bus, opcode, Prefix::None) {
                self.cycles += exec_cycles as u64;
            };
        };
        Result::Ok(())
    }
}
