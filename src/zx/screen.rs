//! ZX Spectrum screen module
//! Consists of ZXScreen type and functions for bitmap addr encode/decode
//! Produces RGBA bitmap screen
//! *Block* in this module is 8x1 pixels chunk
//! *Col* and *Row* are 8 pixels chunks.
//! **Emulated** border is 32 pixels wide and 24 pixels tall

use utils::*;
use super::machine::ZXMachine;

pub const CANVAS_WIDTH: usize = 256;
pub const CANVAS_HEIGHT: usize = 192;
pub const CANVAS_X: usize = 32;
pub const CANVAS_Y: usize = 24;

pub const SCREEN_WIDTH: usize = CANVAS_WIDTH + 32 * 2;
pub const SCREEN_HEIGHT: usize = CANVAS_HEIGHT + 24 * 2;
pub const PIXEL_COUNT: usize = SCREEN_HEIGHT * SCREEN_WIDTH;

pub const ATTR_COLS: usize = CANVAS_WIDTH / 8;
pub const ATTR_ROWS: usize = CANVAS_HEIGHT / 8;

pub const BORDER_COLS: usize = 4;
pub const BORDER_ROWS: usize = 3;

pub const BYTES_PER_PIXEL: usize = 4;

pub const CLOCKS_PER_COL: usize = 4;
pub const PIXELS_PER_CLOCK: usize = 2;

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

/// get bitmap column from address
pub fn get_bitmap_col(addr: u16) -> usize {
    let (_, l) = split_word(addr);
    // extract lowest 5 bits as x coordinate base
    (l & 0x1F) as usize
}

/// get attribute row from address
pub fn get_attr_row(addr: u16) -> usize {
    ((addr - 0x5800) / ATTR_COLS as u16) as usize
}

/// get attribute column from address
pub fn get_attr_col(addr: u16) -> usize {
    ((addr - 0x5800) % ATTR_COLS as u16) as usize
}

/// ZX Spectrum color enum
/// Constructs self from 3-bit value
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

#[derive(Clone, Copy)]
struct ZXScreenPixel {
    line: usize,
    pixel: usize,
}

impl ZXScreenPixel {
    fn first() -> ZXScreenPixel {
        ZXScreenPixel {
            line: 0,
            pixel: 0,
        }
    }

    fn at_yx(line: usize, pixel: usize) -> ZXScreenPixel {
        ZXScreenPixel {
            line: line,
            pixel: pixel,
        }
    }

    fn is_first(self) -> bool {
        (self.line == 0) && (self.pixel == 0)
    }
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
    ink: ZXColor,
    paper: ZXColor,
    flash: bool,
    bright: bool,
}

impl ZXAttribute {
    /// Constructs self from byte
    pub fn from_byte(data: u8) -> ZXAttribute {
        ZXAttribute {
            ink: ZXColor::from_bits(data & 0x07),
            paper: ZXColor::from_bits((data >> 3) & 0x07),
            flash: (data & 0x80) != 0,
            bright: (data & 0x40) != 0,
        }
    }
}


/// Structure, that holds palette information.
/// It have method to transform ZX Spectrum screen data
/// to 4-byte rgba bixel
pub struct ZXPalette;

