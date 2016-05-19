//! ZX Spectrum screen module
//! Consists of ZXScreen type and functions for bitmap addr encode/decode
//! Produces RGBA bitmap screen

use utils::*;
use z80::Clocks;
use super::machine::ZXMachine;

// TODO CHECK USING OF CONST IN PROJECT
// SCREEN_* NOW FULL SCREEN SIZE, CANVAS_* - working area
const SCREEN_WIDTH: usize = 384;
const SCREEN_HEIGHT: usize = 288;
pub const PIXEL_COUNT: usize = SCREEN_HEIGHT * SCREEN_WIDTH;

pub const CANVAS_WIDTH: usize = 256;
pub const CANVAS_HEIGHT: usize = 192;
pub const CANVAS_X: usize = 64;
pub const CANVAS_Y: usize = 48;

pub const ATTR_COLS: usize = CANVAS_WIDTH / 8;
pub const ATTR_ROWS: usize = CANVAS_HEIGHT / 8;

pub const BORDER_COLS: usize = 4;
pub const BORDER_ROWS: usize = 3;


pub const BYTES_PER_PIXEL: usize = 4;


/// Encode line number to read memory address
pub fn get_bitmap_line_addr(line: u16) -> u16 {
    // 0 1 0 Y7 Y6 Y2 Y1 Y0 | Y5 Y4 Y3 X4 X3 X2 X1 X0
    0x4000 | (line << 5) & 0x1800 | (line << 8) & 0x0700 | (line << 2) & 0x00E0
}

/// Get pixel id from address
pub fn get_bitmap_line(addr: u16) -> usize {
    let (h, l) = split_word(addr);
    // extract lowest 5 bits as x coordinate base
    let y = (h & 0x07) | ((l >> 2) & 0x38) | ((h << 3) & 0xC0);
    y as usize
}

pub fn get_bitmap_col(addr: u16) -> usize {
    let (_, l) = split_word(addr);
    // extract lowest 5 bits as x coordinate base
    (l & 0x1F) as usize
}

pub fn get_attr_row(addr: u16) -> usize {
    ((addr - 0x5800) / ATTR_COLS as u16) as usize
}

pub fn get_attr_col(addr: u16) -> usize {
    ((addr - 0x5800) % ATTR_COLS as u16) as usize
}


#[derive(Clone, Copy)]
enum ZXColor {
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

#[derive(Clone, Copy)]
pub struct ZXAttribute {
    ink: ZXColor,
    paper: ZXColor,
    flash: bool,
    bright: bool,
}

impl ZXAttribute {
    pub fn from_byte(data: u8) -> ZXAttribute {
        ZXAttribute {
            ink: ZXColor::from_bits(data & 0x07),
            paper: ZXColor::from_bits((data >> 3) & 0x07),
            flash: (data & 0x80) != 0,
            bright: (data & 0x40) != 0,
        }
    }
}


pub struct ZXPalette;

impl ZXPalette {
    pub fn default() -> ZXPalette {
        ZXPalette
    }
    pub fn get_rgba(&self, attr: &ZXAttribute, state: bool, flash_state: bool)
        -> [u8; BYTES_PER_PIXEL] {
        let base_color = if attr.bright {
            0xFF
        } else {
            0x88
        };
        let color = if state ^ (attr.flash & flash_state) {
            attr.ink
        } else {
            attr.paper
        };
        match color {
            ZXColor::Black => [0x00, 0x00, 0x00, 0xFF],
            ZXColor::Blue => [0x00, 0x00, base_color, 0xFF],
            ZXColor::Red => [base_color, 0x00, 0x00, 0xFF],
            ZXColor::Purple => [base_color, 0x00, base_color, 0xFF],
            ZXColor::Green => [0x00, base_color, 0x00, 0xFF],
            ZXColor::Cyan => [0x00, base_color, base_color, 0xFF],
            ZXColor::Yellow => [base_color, base_color, 0x00, 0xFF],
            ZXColor::White => [base_color, base_color, base_color, 0xFF],
        }
    }
}

/// Screen
pub struct ZXScreen {
    // 4 rgba bytes per pixel
    attributes: [[ZXAttribute; ATTR_COLS]; ATTR_ROWS],
    bitmap: [[u8; ATTR_COLS]; CANVAS_HEIGHT],
    buffer: [u8; PIXEL_COUNT * BYTES_PER_PIXEL],
    machine: ZXMachine,
    palette: ZXPalette,
    flash: bool,
    frame_counter: u64,
}

impl ZXScreen {
    /// Returns new screen intance
    pub fn new(machine_type: ZXMachine, palette_type: ZXPalette) -> ZXScreen {
        ZXScreen {
            attributes: [[ZXAttribute::from_byte(0); ATTR_COLS]; ATTR_ROWS],
            bitmap: [[0; ATTR_COLS]; CANVAS_HEIGHT],
            buffer: [0; PIXEL_COUNT * BYTES_PER_PIXEL],
            machine: machine_type,
            palette: palette_type,
            flash: false,
            frame_counter: 0,
        }
    }

