use cpu::registers::{RegName8, RegName16, Regs};
use cpu::tables;
use utils::make_word;
use cpu::decoders::{ConditionDecoder, RegNameDecoder};

/// Z80 processor System bus
pub trait Z80Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}


/// Instruction prefix
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prefix {
    CB,
    DD,
    ED,
    FD,
}
impl Prefix {
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
/// ```
///           ___y____
///         /         \
/// | 1  1 | 1  1 | 1 | 1  1  1|
/// \_____/\_____/\__/\_______/
///    x      p    q      z
/// ```
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
    Executed(u8),
    NonInstuction,
    Fail,
}
/// Modificate register with prefix
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

/// Modificate register with prefix
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

/// Z80 Processor
pub struct Z80 {
    regs: Regs,
    /// cycles, which were uncounted from previous emulation
    uncounted_cycles: u64,
}

impl Z80 {
    /// new cpu
    pub fn new() -> Z80 {
        Z80 {
            regs: Regs::new(),
            uncounted_cycles: 0,
        }
    }

    /// read byte from rom and, pc += 1
    fn rom_next_byte(&mut self, bus: &mut Z80Bus) -> u8 {
        let pc = self.regs.inc_reg_16(RegName16::PC, 1);
        bus.read(pc - 1)
    }

    /// read word from rom and, pc += 2
    fn rom_next_word(&mut self, bus: &mut Z80Bus) -> u16 {
        let pc = self.regs.inc_reg_16(RegName16::PC, 2);
        make_word(bus.read(pc - 1), bus.read(pc - 2))
    }

    /// normal execution group, can be modified with prefixes DD, FD
    fn execute_normal(&mut self,
                      bus: &mut Z80Bus,
                      opcode: Opcode,
                      prefix: Option<Prefix>)
                      -> ExecResult {
        // 2 first bits of opcode
        match opcode.x {
            // ---------------------------------
            // [0x00yyyzzz] instruction section
            // ---------------------------------
            // [0x00yyy000] instruction group
            0 if opcode.z == 0 => {
                match opcode.y {
                    // NOP
                    // [0x00000000] = 0x00
                    0 => {}
                    // EX AF, AF'
                    // [0x00001000] = 0x08
                    1 => {
                        println!("EX AF,AF'");
                        self.regs.swap_af_alt();
                    }
                    // DJNZ offset;   13/8 clocks
                    // [0x00010000] = 0x10
                    2 => {
                        println!("DJNZ d");
                        let offset = self.rom_next_byte(bus) as i8;
                        // preform jump
                        if self.regs.dec_reg_8(RegName8::B, 1) != 0 {
                            self.regs.shift_pc(offset);
                            return ExecResult::Executed(13);
                        } else {
                            return ExecResult::Executed(8);
                        }
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
                            return ExecResult::Executed(12);
                        } else {
                            return ExecResult::Executed(7);
                        }
                    }
                    _ => return ExecResult::Fail,
                }
            }
            // [0x00ppq001] instruction group
            0 if opcode.z == 1 => {
                match opcode.q {
                    // LD rp[p], nn
                    // [0x00pp0001] : 0x01, 0x11, 0x21, 0x31
                    0 => {
                        let mut reg = RegNameDecoder::reg_16_with_sp(opcode.p);
                        // mod by prefix
                        if let Some(prefix) = prefix {
                            reg = reg_with_prefix_16(reg, prefix);
                        }
                        let data = self.rom_next_word(bus);
                        self.regs.set_reg_16(reg, data);
                    }
                    // ADD HL, ss
                    1 => {
                        // Work in progress
                        //
                    }
                    _ => return ExecResult::Fail,
                }
            }
            0 if opcode.z == 2 => {}
            0 if opcode.z == 3 => {}
            0 if opcode.z == 4 => {}
            0 if opcode.z == 5 => {}
            0 if opcode.z == 6 => {}
            0 if opcode.z == 7 => {}

            _ => panic!("Opcode {:#b} unimplented", opcode.to_byte()),
        };
        ExecResult::Executed(tables::CLOCKS_NORMAL[opcode.to_byte() as usize])
    }


    /// emulation cycle
    pub fn emulate(&mut self, bus: &mut Z80Bus, cycles: u64) {
        // cycle_counter initial value
        let mut cycle_counter = self.uncounted_cycles;
        loop {
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
                                }
                                Prefix::CB if prefix_hi == Prefix::DD => {
                                    // DDCB prefixed
                                }
                                Prefix::CB => {
                                    // FDCB prefixed
                                }
                            }
                        } else {
                            match prefix_single {
                                Prefix::DD => {
                                    // DD-prefixed
                                }
                                _ => {
                                    // FD-prefixed
                                }
                            }
                        }
                    }
                    // CB-prefixed
                    Prefix::CB => {}
                    // ED-prefixed
                    Prefix::ED => {}
                }
            } else {
                let opcode = Opcode::from_byte(self.rom_next_byte(bus));
                if let ExecResult::Executed(exec_cycles) = self.execute_normal(bus, opcode, None) {
                    cycle_counter += exec_cycles as u64;
                }
            }

            if cycle_counter >= cycles {
                self.uncounted_cycles = cycle_counter - cycles;
                break;
            }
        }

    }
}
