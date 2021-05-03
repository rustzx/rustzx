mod io;

use std::{
    path::Path,
    fs::File,
};
use rustzx_core::{
    host::{Host, Snapshot, Tape},
    settings::RustzxSettings,
    zx::ZXMachine,
};
use anyhow::bail;
use std::path::PathBuf;
use io::FileAsset;

pub struct GuiHost {
    settings: RustzxSettings,
    roms: Vec<PathBuf>,
    snapshot: Option<PathBuf>,
    tape: Option<PathBuf>,
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

    pub fn with_rom(mut self, path: &Path) -> anyhow::Result<Self> {
        match self.settings.machine {
            ZXMachine::Sinclair48K => {
                if !path.exists() {
                    bail!("Provided 48K ROM file does not exist")
                }
                self.roms = vec![path.to_owned()];
            }
            ZXMachine::Sinclair128K => {
                let rom0_path = path;
                if !rom0_path.extension().map_or(false, |e| e == "0") {
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
        Ok(self)
    }

    pub fn with_snapshot(mut self, path: &Path) -> anyhow::Result<Self> {
        if !path.extension().unwrap_or_default().eq_ignore_ascii_case("sna") {
            bail!("Invalid snapshot format");
        }

        if !path.exists() {
            bail!("Provided snapshot file does not exist");
        }

        self.snapshot.replace(path.to_owned());
        Ok(self)
    }

    pub fn with_tape(mut self, path: &Path) -> anyhow::Result<Self> {
        if !path.extension().unwrap_or_default().eq_ignore_ascii_case("tap") {
            bail!("Invalid tape format");
        }

        if !path.exists() {
            bail!("Provided tape file does not exist");
        }

        self.tape.replace(path.to_owned());
        Ok(self)
    }
}

impl Host for GuiHost {
    type RomAssetImpl = FileAsset;
    type SnapshotAssetImpl = FileAsset;
    type TapeAssetImpl = FileAsset;

    fn rom(&self, page: usize) -> Option<FileAsset> {
        self.roms.get(page).and_then(|path| {
            File::open(path)
                .map_err(|e| {
                    log::error!("Failed to open ROM file: {}", e)
                })
                .ok()
                .map(|file| file.into())
        })
    }

    fn snapshot(&self) -> Option<Snapshot<FileAsset>> {
        self.snapshot.as_ref().and_then(|path| {
            File::open(path)
                .map_err(|e| {
                    log::error!("Failed to open snapshot file: {}", e)
                })
                .ok()
                .map(|file| {
                    Snapshot::Sna(file.into())
                })
        })
    }

    fn tape(&self) -> Option<Tape<FileAsset>> {
        self.tape.as_ref().and_then(|path| {
            File::open(path)
                .map_err(|e| {
                    log::error!("Failed to open tape file: {}", e)
                })
                .ok()
                .map(|file| {
                    Tape::Tap(file.into())
                })
        })
    }

    fn settings(&self) -> &RustzxSettings {
        &self.settings
    }
}
