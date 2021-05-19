/// Represents color brightness
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

impl From<ZXColor> for u8 {
    fn from(color: ZXColor) -> Self {
        match color {
            ZXColor::Black => 0,
            ZXColor::Blue => 1,
            ZXColor::Red => 2,
            ZXColor::Purple => 3,
            ZXColor::Green => 4,
            ZXColor::Cyan => 5,
            ZXColor::Yellow => 6,
            ZXColor::White => 7,
        }
    }
}

/// ZX Spectrum attribute structure
/// It contains information about ink, paper color,
/// flash attribute and brightness
#[derive(Clone, Copy)]
pub(crate) struct ZXAttribute {
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

    /// Returns active color of pixel in current attribute
    pub fn active_color(&self, state: bool, enable_flash: bool) -> ZXColor {
        if state ^ (self.flash && enable_flash) {
            self.ink
        } else {
            self.paper
        }
    }
}
