use rustzx_core::zx::screen::colors::{ZXBrightness, ZXColor};

type ColorRgba = [u8; 4];

/// represents set of colors
struct ColorSet {
    black: ColorRgba,
    blue: ColorRgba,
    red: ColorRgba,
    purple: ColorRgba,
    green: ColorRgba,
    cyan: ColorRgba,
    yellow: ColorRgba,
    white: ColorRgba,
}
pub struct Palette {
    bright: ColorSet,
    normal: ColorSet,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            normal: ColorSet {
                black: 0x000000FF_u32.to_be_bytes(),
                blue: 0x0000CDFF_u32.to_be_bytes(),
                red: 0xCD0000FF_u32.to_be_bytes(),
                purple: 0xCD00CDFF_u32.to_be_bytes(),
                green: 0x00CD00FF_u32.to_be_bytes(),
                cyan: 0x00CDCDFF_u32.to_be_bytes(),
                yellow: 0xCDCD00FF_u32.to_be_bytes(),
                white: 0xCDCDCDFF_u32.to_be_bytes(),
            },
            bright: ColorSet {
                black: 0x000000FF_u32.to_be_bytes(),
                blue: 0x0000FFFF_u32.to_be_bytes(),
                red: 0xFF0000FF_u32.to_be_bytes(),
                purple: 0xFF00FFFF_u32.to_be_bytes(),
                green: 0x00FF00FF_u32.to_be_bytes(),
                cyan: 0x00FFFFFF_u32.to_be_bytes(),
                yellow: 0xFFFF00FF_u32.to_be_bytes(),
                white: 0xFFFFFFFF_u32.to_be_bytes(),
            },
        }
    }
}

impl Palette {
    pub fn get_rgba(&self, color: ZXColor, brightness: ZXBrightness) -> ColorRgba {
        let set = match brightness {
            ZXBrightness::Normal => &self.normal,
            ZXBrightness::Bright => &self.bright,
        };
        match color {
            ZXColor::Black => set.black,
            ZXColor::Blue => set.blue,
            ZXColor::Red => set.red,
            ZXColor::Purple => set.purple,
            ZXColor::Green => set.green,
            ZXColor::Cyan => set.cyan,
            ZXColor::Yellow => set.yellow,
            ZXColor::White => set.white,
        }
    }
}
