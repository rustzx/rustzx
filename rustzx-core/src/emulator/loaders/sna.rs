use crate::{
    emulator::Emulator,
    host::{Host, LoadableAsset},
    utils::{make_word, Clocks},
    z80::{opcodes::execute_pop_16, RegName16},
    zx::colors::ZXColor,
    Result,
};
use alloc::vec::Vec;

/// SNA snapshot loading function
pub fn load_sna<H: Host>(emulator: &mut Emulator<H>, mut asset: H::SnapshotAsset) -> Result<()> {
    // TODO(#54): Eliminate loading a whole file to vector in sna loader
    let mut data = Vec::new();
    asset.read_to_end(&mut data)?;
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
    let iff = (data[19] & 0x04) != 0;
    emulator.cpu.regs.set_iff1(iff);
    emulator.cpu.regs.set_iff2(iff);
    // r
    emulator.cpu.regs.set_r(data[20]);
    // af
    emulator.cpu.regs.set_af(make_word(data[22], data[21]));
    // sp
    emulator.cpu.regs.set_sp(make_word(data[24], data[23]));
    // interrupt mode
    emulator.cpu.set_im(data[25] & 0x03);
    // set border
    emulator
        .controller
        .border
        .set_border(Clocks(0), ZXColor::from_bits(data[26] & 0x07));
    // ram pages
    let page = emulator.controller.memory.ram_page_data_mut(0);
    page.copy_from_slice(&data[27..16411]);
    let page = emulator.controller.memory.ram_page_data_mut(1);
    page.copy_from_slice(&data[16411..32795]);
    let page = emulator.controller.memory.ram_page_data_mut(2);
    page.copy_from_slice(&data[32795..49179]);

    // RET
    execute_pop_16(
        &mut emulator.cpu,
        &mut emulator.controller,
        RegName16::PC,
        Clocks(0),
    );
    Ok(())
}
