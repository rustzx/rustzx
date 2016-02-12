#![allow(dead_code)]
// TODO: Reformat all code with rustfmt
mod utils;
mod cpu;
mod zx;

use zx::ZXComputer;
fn main() {
    let mut comp = ZXComputer::new();
    comp.load_default_rom();
    comp.emulate();
}
