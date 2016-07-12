//! Module with ZX Spectrum related things
//! One of core platform-independent modules
pub mod memory;
pub mod controller;
pub mod screen;
pub mod keys;
pub mod tape;
pub mod machine;
pub mod constants;
pub mod roms;
pub mod sound;

// re-export most of things
// TODO: in-deep rewiew re-exports recursively
pub use self::controller::ZXController;
pub use self::machine::{ZXMachine, ZXSpecs};
pub use self::memory::{ZXMemory, RomType, RamType};
pub use self::screen::*;
pub use self::keys::*;
pub use self::tape::ZXTape;
