mod tap;
pub use self::tap::Tap;
use utils::Clocks;
pub enum InsertResult {
    Ok,
    Err(&'static str),
}

pub trait ZXTape {
    fn current_bit(&self) -> bool;
    fn process_clocks(&mut self, clocks: Clocks);
    fn insert(&mut self, path: &str) -> InsertResult;
    fn eject(&mut self);
    fn stop(&mut self);
    fn play(&mut self);
    fn rewind(&mut self);
}
