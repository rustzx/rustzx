mod empty;
mod tap;

pub use self::{empty::Empty, tap::Tap};

use crate::{host::LoadableAsset, utils::Clocks, Result};

use enum_dispatch::enum_dispatch;

#[enum_dispatch(TapeImpl)]
pub enum ZXTape<A: LoadableAsset> {
    Tap(Tap<A>),
    Empty(Empty),
}

impl<A: LoadableAsset> Default for ZXTape<A> {
    fn default() -> Self {
        Self::Empty(Empty)
    }
}

#[enum_dispatch]
pub trait TapeImpl {
    fn can_fast_load(&self) -> bool;
    /// Returns byte of current block or `None` if block has ended
    fn next_block_byte(&mut self) -> Result<Option<u8>>;
    /// Loads next block. Retruns false if end of the tape is reached
    fn next_block(&mut self) -> Result<bool>;
    /// Returns current tape (`ear`) bit
    fn current_bit(&self) -> bool;
    /// Perform tape processing emulation within `clocks` time limit
    fn process_clocks(&mut self, clocks: Clocks) -> Result<()>;
    fn stop(&mut self);
    fn play(&mut self);
    /// Rewinds tape content to the beginning
    fn rewind(&mut self) -> Result<()>;
}
