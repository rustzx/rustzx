// SZX support by Arjun
use crate::{
    emulator::Emulator,
    error::{IoError, SnapshotLoadError},
    host::{DataRecorder, Host, LoadableAsset, SeekFrom, SeekableAsset},
    zx::{machine::ZXMachine, video::colors::ZXColor},
    Result,
};
use std::{println, str::from_utf8};

pub enum ZxType {
    ZXSTMID_16K = 0,
    ZXSTMID_48K,
    ZXSTMID_128K,
    ZXSTMID_PLUS2,
    ZXSTMID_PLUS2A,
    ZXSTMID_PLUS3,
    ZXSTMID_PLUS3E,
    ZXSTMID_PENTAGON128,
    ZXSTMID_TC2048,
    ZXSTMID_TC2068,
    ZXSTMID_SCORPION,
    ZXSTMID_SE,
    ZXSTMID_TS2068,
    ZXSTMID_PENTAGON512,
    ZXSTMID_PENTAGON1024,
    ZXSTMID_NTSC48K,
    ZXSTMID_128KE,
}

pub const ZXSTZF_EILAST: u32 = 1;
pub const ZXSTZF_HALTED: u32 = 2;
pub const ZXSTRF_COMPRESSED: u32 = 1;
pub const ZXSTKF_ISSUE2: u32 = 1;
pub const ZXSTMF_ALTERNATETIMINGS: u32 = 1;
pub const SZX_VERSION_SUPPORTED_MAJOR: u32 = 1;
pub const SZX_VERSION_SUPPORTED_MINOR: u32 = 5;
pub const ZXST_HEADER_SIZE: usize = 8; // The zx-state header
pub const ZXST_BLOCK_HEADER_SIZE: usize = 8; // The header for each block
/// SZX snapshot loading function
pub fn load<H, A>(emulator: &mut Emulator<H>, mut asset: A) -> Result<()>
where
    H: Host,
    A: LoadableAsset + SeekableAsset,
{
    let size = asset.seek(SeekFrom::End(0))?;
    let mut cursor_pos = 0;
    asset.seek(SeekFrom::Start(0))?;

    // ZXST Header
    let mut header = [0u8; ZXST_HEADER_SIZE];
    asset.read_exact(&mut header)?;
    cursor_pos += ZXST_HEADER_SIZE;
    let magic_bytes = &[header[0], header[1], header[2], header[3]];
    let magic_str = from_utf8(magic_bytes).unwrap();
    if !magic_str.eq("ZXST") {
        return Err(SnapshotLoadError::InvalidSZXFile.into());
    }
    let major_version = header[4];
    let minor_version = header[5];
    let machine_id = header[6];
    let flags = header[7];
    println!("SZX version: {major_version}.{minor_version}");

    // ZXST Block Header
    asset.seek(SeekFrom::Start(cursor_pos))?;
    let mut block_header = [0u8; ZXST_BLOCK_HEADER_SIZE];
    while !asset.read_exact(&mut block_header).is_err() {
        let id: u32 = u32::from_le_bytes([
            block_header[0],
            block_header[1],
            block_header[2],
            block_header[3],
        ]);
        let size: u32 = u32::from_le_bytes([
            block_header[4],
            block_header[5],
            block_header[6],
            block_header[7],
        ]);
        let crtr_bytes = &[
            block_header[0],
            block_header[1],
            block_header[2],
            block_header[3],
        ];
        let id_str = from_utf8(crtr_bytes).unwrap().to_uppercase();
        println!("Block id: {id_str}, size: {size}");
        cursor_pos += ZXST_BLOCK_HEADER_SIZE;

        // ZXST Block Data
        asset.seek(SeekFrom::Start(cursor_pos))?;
        let mut block_data = vec![0; size as usize];

        asset.read_exact(&mut block_data)?;
        match id_str.as_str() {
            "CRTR" => {
                let crtr_name_bytes = &block_data[0..33];
                let crtr_str = from_utf8(crtr_name_bytes).unwrap();
                println!("\tCreator name: {crtr_str}");
                let major_version = u16::from_le_bytes([block_data[33], block_data[34]]);
                let minor_version = u16::from_le_bytes([block_data[35], block_data[36]]);
                println!("\tCreator version: {major_version}.{minor_version}");
            }
            "Z80R" => {
                // AF
                emulator
                    .cpu
                    .regs
                    .set_af(u16::from_le_bytes([block_data[0], block_data[1]]));

                // BC
                emulator
                    .cpu
                    .regs
                    .set_bc(u16::from_le_bytes([block_data[2], block_data[3]]));

                // DE
                emulator
                    .cpu
                    .regs
                    .set_de(u16::from_le_bytes([block_data[4], block_data[5]]));

                // HL
                emulator
                    .cpu
                    .regs
                    .set_hl(u16::from_le_bytes([block_data[6], block_data[7]]));

                // Set alternate regs by swapping with main.
                // AF1
                emulator.cpu.regs.swap_af_alt();
                emulator
                    .cpu
                    .regs
                    .set_af(u16::from_le_bytes([block_data[8], block_data[9]]));
                emulator.cpu.regs.swap_af_alt();

                emulator.cpu.regs.exx();
                // BC1
                emulator
                    .cpu
                    .regs
                    .set_bc(u16::from_le_bytes([block_data[10], block_data[11]]));

                // DE1
                emulator
                    .cpu
                    .regs
                    .set_de(u16::from_le_bytes([block_data[12], block_data[13]]));

                // HL1
                emulator
                    .cpu
                    .regs
                    .set_hl(u16::from_le_bytes([block_data[14], block_data[15]]));

                emulator.cpu.regs.exx();

                // IX
                emulator
                    .cpu
                    .regs
                    .set_ix(u16::from_le_bytes([block_data[16], block_data[17]]));

                // IY
                emulator
                    .cpu
                    .regs
                    .set_iy(u16::from_le_bytes([block_data[18], block_data[19]]));

                // SP
                emulator
                    .cpu
                    .regs
                    .set_sp(u16::from_le_bytes([block_data[20], block_data[21]]));

                // PC
                emulator
                    .cpu
                    .regs
                    .set_pc(u16::from_le_bytes([block_data[22], block_data[23]]));

                // I
                emulator.cpu.regs.set_i(block_data[24]);

                // R
                emulator.cpu.regs.set_r(block_data[25]);

                // IFF1
                emulator.cpu.regs.set_iff1(block_data[26] > 0);

                // IFF2
                emulator.cpu.regs.set_iff2(block_data[27] > 0);

                // IM
                emulator.cpu.set_im(block_data[24]);

                // dwCyclesStart
                emulator.controller.frame_clocks = u32::from_le_bytes([
                    block_data[25],
                    block_data[26],
                    block_data[27],
                    block_data[28],
                ]) as usize;

                // chHoldIntReqCycles
                // Ignored block_data 29

                // chFlags
                emulator.cpu.skip_interrupt = (block_data[30] as u32) & ZXSTZF_EILAST != 0;
                emulator.cpu.halted = (block_data[30] as u32) & ZXSTZF_HALTED != 0;
            }
            _ => (),
        }
        // skip block data
        cursor_pos += size as usize;

        asset.seek(SeekFrom::Start(cursor_pos))?;
    }
    println!("SZX file ended.");
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
            emulator.cpu.push_pc_to_stack(&mut emulator.controller);
        }

        Self { emulator, is_48k }
    }
}

impl<'a, H: Host> Drop for ScopedSnapshotState<'a, H> {
    fn drop(&mut self) {
        if self.is_48k {
            self.emulator
                .cpu
                .pop_pc_from_stack(&mut self.emulator.controller);
        }
    }
}

pub fn save<H, R>(emulator: &mut Emulator<H>, mut recorder: R) -> Result<()>
where
    H: Host,
    R: DataRecorder,
{
    Ok(())
}
