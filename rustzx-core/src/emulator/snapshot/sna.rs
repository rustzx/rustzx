use crate::{
    emulator::Emulator,
    error::IoError,
    host::{DataRecorder, Host, LoadableAsset, SeekFrom, SeekableAsset},
    utils::{make_word, split_word, Clocks},
    z80::{
        opcodes::{execute_pop_16, execute_push_16},
        RegName16,
    },
    zx::{machine::ZXMachine, video::colors::ZXColor},
    Result,
};

const SNA_HEADER_SIZE: usize = 27;
const SNA_128K_SECONDARY_HEADER_SIZE: usize = 4;
const SNA_48K_SIZE: usize = 49179;
const SNA_128K_SECONDARY_HEADER_OFFSET: usize = SNA_48K_SIZE;
const SNA_128K_SECONDARY_TAIL_BANKS_OFFSET: usize = 49183;
const SNA_128K_PERSISTENT_BANK_0: u8 = 5;
const SNA_128K_PERSISTENT_BANK_1: u8 = 2;
const SNA_IFF2_BIT_MASK: u8 = 0x04;
const SNA_INTERRUPT_MODE_MASK: u8 = 0x03;
const SNA_BORDER_COLOR_MASK: u8 = 0x07;
const SNA_128K_TAIL_BANKS: &[u8] = &[0, 1, 3, 4, 6, 7];
const SNA_PAGINATED_PAGED_BANK_ADDRESS: u16 = 0xFFFF;
const SNA_48K_RAM_PAGES_COUNT: u8 = 3;

/// SNA snapshot loading function
pub fn load<H: Host>(emulator: &mut Emulator<H>, mut asset: H::SnapshotAsset) -> Result<()> {
    let size = asset.seek(SeekFrom::End(0))?;
    asset.seek(SeekFrom::Start(0))?;

    let is_128k = size > SNA_48K_SIZE;

    if !is_128k && size < SNA_48K_SIZE {
        return Err(IoError::UnexpectedEof.into());
    }

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
    // iff2, iff1
    let iff = (header[19] & SNA_IFF2_BIT_MASK) != 0;
    emulator.cpu.regs.set_iff1(iff);
    emulator.cpu.regs.set_iff2(iff);
    // r
    emulator.cpu.regs.set_r(header[20]);
    // af
    emulator.cpu.regs.set_af(make_word(header[22], header[21]));
    // sp
    emulator.cpu.regs.set_sp(make_word(header[24], header[23]));
    // interrupt mode
    emulator.cpu.set_im(header[25] & SNA_INTERRUPT_MODE_MASK);
    // Border color
    emulator.controller.set_border_color(
        Clocks(0),
        ZXColor::from_bits(header[26] & SNA_BORDER_COLOR_MASK),
    );
    if is_128k {
        // PC, 7ffd port, trdos pagination status
        let mut tmp = [0u8; SNA_128K_SECONDARY_HEADER_SIZE];
        asset.seek(SeekFrom::Start(SNA_128K_SECONDARY_HEADER_OFFSET))?;
        asset.read_exact(&mut tmp)?;
        emulator.cpu.regs.set_pc(make_word(tmp[1], tmp[0]));
        let port_7ffd = tmp[2];
        let _trdos_paged = tmp[3];
        // This will alsto setup required memory map before banks restore
        emulator.controller.write_7ffd(port_7ffd);

        // Go to the previous position
        asset.seek(SeekFrom::Start(SNA_HEADER_SIZE))?;
        let paginated_bank = match emulator
            .controller
            .memory
            .get_page(SNA_PAGINATED_PAGED_BANK_ADDRESS)
        {
            crate::zx::memory::Page::Ram(bank) => bank,
            crate::zx::memory::Page::Rom(_) => 0,
        };

        // write 3 head banks
        let head_banks = &[
            SNA_128K_PERSISTENT_BANK_0,
            SNA_128K_PERSISTENT_BANK_1,
            paginated_bank,
        ];
        for bank in head_banks {
            let page = emulator.controller.memory.ram_page_data_mut(*bank);
            asset.read_exact(page)?;
        }

        // tail banks
        asset.seek(SeekFrom::Start(SNA_128K_SECONDARY_TAIL_BANKS_OFFSET))?;
        for bank in SNA_128K_TAIL_BANKS {
            if *bank == paginated_bank {
                continue;
            }
            let page = emulator.controller.memory.ram_page_data_mut(*bank);
            asset.read_exact(page)?;
        }
    } else {
        for page_index in 0..SNA_48K_RAM_PAGES_COUNT {
            let page = emulator.controller.memory.ram_page_data_mut(page_index);
            asset.read_exact(page)?;
        }

        // Perform RET as 48K sna snapshot stores it in the machine stack
        execute_pop_16(
            &mut emulator.cpu,
            &mut emulator.controller,
            RegName16::PC,
            Clocks(0),
        );
    }

    // Refresh screen and other memory-dependent peripheral
    emulator.controller.refresh_memory_dependent_devices();

    Ok(())
}

