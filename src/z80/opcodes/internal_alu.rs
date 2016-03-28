use z80::*;
use utils::*;

/// 8-bit ALU operations
pub fn execute_alu_8(cpu: &mut Z80, alu_code: U3, operand: u8) {
    let acc = cpu.regs.get_acc(); // old acc
    let result;
    // all flags are changing after alu
    let (sign, zero, f5, f3, half_carry, pv, sub, carry);
    match alu_code {
        // ADD A, Operand
        U3::N0 => {
            let (r, c) = acc.overflowing_add(operand);
            result = r;
            carry = c;
            sub = false;
            pv = check_add_overflow_8(acc as i8, operand as i8);
            half_carry = half_carry_8(acc, operand);
        }
        // ADC A, Operand
        U3::N1 => {
            let prev_carry = bool_to_u8(cpu.regs.get_flag(Flag::Carry));
            let (r_tmp, c1) = acc.overflowing_add(operand);
            let (r, c2) = r_tmp.overflowing_add(prev_carry);
            result = r;
            carry = c1 | c2;
            sub = false;
            pv = check_add_overflow_8(acc as i8, operand as i8) |
                 check_add_overflow_8(r_tmp as i8, prev_carry as i8);
            half_carry = half_carry_8(acc, operand) | half_carry_8(r_tmp, prev_carry);
        }
        // SUB A, Operand
        U3::N2 | U3::N7 => {
            let (r, c) = acc.overflowing_sub(operand);
            result = r;
            carry = c;
            sub = true;
            pv = check_sub_overflow_8(acc as i8, operand as i8);
            half_carry = half_borrow_8(acc, operand);
        }
        // SBC A, Operand; CP A, Operand
        U3::N3 => {
            let prev_carry = bool_to_u8(cpu.regs.get_flag(Flag::Carry));
            let (r_tmp, c1) = acc.overflowing_sub(operand);
            let (r, c2) = r_tmp.overflowing_sub(prev_carry);
            result = r;
            carry = c1 | c2;
            sub = true;
            pv = check_sub_overflow_8(acc as i8, operand as i8) |
                 check_sub_overflow_8(r_tmp as i8, prev_carry as i8);
            half_carry = half_borrow_8(acc, operand) | half_borrow_8(r_tmp, prev_carry);
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
