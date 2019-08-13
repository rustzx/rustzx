//! Module contains different media loaders

mod sna;
mod tap;
pub use self::sna::*;
pub use self::tap::*;
use emulator::Emulator;
use std::convert::AsRef;
use std::path::Path;

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
        Some(ref s) if s == "sna" => {
            // TODO: load_sna should take Path for filename
            match file.as_ref().to_str() {
                Some(file_str) => load_sna(emulator, file_str),
                None => (),
            }
        }
        Some(ref s) if s == "tap" => {
            // TODO: insert should take Path for filename
            match file.as_ref().to_str() {
                Some(file_str) => {
                    emulator.controller.tape.insert(file_str);
                }
                None => (),
            }
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use settings::RustzxSettings;
    use std::path::PathBuf;

    #[test]
    fn load_file_autodetect_load_tap() {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "test", "tapes", "simple.tap"]
            .iter()
            .collect();
        let settings = RustzxSettings::new();
        let mut emulator = Emulator::new(&settings);
        assert_eq!(emulator.controller.tape.block_byte(0), None);
        load_file_autodetect(&mut emulator, &path);
        assert_eq!(emulator.controller.tape.block_byte(0), Some(0));
    }
}
