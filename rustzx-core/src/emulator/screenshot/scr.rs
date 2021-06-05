use crate::{
    emulator::Emulator,
    error::ScreenLoadError,
    host::{Host, LoadableAsset, SeekFrom, SeekableAsset},
    zx::memory::Page,
    Result,
};
use rustzx_z80::CodeGenerator;

const PRIMARY_SCREEN_MEMORY_SIZE: usize = 6912;

/// Loads `*.scr` screenshot file
pub fn load<H, A>(emulator: &mut Emulator<H>, mut asset: A) -> Result<()>
where
    H: Host,
    A: LoadableAsset + SeekableAsset,
{
    const SCREEN_ADDR: u16 = 0x4000;
    const LOOP_ADDR: u16 = 0x8000;

    let file_size = asset.seek(SeekFrom::End(0))?;

    if file_size != PRIMARY_SCREEN_MEMORY_SIZE {
        return Err(ScreenLoadError::InvalidScrFile.into());
    }

    asset.seek(SeekFrom::Start(0))?;

    let bank = match emulator.controller.memory.get_page(SCREEN_ADDR) {
        Page::Ram(page) => page,
        Page::Rom(_) => {
            // Machine with such memory map is not implemented yet
            return Err(ScreenLoadError::MachineNotSupported.into());
        }
    };

    // Generate infinite loop and jump to it
    CodeGenerator::new(&mut emulator.controller)
        .codegen_set_addr(LOOP_ADDR)
        .jump(LOOP_ADDR);
    emulator.cpu.regs.set_pc(LOOP_ADDR);

    // Directly load screen memory from the asset
    let memory = emulator.controller.memory.ram_page_data_mut(bank);
    asset.read_exact(&mut memory[..PRIMARY_SCREEN_MEMORY_SIZE])?;

    // Update screen
    emulator.controller.refresh_memory_dependent_devices();

    Ok(())
}
