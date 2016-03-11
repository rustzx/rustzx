use super::*;
use z80::*;
use utils::*;

/// Instruction group which operatis with bits
/// Includes rotations, setting, reseting, testing.
/// covers CB, DDCB and FDCB execution group
/// `prefix` param stands for first byte in double-prefixed instructions
pub fn execute_bits(cpu: &mut Z80, bus: &mut Z80Bus, opcode: Opcode, prefix: Prefix) -> Clocks {
    // at first = check prefix. if exists - swap opcode and displacement.
    // this must be happened because DDCB/FDCB instructions looks like
    // DD CB displacement opcode
    let displacement;
    let opcode = if prefix != Prefix::None {
        displacement = opcode.byte as i8;
        Opcode::from_byte(cpu.rom_next_byte(bus))
    } else {
        displacement = 0i8;
        opcode
    };

    let mut clocks = 0;
    // determinate data to rotate
    // (HL) selected if z is 6 in non-prefixed or if opcode is prefixed
    let operand = if (opcode.z == U3::N6) | (prefix != Prefix::None) {
        // of non prefixed, reg will become HL, else prefix corrects if to IX or IY
        let reg = RegName16::HL.with_prefix(prefix);
        // displacement will be equal zero if prefix isn't set, so next code is ok
        let addr = word_displacement(cpu.regs.get_reg_16(reg), displacement);
        RotOperand8::Indirect(addr)
    } else {
        // opcode.z will never be 6 at this moment, so unwrap
        RotOperand8::Reg(RegName8::from_u3(opcode.z).unwrap())
    };
    // if opcode is prefixed and z != 6 then we must copy
    // result to register (B, C, D, E, F, H, L, A), selected by z
    let copy_reg = if (opcode.z != U3::N6) & (prefix != Prefix::None) {
        Some(RegName8::from_u3(opcode.z).unwrap())
    } else {
        None
    };

    // valiable to store result of next computations,
    // used in DDCB, FDCB opcodes for result store
    let result;
    // parse opcode
    match opcode.x {
        // Rotate group. 0x00...0x3F
        U2::N0 => {
            result = execute_rot(cpu, bus, opcode.y, operand);
        }
        // Bit test, set, reset group
        U2::N1 | U2::N2 | U2::N3 => {
            // get bit number and data byte
            let bit_number = opcode.y.as_byte();
            let data = match operand {
                RotOperand8::Indirect(addr) => {
                    bus.read(addr)
                }
                RotOperand8::Reg(reg) => {
                    cpu.regs.get_reg_8(reg)
                }
            };
            match opcode.x {
                // BIT y, r[z]
                // [0b01yyyzzz] : 0x40...0x7F
                U2::N1 => {
                    let bit_is_set = data & (0x01 << bit_number) == 0;
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::HalfCarry, true);
                    cpu.regs.set_flag(Flag::Zero, bit_is_set);
                    cpu.regs.set_flag(Flag::ParityOveflow, bit_is_set);
                    cpu.regs.set_flag(Flag::Sign, bit_is_set && (bit_number == 7));
                    if let RotOperand8::Indirect(addr) = operand {
                        // copy of high address byte 3 and 5 bits
                        cpu.regs.set_flag(Flag::F3, addr & 0x0800 != 0);
                        cpu.regs.set_flag(Flag::F5, addr & 0x2000 != 0);
                    } else {
                        // wierd rules
                        cpu.regs.set_flag(Flag::F3, bit_is_set && (bit_number == 3));
                        cpu.regs.set_flag(Flag::F5, bit_is_set && (bit_number == 5));
                    };
                    result = 0; // mask compiler error
                }
                // RES y, r[z]
                // [0b10yyyzzz] : 0x80...0xBF
                U2::N2 => {
                    result = data & (!(0x01 << bit_number));
                    match operand {
                        RotOperand8::Indirect(addr) => {
                            bus.write(addr, result);
                        }
                        RotOperand8::Reg(reg) => {
                            cpu.regs.set_reg_8(reg, result);
                        }
                    };
                }
                // SET y, r[z]
                // [0b01yyyzzz] : 0xC0...0xFF
                U2::N3 => {
                    result = data | (0x01 << bit_number);
                    match operand {
                        RotOperand8::Indirect(addr) => {
                            bus.write(addr, result);
                        }
                        RotOperand8::Reg(reg) => {
                            cpu.regs.set_reg_8(reg, result);
                        }
                    };
                }
                _ => unreachable!()
            }
        }
    };
    // if result must be copied
    if let Some(reg) = copy_reg {
        // if operation is not BIT
        if opcode.x != U2::N1 {
            cpu.regs.set_reg_8(reg, result);
        };
    };
    if prefix == Prefix::None {
        clocks += tables::CLOCKS_CB[opcode.byte as usize];
    } else {
        clocks += tables::CLOCKS_DDCB_FDCB[opcode.byte as usize];
    };
    cpu.regs.inc_r(2); // DDCB,FDCB or CB prefix double inc R reg (yes, wierd enough)
    Clocks::Some(clocks)
}
