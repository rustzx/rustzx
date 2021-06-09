use rustzx_core::zx::video::colors::{ZXBrightness, ZXColor};
use rustzx_utils::palette::rgba::ORIGINAL as DEFAULT_PALETTE;

type ColorRgba = [u8; 4];

const MAX_COLORS: usize = 16;

pub struct Palette {
    colors: [ColorRgba; MAX_COLORS],
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            colors: DEFAULT_PALETTE,
        }
    }
}

impl Palette {
    pub fn get_rgba(&self, color: ZXColor, brightness: ZXBrightness) -> ColorRgba {
        let index = ((color as u8) + (brightness as u8) * 8) as usize;
        assert!(index < MAX_COLORS);
        self.colors[index]
    }
}
