use utils::*;
use super::*;
use super::opcodes::*;

/// Z80 Processor struct
pub struct Z80 {
    /// CPU Regs struct
    pub regs: Regs,
    /// cycles, which were uncounted from previous emulation
    uncounted_cycles: u64,
    pub halted: bool,
    pub skip_interrupt: bool,
    pub int_mode: IntMode,
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
    pub fn rom_next_byte(&mut self, bus: &Z80Bus) -> u8 {
        let addr = self.regs.get_pc();
        self.regs.inc_pc(1);
        bus.read(addr)
    }

    /// read word from rom and, pc += 2
    pub fn rom_next_word(&mut self, bus: &Z80Bus) -> u16 {
        let (hi, lo);
        lo = self.regs.get_pc();
        hi = self.regs.inc_pc(1);
        self.regs.inc_pc(1);
        make_word(bus.read(hi), bus.read(lo))
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
            return execute_internal_nop(self) as u64;
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
                                cycle_counter += exec_cycles as u64;
                            };
                        }
                        Prefix::CB => {
                            // FDCB prefixed
                            // third byte is not real opcode. it will be transformed
                            // into displacement in execute_bits
                            let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                            if let Clocks::Some(exec_cycles) =
                                execute_bits(self, bus, opcode, Prefix::FD) {
                                cycle_counter += exec_cycles as u64;
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
                                        cycle_counter += exec_cycles as u64;
                                    };
                                }
                                // FD prefixed
                                _ => {
                                    if let Clocks::Some(exec_cycles) =
                                           execute_normal(self, bus, opcode, Prefix::FD) {
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
                    if let Clocks::Some(exec_cycles) = execute_bits(self, bus, opcode,
                                                                                 Prefix::None) {
                        cycle_counter += exec_cycles as u64;
                    };
                }
                // ED-prefixed
                Prefix::ED => {
                    // NOTE: DEBUG
                    println!("Prefix: ED");
                    let opcode = Opcode::from_byte(byte2);
                    if let Clocks::Some(exec_cycles) = execute_extended(self, bus, opcode) {
                        cycle_counter += exec_cycles as u64;
                    };
                }
                _ => unreachable!(),
            };
        } else {
            // Non-prefixed
            let opcode = Opcode::from_byte(byte1);
            if let Clocks::Some(exec_cycles) = execute_normal(self, bus, opcode, Prefix::None) {
                cycle_counter += exec_cycles as u64;
            };
        };

        cycle_counter
    }
}
