use utils::*;
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 192;
pub const PIXEL_COUNT: usize = SCREEN_HEIGHT * SCREEN_WIDTH;
pub const PIXELS_PER_BYTE: u64 = 8;
pub const BYTES_PER_ROW: u64 = SCREEN_WIDTH as u64 / PIXELS_PER_BYTE;
pub const ROWS_PER_ATTR: u64 = 8;

fn get_pixel_base_index(addr: u16) -> usize {
    let (h, l) = split_word(addr);
    // 0 1 0 Y7 Y6 Y2 Y1 Y0 | Y5 Y4 Y3 X4 X3 X2 X1 X0
    // extract lowest 5 bits as x coordinate base
    let x_base = l & 0x1F;
    let y = (h & 0x07) | ((l >> 2) & 0x38) | ((h << 3) & 0xC0);
    return y as usize * SCREEN_WIDTH + x_base as usize * 8;
}

pub struct ZXScreen {
    // 2 bytes per pixel
    // rrrr gggg bbbb aaaa
    // r = flash attribute
    // g = state (on/off)
    // b = paper color
    // a = ink color
    screen: [u8; PIXEL_COUNT * 2],
}

impl ZXScreen {

    pub fn new() -> ZXScreen {
        ZXScreen {
            screen: [0; PIXEL_COUNT * 2],
        }
    }

    pub fn clear(&mut self) {
        // fill with zeros
        self.screen = [0; PIXEL_COUNT * 2];
    }
    pub fn write_bitmap_byte(&mut self, addr: u16, value: u8) {
        assert!((addr & 0xE000) == 0x4000);
        // split bitmap byte on pixels
        let mut pixels = [0_u8; 8];
        let mut value = value;
        for n in 0..8 {
            // check highest bit
            if (value & 0x80) != 0  {
                // if set then fill lower 4 bits
                pixels[n] = 0x0F;
            };
            // shift left one bit
            value <<= 1;
        };
        // drop value, it contains garbage/zero
        drop(value);
        // pixels extracted from bitmap byte.
        // so, we can write them to screen array;
        let base =  get_pixel_base_index(addr);
        for n in 0..8 {
            // clear state
            self.screen[(base + n) * 2 + 1] &= 0xF0;
            // write new color
            self.screen[(base + n) * 2 + 1] |= pixels[n];
        }
    }
    pub fn write_attr_byte(&mut self, addr: u16, value: u8) {
        // TODO: Add assert
        let base = addr - 0x5800;
        // left-top pos of 8x8 area
        let x_base = (base % 32) * 8;
        let y_base = (base / 32) * 8;
        for y in y_base..y_base + 8 {
            for x in x_base..x_base + 8 {
                let index = y as usize * SCREEN_WIDTH + x as usize;
                // clear flash and set to new value
                self.screen[index * 2 + 1] &= 0x0F;
                if (value & 0x80) != 0 {
                    self.screen[index * 2 + 1] |= 0xF0;
                }
                // set colors
                self.screen[index * 2] = (value & 0x07) | ((value << 1) & 0x70);
                // set brightness ( set fourth bit in color values)
                if (value & 0x40) != 0 {
                    self.screen[index * 2] |= 0x88
                }
            }
        }
    }

    pub fn clone_texture(&self) -> &[u8] {
        &self.screen
    }
}
