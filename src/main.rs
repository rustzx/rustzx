// Only if Debug
#![allow(dead_code)]

mod utils;
mod cpu;
mod zx;

use zx::ZXComputer;
fn main() {
    let mut comp = ZXComputer::new();
    // it will just execute 25 NOP's for now.
    comp.emulate();
}
