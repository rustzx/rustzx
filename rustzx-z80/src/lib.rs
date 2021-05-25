#![no_std]
#![allow(dead_code)]

//! Module which contains all CPU-specific structures, functions, constants

mod bus;
mod common_types;
mod cpu;
pub mod opcodes;
mod registers;
pub mod tables;
mod smallnum;
mod clocks;
mod utils;

pub use clocks::Clocks;

// Bring nested types and functions to current scope
pub use self::{bus::*, common_types::*, cpu::*, registers::*};
