//! VTX sound format parsing library
//!
//! See sources of `vtx-bin` crate for usage example
pub mod player;

use byteorder::{LittleEndian, ReadBytesExt};
use delharc::decode::{Decoder, Lh5Decoder};
use num_traits::FromPrimitive;
use thiserror::Error;

/// AY/YM Sound chip register count
pub const AY_REGISTER_COUNT: usize = 14;
/// Special R13 (`vtx.frame_registers[R13_NO_CHANGE_VALUE]`) register value which signals
/// that R13 register should not be changed for the current emulation frame. This should
/// be taken into account during emulation for correct envelope emulation
pub const R13_NO_CHANGE_VALUE: u8 = 0xFF;

/// Stereo configuration
/// See [aym::AyMode] documentation for more info
#[derive(Debug, num_derive::FromPrimitive)]
#[allow(clippy::upper_case_acronyms)]
pub enum Stereo {
    Mono,
    ABC,
    ACB,
    BAC,
    BCA,
    CAB,
    CBA,
}

/// Sound chip type
/// See [aym::SoundChip] documentation for more info
#[derive(Debug)]
pub enum SoundChip {
    AY,
    YM,
}

#[derive(Debug)]
pub struct Vtx {
    /// Sound chip which should be used for this track
    pub chip: SoundChip,
    /// Stereo configuration of the track
    pub stereo: Stereo,
    /// Sound chip frequency (e.g. For ZX Spectrum it is usually 1773400 Hz)
    pub frequency: u32,
    /// Sound frames per second (e.g. For ZX Spectrum usually equals to 50)
    pub player_frequency: u8,
    /// Starting frame index for looped playback (e.g. 0 - beginning of the song)
    pub loop_start_frame: u16,
    /// Year of the track
    pub year: u16,
    /// Title of the track
    pub title: String,
    /// Author of the track
    pub author: String,
    /// Source of the song (e.g. In which game was used)
    pub from: String,
    /// Tracker program used to make this song
    pub tracker: String,
    /// Author comment
    pub comment: String,
    /// Stores sequential blocks of register values for each frame. Each block has size if
    /// `AY_REGISTER_COUNT`. It is advised to use `frame_registers` to access frame data instead of
    /// direct access to `frame_data` field
    pub frame_data: Vec<u8>,
}

impl Vtx {
    /// Returns frame count of the track
    fn frames_count(&self) -> usize {
        self.frame_data.len() / AY_REGISTER_COUNT
    }

    /// Returns slice with register values for the given frame or `None` if index is
    /// out of bounds
    fn frame_registers(&self, index: usize) -> Option<&[u8]> {
        let offset = index * AY_REGISTER_COUNT;
        if offset + AY_REGISTER_COUNT > self.frame_data.len() {
            return None;
        }

        Some(&self.frame_data[offset..offset + AY_REGISTER_COUNT])
    }
}

#[derive(Error, Debug)]
pub enum VtxError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid VTX header: {}", message)]
    InvalidHeader { message: &'static str },
    #[error("Failed to decode lh5 compressed data")]
    DecompressFailure,
}

