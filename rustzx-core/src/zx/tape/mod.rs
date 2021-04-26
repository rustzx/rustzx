//! Contains Tape handling type and functions

mod tap;
// reexport Tap Tape player
pub use self::tap::Tap;

use crate::utils::Clocks;
#[cfg(feature = "std")]
use std::path::Path;

/// Result of tape insertion,
/// `Err` contains string, which describes
/// what caused errror
pub enum InsertResult {
    Ok,
    Err(&'static str),
}

pub trait ZXTape {
    // -----------------
    // FAST LOAD SECTION
    // -----------------
    /// is this type of tape is allowed to fast load blocks?
    fn can_fast_load(&self) -> bool;
    /// Returns byte of current block or `None` if offset exceeds block Size
    fn block_byte(&self, offset: usize) -> Option<u8>;
    /// Moves tape pointer to next block
    fn next_block(&mut self);
    /// Resets relative position in block to zero
    fn reset_pos_in_block(&mut self);
    // -----------------
    //  GENERAL SECTION
    // -----------------
    /// Returns current ear bit
    fn current_bit(&self) -> bool;
    /// Makes procession of type in definite time
    fn process_clocks(&mut self, clocks: Clocks);
    /// insert new media
    #[cfg(feature = "std")]
    fn insert(&mut self, path: &Path) -> InsertResult;
    /// ejects tape
    fn eject(&mut self);
    /// stops tape
    fn stop(&mut self);
    /// plays tape
    fn play(&mut self);
    /// rewinds tape
    fn rewind(&mut self);
}
