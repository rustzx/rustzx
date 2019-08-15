//! Module with ZX Spectrum related things
//! One of core platform-independent modules
pub mod controller;
pub mod memory;
pub mod screen;
pub mod keys;
pub mod constants;
pub mod joy;
pub mod machine;
pub mod roms;
pub mod sound;
pub mod tape;

// re-export most of things
// TODO: in-deep rewiew re-exports recursively
pub use self::controller::ZXController;
pub use self::keys::*;
pub use self::machine::{ZXMachine, ZXSpecs};
pub use self::memory::{RamType, RomType, ZXMemory};
pub use self::screen::*;
pub use self::tape::ZXTape;
