//! Module with ZX Spectrum related things
//! One of core platform-independent modules
pub mod constants;
pub mod controller;
pub mod joy;
pub mod keys;
pub mod machine;
pub mod memory;
pub mod roms;
pub mod sound;
pub mod tape;
pub mod video;

// re-export most of things
// TODO(#48): Reorganize imports
pub use self::{
    keys::*,
    machine::{ZXMachine, ZXSpecs},
    memory::{RamType, RomType, ZXMemory},
    tape::ZXTape,
    video::*,
};
