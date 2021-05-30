use crate::{
    opcode::BlockDir,
    tables::{lookup8_r12, HALF_CARRY_SUB_TABLE, PARITY_TABLE, SZF3F5_TABLE},
    RegName16, RegName8, Z80Bus, FLAG_CARRY, FLAG_F3, FLAG_F5, FLAG_HALF_CARRY, FLAG_PV, FLAG_SIGN,
    FLAG_SUB, FLAG_ZERO, Z80,
};

/// ldi/ldd instruction group
pub fn execute_ldi_ldd(cpu: &mut Z80, bus: &mut impl Z80Bus, dir: BlockDir) {
    let src = bus.read(cpu.regs.get_hl(), 3);
    let bc = cpu.regs.dec_reg_16(RegName16::BC);
    bus.write(cpu.regs.get_de(), src, 3);
    bus.wait_loop(cpu.regs.get_de(), 2);
    match dir {
        BlockDir::Inc => {
            cpu.regs.inc_reg_16(RegName16::HL);
            cpu.regs.inc_reg_16(RegName16::DE);
        }
        BlockDir::Dec => {
            cpu.regs.dec_reg_16(RegName16::HL);
            cpu.regs.dec_reg_16(RegName16::DE);
        }
    }
    let mut flags = cpu.regs.get_flags();
    flags &= !(FLAG_SUB | FLAG_HALF_CARRY | FLAG_PV | FLAG_F3 | FLAG_F5);
    flags |= (bc != 0) as u8 * FLAG_PV;
    let src_plus_a = src.wrapping_add(cpu.regs.get_acc());
    flags |= (src_plus_a & 0x08 != 0) as u8 * FLAG_F3;
    flags |= (src_plus_a & 0x02 != 0) as u8 * FLAG_F5;
    cpu.regs.set_flags(flags);
    // Clocks: <4 + 4> + 3 + 3 + 2 = 16
}

/// cpi/cpd instruction group
pub fn execute_cpi_cpd(cpu: &mut Z80, bus: &mut impl Z80Bus, dir: BlockDir) -> bool {
    let src = bus.read(cpu.regs.get_hl(), 3);
    bus.wait_loop(cpu.regs.get_hl(), 5);
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL),
    };
    let bc = cpu.regs.dec_reg_16(RegName16::BC);
    let acc = cpu.regs.get_acc();
    let tmp = acc.wrapping_sub(src);
    let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
    flags |= FLAG_SUB;
    flags |= (bc != 0) as u8 * FLAG_PV;
    flags |= (tmp == 0) as u8 * FLAG_ZERO;
    flags |= tmp & FLAG_SIGN;
    let lookup = lookup8_r12(acc, src, tmp);
    let half_borrow = HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
    flags |= half_borrow;
    let tmp2 = if half_borrow != 0 {
        tmp.wrapping_sub(1)
    } else {
        tmp
    };
    flags |= ((tmp2 & 0x08) != 0) as u8 * FLAG_F3;
    flags |= ((tmp2 & 0x02) != 0) as u8 * FLAG_F5;
    cpu.regs.set_flags(flags);
    // Clocks: <4 + 4> + 3 + 5 = 16
    tmp == 0
}

/// ini/ind instruction group
pub fn execute_ini_ind(cpu: &mut Z80, bus: &mut impl Z80Bus, dir: BlockDir) {
    bus.wait_no_mreq(cpu.regs.get_ir(), 1);
    let src = bus.read_io(cpu.regs.get_bc());
    bus.write(cpu.regs.get_hl(), src, 3);
    let b = cpu.regs.dec_reg_8(RegName8::B);
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL),
    };
    let mut flags = 0u8;
    flags |= SZF3F5_TABLE[b as usize];
    flags |= ((src & 0x80) != 0) as u8 * FLAG_SUB;
    let c = match dir {
        BlockDir::Inc => cpu.regs.get_reg_8(RegName8::C).wrapping_add(1),
        BlockDir::Dec => cpu.regs.get_reg_8(RegName8::C).wrapping_sub(1),
    };
    // (HL) + ( C (+ or -) 1) & 0xFF
    let (k, k_carry) = c.overflowing_add(src);
    flags |= k_carry as u8 * (FLAG_CARRY | FLAG_HALF_CARRY);
    // Parity of (k & 7) xor B is PV flag
    flags |= PARITY_TABLE[((k & 0x07) ^ b) as usize];
    cpu.regs.set_flags(flags);
}

/// outi/outd instruction group
pub fn execute_outi_outd(cpu: &mut Z80, bus: &mut impl Z80Bus, dir: BlockDir) {
    bus.wait_no_mreq(cpu.regs.get_ir(), 1);
    let src = bus.read(cpu.regs.get_hl(), 3);
    let b = cpu.regs.dec_reg_8(RegName8::B);
    bus.write_io(cpu.regs.get_bc(), src);
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL),
    };
    let l = cpu.regs.get_l();
    let mut flags = 0u8;
    flags |= SZF3F5_TABLE[b as usize];
    flags |= ((src & 0x80) != 0) as u8 * FLAG_SUB;
    // L + (HL)
    let (k, k_carry) = l.overflowing_add(src);
    flags |= k_carry as u8 * (FLAG_CARRY | FLAG_HALF_CARRY);
    // Parity of (k & 7) xor B is PV flag
    flags |= PARITY_TABLE[((k & 0x07) ^ b) as usize];
    cpu.regs.set_flags(flags);
}
