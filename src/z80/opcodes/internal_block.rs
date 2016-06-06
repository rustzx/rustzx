use super::*;
use z80::*;
use z80::tables::*;
use utils::*;

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
    let mut flags = cpu.regs.get_flags();
    // reset affected flags
    flags &= !(FLAG_SUB | FLAG_HALF_CARRY | FLAG_PV | FLAG_F3 | FLAG_F5);
    // set PV if bc != 0
    flags |= bool_to_u8(bc != 0) * FLAG_PV;
    let src_plus_a = src.wrapping_add(cpu.regs.get_acc());
    // bit 1 for F5 and bit 3 for F3
    flags |= bool_to_u8(src_plus_a & 0x08 != 0) * FLAG_F3;
    flags |= bool_to_u8(src_plus_a & 0x02 != 0) * FLAG_F5;
    cpu.regs.set_flags(flags);
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
    // flags, only carry unaffected
    let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
    flags |= FLAG_SUB;
    flags |= bool_to_u8(bc != 0) * FLAG_PV;
    flags |= bool_to_u8(tmp == 0) * FLAG_ZERO;
    flags |= tmp & FLAG_SIGN;
    let lookup = lookup8_r12(acc, src, tmp);
    let half_borrow = HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
    flags |= half_borrow;
    let tmp2 = if half_borrow != 0 {
        tmp.wrapping_sub(1)
    } else {
        tmp
    };
    flags |= bool_to_u8((tmp2 & 0x08) != 0) * FLAG_F3;
    flags |= bool_to_u8((tmp2 & 0x02) != 0) * FLAG_F5;
    cpu.regs.set_flags(flags);
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
    let mut flags = 0u8;
    // as in dec b
    flags |= SZF3F5_TABLE[b as usize];
    // 7 bit from input value
    flags |= bool_to_u8((src & 0x80) != 0) * FLAG_SUB;
    // get C reg and modify it according to instruction type
    let c = match dir {
        BlockDir::Inc => cpu.regs.get_reg_8(RegName8::C).wrapping_add(1),
        BlockDir::Dec => cpu.regs.get_reg_8(RegName8::C).wrapping_sub(1),
    };
    // k_carry from (HL) + ( C (+ or -) 1) & 0xFF
    let (k, k_carry) = c.overflowing_add(src);
    flags |= bool_to_u8(k_carry) * (FLAG_CARRY | FLAG_HALF_CARRY);
    // Parity of (k & 7) xor B is PV flag
    flags |= PARITY_TABLE[((k & 0x07) ^ b) as usize];
    cpu.regs.set_flags(flags);
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
    let mut flags = 0u8;
    // as in dec b
    flags |= SZF3F5_TABLE[b as usize];
    // 7 bit of output value [(HL)]
    flags |= bool_to_u8((src & 0x80) != 0) * FLAG_SUB;
    // temporary k is L + (HL)
    let (k, k_carry) = l.overflowing_add(src);
    flags |= bool_to_u8(k_carry) * (FLAG_CARRY | FLAG_HALF_CARRY);
    // Parity of (k & 7) xor B is PV flag
    flags |= PARITY_TABLE[((k & 0x07) ^ b) as usize];
    cpu.regs.set_flags(flags);
}
