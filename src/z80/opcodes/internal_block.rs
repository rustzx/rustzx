use super::*;
use z80::*;
use z80::tables::*;
use utils::Clocks;

/// ldi or ldd instruction
pub fn execute_ldi_ldd(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) {
    // read (HL)
    let src = bus.read(cpu.regs.get_hl(), Clocks(3));
    let bc = cpu.regs.dec_reg_16(RegName16::BC, 1);
    // write (HL) to (DE)
    bus.write(cpu.regs.get_de(), src, Clocks(3));
    bus.wait_loop(cpu.regs.get_de(), Clocks(2));
    // inc or dec HL and DE
    match dir {
        BlockDir::Inc => {
            cpu.regs.inc_reg_16(RegName16::HL, 1);
            cpu.regs.inc_reg_16(RegName16::DE, 1);
        }
        BlockDir::Dec => {
            cpu.regs.dec_reg_16(RegName16::HL, 1);
            cpu.regs.dec_reg_16(RegName16::DE, 1);
        }
    }
    // flags
    cpu.regs.set_flag(Flag::Sub, false);
    cpu.regs.set_flag(Flag::HalfCarry, false);
    cpu.regs.set_flag(Flag::ParityOveflow, bc != 0);
    let src_plus_a = src.wrapping_add(cpu.regs.get_acc());
    cpu.regs.set_flag(Flag::F3, (src_plus_a & 0x08) != 0);
    cpu.regs.set_flag(Flag::F5, (src_plus_a & 0x02) != 0);
    // Clocks: <4 + 4> + 3 + 3 + 2 = 16
}

/// cpi or cpd instruction
pub fn execute_cpi_cpd(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) -> bool {
    // read (HL)
    let src = bus.read(cpu.regs.get_hl(), Clocks(3));
    bus.wait_loop(cpu.regs.get_hl(), Clocks(5));
    // move pointer
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL, 1),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL, 1),
    };
    // dec bc
    let bc = cpu.regs.dec_reg_16(RegName16::BC, 1);
    let acc = cpu.regs.get_acc();
    // variable to store CP (HL) subtract result
    let tmp = acc.wrapping_sub(src);
    // flags
    cpu.regs.set_flag(Flag::Sub, true);
    cpu.regs.set_flag(Flag::ParityOveflow, bc != 0);
    cpu.regs.set_flag(Flag::Zero, tmp == 0);
    cpu.regs.set_flag(Flag::Sign, (tmp & 0x80) != 0);
    let lookup = lookup8_r12(acc, src, tmp);
    let half_borrow = HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize] != 0;
    cpu.regs.set_flag(Flag::HalfCarry, half_borrow);
    let tmp2 = if half_borrow {
        tmp.wrapping_sub(1)
    } else {
        tmp
    };
    cpu.regs.set_flag(Flag::F3, (tmp2 & 0b1000) != 0);
    cpu.regs.set_flag(Flag::F5, (tmp2 & 0b10) != 0);
    // Clocks: <4 + 4> + 3 + 5 = 16
    tmp == 0 // return comarison result
}

/// ini or ind instruction
pub fn execute_ini_ind(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) {
    bus.wait_no_mreq(cpu.regs.get_ir(), Clocks(1));
    // get from port and write to memory
    let src = bus.read_io(cpu.regs.get_bc());
    bus.write(cpu.regs.get_hl(), src, Clocks(3));
    // dec counter
    let b = cpu.regs.dec_reg_8(RegName8::B, 1);
    // move pointer
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL, 1),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL, 1),
    };
    // as in dec b
    cpu.regs.set_flag(Flag::Zero, b == 0);
    cpu.regs.set_flag(Flag::Sign, (b & 0x80) != 0);
    cpu.regs.set_flag(Flag::F3, (b & 0x08) != 0);
    cpu.regs.set_flag(Flag::F5, (b & 0x20) != 0);
    // 7 bit from input value
    cpu.regs.set_flag(Flag::Sub, (src & 0x80) != 0);
    // get C reg and modify it according to instruction type
    let c = match dir {
        BlockDir::Inc => cpu.regs.get_reg_8(RegName8::C).wrapping_add(1),
        BlockDir::Dec => cpu.regs.get_reg_8(RegName8::C).wrapping_sub(1),
    };
    // k_carry from (HL) + ( C (+ or -) 1) & 0xFF
    let (k, k_carry) = c.overflowing_add(src);
    cpu.regs.set_flag(Flag::Carry, k_carry);
    cpu.regs.set_flag(Flag::HalfCarry, k_carry);
    // Parity of (k & 7) xor B is PV flag
    cpu.regs.set_flag(Flag::ParityOveflow,
                      tables::PARITY_BIT[((k & 0x07) ^ b) as usize] != 0);
}

/// outi or outd instruction
pub fn execute_outi_outd(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) {
    bus.wait_no_mreq(cpu.regs.get_ir(), Clocks(1));
    // get input data
    let src = bus.read(cpu.regs.get_hl(), Clocks(3));
    let b = cpu.regs.dec_reg_8(RegName8::B, 1);
    bus.write_io(cpu.regs.get_bc(), src);
    // move pointer
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL, 1),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL, 1),
    };
    let l = cpu.regs.get_l();
    // as in dec b
    cpu.regs.set_flag(Flag::Zero, b == 0);
    cpu.regs.set_flag(Flag::Sign, (b & 0x80) != 0);
    cpu.regs.set_flag(Flag::F3, (b & 0x08) != 0);
    cpu.regs.set_flag(Flag::F5, (b & 0x20) != 0);
    // 7 bit of output value [(HL)]
    cpu.regs.set_flag(Flag::Sub, (src & 0x80) != 0);
    // temporary k is L + (HL)
    let (k, k_carry) = l.overflowing_add(src);
    cpu.regs.set_flag(Flag::Carry, k_carry);
    cpu.regs.set_flag(Flag::HalfCarry, k_carry);
    // Parity of (k & 7) xor B is PV flag
    cpu.regs.set_flag(Flag::ParityOveflow,
                      tables::PARITY_BIT[((k & 0x07) ^ b) as usize] != 0);
}
