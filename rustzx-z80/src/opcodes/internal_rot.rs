use crate::{
    Clocks,
    smallnum::U3,
    utils::bool_to_u8,
    opcodes::BitOperand8,
    tables::SZPF3F5_TABLE,
    Flag,
    Z80Bus,
    FLAG_CARRY,
    Z80,
};

/// Rotate operations (RLC, RRC, RL, RR, SLA, SRA, SLL, SRL)
/// returns result (can be useful with DDCB/FDCB instructions)
pub fn execute_rot(cpu: &mut Z80, bus: &mut dyn Z80Bus, rot_code: U3, operand: BitOperand8) -> u8 {
    // get byte which will be rotated
    let mut data = match operand {
        BitOperand8::Indirect(addr) => {
            let tmp = bus.read(addr, Clocks(3));
            bus.wait_no_mreq(addr, Clocks(1));
            tmp
        }
        BitOperand8::Reg(reg) => cpu.regs.get_reg_8(reg),
    };
    let mut flags = 0u8;
    let old_carry = cpu.regs.get_flag(Flag::Carry);
    let carry_bit;
    match rot_code {
        // RLC
        U3::N0 => {
            carry_bit = (data & 0x80) != 0;
            // shift left and clear lowerest bit
            data = (data << 1) & 0xFE;
            // set lsb if msb was set
            if carry_bit {
                data |= 0x01;
            };
        }
        // RRC
        U3::N1 => {
            carry_bit = (data & 0x01) != 0;
            // shift left and clear highest bit
            data = (data >> 1) & 0x7F;
            // set msb if lsb was set
            if carry_bit {
                data |= 0x80;
            };
        }
        // RL
        U3::N2 => {
            carry_bit = (data & 0x80) != 0;
            // shift left and clear lowerest bit
            data = (data << 1) & 0xFE;
            // set lsb if msb was set
            if old_carry {
                data |= 0x01;
            };
        }
        // RR
        U3::N3 => {
            carry_bit = (data & 0x01) != 0;
            // shift right and clear highest bit
            data = (data >> 1) & 0x7F;
            // set msb if lsb was set
            if old_carry {
                data |= 0x80;
            };
        }
        // SLA
        U3::N4 => {
            carry_bit = (data & 0x80) != 0;
            // shift left and clear lowerest bit
            data = (data << 1) & 0xFE;
        }
        // SRA
        U3::N5 => {
            carry_bit = (data & 0x01) != 0;
            // shift left and leave highest bit unchange4
            data = ((data >> 1) & 0x7F) | (data & 0x80);
        }
        // SLL
        U3::N6 => {
            carry_bit = (data & 0x80) != 0;
            // shift left and set lowerest bit
            data = (data << 1) | 0x01;
        }
        // SRL
        U3::N7 => {
            carry_bit = (data & 0x01) != 0;
            // shift left and leave highest bit unchanged
            data = (data >> 1) & 0x7F;
        }
    };
    flags |= bool_to_u8(carry_bit) * FLAG_CARRY;
    flags |= SZPF3F5_TABLE[data as usize];
    // write result
    match operand {
        BitOperand8::Indirect(addr) => {
            bus.write(addr, data, Clocks(3));
        }
        BitOperand8::Reg(reg) => {
            cpu.regs.set_reg_8(reg, data);
        }
    };
    cpu.regs.set_flags(flags);
    data
}