/// Helper class to place emulator in the state required for
/// snapshoting and return to normal state afterwards
struct ScopedSnapshotState<'a, H: Host> {
    pub emulator: &'a mut Emulator<H>,
    pub is_48k: bool,
}

impl<'a, H: Host> ScopedSnapshotState<'a, H> {
    fn enter(emulator: &'a mut Emulator<H>) -> Self {
        let is_48k = emulator.settings.machine == ZXMachine::Sinclair48K;
        if is_48k {
            execute_push_16(
                &mut emulator.cpu,
                &mut emulator.controller,
                RegName16::PC,
                Clocks(0),
            );
        }

        Self { emulator, is_48k }
    }
}

impl<'a, H: Host> Drop for ScopedSnapshotState<'a, H> {
    fn drop(&mut self) {
        if self.is_48k {
            execute_pop_16(
                &mut self.emulator.cpu,
                &mut self.emulator.controller,
                RegName16::PC,
                Clocks(0),
            );
        }
    }
}

pub fn save<H: Host>(emulator: &mut Emulator<H>, mut recorder: H::SnapshotRecorder) -> Result<()> {
    let state = ScopedSnapshotState::enter(emulator);
    let ScopedSnapshotState { emulator, is_48k } = &state;

    let mut header = [0u8; SNA_HEADER_SIZE];
    // interrupt register
    header[0] = emulator.cpu.regs.get_i();
    // alt register pairs
    header[1] = emulator.cpu.regs.get_l_alt();
    header[2] = emulator.cpu.regs.get_h_alt();
    header[3] = emulator.cpu.regs.get_e_alt();
    header[4] = emulator.cpu.regs.get_d_alt();
    header[5] = emulator.cpu.regs.get_c_alt();
    header[6] = emulator.cpu.regs.get_b_alt();
    header[7] = emulator.cpu.regs.get_flags_alt();
    header[8] = emulator.cpu.regs.get_acc_alt();
    // hl, de, bc, iy, ix
    header[9] = emulator.cpu.regs.get_l();
    header[10] = emulator.cpu.regs.get_h();
    header[11] = emulator.cpu.regs.get_e();
    header[12] = emulator.cpu.regs.get_d();
    header[13] = emulator.cpu.regs.get_c();
    header[14] = emulator.cpu.regs.get_b();
    let (iyh, iyl) = split_word(emulator.cpu.regs.get_iy());
    header[15] = iyl;
    header[16] = iyh;
    let (ixh, ixl) = split_word(emulator.cpu.regs.get_ix());
    header[17] = ixl;
    header[18] = ixh;
    // iff2
    if emulator.cpu.regs.get_iff2() {
        header[19] = SNA_IFF2_BIT_MASK;
    }
    // r
    header[20] = emulator.cpu.regs.get_r();
    // AF
    header[21] = emulator.cpu.regs.get_flags();
    header[22] = emulator.cpu.regs.get_acc();
    // SP
    let (sph, spl) = split_word(emulator.cpu.regs.get_sp());
    header[23] = spl;
    header[24] = sph;
    // Interrupt mode
    header[25] = emulator.cpu.get_im().into();
    // Border color
    header[26] = emulator.controller.border_color.into();

    recorder.write_all(&header)?;

    if *is_48k {
        for page_index in 0..SNA_48K_RAM_PAGES_COUNT {
            let page = emulator.controller.memory.ram_page_data(page_index);
            recorder.write_all(page)?;
        }
    } else {
        let paginated_bank = match emulator
            .controller
            .memory
            .get_page(SNA_PAGINATED_PAGED_BANK_ADDRESS)
        {
            crate::zx::memory::Page::Ram(bank) => bank,
            crate::zx::memory::Page::Rom(_) => 0,
        };
        let head_banks = &[
            SNA_128K_PERSISTENT_BANK_0,
            SNA_128K_PERSISTENT_BANK_1,
            paginated_bank,
        ];
        for bank in head_banks {
            let page = emulator.controller.memory.ram_page_data(*bank);
            recorder.write_all(page)?;
        }

        // PC, 7ffd, trdos
        let (pch, pcl) = split_word(emulator.cpu.regs.get_pc());
        let port_7ffd = emulator.controller.read_7ffd();
        let trdos_paged = 0x00;
        recorder.write_all(&[pcl, pch, port_7ffd, trdos_paged])?;

        // remaining banks
        for bank in SNA_128K_TAIL_BANKS {
            if *bank == paginated_bank {
                continue;
            }
            let page = emulator.controller.memory.ram_page_data(*bank);
            recorder.write_all(page)?;
        }
    }

    Ok(())
}