    /// Clears screen
    pub fn clear(&mut self) {
        // fill with zeros
        self.attributes = [[ZXAttribute::from_byte(0); ATTR_COLS]; ATTR_ROWS];
        self.bitmap = [[0; ATTR_COLS]; CANVAS_HEIGHT];
        self.buffer = [0; PIXEL_COUNT * BYTES_PER_PIXEL];
    }

    pub fn new_frame(&mut self) {
        self.frame_counter += 1;
        if self.frame_counter % 32 == 0 {
            self.flash = !self.flash;
            //println!("Flash!", );
        }
        for line in 0..CANVAS_HEIGHT {
            for col in 0.. ATTR_COLS {
                self.update_buffer_block(line, col);
            }
        }
    }

    fn update_buffer_block(&mut self, line: usize, col: usize) {
        let data = self.bitmap[line][col];
        let row = line /8;
        // get base block index (8x1 stripe)
        let block_base_index = (((line + CANVAS_Y) * SCREEN_WIDTH) + col * 8 + CANVAS_X) *
            BYTES_PER_PIXEL;
        // current attribute of block
        let block_attr = self.attributes[row][col];
        // write pixels to buffer
        for bit in 0..8 {
            let pixel = block_base_index + bit * BYTES_PER_PIXEL;
            let state = ((data << bit) & 0x80) != 0;
            let color = self.palette.get_rgba(&block_attr, state, self.flash);
            self.buffer[pixel..pixel + BYTES_PER_PIXEL]
                .clone_from_slice(&color);
        }
    }
    /// Writes bitmap with `address` to screen representation
    /// # Panics
    /// Panics when addr in not in 0x4000..0x5800 range
    pub fn write_bitmap_byte(&mut self, addr: u16, clocks: Clocks, data: u8) {
        // check address boundaries
        assert!(addr >= 0x4000 && addr <= 0x57FF);
        let line = get_bitmap_line(addr);
        let col = get_bitmap_col(addr);
        self.bitmap[line][col] = data;
        let specs = self.machine.specs();
        // taking into acount that contention starts from first pixel clocks - 1
        let block_time = (specs.clocks_first_pixel - 1) + (line as u64) * specs.clocks_line +
            ((col as u64) / 2) * 8;
        if clocks.count() < block_time as usize {
            self.update_buffer_block(line, col);
        }
    }

    /// Writes attribute with `address` to screen representation
    pub fn write_attr_byte(&mut self, addr: u16, clocks: Clocks, value: u8) {
        assert!(addr >= 0x5800 && addr <= 0x5AFF);
        let row = get_attr_row(addr);
        let col = get_attr_col(addr);
        self.attributes[row][col] = ZXAttribute::from_byte(value);
        let specs = self.machine.specs();
        // taking into acount that contention starts from first pixel clocks - 1
        let last_block_time = (specs.clocks_first_pixel - 1) +
        ((row as u64) * 8 + 7) * specs.clocks_line + (col as u64 / 2) * 8;

        if clocks.count() <= last_block_time as usize {
            let beam_line = if clocks.count() < (specs.clocks_first_pixel as usize - 1) {
                0
            } else {
                ((clocks.count() -
                    (specs.clocks_first_pixel as usize - 1)) / specs.clocks_line as usize)
            };
            self.update_buffer_block(row * 8 + beam_line % 8, col);
            for line_shift in (beam_line % 8)..8 {
                self.update_buffer_block(row * 8 + line_shift, col);
            }
        }
    }

    /// Clones screen texture
    pub fn clone_texture(&self) -> &[u8] {
        &self.buffer
    }
}
