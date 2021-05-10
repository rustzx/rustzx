use crate::{utils::Clocks, zx::tape::TapeImpl};

pub struct Empty;

impl TapeImpl for Empty {
    fn can_fast_load(&self) -> bool {
        false
    }
    fn block_byte(&self, _offset: usize) -> Option<u8> {
        Some(0)
    }
    fn next_block(&mut self) {}
    fn reset_pos_in_block(&mut self) {}
    fn current_bit(&self) -> bool {
        false
    }
    fn process_clocks(&mut self, _clocks: Clocks) {}
    fn stop(&mut self) {}
    fn play(&mut self) {}
    fn rewind(&mut self) {}
}
