mod frame_buffer;

use anyhow::{anyhow, bail, Context};
use frame_buffer::{FrameBufferContext, RgbaFrameBuffer};
use rustzx_core::{
    host::{
        FrameBuffer, Host, HostContext, RomFormat, RomSet, Screen, Snapshot, StubIoExtender, Tape,
    },
    zx::machine::ZXMachine,
};
use rustzx_utils::{
    io::{DynamicAsset, FileAsset, GzipAsset},
    stopwatch::InstantStopwatch,
};
use std::{collections::VecDeque, fs::File, path::Path};

const SUPPORTED_SNAPSHOT_FORMATS: [&str; 1] = ["sna"];
const SUPPORTED_TAPE_FORMATS: [&str; 1] = ["tap"];
const SUPPORTED_SCREEN_FORMATS: [&str; 1] = ["scr"];

pub struct AppHost;

impl Host for AppHost {
    type Context = AppHostContext;
    type EmulationStopwatch = InstantStopwatch;
    type FrameBuffer = RgbaFrameBuffer;
    type IoExtender = StubIoExtender;
    type TapeAsset = DynamicAsset;
}

pub struct AppHostContext;

impl HostContext<AppHost> for AppHostContext {
    fn frame_buffer_context(&self) -> <<AppHost as Host>::FrameBuffer as FrameBuffer>::Context {
        FrameBufferContext
    }
}

pub struct FileRomSet {
    pages: VecDeque<DynamicAsset>,
}

impl RomSet for FileRomSet {
    type Asset = DynamicAsset;

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
    Screen,
}

pub enum DetectedContainerKind {
    None,
    Gzip,
}

pub fn load_asset(path: &Path) -> anyhow::Result<DynamicAsset> {
    let container_kind = detect_container(path);

    let file = File::open(path).with_context(|| "Failed to open tape file")?;

    match container_kind {
        DetectedContainerKind::None => Ok(FileAsset::from(file).into()),
        DetectedContainerKind::Gzip => {
            let gzip = GzipAsset::new(file)?;
            Ok(gzip.into())
        }
    }
}

pub fn load_tape(path: &Path) -> anyhow::Result<Tape<DynamicAsset>> {
    if !file_extension_matches_one_of(path, &SUPPORTED_TAPE_FORMATS) {
        bail!("Invalid tape format");
    }

    if !path.exists() {
        bail!("Provided tape file does not exist");
    }

    load_asset(path)
        .map(Tape::Tap)
        .with_context(|| "Failed to load tape file")
}

pub fn load_snapshot(path: &Path) -> anyhow::Result<Snapshot<DynamicAsset>> {
    if !file_extension_matches_one_of(path, &SUPPORTED_SNAPSHOT_FORMATS) {
        bail!("Invalid snapshot format");
    }

    if !path.exists() {
        bail!("Provided snapshot file does not exist");
    }

    load_asset(path)
        .map(Snapshot::Sna)
        .with_context(|| "Failed to load snapshot file")
}

pub fn load_screen(path: &Path) -> anyhow::Result<Screen<DynamicAsset>> {
    if !file_extension_matches_one_of(path, &SUPPORTED_SCREEN_FORMATS) {
        bail!("Invalid screen format");
    }

    if !path.exists() {
        bail!("Provided screen file does not exist");
    }

    load_asset(path)
        .map(Screen::Scr)
        .with_context(|| "Failed to load screen file")
}

fn load_rom_asset(path: &Path) -> anyhow::Result<DynamicAsset> {
    load_asset(path).with_context(|| "Failed to load rom asset")
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
            let rom1_path = if is_container(rom0_path) {
                let container_ext = rom0_path.extension().unwrap().to_string_lossy();
                let mut new_path = rom0_path.to_owned();
                new_path.set_extension(""); // removes just container extension
                new_path.with_extension(format!("1.{}", container_ext))
            } else {
                rom0_path.to_owned().with_extension("1")
            };

            if !rom1_path.exists() {
                bail!("Provided 128K ROM1 file does not exist");
            }

            Ok(FileRomSet {
                pages: VecDeque::from(vec![
                    load_rom_asset(rom0_path).with_context(|| "128K ROM0 load failed")?,
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
    } else if file_extension_matches_one_of(path, &SUPPORTED_SCREEN_FORMATS) {
        Ok(DetectedFileKind::Screen)
    } else {
        Err(anyhow!("Not supported file format"))
    }
}

fn is_container(path: &Path) -> bool {
    !matches!(detect_container(path), DetectedContainerKind::None)
}

fn detect_container(path: &Path) -> DetectedContainerKind {
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();

    match ext.as_str() {
        "gz" => DetectedContainerKind::Gzip,
        _ => DetectedContainerKind::None,
    }
}

fn file_extension_matches(path: &Path, expected: &str) -> bool {
    let mut path = path.to_owned();
    // Ignore outer container extension during comparison
    if is_container(&path) {
        path.set_extension("");
    }

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
        assert!(file_extension_matches(&Path::new("test.tap.gz"), "tap"));
        assert!(file_extension_matches(&Path::new("test.tap.gZ"), "tap"));
    }

    #[test]
    fn file_extension_matches_returns_false() {
        assert!(!file_extension_matches(&Path::new("test.tap"), "sna"));
    }
}
