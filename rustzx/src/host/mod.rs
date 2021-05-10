mod frame_buffer;
mod io;

use anyhow::{anyhow, bail, Context};
use frame_buffer::{FrameBufferContext, RgbaFrameBuffer};
use io::FileAsset;
use rustzx_core::{
    host::{FrameBuffer, Host, HostContext, RomFormat, RomSet, Snapshot, Tape},
    zx::machine::ZXMachine,
};
use std::{collections::VecDeque, fs::File, path::Path};

const SUPPORTED_SNAPSHOT_FORMATS: [&str; 1] = ["sna"];
const SUPPORTED_TAPE_FORMATS: [&str; 1] = ["tap"];

pub struct AppHost;

impl Host for AppHost {
    type Context = AppHostContext;
    type FrameBuffer = RgbaFrameBuffer;
    type RomSet = FileRomSet;
    type SnapshotAsset = FileAsset;
    type TapeAsset = FileAsset;
}

pub struct AppHostContext;

impl HostContext<AppHost> for AppHostContext {
    fn frame_buffer_context(&self) -> <<AppHost as Host>::FrameBuffer as FrameBuffer>::Context {
        FrameBufferContext
    }
}

pub struct FileRomSet {
    pages: VecDeque<FileAsset>,
}

impl RomSet for FileRomSet {
    type Asset = FileAsset;

    fn format(&self) -> RomFormat {
        RomFormat::Binary16KPages
    }

    fn next_asset(&mut self) -> Option<Self::Asset> {
        self.pages.pop_front()
    }
}

pub enum DetectedFileKind {
    Tape,
    Snapshot,
}

pub fn load_tape(path: &Path) -> anyhow::Result<Tape<FileAsset>> {
    if !file_extension_matches_one_of(path, &SUPPORTED_TAPE_FORMATS) {
        bail!("Invalid tape format");
    }

    if !path.exists() {
        bail!("Provided tape file does not exist");
    }

    File::open(path)
        .with_context(|| "Failed to open tape file")
        .map(|file| Tape::Tap(file.into()))
}

pub fn load_snapshot(path: &Path) -> anyhow::Result<Snapshot<FileAsset>> {
    if !file_extension_matches_one_of(path, &SUPPORTED_SNAPSHOT_FORMATS) {
        bail!("Invalid snapshot format");
    }

    if !path.exists() {
        bail!("Provided snapshot file does not exist");
    }

    File::open(path)
        .with_context(|| "Failed to open snapshot file")
        .map(|file| Snapshot::Sna(file.into()))
}

fn load_rom_asset(path: &Path) -> anyhow::Result<FileAsset> {
    File::open(path)
        .with_context(|| "Failed to load rom asset")
        .map(|file| file.into())
}

pub fn load_rom(path: &Path, machine: ZXMachine) -> anyhow::Result<FileRomSet> {
    match machine {
        ZXMachine::Sinclair48K => {
            if !path.exists() {
                bail!("Provided 48K ROM file does not exist")
            }

            Ok(FileRomSet {
                pages: VecDeque::from(vec![
                    load_rom_asset(path).with_context(|| "48K ROM load failed")?
                ]),
            })
        }
        ZXMachine::Sinclair128K => {
            let rom0_path = path;
            if !file_extension_matches(rom0_path, "0") {
                bail!("128K ROM filename should end with '.0' extension");
            }
            if !rom0_path.exists() {
                bail!("Provided 128K ROM0 file does not exist");
            }
            let rom1_path = rom0_path.to_owned().with_extension("1");
            if !rom1_path.exists() {
                bail!("Provided 128K ROM1 file does not exist");
            }

            Ok(FileRomSet {
                pages: VecDeque::from(vec![
                    load_rom_asset(&rom0_path).with_context(|| "128K ROM0 load failed")?,
                    load_rom_asset(&rom1_path).with_context(|| "128K ROM1 load failed")?,
                ]),
            })
        }
    }
}

pub fn detect_file_type(path: &Path) -> anyhow::Result<DetectedFileKind> {
    if file_extension_matches_one_of(path, &SUPPORTED_TAPE_FORMATS) {
        load_tape(path)?;
        Ok(DetectedFileKind::Tape)
    } else if file_extension_matches_one_of(path, &SUPPORTED_SNAPSHOT_FORMATS) {
        load_snapshot(path)?;
        Ok(DetectedFileKind::Snapshot)
    } else {
        Err(anyhow!("Not supported file format"))
    }
}

fn file_extension_matches(path: &Path, expected: &str) -> bool {
    let actual = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();

    actual == expected
}

fn file_extension_matches_one_of(path: &Path, extensions: &[&str]) -> bool {
    extensions
        .iter()
        .copied()
        .any(|ext| file_extension_matches(path, ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn file_extension_matches_returns_true() {
        assert!(file_extension_matches(&Path::new("test.tap"), "tap"));
        assert!(file_extension_matches(&Path::new("test.TAP"), "tap"));
        assert!(file_extension_matches(&Path::new("test.tAp"), "tap"));
    }

    #[test]
    fn file_extension_matches_returns_false() {
        assert!(!file_extension_matches(&Path::new("test.tap"), "sna"));
    }
}
