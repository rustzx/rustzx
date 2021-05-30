use crate::{RegName16, Z80Bus, Z80};

/// Pushes 16 bit value to the stack
pub fn execute_push_16(
    cpu: &mut Z80,
    bus: &mut impl Z80Bus,
    reg: RegName16,
    clocks_half_word_write: usize,
) {
    // h then l
    let [l, h] = cpu.regs.get_reg_16(reg).to_le_bytes();
    bus.write(cpu.regs.dec_sp(), h, clocks_half_word_write);
    bus.write(cpu.regs.dec_sp(), l, clocks_half_word_write);
}

/// Pops 16 bit value from the stack
pub fn execute_pop_16(
    cpu: &mut Z80,
    bus: &mut impl Z80Bus,
    reg: RegName16,
    clocks_half_word_read: usize,
) {
    let (h, l);
    l = bus.read(cpu.regs.get_sp(), clocks_half_word_read);
    cpu.regs.inc_sp();
    h = bus.read(cpu.regs.get_sp(), clocks_half_word_read);
    cpu.regs.inc_sp();
    cpu.regs.set_reg_16(reg, u16::from_le_bytes([l, h]));
}
