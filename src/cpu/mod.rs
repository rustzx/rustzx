//! Module which contains all CPU-specific structures, functions, constants
//! exports `Z80` and `Z80Bus` for dirrect use
pub mod tables;
mod common_types;
mod registers;
mod z80;

// Bring nested types and functions to current scope
pub use self::common_types::*;
pub use self::registers::*;
pub use self::z80::*;
