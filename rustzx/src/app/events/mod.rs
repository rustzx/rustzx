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
        mouse::kempston::{KempstonMouseButton, KempstonMouseWheelDirection},
    },
    EmulationMode,
};
use std::path::PathBuf;

pub use events_sdl::EventsSdl;

// Event type
pub enum Event {
    ZXKey(ZXKey, bool),
    CompoundKey(CompoundKey, bool),
    Kempston(KempstonKey, bool),
    Sinclair(SinclairJoyNum, SinclairKey, bool),
    MouseMove { x: i8, y: i8 },
    MouseButton(KempstonMouseButton, bool),
    MouseWheel(KempstonMouseWheelDirection),
    SwitchFrameTrace,
    ChangeJoyKeyboardLayer(bool),
    ChangeSpeed(EmulationMode),
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
