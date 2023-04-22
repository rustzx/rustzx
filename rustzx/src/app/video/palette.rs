use rustzx_core::zx::video::colors::{ZXBrightness, ZXColor};
use rustzx_utils::palette::rgba::ORIGINAL as DEFAULT_PALETTE;

pub type ColorRgba = u32;
pub type ColorIndexed = u8;
pub const PALETTE_SIZE: usize = 16;

/// Converts ZX Spectrum color and brightness to palette index (0..16)
pub fn zx_color_to_index(color: ZXColor, brightness: ZXBrightness) -> u8 {
    (color as u8) | (brightness as u8) << 3
}

pub struct Palette {
    colors: [ColorRgba; PALETTE_SIZE],
}

impl Palette {
    /// Get RGBA color from palette by its index
    pub fn get_color(&self, index: u8) -> ColorRgba {
        self.colors[index as usize]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            colors: DEFAULT_PALETTE,
        }
    }
}
