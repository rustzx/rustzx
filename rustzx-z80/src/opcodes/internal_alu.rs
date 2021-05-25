use crate::{
    smallnum::U3,
    utils::bool_to_u8,
    tables::{
        lookup8_r12, F3F5_TABLE, HALF_CARRY_ADD_TABLE, HALF_CARRY_SUB_TABLE,
        OVERFLOW_ADD_TABLE, OVERFLOW_SUB_TABLE, PARITY_TABLE,
    },
    Flag, FLAG_CARRY, FLAG_HALF_CARRY, FLAG_SIGN, FLAG_SUB, FLAG_ZERO, Z80,
};

/// 8-bit ALU operations
pub fn execute_alu_8(cpu: &mut Z80, alu_code: U3, operand: u8) {
    let acc = cpu.regs.get_acc(); // old acc
    let result;
    // all flags are changing after alu, so init flags with zero
    let prev_carry = cpu.regs.get_flag(Flag::Carry);
    let mut flags = 0u8;
    match alu_code {
        // ADD A, Operand
        U3::N0 => {
            let temp: u16 = (acc as u16).wrapping_add(operand as u16);
            result = temp as u8;
            // get lookup code in r12 form [read file overflows.rs in z80/tables module]
            // high nibble will be bit 7 in r12 form, low nibble will be 3 bit in same form
            let lookup = lookup8_r12(acc, operand, temp as u8);
            // using lookup for finding overflow and half carry flags
            flags |= OVERFLOW_ADD_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
            flags |= bool_to_u8(temp > 0xFF) * FLAG_CARRY;
        }
        // ADC A, Operand
        U3::N1 => {
            let temp: u16 = (acc as u16)
                .wrapping_add(operand as u16)
                .wrapping_add(prev_carry as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_ADD_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
            flags |= bool_to_u8(temp > 0xFF) * FLAG_CARRY;
        }
        // SUB A, Operand
        U3::N2 | U3::N7 => {
            let temp: u16 = (acc as u16).wrapping_sub(operand as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_SUB_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
            flags |= bool_to_u8(temp > 0xFF) * FLAG_CARRY;
            flags |= FLAG_SUB;
        }
        // SBC A, Operand; CP A, Operand
        U3::N3 => {
            let temp: u16 = (acc as u16)
                .wrapping_sub(operand as u16)
                .wrapping_sub(prev_carry as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_SUB_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
            flags |= bool_to_u8(temp > 0xFF) * FLAG_CARRY;
            flags |= FLAG_SUB;
        }
        // AND A, Operand
        U3::N4 => {
            result = acc & operand;
            flags |= PARITY_TABLE[result as usize];
            flags |= FLAG_HALF_CARRY;
        }
        // XOR A, Operand
        U3::N5 => {
            result = acc ^ operand;
            flags |= PARITY_TABLE[result as usize];
        }
        // OR A, Operand
        U3::N6 => {
            result = acc | operand;
            flags |= PARITY_TABLE[result as usize];
        }
    };
    // CP, f3 and f5 from acc, else from result
    if alu_code == U3::N7 {
        flags |= F3F5_TABLE[operand as usize];
    } else {
        flags |= F3F5_TABLE[result as usize];
        cpu.regs.set_acc(result);
    };
    flags |= bool_to_u8(result == 0) * FLAG_ZERO;
    flags |= result & FLAG_SIGN;
    cpu.regs.set_flags(flags);
}
