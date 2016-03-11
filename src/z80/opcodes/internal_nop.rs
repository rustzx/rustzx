use z80::*;

/// returns clocks, needed for executing NOP instruction
pub fn execute_internal_nop(cpu: &mut Z80) -> u8 {
    cpu.regs.inc_r(1); // inc R register for emulating memory refresh
    tables::CLOCKS_NORMAL[0x00] // return clocks, needed for NOP
}


/// No operation, no interrupts
pub fn execute_internal_noni(cpu: &mut Z80) -> u8 {
    cpu.skip_interrupt = true;
    execute_internal_nop(cpu)
}
