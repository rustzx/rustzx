// std
use std::io::Read;
use std::fs::File;
use std::path::Path;
// emulator
use emulator::Emulator;
use z80::opcodes::execute_pop_16;
use z80::RegName16;
use utils::{Clocks, make_word};
use zx::colors::ZXColor;

/// SNA snapshot loading function
pub fn load_sna(emulator: &mut Emulator, file: impl AsRef<Path>) {
    let mut data = Vec::new();
    File::open(file).unwrap().read_to_end(&mut data).unwrap();
    assert!(data.len() == 49179);
    // i-reg
    emulator.cpu.regs.set_i(data[0]);
    // alt-regs
    emulator.cpu.regs.set_hl(make_word(data[2], data[1]));
    emulator.cpu.regs.set_de(make_word(data[4], data[3]));
    emulator.cpu.regs.set_bc(make_word(data[6], data[5]));
    emulator.cpu.regs.exx();
    // af'
    emulator.cpu.regs.set_af(make_word(data[8], data[7]));
    emulator.cpu.regs.swap_af_alt();
    // regs
    emulator.cpu.regs.set_hl(make_word(data[10], data[9]));
    emulator.cpu.regs.set_de(make_word(data[12], data[11]));
    emulator.cpu.regs.set_bc(make_word(data[14], data[13]));
    // index regs
    emulator.cpu.regs.set_iy(make_word(data[16], data[15]));
    emulator.cpu.regs.set_ix(make_word(data[18], data[17]));
    // iff1, iff2
    emulator.cpu.regs.set_iff1((data[19] & 0x01) != 0);
    emulator.cpu.regs.set_iff1((data[19] & 0x04) != 0);
    // r
    emulator.cpu.regs.set_r(data[20]);
    // af
    emulator.cpu.regs.set_af(make_word(data[22], data[21]));
    // sp
    emulator.cpu.regs.set_sp(make_word(data[24], data[23]));
    // interrupt mode
    emulator.cpu.set_im(data[25]);
    // set border
    emulator.controller.border.set_border(Clocks(0), ZXColor::from_bits(data[26]));
    // ram pages
    emulator.controller.memory.load_ram(0, &data[27..16411]);
    // validate screen, it has been changed
    emulator.controller.memory.load_ram(1, &data[16411..32795]);
    emulator.controller.memory.load_ram(2, &data[32795..49179]);
    // RET
    execute_pop_16(&mut emulator.cpu,
                   &mut emulator.controller,
                   RegName16::PC,
                   Clocks(0));
}
