use super::*;
use z80::*;
use utils::*;

/// Instruction group which operatis with bits
/// Includes rotations, setting, reseting, testing.
/// covers CB, DDCB and FDCB execution group
/// `prefix` param stands for first byte in double-prefixed instructions
pub fn execute_bits(cpu: &mut Z80, bus: &mut Z80Bus, prefix: Prefix) {
    // at first = check prefix. if exists - swap opcode and displacement.
    // this must be happened because DDCB/FDCB instructions looks like
    // DD CB displacement opcode
    // let displacement;
    // let operand =
    // let opcode = if prefix != Prefix::None {
    //     displacement = opcode.byte as i8;
    //     Opcode::from_byte(cpu.fetch_byte(bus, Clocks(3)))
    // } else {
    //     displacement = 0i8;
    //     opcode
    // };
    let (opcode, operand) = if prefix == Prefix::None {
        // normal opcode fetch
        let tmp_opcode = Opcode::from_byte(cpu.fetch_byte(bus, Clocks(4)));
        if let Some(reg) = RegName8::from_u3(tmp_opcode.z) {
            (tmp_opcode, RotOperand8::Reg(reg))
        } else {
            (tmp_opcode, RotOperand8::Indirect(cpu.regs.get_hl()))
        }
    } else {
        // xx xx dd nn format opcode fetch
        let d = cpu.fetch_byte(bus, Clocks(3)) as i8;
        let addr = word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)), d);
        let tmp_opcode = Opcode::from_byte(bus.read(cpu.regs.get_pc(), Clocks(3)));
        bus.wait_loop(cpu.regs.get_pc(), Clocks(2));
        cpu.regs.inc_pc(1);
        (tmp_opcode, RotOperand8::Indirect(addr))
    };

    // determinate data to rotate
    // (HL) selected if z is 6 in non-prefixed or if opcode is prefixed
    // let operand = if (opcode.z == U3::N6) | (prefix != Prefix::None) {
    //     // of non prefixed, reg will become HL, else prefix corrects if to IX or IY
    //     let reg = RegName16::HL.with_prefix(prefix);
    //     // displacement will be equal zero if prefix isn't set, so next code is ok
    //     let addr = word_displacement(cpu.regs.get_reg_16(reg), displacement);
    //     RotOperand8::Indirect(addr)
    // } else {
    //     // opcode.z will never be 6 at this moment, so unwrap
    //     RotOperand8::Reg(RegName8::from_u3(opcode.z).unwrap())
    // };
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
                    let tmp = bus.read(addr, Clocks(3));
                    bus.wait_no_mreq(addr, Clocks(1));
                    tmp
                }
                RotOperand8::Reg(reg) => {
                    cpu.regs.get_reg_8(reg)
                }
            };
            match opcode.x {
                // BIT y, r[z]
                // [0b01yyyzzz] : 0x40...0x7F
                U2::N1 => {
                    let bit_is_set = (data & (0x01 << bit_number)) != 0;
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::HalfCarry, true);
                    cpu.regs.set_flag(Flag::Zero, !bit_is_set);
                    cpu.regs.set_flag(Flag::ParityOveflow, !bit_is_set);
                    cpu.regs.set_flag(Flag::Sign, bit_is_set && (bit_number == 7));
                    if let RotOperand8::Indirect(addr) = operand {
                        cpu.regs.set_flag(Flag::F3, addr & 0x0800 != 0);
                        cpu.regs.set_flag(Flag::F5, addr & 0x2000 != 0);
                    } else {
                        cpu.regs.set_flag(Flag::F3, (data & 0x08) != 0);
                        cpu.regs.set_flag(Flag::F5, (data & 0x20) != 0);
                    };
                    result = 0; // mask compiler error
                }
                // RES y, r[z]
                // [0b10yyyzzz] : 0x80...0xBF
                U2::N2 => {
                    result = data & (!(0x01 << bit_number));
                    match operand {
                        RotOperand8::Indirect(addr) => {
                            bus.write(addr, result, Clocks(3));
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
                            bus.write(addr, result, Clocks(3));
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
    cpu.regs.inc_r(2); // DDCB,FDCB or CB prefix double inc R reg (yes, wierd enough)
}
