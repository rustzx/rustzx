use crate::{opcode::BitOperand8, smallnum::U3, tables::SZPF3F5_TABLE, Z80Bus, FLAG_CARRY, Z80};

/// Rotate operations (RLC, RRC, RL, RR, SLA, SRA, SLL, SRL)
/// returned result is required for with DDCB/FDCB-prefixed instructions
pub fn execute_rot(cpu: &mut Z80, bus: &mut impl Z80Bus, rot_code: U3, operand: BitOperand8) -> u8 {
    let mut data = match operand {
        BitOperand8::Indirect(addr) => {
            let tmp = bus.read(addr, 3);
            bus.wait_no_mreq(addr, 1);
            tmp
        }
        BitOperand8::Reg(reg) => cpu.regs.get_reg_8(reg),
    };
    let mut flags = 0u8;
    let old_carry = cpu.regs.get_flags() & FLAG_CARRY;
    let carry_bit;
    match rot_code {
        // RLC
        U3::N0 => {
            carry_bit = (data & 0x80) != 0;
            data = (data << 1) & 0xFE;
            if carry_bit {
                data |= 0x01;
            };
        }
        // RRC
        U3::N1 => {
            carry_bit = (data & 0x01) != 0;
            data = (data >> 1) & 0x7F;
            if carry_bit {
                data |= 0x80;
            };
        }
        // RL
        U3::N2 => {
            carry_bit = (data & 0x80) != 0;
            data = (data << 1) & 0xFE;
            if old_carry != 0 {
                data |= 0x01;
            };
        }
        // RR
        U3::N3 => {
            carry_bit = (data & 0x01) != 0;
            data = (data >> 1) & 0x7F;
            if old_carry != 0 {
                data |= 0x80;
            };
        }
        // SLA
        U3::N4 => {
            carry_bit = (data & 0x80) != 0;
            data = (data << 1) & 0xFE;
        }
        // SRA
        U3::N5 => {
            carry_bit = (data & 0x01) != 0;
            data = ((data >> 1) & 0x7F) | (data & 0x80);
        }
        // SLL
        U3::N6 => {
            carry_bit = (data & 0x80) != 0;
            data = (data << 1) | 0x01;
        }
        // SRL
        U3::N7 => {
            carry_bit = (data & 0x01) != 0;
            data = (data >> 1) & 0x7F;
        }
    };
    flags |= carry_bit as u8 * FLAG_CARRY;
    flags |= SZPF3F5_TABLE[data as usize];
    match operand {
        BitOperand8::Indirect(addr) => {
            bus.write(addr, data, 3);
        }
        BitOperand8::Reg(reg) => {
            cpu.regs.set_reg_8(reg, data);
        }
    };
    cpu.regs.set_flags(flags);
    data
}
