use crate::{
    smallnum::U3,
    tables::{
        lookup8_r12, F3F5_TABLE, HALF_CARRY_ADD_TABLE, HALF_CARRY_SUB_TABLE, OVERFLOW_ADD_TABLE,
        OVERFLOW_SUB_TABLE, PARITY_TABLE,
    },
    FLAG_CARRY, FLAG_HALF_CARRY, FLAG_SIGN, FLAG_SUB, FLAG_ZERO, Z80,
};

/// 8-bit ALU operations
pub fn execute_alu_8(cpu: &mut Z80, alu_code: U3, operand: u8) {
    let acc = cpu.regs.get_acc();
    let result;
    let with_carry = (cpu.regs.get_flags() & FLAG_CARRY) != 0;
    let mut flags = 0u8;
    match alu_code {
        // ADD A, Operand
        U3::N0 => {
            let temp: u16 = (acc as u16).wrapping_add(operand as u16);
            result = temp as u8;
            // get lookup code in r12 form [read file overflows.rs in `tables` module]
            // high nibble will be bit 7 in r12 form, low nibble will be 3 bit in same form
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_ADD_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
            flags |= (temp > 0xFF) as u8 * FLAG_CARRY;
        }
        // ADC A, Operand
        U3::N1 => {
            let temp: u16 = (acc as u16)
                .wrapping_add(operand as u16)
                .wrapping_add(with_carry as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_ADD_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
            flags |= (temp > 0xFF) as u8 * FLAG_CARRY;
        }
        // SUB A, Operand
        U3::N2 | U3::N7 => {
            let temp: u16 = (acc as u16).wrapping_sub(operand as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_SUB_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
            flags |= (temp > 0xFF) as u8 * FLAG_CARRY;
            flags |= FLAG_SUB;
        }
        // SBC A, Operand; CP A, Operand
        U3::N3 => {
            let temp: u16 = (acc as u16)
                .wrapping_sub(operand as u16)
                .wrapping_sub(with_carry as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            flags |= OVERFLOW_SUB_TABLE[(lookup >> 4) as usize];
            flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
            flags |= (temp > 0xFF) as u8 * FLAG_CARRY;
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
    flags |= (result == 0) as u8 * FLAG_ZERO;
    flags |= result & FLAG_SIGN;
    cpu.regs.set_flags(flags);
}
