#![no_std]

//! Module which contains all CPU-specific structures, functions, constants

mod bus;
mod codegen;
mod cpu;
mod opcode;
mod registers;
mod smallnum;
mod tables;

pub use bus::Z80Bus;
pub use codegen::{CodeGenerator, CodegenMemorySpace};
pub use cpu::{IntMode, Z80};
pub use opcode::{Opcode, Prefix};
pub use registers::{
    RegName16, RegName8, Regs, FLAG_CARRY, FLAG_F3, FLAG_F5, FLAG_HALF_CARRY, FLAG_PV, FLAG_SIGN,
    FLAG_SUB, FLAG_ZERO,
};
