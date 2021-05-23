use super::{Rect, TextureInfo, VideoDevice};
use crate::{app::settings::Settings, backends::SDL_CONTEXT};
use rustzx_core::zx::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::{
    pixels::PixelFormatEnum as PixelFormat,
    rect::Rect as SdlRect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use std::collections::HashMap;

/// Represents real SDL video backend
pub struct VideoSdl {
    renderer: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    texteres: HashMap<TextureInfo, Texture>,
    next_tex_id: usize,
}

impl VideoSdl {
    /// constructs new renderer with application settings
    pub fn new(settings: &Settings) -> VideoSdl {
        // init video subsystem
        let mut video_subsystem = None;
        SDL_CONTEXT.with(|sdl| {
            video_subsystem = sdl.borrow_mut().video().ok();
        });
        if let Some(video) = video_subsystem {
            // construct window and renderer form it
            let (width, height) = (
                SCREEN_WIDTH * settings.scale,
                SCREEN_HEIGHT * settings.scale,
            );
            let window = video
                .window("RustZX", width as u32, height as u32)
                .position_centered()
                .opengl()
                .build()
                .expect("[ERROR] Sdl window buil fail");
            let renderer = window
                .into_canvas()
                .present_vsync()
                .build()
                .expect("[ERROR] Sdl Canvas build error");
            let texture_creator = renderer.texture_creator();
            VideoSdl {
                renderer,
                texture_creator,
                texteres: HashMap::new(),
                next_tex_id: 0,
            }
        } else {
            panic!("[ERROR] Sdl video init fail!");
        }
    }
}

impl VideoDevice for VideoSdl {
    fn gen_texture(&mut self, width: u32, height: u32) -> TextureInfo {
        let id = self.next_tex_id;
        // create texture in backend
        let tex = self
            .texture_creator
            .create_texture_streaming(PixelFormat::ABGR8888, width, height)
            .expect("[ERROR] Sdl texture creation error");
        let tex_info = TextureInfo { id, width, height };
        // bind id in map
        self.texteres.insert(tex_info, tex);
        self.next_tex_id += 1;
        tex_info
    }

    fn set_title(&mut self, title: &str) {
        self.renderer.window_mut().set_title(title).unwrap();
    }

    fn update_texture(&mut self, tex: TextureInfo, buffer: &[u8]) {
        // find texture
        let tex_sdl = self
            .texteres
            .get_mut(&tex)
            .expect("[ERROR] Wrong texrure ID on update");
        // send data
        tex_sdl
            .with_lock(None, |out, pitch| {
                for y in 0..tex.height {
                    for x in 0..tex.width {
                        let offset_dest = (y * pitch as u32 + x * 4) as usize;
                        let offset_src = (y * tex.width * 4 + x * 4) as usize;
                        out[offset_dest..(4 + offset_dest)]
                            .clone_from_slice(&buffer[offset_src..(4 + offset_src)]);
                    }
                }
            })
            .expect("[ERROR] Texture update error");
    }

    fn begin(&mut self) {
        // clear surface
        self.renderer.clear();
    }

    fn draw_texture_2d(&mut self, tex: TextureInfo, rect: Option<Rect>) {
        // find texture
        let tex = self
            .texteres
            .get_mut(&tex)
            .expect("[ERROR] Wrong texrure ID on draw");
        // construct sdl rect
        let dest_rect = rect.map(|rect| SdlRect::new(rect.x, rect.y, rect.w, rect.h));
        // render
        self.renderer
            .copy(tex, None, dest_rect)
            .expect("[ERROR] Can't draw texture");
    }

    fn end(&mut self) {
        // display buffer
        self.renderer.present();
    }
}
