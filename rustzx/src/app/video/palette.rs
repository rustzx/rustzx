use rustzx_core::zx::video::colors::{ZXBrightness, ZXColor};

type ColorRgba = [u8; 4];

const MAX_COLORS: usize = 16;

pub struct Palette {
    colors: [ColorRgba; MAX_COLORS],
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            colors: [
                // normal
                0x000000FF_u32.to_be_bytes(),
                0x0000CDFF_u32.to_be_bytes(),
                0xCD0000FF_u32.to_be_bytes(),
                0xCD00CDFF_u32.to_be_bytes(),
                0x00CD00FF_u32.to_be_bytes(),
                0x00CDCDFF_u32.to_be_bytes(),
                0xCDCD00FF_u32.to_be_bytes(),
                0xCDCDCDFF_u32.to_be_bytes(),
                // bright
                0x000000FF_u32.to_be_bytes(),
                0x0000FFFF_u32.to_be_bytes(),
                0xFF0000FF_u32.to_be_bytes(),
                0xFF00FFFF_u32.to_be_bytes(),
                0x00FF00FF_u32.to_be_bytes(),
                0x00FFFFFF_u32.to_be_bytes(),
                0xFFFF00FF_u32.to_be_bytes(),
                0xFFFFFFFF_u32.to_be_bytes(),
            ],
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