impl ZXPalette {
    /// Returns default palette
    /// TODO: Use `Default` trait?
    pub fn default() -> ZXPalette {
        ZXPalette
    }
    /// Returns rgba pixel from screen data
    pub fn get_rgba(&self, attr: &ZXAttribute, state: bool,
        flash_state: bool) -> [u8; BYTES_PER_PIXEL] {
        let base_color = if attr.bright {
            0xFF
        } else {
            0x88
        };
        let color = if state ^  (attr.flash & flash_state) {
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

/// ZXSpectrum screen sctruct
pub struct ZXScreen {
    // 4 rgba bytes per pixel
    attributes: [[ZXAttribute; ATTR_COLS]; ATTR_ROWS],
    bitmap: [[u8; ATTR_COLS]; CANVAS_HEIGHT],
    buffer: [u8; PIXEL_COUNT * BYTES_PER_PIXEL],
    machine: ZXMachine,
    palette: ZXPalette,
    flash: bool,
    frame_counter: u64,
    last_border_pixel: ZXScreenPixel,
    last_border_color: u8,
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
            last_border_pixel: ZXScreenPixel::first(),
            last_border_color: 0,
        }
    }

    /// fill stripe in frame buffer
    fn fill_border_stripe(&mut self, color: [u8; BYTES_PER_PIXEL], begin: usize, count: usize) {
        for pixel in begin..begin + count {
            if (pixel % SCREEN_WIDTH < BORDER_COLS * 8) ||
                (pixel % SCREEN_WIDTH >= BORDER_COLS * 8 + CANVAS_WIDTH) ||
                (pixel / SCREEN_WIDTH < BORDER_ROWS * 8) ||
                (pixel / SCREEN_WIDTH >= CANVAS_HEIGHT + BORDER_ROWS * 8) {
                self.buffer[pixel * BYTES_PER_PIXEL..(pixel + 1) * BYTES_PER_PIXEL]
                    .clone_from_slice(&color);
            }
        }
    }

    /// Changes border at given time
    pub fn set_border(&mut self, color: u8, clocks: Clocks) {
        let last_pixel = self.last_border_pixel;
        let last_color = self.last_border_color;
        let last_color_arr = self.palette.get_rgba(&ZXAttribute::from_byte(last_color), true, false);
        let next_pixel = self.next_border_pixel(clocks);
        if next_pixel.is_first() {
            return;
        }
        // fill middle
        let line_dt = next_pixel.line - self.last_border_pixel.line;
        // if we have full lines
        if line_dt > 1 {
            for line in (self.last_border_pixel.line + 1)..next_pixel.line {
                self.fill_border_stripe(last_color_arr, line * SCREEN_WIDTH, SCREEN_WIDTH);
            }
        };
        if line_dt == 0 {
            // if same line
            self.fill_border_stripe(last_color_arr,
                last_pixel.line * SCREEN_WIDTH + last_pixel.pixel,
                next_pixel.pixel - last_pixel.pixel);
        } else {
            // top half-stripe
            self.fill_border_stripe(last_color_arr,
                last_pixel.line * SCREEN_WIDTH + last_pixel.pixel,
                SCREEN_WIDTH - last_pixel.pixel);
            // bottom half-stripe
            self.fill_border_stripe(last_color_arr,
                next_pixel.line * SCREEN_WIDTH,
                next_pixel.pixel);
        }
        self.last_border_pixel = next_pixel;
        self.last_border_color = color;
    }
    /// Invokes actions, preformed at frame start (screen redraw)
    pub fn new_frame(&mut self) {

        self.frame_counter += 1;
        if self.frame_counter % 32 == 0 {
            self.flash = !self.flash;
        }
        for line in 0..CANVAS_HEIGHT {
            for col in 0.. ATTR_COLS {
                self.update_buffer_block(line, col);
            }
        }
        if !self.last_border_pixel.is_first() {
            let specs = self.machine.specs();
            let last_border_color = self.last_border_color;
            self.set_border(last_border_color, Clocks(specs.clocks_frame as usize));
        }
        self.last_border_pixel = ZXScreenPixel::first();
    }

    /// Updates given 8x1 block in pixel buffer
    fn update_buffer_block(&mut self, line: usize, col: usize) {
        let data = self.bitmap[line][col];
        let row = line / 8;
        // get base block index (8x1 stripe)
        let block_base_index = (((line + CANVAS_Y) * SCREEN_WIDTH) + CANVAS_X + col * 8) *
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


    /// ULA draws 2 pixels per TState.
    /// This function helps to determine pixel, which will be rendered at specific time
    fn next_border_pixel(&self, clocks: Clocks) -> ZXScreenPixel {
        let specs = self.machine.specs();
        // begining of the first line (first pixel timing minus border lines
        // minus left border columns)
        let clocks_origin = specs.clocks_first_pixel as usize + 2 -
            8 * BORDER_ROWS * specs.clocks_line as usize - BORDER_COLS * CLOCKS_PER_COL;
        // return first pixel index
        if clocks.count() < clocks_origin {
            return ZXScreenPixel::first();
        }
        let clocks = clocks.count() - clocks_origin;
        let mut line = clocks / specs.clocks_line as usize;
        // so, next pixel will be current + 2
        let mut pixel = (clocks % specs.clocks_line as usize + 1)
            * PIXELS_PER_CLOCK as usize;
        // if beam out of screen on horizontal pos.
        if pixel >= SCREEN_WIDTH {
            // first pixel of next line
            pixel = 0;
            line += 1;
        }
        // if beam out of screen on vertical pos.
        if line >= SCREEN_HEIGHT {
            return ZXScreenPixel::first();
        } else {
            return ZXScreenPixel::at_yx(line, pixel);
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

        let clocks_origin = specs.clocks_first_pixel as usize + 2;
        // taking into acount that contention starts from first pixel clocks - 1
        let block_time = clocks_origin + line * specs.clocks_line as usize +
            (col / 2) * 8;
        if clocks.count() < block_time {
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

        let clocks_origin = specs.clocks_first_pixel as usize + 2;
        let beam_line = if clocks.count() < clocks_origin + (col / 2 * 8) {
            0
        } else {
            ((clocks.count() - (clocks_origin + (col / 2 * 8))) / specs.clocks_line as usize) + 1
        };
        let block_time = if beam_line <= row * 8 {
            clocks_origin + (row * 8) * specs.clocks_line as usize + (col / 2) * 8
        }  else if beam_line < (row + 1) * 8 {
            clocks_origin + (row * 8 + beam_line % 8) * specs.clocks_line as usize + (col / 2) * 8
        } else {
            clocks_origin + ((row * 8) + 7) * specs.clocks_line as usize + (col / 2) * 8
        };
        // if next line of beam is smaller than next attr block
        if clocks.count() < block_time as usize {
            for line_shift in (beam_line % 8 + row * 8)..((row + 1) * 8) {
                self.update_buffer_block(line_shift, col);
            }
        }
    }

    /// Clones screen texture
    pub fn clone_texture(&self) -> &[u8] {
        &self.buffer
    }
}
