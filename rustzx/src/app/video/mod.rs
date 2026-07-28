//! platform-independent traits. Submodules with backends will be selectable
//! via cargo features in future
mod palette;

pub mod wgpu;

pub use palette::{zx_color_to_index, ColorIndexed, Palette, PALETTE_SIZE};
