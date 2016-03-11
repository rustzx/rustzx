use super::*;
use z80::*;
use utils::*;

/// ldi or ldd instruction
pub fn execute_ldi_ldd(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) {
    // read (HL)
    let src = bus.read(cpu.regs.get_hl());
    // write (HL) to (DE)
    bus.write(cpu.regs.get_de(), src);
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
    // dec BC
    let bc = cpu.regs.dec_reg_16(RegName16::BC, 1);
    // flags
    cpu.regs.set_flag(Flag::Sub, false);
    cpu.regs.set_flag(Flag::HalfCarry, false);
    cpu.regs.set_flag(Flag::ParityOveflow, bc != 0);
    let src_plus_a = src.wrapping_add(cpu.regs.get_acc());
    cpu.regs.set_flag(Flag::F3, (src_plus_a & 0b1000) != 0);
    cpu.regs.set_flag(Flag::F5, (src_plus_a & 0b10) != 0);
}

/// cpi or cpd instruction
pub fn execute_cpi_cpd(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) -> bool {
    // read (HL)
    let src = bus.read(cpu.regs.get_hl());
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
    let half_borrow = half_borrow_8(acc, src);
    cpu.regs.set_flag(Flag::HalfCarry, half_borrow);
    let tmp2 = if half_borrow {
        tmp.wrapping_sub(1)
    } else {
        tmp
    };
    cpu.regs.set_flag(Flag::F3, (tmp2 & 0b1000) != 0);
    cpu.regs.set_flag(Flag::F5, (tmp2 & 0b10) != 0);
    tmp == 0
}

/// ini or ind instruction
pub fn execute_ini_ind(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) {
    // get input data
    let src = bus.read_io(cpu.regs.get_bc());
    // write to memory
    bus.write(cpu.regs.get_hl(), src);
    // dec b
    let b = cpu.regs.dec_reg_8(RegName8::B, 1);
    // move pointer
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL, 1),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL, 1),
    };
    // flags
    cpu.regs.set_flag(Flag::Zero, b == 0);
    cpu.regs.set_flag(Flag::Sign, (b & 0x80) != 0);
    cpu.regs.set_flag(Flag::F3, (b & 0b1000) != 0);
    cpu.regs.set_flag(Flag::F5, (b & 0b10) != 0);
    cpu.regs.set_flag(Flag::Sub, (src & 0x80) != 0);
    let c = cpu.regs.get_reg_8(RegName8::C);
    let cc = match dir {
        BlockDir::Inc => c.wrapping_add(1),
        BlockDir::Dec => c.wrapping_sub(1),
    };
    let (_, carry) = cc.overflowing_add(src);
    cpu.regs.set_flag(Flag::Carry, carry);
    cpu.regs.set_flag(Flag::HalfCarry, carry);
    // and now most hard. P/V flag :D
    // at first, build "Temp1"
    let temp1_operands = (bool_to_u8(bit(c, 1)) << 3) |
                         (bool_to_u8(bit(c, 0)) << 2) |
                         (bool_to_u8(bit(src, 1)) << 1) |
                         (bool_to_u8(bit(src, 0)));
    // obtain temp1
    let temp1 = match dir {
        BlockDir::Inc => tables::IO_INC_TEMP1[temp1_operands as usize] != 0,
        BlockDir::Dec => tables::IO_DEC_TEMP1[temp1_operands as usize] != 0,
    };
    // TODO: rewrite as table, described in Z80 Undocumended documented
    let temp2 = if (b & 0x0F) == 0 {
        (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 4) | (bit(b, 6) & (!bit(b, 5))))
    } else {
        (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 0) | (bit(b, 2) & (!bit(b, 1))))
    };
    cpu.regs.set_flag(Flag::ParityOveflow, temp1 ^ temp2 ^ bit(c, 2) ^ bit(src, 2));
    // Oh, this was pretty hard.
    // TODO: place pv falag detection in separate function
}

/// outi or outd instruction
pub fn execute_outi_outd(cpu: &mut Z80, bus: &mut Z80Bus, dir: BlockDir) {
    // get input data
    let src = bus.read(cpu.regs.get_hl());
    // acсording to the official docs, b decrements before moving it to the addres bus
    // dec b
    let b = cpu.regs.dec_reg_8(RegName8::B, 1);
    bus.write_io(cpu.regs.get_bc(), src);
    // move pointer
    match dir {
        BlockDir::Inc => cpu.regs.inc_reg_16(RegName16::HL, 1),
        BlockDir::Dec => cpu.regs.dec_reg_16(RegName16::HL, 1),
    };
    // flags
    cpu.regs.set_flag(Flag::Zero, b == 0);
    cpu.regs.set_flag(Flag::Sign, (b & 0x80) != 0);
    cpu.regs.set_flag(Flag::F3, (b & 0b1000) != 0);
    cpu.regs.set_flag(Flag::F5, (b & 0b10) != 0);
    cpu.regs.set_flag(Flag::Sub, (src & 0x80) != 0);
    let c = cpu.regs.get_reg_8(RegName8::C);
    let cc = match dir {
        BlockDir::Inc => c.wrapping_add(1),
        BlockDir::Dec => c.wrapping_sub(1),
    };
    let (_, carry) = cc.overflowing_add(src);
    cpu.regs.set_flag(Flag::Carry, carry);
    cpu.regs.set_flag(Flag::HalfCarry, carry);
    // and now most hard. P/V flag :D
    // at first, build "Temp1"
    let temp1_operands = (bool_to_u8(bit(c, 1)) << 3) |
                         (bool_to_u8(bit(c, 0)) << 2) |
                         (bool_to_u8(bit(src, 1)) << 1) |
                         (bool_to_u8(bit(src, 0)));
    // obtain temp1
    let temp1 = match dir {
        BlockDir::Inc => tables::IO_INC_TEMP1[temp1_operands as usize] != 0,
        BlockDir::Dec => tables::IO_DEC_TEMP1[temp1_operands as usize] != 0,
    };
    // TODO: rewrite as table, described in Z80 Undocumended documented
    let temp2 = if (b & 0x0F) == 0 {
        (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 4) | (bit(b, 6) & (!bit(b, 5))))
    } else {
        (tables::PARITY_BIT[b as usize] != 0) ^ (bit(b, 0) | (bit(b, 2) & (!bit(b, 1))))
    };
    cpu.regs.set_flag(Flag::ParityOveflow, temp1 ^ temp2 ^ bit(c, 2) ^ bit(src, 2));
    // Oh, this was pretty hard.
    // TODO: place pv falag detection in separate function
}
