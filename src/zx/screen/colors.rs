pub const BYTES_PER_PIXEL: usize = 4;
pub type ColorArray = [u8; BYTES_PER_PIXEL];
#[cfg_attr(rustfmt, rustfmt_skip)]
fn split_in_bytes(val: usize) -> ColorArray {
    return [
        ((val >> 24) & 0xFF) as u8,
        ((val >> 16) & 0xFF) as u8,
        ((val >>  8) & 0xFF) as u8,
        ((val >>  0) & 0xFF) as u8
    ];
}

#[derive(Clone, Copy)]
pub enum ZXBrightness {
    Normal,
    Bright,
}

/// ZX Spectrum color enum
/// Constructs self from 3-bit value
#[derive(Clone, Copy)]
pub enum ZXColor {
    Black,
    Blue,
    Red,
    Purple,
    Green,
    Cyan,
    Yellow,
    White,
}

impl ZXColor {
    /// Returns ZXColor from 3 bits
    /// # Panics
    /// Panics when input color is bigger than 7
    pub fn from_bits(bits: u8) -> ZXColor {
        assert!(bits <= 7);
        match bits {
            0 => ZXColor::Black,
            1 => ZXColor::Blue,
            2 => ZXColor::Red,
            3 => ZXColor::Purple,
            4 => ZXColor::Green,
            5 => ZXColor::Cyan,
            6 => ZXColor::Yellow,
            7 => ZXColor::White,
            _ => unreachable!(),
        }
    }
}

/// ZX Spectrum attribute structure
/// It contains information about ink, paper color,
/// flash attribute and brightness
#[derive(Clone, Copy)]
pub struct ZXAttribute {
    pub ink: ZXColor,
    pub paper: ZXColor,
    pub brightness: ZXBrightness,
    pub flash: bool,
}

impl ZXAttribute {
    /// Constructs self from byte
    pub fn from_byte(data: u8) -> ZXAttribute {
        ZXAttribute {
            ink: ZXColor::from_bits(data & 0x07),
            paper: ZXColor::from_bits((data >> 3) & 0x07),
            flash: (data & 0x80) != 0,
            brightness: if (data & 0x40) != 0 {
                ZXBrightness::Bright
            } else {
                ZXBrightness::Normal
            },
        }
    }

    pub fn active_color(&self, state: bool, enable_flash: bool) -> ZXColor {
        if state ^ (self.flash && enable_flash) {
            self.ink
        } else {
            self.paper
        }
    }
}
// TODO: make non-copyable and pass as ref in
// other types
#[derive(Clone, Copy)]
struct ColorSet {
    black: ColorArray,
    blue: ColorArray,
    red: ColorArray,
    purple: ColorArray,
    green: ColorArray,
    cyan: ColorArray,
    yellow: ColorArray,
    white: ColorArray,
}
/// Structure, that holds palette information.
/// It have method to transform ZX Spectrum screen data
/// to 4-byte rgba bixel
#[derive(Clone, Copy)]
pub struct ZXPalette {
    transparent: ColorArray,
    bright: ColorSet,
    normal: ColorSet,
}

impl ZXPalette {
    /// Returns default palette
    /// TODO: Use `Default` trait?
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn default() -> ZXPalette {
        ZXPalette {
            transparent: split_in_bytes(0x00000000),
            normal: ColorSet {
                black:   split_in_bytes(0x000000FF),
                blue:    split_in_bytes(0x000088FF),
                red:     split_in_bytes(0x880000FF),
                purple:  split_in_bytes(0x880088FF),
                green:   split_in_bytes(0x008800FF),
                cyan:    split_in_bytes(0x008888FF),
                yellow:  split_in_bytes(0x888800FF),
                white:   split_in_bytes(0x888888FF),
            },
            bright: ColorSet {
                black:   split_in_bytes(0x000000FF),
                blue:    split_in_bytes(0x0000FFFF),
                red:     split_in_bytes(0xFF0000FF),
                purple:  split_in_bytes(0xFF00FFFF),
                green:   split_in_bytes(0x00FF00FF),
                cyan:    split_in_bytes(0x00FFFFFF),
                yellow:  split_in_bytes(0xFFFF00FF),
                white:   split_in_bytes(0xFFFFFFFF),
            }
        }
    }
    /// Returns rgba pixel from screen data
    pub fn get_rgba(&self, color: ZXColor, brightness: ZXBrightness) -> &ColorArray {
        // select palette
        let set = match brightness {
            ZXBrightness::Normal => &self.normal,
            ZXBrightness::Bright => &self.bright,
        };
        return match color {
            ZXColor::Black => &set.black,
            ZXColor::Blue => &set.blue,
            ZXColor::Red => &set.red,
            ZXColor::Purple => &set.purple,
            ZXColor::Green => &set.green,
            ZXColor::Cyan => &set.cyan,
            ZXColor::Yellow => &set.yellow,
            ZXColor::White => &set.white,
        };
    }
}
