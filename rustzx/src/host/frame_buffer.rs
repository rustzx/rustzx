use crate::app::video::Palette;
use rustzx_core::{
    host::{FrameBuffer, FrameBufferSource},
    zx::video::colors::{ZXBrightness, ZXColor},
};

const RGBA_PIXEL_SIZE: usize = 4;

#[derive(Clone)]
pub struct FrameBufferContext;

pub struct RgbaFrameBuffer {
    buffer: Vec<u8>,
    palette: Palette,
    buffer_row_size: usize,
}

impl FrameBuffer for RgbaFrameBuffer {
    type Context = FrameBufferContext;

    fn new(
        width: usize,
        height: usize,
        _source: FrameBufferSource,
        _context: Self::Context,
    ) -> Self {
        Self {
            buffer: vec![0u8; width * height * RGBA_PIXEL_SIZE],
            palette: Palette::default(),
            buffer_row_size: width * RGBA_PIXEL_SIZE,
        }
    }

    fn set_color(&mut self, x: usize, y: usize, color: ZXColor, brightness: ZXBrightness) {
        let buffer_pos = y * self.buffer_row_size + x * RGBA_PIXEL_SIZE;

        self.palette
            .get_rgba(color, brightness)
            .iter()
            .copied()
            .zip(&mut self.buffer[buffer_pos..buffer_pos + RGBA_PIXEL_SIZE])
            .for_each(|(source, dest)| *dest = source);
    }
}

impl RgbaFrameBuffer {
    pub fn rgba_data(&self) -> &[u8] {
        &self.buffer
    }
}
