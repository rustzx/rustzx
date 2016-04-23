use z80::*;

// push 16 bit value to stack. Clocks for each byte
pub fn execute_push_16(cpu: &mut Z80, bus: &mut Z80Bus, reg: RegName16, clk: Clocks) {
    let data = cpu.regs.get_reg_16(reg);
    bus.write_word(cpu.regs.dec_sp(2), data, clk);
}
// pop 16 bit value from stack. Clocks for each byte
pub fn execute_pop_16(cpu: &mut Z80, bus: &mut Z80Bus, reg: RegName16, clk: Clocks) {
    let data = bus.read_word(cpu.regs.get_sp(), clk);
    cpu.regs.inc_sp(2);
    cpu.regs.set_reg_16(reg, data);
}
