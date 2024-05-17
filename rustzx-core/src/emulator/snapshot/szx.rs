use crate::{
    emulator::Emulator,
    error::{SnapshotLoadError, SnapshotSaveError},
    host::{DataRecorder, Host, LoadableAsset, SeekFrom, SeekableAsset},
    zx::{joy::kempston, mouse::kempston::KempstonMouse, video::colors::ZXColor},
    Result,
};

use alloc::vec::Vec;
use alloc::{str::from_utf8, vec};
#[cfg(feature = "zlib")]
use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;

use rustzx_z80::Z80Bus;

const ZXST_MID_128K: u32 = 2;

const ZXSTZF_EILAST: u32 = 1;
const ZXSTZF_HALTED: u32 = 2;
const ZXSTZF_FSET: u32 = 4;

const ZXSTAYF_128AY: u32 = 2;

const ZXSTKJT_KEMPSTON: u32 = 1;

const ZXSTM_KEMPSTON: u32 = 2;

const ZXSTRF_COMPRESSED: u32 = 1;

const ZXST_HEADER_SIZE: usize = 8; // The zx-state header
const ZXST_BLOCK_HEADER_SIZE: usize = 8; // The header for each block

// Process Creator (CRTR) block
fn process_crtr_block<H: Host>(_: &mut Emulator<H>, block_data: &[u8]) {
    let crtr_name_bytes = &block_data[0..33];
    let _ = from_utf8(crtr_name_bytes).unwrap();
    let _ = u16::from_le_bytes([block_data[33], block_data[34]]);
    let _ = u16::from_le_bytes([block_data[35], block_data[36]]);
}

// Process ZXSTZ80REGS (Z80R) block
fn process_z80r_block<H: Host>(emulator: &mut Emulator<H>, block_data: &[u8]) {
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
    emulator.cpu.set_im(block_data[28]);

    // dwCyclesStart
    emulator.controller.frame_clocks = u32::from_le_bytes([
        block_data[29],
        block_data[30],
        block_data[31],
        block_data[32],
    ]) as usize;

    // chHoldIntReqCycles
    // Ignored block_data 33

    // chFlags
    let flags = block_data[34] as u32;
    emulator.cpu.skip_interrupt = flags & ZXSTZF_EILAST != 0;
    emulator.cpu.halted = flags & ZXSTZF_HALTED != 0;

    if emulator.cpu.halted {
        emulator.cpu.regs.inc_pc();
    }

    // v1.5
    if flags & ZXSTZF_FSET != 0 {
        emulator.cpu.regs.set_q()
    } else {
        emulator.cpu.regs.clear_q()
    };

    // wMemPtr
    emulator
        .cpu
        .regs
        .set_mem_ptr(u16::from_le_bytes([block_data[35], block_data[36]]));
}

// Process ZXSTSPECREGS (SPCR) block
fn process_spcr_block<H: Host>(emulator: &mut Emulator<H>, machine_id: u32, block_data: &[u8]) {
    // ch7ffd
    if machine_id < ZXST_MID_128K {
        emulator.controller.write_7ffd(0); // Always 0 for 16k and 48k
    } else {
        emulator.controller.write_7ffd(block_data[1]);
    }

    // ch1ffd
    // For +2a/+3 and Scorpion models.
    // Only 128 and 48k models supported currently. Skipping block_data[2] (union)

    // chEff7
    // For Pentagon 1024 model.
    // Only 128 and 48k models supported currently. Skipping block_data[2] (union)

    // chFe
    emulator.controller.write_io(0x0fe, block_data[3]);

    // chBorder
    // Setting the border after the out to 0xfe above because that too
    // sets the border color.
    emulator.controller.border_color = ZXColor::from_bits(block_data[0]);
}

// Process ZXSTAYBLOCK (AY00)
#[cfg(all(feature = "sound", feature = "ay"))]
fn process_ay_block<H: Host>(emulator: &mut Emulator<H>, machine_id: u32, block_data: &[u8]) {
    // chFlags
    let flags = block_data[0] as u32;
    if machine_id < ZXST_MID_128K {
        // If AY needs enabling and it isn't enabled already, enable it.
        if (flags & ZXSTAYF_128AY != 0) && (!emulator.settings.ay_enabled) {
            emulator.set_ay_enabled(true);
        }
        // AY needs disabling and is enabled currently, disable it.
        else if (flags & ZXSTAYF_128AY == 0) && (emulator.settings.ay_enabled) {
            emulator.set_ay_enabled(false);
        }
    }

    if emulator.settings.ay_enabled {
        // chCurrentRegister
        let ay_reg = block_data[1];
        emulator.controller.mixer.ay.select_reg(ay_reg);

        // chAyRegs
        emulator.controller.mixer.ay.set_regs(&block_data[2..]);
    }
}

// Process ZXSTKEYB (KEYB)
fn process_keyb_block<H: Host>(emulator: &mut Emulator<H>, block_data: &[u8]) {
    // dwFlags
    // ignored for now as only issue 2 is emulated
    let _flags = u32::from_le_bytes([block_data[0], block_data[1], block_data[2], block_data[3]]);

    // chKeyboardJoystick
    let joystick = block_data[4] as u32;
    if joystick & ZXSTKJT_KEMPSTON != 0 {
        emulator.controller.kempston = Some(kempston::KempstonJoy::default())
    } else {
        emulator.controller.kempston = None;
    }
}

