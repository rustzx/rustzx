//! ZX Spectrum screen module
//! Consists of ZXCanvas type and functions for bitmap addr encode/decode
//! Produces RGBA bitmap screen
//! *Block* in this module is 8x1 pixels chunk
//! *Col* and *Row* are 8 pixels chunks.
//! **Emulated** border is 32 pixels wide and 24 pixels tall

use utils::*;
use super::colors::*;
use zx::machine::ZXMachine;
use zx::constants::*;
use utils::screen::*;

/// ZXSpectrum screen sctruct
pub struct ZXCanvas {
    attributes: [[ZXAttribute; ATTR_COLS]; ATTR_ROWS],
    bitmap: [[u8; ATTR_COLS]; CANVAS_HEIGHT],
    // Output texture
    backbuffer: [u8; PIXEL_COUNT * BYTES_PER_PIXEL],
    buffer: [u8; PIXEL_COUNT * BYTES_PER_PIXEL],
    machine: ZXMachine,
    palette: ZXPalette,
    flash: bool,
    frame_counter: u64,
}

impl ZXCanvas {
    /// Returns new screen intance
    pub fn new(machine_type: ZXMachine, palette_type: ZXPalette) -> ZXCanvas {
        ZXCanvas {
            attributes: [[ZXAttribute::from_byte(0); ATTR_COLS]; ATTR_ROWS],
            bitmap: [[0; ATTR_COLS]; CANVAS_HEIGHT],
            buffer: [0; PIXEL_COUNT * BYTES_PER_PIXEL],
            backbuffer: [0; PIXEL_COUNT * BYTES_PER_PIXEL],
            machine: machine_type,
            palette: palette_type,
            flash: false,
            frame_counter: 0,
        }
    }

    /// Invokes actions, preformed at frame start (screen redraw)
    pub fn new_frame(&mut self) {
        self.frame_counter += 1;
        if self.frame_counter % 16 == 0 {
            self.flash = !self.flash;
        }
        self.backbuffer.clone_from_slice(&self.buffer);
        for line in 0..CANVAS_HEIGHT {
            for col in 0.. ATTR_COLS {
                self.update_buffer_block(line, col);
            }
        }
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
            let state = (((data << bit) & 0x80) != 0) ^ (block_attr.flash & self.flash);
            let color = if state {
                block_attr.ink
            } else {
                block_attr.paper
            };
            let color_array = self.palette.get_rgba(color,
                block_attr.brightness
            );
            self.buffer[pixel..pixel + BYTES_PER_PIXEL].clone_from_slice(color_array);
        }
    }

    /// Writes bitmap with `address` to screen representation
    /// # Panics
    /// Panics when addr in not in 0x4000..0x5800 range
    pub fn write_bitmap_byte(&mut self, addr: u16, clocks: Clocks, data: u8) {
        // check address boundaries
        assert!(addr >= BITMAP_BASE_ADDR && addr < ATTR_BASE_ADDR);
        let line = get_bitmap_line(addr);
        let col = get_bitmap_col(addr);
        self.bitmap[line][col] = data;
        let specs = self.machine.specs();
        let clocks_origin = specs.clocks_ula_read_origin;
        // taking into acount that contention starts from first pixel clocks - 1
        let block_time = clocks_origin + line * specs.clocks_line as usize + (col / 2) * 8;
        if clocks.count() < block_time {
            self.update_buffer_block(line, col);
        }
    }

    /// Writes attribute with `address` to screen representation
    pub fn write_attr_byte(&mut self, addr: u16, clocks: Clocks, value: u8) {
        assert!(addr >= ATTR_BASE_ADDR && addr <= ATTR_MAX_ADDR);
        let row = get_attr_row(addr);
        let col = get_attr_col(addr);
        self.attributes[row][col] = ZXAttribute::from_byte(value);
        let specs = self.machine.specs();

        let clocks_origin = specs.clocks_ula_read_origin;
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
    pub fn texture(&self) -> &[u8] {
        &self.backbuffer
    }
}
