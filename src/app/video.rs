use std::borrow::Cow;

use glium::{Surface, VertexBuffer, Program};
use glium::uniforms::*;
use glium::texture::{RawImage2d, ClientFormat};
use glium::texture::unsigned_texture2d::UnsignedTexture2d;
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::backend::glutin_backend::GlutinFacade;

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coord);

const SCREEN_VERTS: [Vertex; 6] = [
    Vertex { position: [-1.0, -1.0], tex_coord: [0.0 , 1.0] },
    Vertex { position: [ 1.0, -1.0], tex_coord: [1.0 , 1.0] },
    Vertex { position: [-1.0,  1.0], tex_coord: [0.0 , 0.0] },
    Vertex { position: [-1.0,  1.0], tex_coord: [0.0 , 0.0] },
    Vertex { position: [ 1.0, -1.0], tex_coord: [1.0 , 1.0] },
    Vertex { position: [ 1.0,  1.0], tex_coord: [1.0 , 0.0] },
];

const BORDER_MATRIX: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

pub struct ZXScreenRenderer {
    screen_vb: VertexBuffer<Vertex>,
    screen_idx: NoIndices,
    shader: Program,
    screen_matrix: [[f32; 4]; 4],
    border_color: u8,
    blink: bool,
}

impl ZXScreenRenderer {
    pub fn new<F: Facade>(display: &F) -> ZXScreenRenderer {
        let vb = VertexBuffer::new(display, &SCREEN_VERTS).unwrap();
        let idx  = NoIndices(PrimitiveType::TrianglesList);
        let vert_shader = include_str!("shaders/vert.glsl");
        let frag_shader = include_str!("shaders/frag.glsl");
        let program = Program::from_source(display, vert_shader, frag_shader, None).unwrap();
        // 384х288
        // 384 = 256 + 64 + 64
        // 288 = 192 + 48 + 48
        let sx = 256.0 / 384.0;
        let sy = 192.0 / 288.0;
        let mat = [
            [ sx, 0.0, 0.0, 0.0],
            [0.0,  sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        ZXScreenRenderer {
            screen_vb: vb,
            screen_idx: idx,
            shader: program,
            screen_matrix: mat,
            border_color: 0,
            blink: false,
        }
    }
    pub fn set_border_color(&mut self, col: u8) {
        self.border_color = col;
    }
    pub fn invert_blink(&mut self) {
        self.blink = !self.blink;
    }
    pub fn draw_screen(&self, display: &GlutinFacade, screen: &[u8]) {
        let screen_raw = RawImage2d {
            data: Cow::Borrowed(screen),
            width: 256,
            height: 192,
            format: ClientFormat::U4U4U4U4,
        };
        let bcolor = [self.border_color << 4, 0x00];
        let border_raw = RawImage2d {
            data: Cow::Borrowed(&bcolor),
            width: 1,
            height: 1,
            format: ClientFormat::U4U4U4U4
        };
        let screen_tex = UnsignedTexture2d::new(display, screen_raw).unwrap();
        let border_tex = UnsignedTexture2d::new(display, border_raw).unwrap();
        let uniforms_screen = uniform![
            tex: Sampler::new(&screen_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            matrix: self.screen_matrix,
            blink: self.blink,
        ];
        let uniforms_border = uniform![
            tex: Sampler::new(&border_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            matrix: BORDER_MATRIX,
            blink: false,
        ];
        let mut target = display.draw();
        target.draw(&self.screen_vb, &self.screen_idx,
                    &self.shader, &uniforms_border, &Default::default()).unwrap();
        target.draw(&self.screen_vb, &self.screen_idx,
                    &self.shader, &uniforms_screen, &Default::default()).unwrap();

        target.finish().unwrap();
    }


}
