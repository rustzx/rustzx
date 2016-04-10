use std::path::Path;

mod tap;
pub use self::tap::Tap;

pub enum InsertResult {
    Ok,
    Err(&'static str),
}

pub trait ZXTape {
    fn current_bit(&self) -> bool;
    fn process_clocks(&mut self, clocks: u64);
    fn insert<P>(&mut self, path: P) -> InsertResult where P: AsRef<Path>;
    fn eject(&mut self);
    fn stop(&mut self);
    fn play(&mut self);
    fn rewind(&mut self);
}