impl Vtx {
    /// Loads VTX file using provided reader
    pub fn load<R>(mut reader: R) -> Result<Self, VtxError>
    where
        R: std::io::Read + std::io::Seek,
    {
        let mut magic = [0u8; 2];
        reader
            .read_exact(&mut magic)
            .map_err(|_| VtxError::InvalidHeader {
                message: "Failed to read file identifier",
            })?;

        let chip = match magic {
            [b'a', b'y'] => SoundChip::AY,
            [b'y', b'm'] => SoundChip::YM,
            _ => {
                return Err(VtxError::InvalidHeader {
                    message: "Not supported file identifier",
                });
            }
        };

        let stereo = Stereo::from_u8(reader.read_u8()?).ok_or(VtxError::InvalidHeader {
            message: "Invalid stereo configuration",
        })?;

        let loop_start_rame = reader.read_u16::<LittleEndian>()?;
        let frequency = reader.read_u32::<LittleEndian>()?;
        let player_frequency = reader.read_u8()?;
        let year = reader.read_u16::<LittleEndian>()?;
        let decompressed_frames_size = reader.read_u32::<LittleEndian>()?;

        if decompressed_frames_size % AY_REGISTER_COUNT as u32 != 0 {
            return Err(VtxError::InvalidHeader {
                message: "Invalid decompressed frames data size",
            });
        }

        let strings_start = reader.stream_position()?;

        const READ_STRING_BUFFER_SIZE: usize = 256;
        const EXPECTED_STRINGS_COUNT: usize = 5;

        let mut strings_block_size = 0;
        let mut null_terminators_read = 0;
        while null_terminators_read != 5 {
            let mut strings_partial_buffer = [0u8; READ_STRING_BUFFER_SIZE];
            let bytes_read = reader.read(&mut strings_partial_buffer)?;
            let mut current_buffer_bytes_count = 0;
            while current_buffer_bytes_count < bytes_read {
                if let Some(pos) = strings_partial_buffer[current_buffer_bytes_count..]
                    .iter()
                    .position(|x| *x == b'\0')
                {
                    null_terminators_read += 1;
                    current_buffer_bytes_count += pos + 1;
                } else {
                    current_buffer_bytes_count = bytes_read;
                }
                if null_terminators_read == EXPECTED_STRINGS_COUNT {
                    break;
                }
            }
            strings_block_size += current_buffer_bytes_count;
        }

        if null_terminators_read != EXPECTED_STRINGS_COUNT {
            return Err(VtxError::InvalidHeader {
                message: "Invalid strings block",
            });
        }

        reader.seek(std::io::SeekFrom::Start(strings_start))?;
        let mut strings_buffer = vec![0u8; strings_block_size - 1];
        reader.read_exact(&mut strings_buffer)?;
        if reader.read_u8()? != b'\0' {
            return Err(VtxError::InvalidHeader {
                message: "Missing strings block terminator",
            });
        }
        let mut strings = strings_buffer
            .split(|b| *b == b'\0')
            .map(|buf| String::from_utf8_lossy(buf).into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            strings.len(),
            EXPECTED_STRINGS_COUNT,
            "Iterator size should be assured above"
        );

        let comment = strings.pop().unwrap();
        let tracker = strings.pop().unwrap();
        let from = strings.pop().unwrap();
        let author = strings.pop().unwrap();
        let title = strings.pop().unwrap();

        let mut transposed_frame_data = vec![0u8; decompressed_frames_size as usize];
        let mut decoder = Lh5Decoder::new(reader);
        decoder
            .fill_buffer(&mut transposed_frame_data)
            .map_err(|_| VtxError::DecompressFailure)?;

        // VTX originally stores pre-transposed data, therefore we need to tarnspose it
        let frames_count = transposed_frame_data.len() / AY_REGISTER_COUNT;
        let mut frame_data = Vec::with_capacity(transposed_frame_data.len());
        for idx in 0..transposed_frame_data.len() {
            let frame_idx = idx / AY_REGISTER_COUNT;
            let reg_idx = idx % AY_REGISTER_COUNT;
            frame_data.push(transposed_frame_data[reg_idx * frames_count + frame_idx]);
        }

        let vtx = Self {
            chip,
            stereo,
            frequency,
            player_frequency,
            loop_start_frame: loop_start_rame,
            year,
            title,
            author,
            from,
            tracker,
            comment,
            frame_data,
        };

        Ok(vtx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use sha2::{Digest, Sha256};

    fn check_load_vtx(data: &[u8]) -> (Vtx, String) {
        let mut vtx = Vtx::load(std::io::Cursor::new(data)).unwrap();

        let mut frame_data = vec![];
        std::mem::swap(&mut frame_data, &mut vtx.frame_data);

        let mut hasher = Sha256::default();
        hasher.update(&frame_data);
        let frame_data_fingerprint = hasher.finalize();
        (vtx, format!("{:x}", frame_data_fingerprint))
    }

    #[test]
    fn decode_succeeds_1() {
        let (vtx, frame_data_fingerprint) = check_load_vtx(include_bytes!("test/csoon.vtx"));
        expect![[r#"
            Vtx {
                chip: AY,
                stereo: ABC,
                frequency: 1773400,
                player_frequency: 50,
                loop_start_frame: 0,
                year: 0,
                title: "Coming Soon",
                author: "",
                from: "Cube Megademo",
                tracker: "Sound Tracker",
                comment: "",
                frame_data: [],
            }
        "#]]
        .assert_debug_eq(&vtx);
        expect![[r#"c5a486a006be9dd29cc24961b491596c4036b5534f7a6b465a4678974c713874"#]]
            .assert_eq(&frame_data_fingerprint);
    }

    #[test]
    fn decode_succeeds_2() {
        let (vtx, frame_data_fingerprint) = check_load_vtx(include_bytes!("test/secret.vtx"));
        expect![[r#"
            Vtx {
                chip: YM,
                stereo: ABC,
                frequency: 1773400,
                player_frequency: 50,
                loop_start_frame: 0,
                year: 2005,
                title: "\"SECRET LAND\" ( Sandra ' 95 )",
                author: "AY_VER BY IGNEOUS'2000",
                from: "",
                tracker: "",
                comment: "Created by Sergey Bulba's AY-3-8910/12 Emulator v2.6",
                frame_data: [],
            }
        "#]]
        .assert_debug_eq(&vtx);
        expect![[r#"9226e785a21e943e588dde6489284f87673138493ddf87ba483a4470311176de"#]]
            .assert_eq(&frame_data_fingerprint);
    }

    #[test]
    fn decode_succeeds_3() {
        let (vtx, frame_data_fingerprint) = check_load_vtx(include_bytes!("test/sil00.vtx"));
        expect![[r#"
            Vtx {
                chip: AY,
                stereo: ACB,
                frequency: 1773400,
                player_frequency: 50,
                loop_start_frame: 0,
                year: 1989,
                title: "Tune 1",
                author: "Fuxoft",
                from: "Song In Lines 3-5",
                tracker: "",
                comment: "Created by Sergey Bulba's AY-3-8910/12 Emulator v1.5",
                frame_data: [],
            }
        "#]]
        .assert_debug_eq(&vtx);
        expect![[r#"e81a26d67779064af9f8c9826174881d2532212add961b15399b0b8db7eff8c5"#]]
            .assert_eq(&frame_data_fingerprint);
    }

    #[test]
    fn decode_succeeds_4() {
        let (vtx, frame_data_fingerprint) = check_load_vtx(include_bytes!("test/spf21_00.vtx"));
        expect![[r#"
            Vtx {
                chip: AY,
                stereo: ABC,
                frequency: 1773400,
                player_frequency: 50,
                loop_start_frame: 0,
                year: 0,
                title: "Spectrofon 21 main menu tune",
                author: "ARNO",
                from: "Spectrofon 21 magazine",
                tracker: "Pro Tracker v2.1",
                comment: "      Converted to VTX by           Ivan Yuskin  (Krogoth)",
                frame_data: [],
            }
        "#]]
        .assert_debug_eq(&vtx);
        expect![[r#"bfab6a3e657854cb4263fc0492cd2db51451a57e6ba98e5d3050e2bed1f904fb"#]]
            .assert_eq(&frame_data_fingerprint);
    }

    #[test]
    fn frame_indexing_success() {
        let vtx = Vtx::load(std::io::Cursor::new(include_bytes!("test/csoon.vtx"))).unwrap();

        expect![[r#"
            Some(
                [
                    94,
                    0,
                    224,
                    7,
                    94,
                    0,
                    0,
                    16,
                    0,
                    30,
                    0,
                    32,
                    0,
                    255,
                ],
            )
        "#]]
        .assert_debug_eq(&vtx.frame_registers(42));
    }

    #[test]
    fn frame_indexing_failure() {
        let vtx = Vtx::load(std::io::Cursor::new(include_bytes!("test/csoon.vtx"))).unwrap();

        expect![[r#"
            None
        "#]]
        .assert_debug_eq(&vtx.frame_registers(999999));
    }
}
