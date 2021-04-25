//! platform-independent traits. Submodules with backends will be selectable
//! via cargo features in future
mod events_sdl;
pub use self::events_sdl::EventsSdl;

use crate::{
    utils::EmulationSpeed,
    zx::{joy::kempston::KempstonKey, keys::*},
};
use std::path::PathBuf;

// Event type
pub enum Event {
    GameKey(ZXKey, bool),
    Kempston(KempstonKey, bool),
    SwitchDebug,
    ChangeSpeed(EmulationSpeed),
    InsertTape,
    StopTape,
    // QuickSave,
    // QuickLoad,
    // Pause,
    OpenFile(PathBuf),
    Exit,
}

/// provides event response interface
pub trait EventDevice {
    // get last event
    fn pop_event(&mut self) -> Option<Event>;
}
