mod empty;
mod tap;

pub use empty::Empty;
pub use tap::Tap;

use crate::{
    host::{LoadableAsset, SeekableAsset},
    Result,
};

use enum_dispatch::enum_dispatch;

#[allow(clippy::large_enum_variant)]
#[enum_dispatch(TapeImpl)]
pub enum ZXTape<A: LoadableAsset + SeekableAsset> {
    Tap(Tap<A>),
    Empty(Empty),
}

impl<A: LoadableAsset + SeekableAsset> Default for ZXTape<A> {
    fn default() -> Self {
        Self::Empty(Empty)
    }
}

#[enum_dispatch]
pub trait TapeImpl {
    fn can_fast_load(&self) -> bool;
    /// Returns byte of current block or `None` if block has ended
    fn next_block_byte(&mut self) -> Result<Option<u8>>;
    /// Loads next block. Returns false if end of the tape is reached
    fn next_block(&mut self) -> Result<bool>;
    /// Returns current tape (`ear`) bit
    fn current_bit(&self) -> bool;
    /// Perform tape processing emulation within `clocks` time limit
    fn process_clocks(&mut self, clocks: usize) -> Result<()>;
    fn stop(&mut self);
    fn play(&mut self);
    /// Rewinds tape content to the beginning
    fn rewind(&mut self) -> Result<()>;
}
