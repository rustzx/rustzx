#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod palette;
#[cfg(feature = "std")]
pub mod stopwatch;

#[cfg(all(feature = "std"))]
pub mod io;
