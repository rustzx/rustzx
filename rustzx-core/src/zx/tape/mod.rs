//! Contains Tape handling type and functions

mod tap;
// reexport Tap Tape player
pub use self::tap::Tap;

use crate::utils::Clocks;

use enum_dispatch::enum_dispatch;

#[enum_dispatch(TapeImpl)]
pub enum ZXTape {
    Tap(Tap),
}

#[enum_dispatch]
pub trait TapeImpl {
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
    /// stops tape
    fn stop(&mut self);
    /// plays tape
    fn play(&mut self);
    /// rewinds tape
    fn rewind(&mut self);
}