// Process ZXSTMOUSE (AMXM)
fn process_amxm_block<H: Host>(emulator: &mut Emulator<H>, block_data: &[u8]) {
    // chType
    // Only Kempston mouse is supported
    let mouse = block_data[0] as u32;
    if mouse != 0 {
        if mouse & ZXSTM_KEMPSTON > 0 {
            emulator.controller.mouse = Some(KempstonMouse::default());
        } else {
            emulator.controller.mouse = None;
        }
    } else {
        emulator.controller.mouse = None;
    }
}

// Process ZXSTRAMPAGE (RAMP)
fn process_ramp_block<H: Host>(
    emulator: &mut Emulator<H>,
    machine_id: u32,
    block_data: &[u8],
) -> Result<()> {
    // wFlags
    let flags = u16::from_le_bytes([block_data[0], block_data[1]]) as u32;

    // chPageNo
    let mut page_num = block_data[2];
    // Remap page numbers for 16k/48k machines
    if machine_id < ZXST_MID_128K {
        page_num = match page_num {
            5 => 0,
            2 => 1,
            0 => 2,
            _ => page_num,
        };
    }

    let page_data = emulator.controller.memory.ram_page_data_mut(page_num);

    if flags & ZXSTRF_COMPRESSED != 0 {
        if cfg!(not(feature = "zlib")) {
            return Err(SnapshotLoadError::ZlibNotSupported.into());
        }
        #[cfg(feature = "zlib")]
        {
            let compressed_data: Vec<u8> = block_data[3..].to_vec();
            match decompress_zlib_stream(&compressed_data) {
                Ok(data) => {
                    return {
                        page_data.copy_from_slice(&data[..page_data.len()]);
                        Ok(())
                    }
                }
                Err(_) => return Err(SnapshotLoadError::InvalidSZXFile.into()),
            }
        }
    } else {
        let uncompressed_data: Vec<u8> = block_data[3..].to_vec();
        page_data.copy_from_slice(&uncompressed_data[..page_data.len()]);
    }

    Ok(())
}

#[cfg(feature = "zlib")]
fn decompress_zlib_stream(bytes: &[u8]) -> Result<Vec<u8>> {
    const MAX_SIZE: usize = 65535;
    if let Ok(data) = decompress_to_vec_zlib_with_limit(bytes, MAX_SIZE) {
        Ok(data)
    } else {
        Err(SnapshotLoadError::InvalidSZXFile.into())
    }
}

/// SZX snapshot loading function
pub fn load<H, A>(emulator: &mut Emulator<H>, mut asset: A) -> Result<()>
where
    H: Host,
    A: LoadableAsset + SeekableAsset,
{
    let _ = asset.seek(SeekFrom::End(0))?;
    let mut cursor_pos = 0;
    asset.seek(SeekFrom::Start(0))?;

    // ZXST Header
    let mut header = [0u8; ZXST_HEADER_SIZE];
    asset.read_exact(&mut header)?;
    cursor_pos += ZXST_HEADER_SIZE;
    let magic_bytes = &[header[0], header[1], header[2], header[3]];
    let magic_str = from_utf8(magic_bytes);

    match magic_str {
        Ok(magic_data) => {
            if !magic_data.eq("ZXST") {
                return Err(SnapshotLoadError::InvalidSZXFile.into());
            }
        }
        Err(_) => return Err(SnapshotLoadError::InvalidSZXFile.into()),
    }
    let _ = header[4];
    let _ = header[5];

    let machine_id = header[6] as u32;
    if machine_id > ZXST_MID_128K {
        return Err(SnapshotLoadError::MachineNotSupported.into());
    }

    // ZXST Block Header
    asset.seek(SeekFrom::Start(cursor_pos))?;
    let mut block_header = [0u8; ZXST_BLOCK_HEADER_SIZE];
    while asset.read_exact(&mut block_header).is_ok() {
        let size: u32 = u32::from_le_bytes([
            block_header[4],
            block_header[5],
            block_header[6],
            block_header[7],
        ]);
        let id_bytes = &[
            block_header[0],
            block_header[1],
            block_header[2],
            block_header[3],
        ];
        let id_str = from_utf8(id_bytes).unwrap().to_uppercase();
        cursor_pos += ZXST_BLOCK_HEADER_SIZE;

        // ZXST Block Data
        asset.seek(SeekFrom::Start(cursor_pos))?;
        let mut block_data = vec![0; size as usize];

        if asset.read_exact(&mut block_data).is_err() {
            return Err(SnapshotLoadError::InvalidSZXFile.into());
        }

        match id_str.as_str() {
            "CRTR" => {
                process_crtr_block(emulator, &block_data);
            }
            "Z80R" => {
                process_z80r_block(emulator, &block_data);
            }
            "SPCR" => {
                process_spcr_block(emulator, machine_id, &block_data);
            }
            #[cfg(all(feature = "sound", feature = "ay"))]
            "AY\0\0" => {
                process_ay_block(emulator, machine_id, &block_data);
            }
            "KEYB" => {
                process_keyb_block(emulator, &block_data);
            }
            "AMXM" => {
                process_amxm_block(emulator, &block_data);
            }
            "RAMP" => {
                process_ramp_block(emulator, machine_id, &block_data)?;
            }

            _ => (),
        }
        // skip block data
        cursor_pos += size as usize;

        asset.seek(SeekFrom::Start(cursor_pos))?;
    }
    emulator.controller.refresh_memory_dependent_devices();
    Ok(())
}

pub fn save<H, R>(_: &mut Emulator<H>, _: R) -> Result<()>
where
    H: Host,
    R: DataRecorder,
{
    Err(SnapshotSaveError::NotSupported.into())
}
