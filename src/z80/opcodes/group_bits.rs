use super::*;
use utils::*;
use z80::*;
use z80::tables::*;

/// Instruction group which operatis with bits
/// Includes rotations, setting, reseting, testing.
/// covers CB, DDCB and FDCB execution group
/// `prefix` param stands for first byte in double-prefixed instructions
pub fn execute_bits(cpu: &mut Z80, bus: &mut Z80Bus, prefix: Prefix) {
    let (opcode, operand) = if prefix == Prefix::None {
        // normal opcode fetch
        let tmp_opcode = Opcode::from_byte(cpu.fetch_byte(bus, Clocks(4)));
        // inc r when non-prefixed.
        cpu.regs.inc_r(1);
        // return opcode with operand tuple
        if let Some(reg) = RegName8::from_u3(tmp_opcode.z) {
            (tmp_opcode, BitOperand8::Reg(reg))
        } else {
            // non-prefixed, addr is HL
            (tmp_opcode, BitOperand8::Indirect(cpu.regs.get_hl()))
        }
    } else {
        // xx xx dd nn format opcode fetch
        // if prefixed, we need to swap displacement and opcode
        // fetch displacement
        let d = cpu.fetch_byte(bus, Clocks(3)) as i8;
        // build address
        let addr = word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)), d);
        // read next byte
        let tmp_opcode = Opcode::from_byte(bus.read(cpu.regs.get_pc(), Clocks(3)));
        // wait 2 clocks
        bus.wait_loop(cpu.regs.get_pc(), Clocks(2));
        // next byte
        cpu.regs.inc_pc(1);
        (tmp_opcode, BitOperand8::Indirect(addr))
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
                BitOperand8::Indirect(addr) => {
                    let tmp = bus.read(addr, Clocks(3));
                    bus.wait_no_mreq(addr, Clocks(1));
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
                    flags |= bool_to_u8(!bit_is_set) * (FLAG_ZERO | FLAG_PV);
                    // NOTE: according to FUSE.
                    // maybe must be based on current bit or something?
                    flags |= bool_to_u8((data & 0x80 != 0) && (bit_number == 7)) * FLAG_SIGN;
                    if let BitOperand8::Indirect(addr) = operand {
                        flags |= F3F5_TABLE[(addr >> 8) as usize];
                    } else {
                        flags |= F3F5_TABLE[data as usize];
                    }
                    cpu.regs.set_flags(flags);
                    result = 0; // mask compiler error
                }
                // RES y, r[z]
                // [0b10yyyzzz] : 0x80...0xBF
                U2::N2 => {
                    result = data & (!(0x01 << bit_number));
                    match operand {
                        BitOperand8::Indirect(addr) => {
                            bus.write(addr, result, Clocks(3));
                        }
                        BitOperand8::Reg(reg) => {
                            cpu.regs.set_reg_8(reg, result);
                        }
                    };
                }
                // SET y, r[z]
                // [0b01yyyzzz] : 0xC0...0xFF
                U2::N3 => {
                    result = data | (0x01 << bit_number);
                    match operand {
                        BitOperand8::Indirect(addr) => {
                            bus.write(addr, result, Clocks(3));
                        }
                        BitOperand8::Reg(reg) => {
                            cpu.regs.set_reg_8(reg, result);
                        }
                    };
                }
                _ => unreachable!(),
            }
        }
    };
    // if result preifxed
    if prefix != Prefix::None {
        // and z != 6 (must be undocumented)
        if let Some(reg) = RegName8::from_u3(opcode.z) {
            // then copy, if instruction isn't BIT
            if opcode.x != U2::N1 {
                cpu.regs.set_reg_8(reg, result);
            };
        }
    }
}
