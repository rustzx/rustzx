#![allow(dead_code)]
mod utils;
mod cpu;
mod zx;

use zx::ZXComputer;
fn main() {
    let mut comp = ZXComputer::new();
    comp.load_default_rom();
    while !comp.cpu.is_halted() {
        comp.emulate();
    }
}
