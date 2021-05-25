use crate::{zx::tape::TapeImpl, Result};

pub struct Empty;

impl TapeImpl for Empty {
    fn can_fast_load(&self) -> bool {
        false
    }

    fn next_block_byte(&mut self) -> Result<Option<u8>> {
        Ok(None)
    }

    fn next_block(&mut self) -> Result<bool> {
        Ok(false)
    }

    fn current_bit(&self) -> bool {
        false
    }

    fn process_clocks(&mut self, _clocks: usize) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) {}

    fn play(&mut self) {}

    fn rewind(&mut self) -> Result<()> {
        Ok(())
    }
}
