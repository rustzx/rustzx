mod io;

use anyhow::{anyhow, bail};
use io::FileAsset;
use rustzx_core::{
    host::{Host, Snapshot, Tape},
    settings::RustzxSettings,
    zx::ZXMachine,
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub struct GuiHost {
    settings: RustzxSettings,
    roms: Vec<PathBuf>,
    snapshot: Option<PathBuf>,
    tape: Option<PathBuf>,
}

const SUPPORTED_SNAPSHOT_FORMATS: [&str; 1] = ["sna"];
const SUPPORTED_TAPE_FORMATS: [&str; 1] = ["tap"];

pub enum DetectedFileKind {
    Tape,
    Snapshot,
}

impl GuiHost {
    pub fn from_settings(settings: RustzxSettings) -> Self {
        Self {
            settings,
            roms: Default::default(),
            snapshot: None,
            tape: None,
        }
    }

    pub fn load_rom(&mut self, path: &Path) -> anyhow::Result<()> {
        match self.settings.machine {
            ZXMachine::Sinclair48K => {
                if !path.exists() {
                    bail!("Provided 48K ROM file does not exist")
                }
                self.roms = vec![path.to_owned()];
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
                self.roms = vec![rom0_path.to_owned(), rom1_path.to_owned()];
            }
        }
        Ok(())
    }

    pub fn load_snapshot(&mut self, path: &Path) -> anyhow::Result<()> {
        if !file_extension_matches_one_of(path, &SUPPORTED_SNAPSHOT_FORMATS) {
            bail!("Invalid snapshot format");
        }

        if !path.exists() {
            bail!("Provided snapshot file does not exist");
        }

        self.snapshot.replace(path.to_owned());
        Ok(())
    }

    pub fn load_tape(&mut self, path: &Path) -> anyhow::Result<()> {
        if !file_extension_matches_one_of(path, &SUPPORTED_TAPE_FORMATS) {
            bail!("Invalid tape format");
        }

        if !path.exists() {
            bail!("Provided tape file does not exist");
        }

        self.tape.replace(path.to_owned());
        Ok(())
    }

    pub fn load_autodetect(&mut self, path: &Path) -> anyhow::Result<DetectedFileKind> {
        if file_extension_matches_one_of(path, &SUPPORTED_TAPE_FORMATS) {
            self.load_tape(path)?;
            Ok(DetectedFileKind::Tape)
        } else if file_extension_matches_one_of(path, &SUPPORTED_SNAPSHOT_FORMATS) {
            self.load_snapshot(path)?;
            Ok(DetectedFileKind::Snapshot)
        } else {
            Err(anyhow!("Not supported file format"))
        }
    }
}

impl Host for GuiHost {
    type RomAssetImpl = FileAsset;
    type SnapshotAssetImpl = FileAsset;
    type TapeAssetImpl = FileAsset;

    fn rom(&self, page: usize) -> Option<FileAsset> {
        self.roms.get(page).and_then(|path| {
            File::open(path)
                .map_err(|e| log::error!("Failed to open ROM file: {}", e))
                .ok()
                .map(|file| file.into())
        })
    }

    fn snapshot(&self) -> Option<Snapshot<FileAsset>> {
        self.snapshot.as_ref().and_then(|path| {
            File::open(path)
                .map_err(|e| log::error!("Failed to open snapshot file: {}", e))
                .ok()
                .map(|file| Snapshot::Sna(file.into()))
        })
    }

    fn tape(&self) -> Option<Tape<FileAsset>> {
        self.tape.as_ref().and_then(|path| {
            File::open(path)
                .map_err(|e| log::error!("Failed to open tape file: {}", e))
                .ok()
                .map(|file| Tape::Tap(file.into()))
        })
    }

    fn settings(&self) -> &RustzxSettings {
        &self.settings
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
