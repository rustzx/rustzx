use crate::utils::{make_word, split_word, Clocks};
use crate::z80::*;

/// Pushes 16 bit value to the stack. Clocks count using for each byte write
pub fn execute_push_16(cpu: &mut Z80, bus: &mut dyn Z80Bus, reg: RegName16, clk: Clocks) {
    // h then l
    let (h, l) = split_word(cpu.regs.get_reg_16(reg));
    bus.write(cpu.regs.dec_sp(1), h, clk);
    bus.write(cpu.regs.dec_sp(1), l, clk);
}

/// Pops 16 bit value from the stack. Clocks count using for each byte read
pub fn execute_pop_16(cpu: &mut Z80, bus: &mut dyn Z80Bus, reg: RegName16, clk: Clocks) {
    let (h, l);
    l = bus.read(cpu.regs.get_sp(), clk);
    cpu.regs.inc_sp(1);
    h = bus.read(cpu.regs.get_sp(), clk);
    cpu.regs.inc_sp(1);
    cpu.regs.set_reg_16(reg, make_word(h, l));
}
