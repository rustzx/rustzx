use z80::*;
use utils::{split_word, make_word};
// push 16 bit value to stack. Clocks for each byte
pub fn execute_push_16(cpu: &mut Z80, bus: &mut Z80Bus, reg: RegName16, clk: Clocks) {
    // h then l
    let (h, l) = split_word(cpu.regs.get_reg_16(reg));
    bus.write(cpu.regs.dec_sp(1), h, clk);
    bus.write(cpu.regs.dec_sp(1), l, clk);
    //bus.write_word(cpu.regs.dec_sp(2), data, clk);
}
// pop 16 bit value from stack. Clocks for each byte
pub fn execute_pop_16(cpu: &mut Z80, bus: &mut Z80Bus, reg: RegName16, clk: Clocks) {
    let (h, l);
    l = bus.read(cpu.regs.get_sp(), clk);
    cpu.regs.inc_sp(1);
    h = bus.read(cpu.regs.get_sp(), clk);
    cpu.regs.inc_sp(1);
    cpu.regs.set_reg_16(reg, make_word(h, l));
}
