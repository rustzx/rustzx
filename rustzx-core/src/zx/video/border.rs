//! Contains ZXSpectrum border implementation

use crate::{
    host::{FrameBuffer, FrameBufferSource},
    zx::{
        constants::{
            BORDER_COLS, BORDER_ROWS, CLOCKS_PER_COL, PIXELS_PER_CLOCK, SCREEN_HEIGHT, SCREEN_WIDTH,
        },
        machine::ZXMachine,
        video::colors::{ZXBrightness, ZXColor},
    },
};

/// Internal struct, which contains information about beam position and color
#[derive(Clone, Copy)]
struct BeamInfo {
    line: usize,
    pixel: usize,
    color: ZXColor,
}
impl BeamInfo {
    /// constructs self with given color at first pixel pos
    fn first_pixel(color: ZXColor) -> BeamInfo {
        BeamInfo::new(0, 0, color)
    }

    /// constructs self at given pos with given color
    fn new(line: usize, pixel: usize, color: ZXColor) -> BeamInfo {
        BeamInfo { line, pixel, color }
    }

    /// resets position
    fn reset(&mut self) {
        self.line = 0;
        self.pixel = 0;
    }
}

/// ZX Spectrum Border Device
pub struct ZXBorder<FB: FrameBuffer> {
    machine: ZXMachine,
    buffer: FB,
    beam_last: BeamInfo,
    border_changed: bool,
    beam_block: bool,
}
impl<FB: FrameBuffer> ZXBorder<FB> {
    /// Returns new instance of border device
    pub fn new(machine: ZXMachine, context: FB::Context) -> Self {
        ZXBorder {
            machine,
            buffer: FB::new(
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                FrameBufferSource::Border,
                context,
            ),
            beam_last: BeamInfo::first_pixel(ZXColor::White),
            border_changed: true,
            beam_block: false,
        }
    }

    /// ULA draws 2 pixels per TState.
    /// This function helps to determine pixel, which will be rendered at specific time
    /// and bool value, which signals end of frame
    fn next_border_pixel(&self, clocks: usize) -> (usize, usize, bool) {
        let specs = self.machine.specs();
        // beginning of the first line (first pixel timing minus border lines
        // minus left border columns)
        let clocks_origin = specs.clocks_first_pixel
            - 8 * BORDER_ROWS * specs.clocks_line
            - BORDER_COLS * CLOCKS_PER_COL
            + specs.clocks_ula_beam_shift;
        // return first pixel pos
        if clocks < clocks_origin {
            return (0, 0, false);
        }
        // get clocks relative to first pixel
        let clocks = clocks - clocks_origin;
        let mut line = clocks / specs.clocks_line;
        // so, next pixel will be current + 2
        let mut pixel = ((clocks % specs.clocks_line) + 1) * PIXELS_PER_CLOCK;
        // if beam out of screen on horizontal pos.
        // pixel - 2 because we added 2 on prev line
        if pixel - PIXELS_PER_CLOCK >= SCREEN_WIDTH {
            // first pixel of next line
            pixel = 0;
            line += 1;
        }
        // if beam out of screen on vertical pos.
        if line >= SCREEN_HEIGHT {
            (0, 0, true)
        } else {
            (line, pixel, false)
        }
    }

    /// fills pixels from last pos to passed by arguments with
    fn fill_to(&mut self, line: usize, pixel: usize) {
        let last = self.beam_last;
        for p in (last.line * SCREEN_WIDTH + last.pixel)..(line * SCREEN_WIDTH + pixel) {
            self.buffer.set_color(
                p % SCREEN_WIDTH,
                p / SCREEN_WIDTH,
                last.color,
                ZXBrightness::Normal,
            );
        }
    }

    /// starts new frame
    pub fn new_frame(&mut self) {
        // if border was not changed during prev frame then force change color of whole border
        if !self.border_changed {
            self.beam_last.reset();
        }
        // fill to end of screen if not already filled
        if !self.beam_block {
            self.fill_to(SCREEN_HEIGHT - 1, SCREEN_WIDTH);
        }
        // move beam to begin and reset flags
        self.beam_last.reset();
        self.border_changed = false;
        self.beam_block = false;
    }

    /// changes color of border
    pub fn set_border(&mut self, clocks: usize, color: ZXColor) {
        // border updated during frame
        self.border_changed = true;
        let (line, pixel, frame_end) = self.next_border_pixel(clocks);
        if !self.beam_block {
            // if not first pixel then update
            if frame_end {
                self.fill_to(SCREEN_HEIGHT - 1, SCREEN_WIDTH);
                self.beam_block = true;
            }
            self.fill_to(line, pixel);
        }
        self.beam_last = BeamInfo::new(line, pixel, color);
    }

    /// Returns reference to texture
    pub fn frame_buffer(&self) -> &FB {
        &self.buffer
    }
}
