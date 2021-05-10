use crate::{
    emulator::Emulator,
    host::{Host, LoadableAsset},
    utils::{make_word, Clocks},
    z80::{opcodes::execute_pop_16, RegName16},
    zx::video::colors::ZXColor,
    Result,
};

/// SNA snapshot loading function
pub fn load_sna<H: Host>(emulator: &mut Emulator<H>, mut asset: H::SnapshotAsset) -> Result<()> {
    const SNA_HEADER_SIZE: usize = 27;
    let mut header = [0u8; SNA_HEADER_SIZE];
    asset.read_exact(&mut header)?;

    // i-reg
    emulator.cpu.regs.set_i(header[0]);
    // alt-regs
    emulator.cpu.regs.set_hl(make_word(header[2], header[1]));
    emulator.cpu.regs.set_de(make_word(header[4], header[3]));
    emulator.cpu.regs.set_bc(make_word(header[6], header[5]));
    emulator.cpu.regs.exx();
    // af'
    emulator.cpu.regs.set_af(make_word(header[8], header[7]));
    emulator.cpu.regs.swap_af_alt();
    // regs
    emulator.cpu.regs.set_hl(make_word(header[10], header[9]));
    emulator.cpu.regs.set_de(make_word(header[12], header[11]));
    emulator.cpu.regs.set_bc(make_word(header[14], header[13]));
    // index regs
    emulator.cpu.regs.set_iy(make_word(header[16], header[15]));
    emulator.cpu.regs.set_ix(make_word(header[18], header[17]));
    // iff1, iff2
    let iff = (header[19] & 0x04) != 0;
    emulator.cpu.regs.set_iff1(iff);
    emulator.cpu.regs.set_iff2(iff);
    // r
    emulator.cpu.regs.set_r(header[20]);
    // af
    emulator.cpu.regs.set_af(make_word(header[22], header[21]));
    // sp
    emulator.cpu.regs.set_sp(make_word(header[24], header[23]));
    // interrupt mode
    emulator.cpu.set_im(header[25] & 0x03);
    emulator
        .controller
        .set_border_color(Clocks(0), ZXColor::from_bits(header[26] & 0x07));
    // ram pages
    for page_index in 0..3 {
        let page = emulator.controller.memory.ram_page_data_mut(page_index);
        asset.read_exact(page)?;
    }

    // RET
    execute_pop_16(
        &mut emulator.cpu,
        &mut emulator.controller,
        RegName16::PC,
        Clocks(0),
    );
    Ok(())
}
