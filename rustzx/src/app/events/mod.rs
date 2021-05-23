//! platform-independent traits. Submodules with backends will be selectable
//! via cargo features in future
mod events_sdl;

use rustzx_core::{
    zx::{
        joy::{
            kempston::KempstonKey,
            sinclair::{SinclairJoyNum, SinclairKey},
        },
        keys::{CompoundKey, ZXKey},
    },
    EmulationSpeed,
};
use std::path::PathBuf;

pub use events_sdl::EventsSdl;

// Event type
pub enum Event {
    ZXKey(ZXKey, bool),
    CompoundKey(CompoundKey, bool),
    Kempston(KempstonKey, bool),
    Sinclair(SinclairJoyNum, SinclairKey, bool),
    SwitchFrameTrace,
    ChangeJoyKeyboardLayer(bool),
    ChangeSpeed(EmulationSpeed),
    InsertTape,
    StopTape,
    QuickSave,
    QuickLoad,
    OpenFile(PathBuf),
    Exit,
}

/// provides event response interface
pub trait EventDevice {
    // get last event
    fn pop_event(&mut self) -> Option<Event>;
}
