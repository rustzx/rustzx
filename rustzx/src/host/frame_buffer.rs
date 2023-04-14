use crate::app::video::{Palette, zx_color_to_index};
use rustzx_core::{
    host::{FrameBuffer, FrameBufferSource},
    zx::video::colors::{ZXBrightness, ZXColor},
};

#[derive(Clone)]
pub struct FrameBufferContext;

pub struct IndexedFrameBuffer {
    buffer: Vec<u8>,
    row_size: usize,
}

impl FrameBuffer for IndexedFrameBuffer {
    type Context = FrameBufferContext;

    fn new(
        width: usize,
        height: usize,
        _source: FrameBufferSource,
        _context: Self::Context,
    ) -> Self {
        Self {
            buffer: vec![0u8; width * height],
            row_size: width,
        }
    }

    fn set_color(&mut self, x: usize, y: usize, color: ZXColor, brightness: ZXBrightness) {
        self.buffer[ y * self.row_size + x] = zx_color_to_index(color, brightness);
    }
}

impl IndexedFrameBuffer {
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }
}
