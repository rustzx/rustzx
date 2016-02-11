//! Module which contains all CPU-specific structures, functions, constants
//! exports `Z80` and `Z80Bus` for dirrect use
pub mod registers;
pub mod decoders;
pub mod tables;

mod z80;
pub use self::z80::{Z80, Z80Bus};
