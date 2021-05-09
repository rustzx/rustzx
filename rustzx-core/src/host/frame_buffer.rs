use crate::zx::video::colors::{ZXBrightness, ZXColor};

pub enum FrameBufferSource {
    Screen,
    Border,
}

pub trait FrameBuffer {
    /// Creates canvas size with required dimensions (`width`, `height`)
    /// TODO: Add HostFrameBufferContext parameter to be able to pass additional
    /// host info
    fn new(width: usize, height: usize, source: FrameBufferSource) -> Self;
    /// Set `color` with `brightness` for pixel on canvas at (`x`, `y`)
    fn set_color(&mut self, x: usize, y: usize, color: ZXColor, brightness: ZXBrightness);
}
