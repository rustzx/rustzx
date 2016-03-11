#![allow(dead_code)]

extern crate time;

mod utils;
mod z80;
mod zx;

use zx::ZXComputer;
fn main() {
    let mut comp = ZXComputer::new();
    comp.load_default_rom();
    let t1 = time::precise_time_ns();
    while !comp.cpu.is_halted() {
        comp.emulate();
    }
    let t2 = time::precise_time_ns();
    println!("Emulation time: {} ns", t2 - t1);
}
