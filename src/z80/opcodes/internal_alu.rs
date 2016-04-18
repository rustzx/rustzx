use z80::*;
use utils::*;
use z80::tables::*;

/// 8-bit ALU operations
pub fn execute_alu_8(cpu: &mut Z80, alu_code: U3, operand: u8) {
    let acc = cpu.regs.get_acc(); // old acc
    let result;
    // all flags are changing after alu
    let (sign, zero, f5, f3, half_carry, pv, sub, carry);
    match alu_code {
        // ADD A, Operand
        U3::N0 => {
            let temp: u16 =  (acc as u16).wrapping_add(operand as u16);
            result = temp as u8;
            // get lookup code in r12 form [read file overflows.rs in z80/tables module]
            // high nibble will be bit 7 in r12 form, low nibble will be 3 bit in same form
            let lookup = lookup8_r12(acc, operand, temp as u8);
            // using lookup for finding overflow and half carry flags
            pv = OVERFLOW_ADD_TABLE[(lookup >> 4) as usize] != 0;
            half_carry = HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize] != 0;
            carry = temp > 0xFF;
            sub = false;
        }
        // ADC A, Operand
        U3::N1 => {
            let prev_carry = bool_to_u8(cpu.regs.get_flag(Flag::Carry));
            let temp: u16 = (acc as u16).wrapping_add(operand as u16)
                .wrapping_add(prev_carry as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            pv = OVERFLOW_ADD_TABLE[(lookup >> 4) as usize] != 0;
            half_carry = HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize] != 0;
            carry = temp > 0xFF;
            sub = false;
        }
        // SUB A, Operand
        U3::N2 | U3::N7 => {
            let temp: u16 = (acc as u16).wrapping_sub(operand as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            pv = OVERFLOW_SUB_TABLE[(lookup >> 4) as usize] != 0;
            half_carry = HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize] != 0;
            carry = temp > 0xFF;
            sub = true;
        }
        // SBC A, Operand; CP A, Operand
        U3::N3 => {
            let prev_carry = bool_to_u8(cpu.regs.get_flag(Flag::Carry));
            let temp: u16 = (acc as u16).wrapping_sub(operand as u16)
                .wrapping_sub(prev_carry as u16);
            result = temp as u8;
            let lookup = lookup8_r12(acc, operand, temp as u8);
            pv = OVERFLOW_SUB_TABLE[(lookup >> 4) as usize] != 0;
            half_carry = HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize] != 0;
            carry = temp > 0xFF;
            sub = true;
        }
        // AND A, Operand
        U3::N4 => {
            result = acc & operand;
            carry = false;
            sub = false;
            pv = tables::PARITY_BIT[result as usize] != 0;
            half_carry = true;
        }
        // XOR A, Operand
        U3::N5 => {
            result = acc ^ operand;
            carry = false;
            sub = false;
            pv = tables::PARITY_BIT[result as usize] != 0;
            half_carry = false;
        }
        // OR A, Operand
        U3::N6 => {
            result = acc | operand;
            carry = false;
            sub = false;
            pv = tables::PARITY_BIT[result as usize] != 0;
            half_carry = false;
        }
    };
    // CP, f3 and f5 from acc, else from result
    if alu_code == U3::N7 {
        f3 = operand & 0x08 != 0;
        f5 = operand & 0x20 != 0;
        // if CP, don't write result
    } else {
        f3 = result & 0x08 != 0;
        f5 = result & 0x20 != 0;
        cpu.regs.set_acc(result);
    };
    zero = result == 0;
    sign = (result & 0x80) != 0;
    cpu.regs.set_flag(Flag::Carry, carry);
    cpu.regs.set_flag(Flag::Sub, sub);
    cpu.regs.set_flag(Flag::ParityOveflow, pv);
    cpu.regs.set_flag(Flag::F3, f3);
    cpu.regs.set_flag(Flag::HalfCarry, half_carry);
    cpu.regs.set_flag(Flag::F5, f5);
    cpu.regs.set_flag(Flag::Zero, zero);
    cpu.regs.set_flag(Flag::Sign, sign);
}
