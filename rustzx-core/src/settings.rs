use crate::{
    utils::EmulationSpeed,
    zx::{machine::ZXMachine, sound::ay::ZXAYMode},
};

pub struct RustzxSettings {
    pub machine: ZXMachine,
    pub emulation_speed: EmulationSpeed,
    pub tape_fastload: bool,
    pub enable_kempston: bool,
    pub ay_mode: ZXAYMode,
    pub ay_enabled: bool,
    pub beeper_enabled: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
    pub load_default_rom: bool,
}
