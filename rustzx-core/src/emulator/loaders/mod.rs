//! Module contains different media loaders

mod sna;
mod tap;
pub use self::{sna::*, tap::*};
use crate::emulator::Emulator;
use std::{convert::AsRef, path::Path};

/// Loads file into emulator instance, auto-detecting file type and
/// executing appropriate action depending on type. For example, for
/// tape images it inserts tape, for snapshots it restores snapshots.
pub fn load_file_autodetect(emulator: &mut Emulator, file: impl AsRef<Path>) {
    let extension = file
        .as_ref()
        .extension()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_lowercase());
    match extension {
        Some(ref s) if s == "sna" => load_sna(emulator, file),
        Some(ref s) if s == "tap" => {
            emulator.controller.tape.insert(file.as_ref());
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::RustzxSettings;
    use std::path::PathBuf;

    #[test]
    fn load_file_autodetect_load_tap() {
        let path: PathBuf = ["test", "tapes", "simple.tap"]
            .iter()
            .collect();
        let settings = RustzxSettings::new();
        let mut emulator = Emulator::new(&settings);
        assert_eq!(emulator.controller.tape.block_byte(0), None);
        load_file_autodetect(&mut emulator, &path);
        assert_eq!(emulator.controller.tape.block_byte(0), Some(0));
    }
}
