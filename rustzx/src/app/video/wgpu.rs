use crate::app::{
    settings::Settings,
    video::{Rect, TextureInfo, VideoDevice},
};
use rustzx_core::zx::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

/// Represents real SDL video backend
#[derive(Default)]
pub struct Device {}

impl VideoDevice for Device {
    fn gen_texture(&mut self, width: u32, height: u32) -> TextureInfo {
        TextureInfo {
            id: 0,
            width,
            height,
        }
    }

    fn set_title(&mut self, title: &str) {}

    fn update_texture(&mut self, tex: TextureInfo, buffer: &[u8]) {}

    fn begin(&mut self) {}

    fn draw_texture_2d(&mut self, tex: TextureInfo, rect: Option<Rect>) {}

    fn end(&mut self) {}
}
