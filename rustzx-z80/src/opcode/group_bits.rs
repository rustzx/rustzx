use crate::{
    opcode::{execute_rot, BitOperand8, Opcode, Prefix},
    smallnum::U2,
    tables::F3F5_TABLE,
    RegName16, RegName8, Z80Bus, FLAG_CARRY, FLAG_F3, FLAG_F5, FLAG_HALF_CARRY, FLAG_PV, FLAG_SIGN,
    FLAG_ZERO, Z80,
};

/// Instruction group which operates on bits
/// Includes rotations, bit set/reset/test
/// Covers CB, DDCB and FDCB execution group
pub fn execute_bits(cpu: &mut Z80, bus: &mut impl Z80Bus, prefix: Prefix) {
    let (opcode, operand) = if prefix == Prefix::None {
        // non-prefixed bits-related opcode
        let opcode = Opcode::from_byte(cpu.fetch_byte(bus, 4));
        cpu.regs.inc_r();
        let operand = match RegName8::from_u3(opcode.z) {
            Some(reg) => BitOperand8::Reg(reg),
            None => BitOperand8::Indirect(cpu.regs.get_hl()),
        };
        (opcode, operand)
    } else {
        // Prefixed opcode with `xx xx dd nn` format
        let displacement = cpu.fetch_byte(bus, 3) as i8;
        let addr = cpu
            .regs
            .build_addr_with_offset(RegName16::HL.with_prefix(prefix), displacement);
        let opcode = Opcode::from_byte(bus.read(cpu.regs.get_pc(), 3));
        bus.wait_loop(cpu.regs.get_pc(), 2);
        cpu.regs.inc_pc();
        (opcode, BitOperand8::Indirect(addr))
    };

    let result = match opcode.x {
        // Rotate group. 0x00...0x3F
        U2::N0 => execute_rot(cpu, bus, opcode.y, operand),
        // Bit test, set, reset group
        U2::N1 | U2::N2 | U2::N3 => {
            // get bit number and data byte
            let bit_number = opcode.y.as_byte();
            let data = match operand {
                BitOperand8::Indirect(addr) => {
                    let tmp = bus.read(addr, 3);
                    bus.wait_no_mreq(addr, 1);
                    tmp
                }
                BitOperand8::Reg(reg) => cpu.regs.get_reg_8(reg),
            };
            match opcode.x {
                // BIT y, r[z]
                // [0b01yyyzzz] : 0x40...0x7F
                U2::N1 => {
                    let bit_is_set = (data & (0x01 << bit_number)) != 0;
                    // only carry is not affected;
                    let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
                    flags |= FLAG_HALF_CARRY;
                    flags |= (!bit_is_set) as u8 * (FLAG_ZERO | FLAG_PV);
                    // NOTE: according to FUSE.
                    // maybe must be based on current bit or something?
                    flags |= ((data & 0x80 != 0) && (bit_number == 7)) as u8 * FLAG_SIGN;
                    // TODO(critical): Not sure that this is relevant for
                    // non-(HL) (prefixed) operand
                    if let BitOperand8::Indirect(_addr) = operand {
                        flags |= ((cpu.regs.get_mem_ptr() >> 8) as u8) & (FLAG_F3 | FLAG_F5);
                    } else {
                        flags |= F3F5_TABLE[data as usize];
                    }
                    cpu.regs.set_flags(flags);
                    // retuned `0` value actually will not be used
                    0
                }
                // RES y, r[z]
                // [0b10yyyzzz] : 0x80...0xBF
                U2::N2 => {
                    let result = data & (!(0x01 << bit_number));
                    match operand {
                        BitOperand8::Indirect(addr) => {
                            bus.write(addr, result, 3);
                        }
                        BitOperand8::Reg(reg) => {
                            cpu.regs.set_reg_8(reg, result);
                        }
                    };
                    result
                }
                // SET y, r[z]
                // [0b01yyyzzz] : 0xC0...0xFF
                U2::N3 => {
                    let result = data | (0x01 << bit_number);
                    match operand {
                        BitOperand8::Indirect(addr) => {
                            // TODO(critical): Check memptr impl against FUSE
                            bus.write(addr, result, 3);
                        }
                        BitOperand8::Reg(reg) => {
                            cpu.regs.set_reg_8(reg, result);
                        }
                    };
                    result
                }
                _ => unreachable!(),
            }
        }
    };

    if prefix != Prefix::None {
        // and z != 6 (undocumented)
        if let Some(reg) = RegName8::from_u3(opcode.z) {
            // if instruction is not BIT
            if opcode.x != U2::N1 {
                cpu.regs.set_reg_8(reg, result);
            };
        }
    }
}
