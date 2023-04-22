//! platform-independent traits. Submodules with backends will be selectable
//! via cargo features in future
mod palette;

pub mod wgpu;

pub use palette::{Palette, ColorRgba, ColorIndexed, PALETTE_SIZE, zx_color_to_index};

/// Texture id binging
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct TextureInfo {
    id: usize,
    width: u32,
    height: u32,
}

/// Simple rect struct
pub struct Rect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

impl Rect {
    /// Constructs new rect
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }
}

/// provides video functionality through real backend to emulator
pub trait VideoDevice {
    /// generates and returns texture handle
    fn gen_texture(&mut self, width: u32, height: u32) -> TextureInfo;
    /// updates texture data
    fn update_texture(&mut self, tex: TextureInfo, buffer: &[u8]);
    /// starts render block
    fn begin(&mut self);
    /// draws plain texure into destination rect
    fn draw_texture_2d(&mut self, tex: TextureInfo, rect: Option<Rect>);
    /// finishes rendering
    fn end(&mut self);
}
