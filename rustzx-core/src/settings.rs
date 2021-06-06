use crate::{utils::EmulationMode, zx::machine::ZXMachine};

#[cfg(all(feature = "sound", feature = "ay"))]
use crate::zx::sound::ay::ZXAYMode;

pub struct RustzxSettings {
    pub machine: ZXMachine,
    pub emulation_mode: EmulationMode,
    pub tape_fastload_enabled: bool,
    pub kempston_enabled: bool,
    pub mouse_enabled: bool,
    #[cfg(all(feature = "sound", feature = "ay"))]
    pub ay_mode: ZXAYMode,
    #[cfg(all(feature = "sound", feature = "ay"))]
    pub ay_enabled: bool,
    #[cfg(feature = "sound")]
    pub beeper_enabled: bool,
    #[cfg(feature = "sound")]
    pub sound_enabled: bool,
    #[cfg(feature = "sound")]
    pub sound_volume: u8,
    #[cfg(feature = "sound")]
    pub sound_sample_rate: usize,
    #[cfg(feature = "embedded-roms")]
    pub load_default_rom: bool,
    #[cfg(feature = "autoload")]
    pub autoload_enabled: bool,
}
