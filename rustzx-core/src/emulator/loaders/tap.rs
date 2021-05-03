// emulator
use crate::{
    emulator::Emulator,
    utils::{make_word, Clocks},
    z80::*,
    zx::tape::TapeImpl,
    host::Host,
};

pub fn fast_load_tap<H: Host>(emulator: &mut Emulator<H>) {
    // resetting tape pos to beginning.
    emulator.controller.tape.reset_pos_in_block();
    // So, at current moment we at 0x056C in 48K Rom.
    // AF contains some garbage. so we need to swap if wtih A'F'
    emulator.cpu.regs.swap_af_alt();
    // now we have type of block at A and flags before LD-BYTES at F
    let mut f = emulator.cpu.regs.get_flags();
    let mut acc = emulator.cpu.regs.get_acc();
    // variable to store resulting flags
    let mut result_flags;
    // pos relative to block start
    let mut pos = 0;
    // destination address in RAM
    let mut dest = emulator.cpu.regs.get_reg_16(RegName16::IX);
    // remaining length
    let mut length = emulator.cpu.regs.get_reg_16(RegName16::DE);
    // parity accumulator and current byte (h, l) regs
    let (mut parity_acc, mut current_byte) = (0, 0);
    'loader: loop {
        // if we still on block
        if let Some(byte) = emulator.controller.tape.block_byte(pos) {
            // set current byte, shift position and do parity check iteration
            current_byte = byte;
            pos += 1;
            parity_acc ^= current_byte;
            // no bytes left, set A to parity accumulator (works as in ROM)
            // and check parity last time
            if length == 0 {
                acc = parity_acc;
                // consider we CAN have parity error
                result_flags = Some(0);
                // if checksum correct set carry to prevent error
                if acc == 0 {
                    result_flags = Some(FLAG_CARRY);
                }
                break 'loader;
            }
            // block type check, first byte
            if (f & FLAG_ZERO) == 0 {
                acc ^= current_byte;
                // if type wrong
                if acc != 0 {
                    result_flags = Some(0);
                    break 'loader;
                }
                // type check passed, go to next byte;
                f |= FLAG_ZERO;
                continue;
            }
            // LOAD
            if (f & FLAG_CARRY) != 0 {
                emulator.controller.write_internal(dest, current_byte);
            // VERIFY
            } else {
                // check for parity each byte, if this fails - set flags to error state
                acc = emulator.controller.memory.read(dest) ^ current_byte;
                if acc != 0 {
                    result_flags = Some(0);
                    break 'loader;
                }
            }
            // move destination pointer and decrease count of remaining bytes
            dest += 1;
            length -= 1;
        } else {
            // this happens if requested length and provided are not matched
            result_flags = Some(FLAG_ZERO);
            break 'loader;
        }
    }
    // set regs to new state
    emulator.cpu.regs.set_reg_16(RegName16::IX, dest);
    emulator.cpu.regs.set_reg_16(RegName16::DE, length);
    emulator
        .cpu
        .regs
        .set_hl(make_word(parity_acc, current_byte));
    emulator.cpu.regs.set_acc(acc);
    // set new flag, if something changed
    if let Some(new_flags) = result_flags {
        f = new_flags;
        // RET
        opcodes::execute_pop_16(
            &mut emulator.cpu,
            &mut emulator.controller,
            RegName16::PC,
            Clocks(0),
        );
    }
    emulator.cpu.regs.set_flags(f);
    // move to next block
    emulator.controller.tape.next_block();
}
