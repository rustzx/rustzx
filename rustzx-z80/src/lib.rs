#![no_std]
#![allow(dead_code)]

//! Module which contains all CPU-specific structures, functions, constants

mod bus;
mod cpu;
mod opcode;
mod registers;
mod smallnum;
mod tables;
mod utils;

pub use bus::Z80Bus;
pub use cpu::{IntMode, Z80};
pub use registers::{
    Condition, Flag, RegName16, RegName8, Regs, FLAG_CARRY, FLAG_F3, FLAG_F5, FLAG_HALF_CARRY,
    FLAG_PV, FLAG_SIGN, FLAG_SUB, FLAG_ZERO,
};
