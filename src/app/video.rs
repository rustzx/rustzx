//! Module with glium-related types and functions for rendering screen
//! contains `ZXScreenRenderer`

use glium::{Surface, VertexBuffer, Program};
use glium::uniforms::*;
use glium::texture::RawImage2d;
use glium::texture::texture2d::Texture2d;
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::backend::glutin_backend::GlutinFacade;

use zx::screen::*;

/// Custom vertex type for glium
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coord);

/// Coordinates of screen quad, constructed from two triangles
#[cfg_attr(rustfmt, rustfmt_skip)]
const SCREEN_VERTS: [Vertex; 6] = [
    Vertex { position: [-1.0, -1.0], tex_coord: [0.0 , 1.0] },
    Vertex { position: [ 1.0, -1.0], tex_coord: [1.0 , 1.0] },
    Vertex { position: [-1.0,  1.0], tex_coord: [0.0 , 0.0] },
    Vertex { position: [-1.0,  1.0], tex_coord: [0.0 , 0.0] },
    Vertex { position: [ 1.0, -1.0], tex_coord: [1.0 , 1.0] },
    Vertex { position: [ 1.0,  1.0], tex_coord: [1.0 , 0.0] },
];

/// const matrix of border object
#[cfg_attr(rustfmt, rustfmt_skip)]
const BORDER_MATRIX: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
];

/// Renderer object
pub struct ZXScreenRenderer {
    screen_vb: VertexBuffer<Vertex>,
    screen_idx: NoIndices,
    shader: Program,
    screen_matrix: [[f32; 4]; 4],
    border_color: u8,
}

impl ZXScreenRenderer {
    /// Returns new Renderer based on glium backend object (`Facade`)
    pub fn new<F: Facade>(display: &F) -> ZXScreenRenderer {
        let vb = VertexBuffer::new(display, &SCREEN_VERTS).unwrap();
        let idx = NoIndices(PrimitiveType::TrianglesList);
        let vert_shader = include_str!("shaders/vert.glsl");
        let frag_shader = include_str!("shaders/frag.glsl");
        let program = Program::from_source(display, vert_shader, frag_shader, None).unwrap();
        let mat = [[1.0, 0.0, 0.0, 0.0],
                   [0.0,1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]];
        ZXScreenRenderer {
            screen_vb: vb,
            screen_idx: idx,
            shader: program,
            screen_matrix: mat,
            border_color: 0,
        }
    }
    /// selects border color
    pub fn set_border_color(&mut self, col: u8) {
        self.border_color = col;
    }

    /// Main screen rendering function
    pub fn draw_screen(&self, display: &GlutinFacade, screen: &[u8]) {
        let bitmap = RawImage2d::from_raw_rgba(screen.to_vec(),
            (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32));
        let screen_tex = Texture2d::new(display, bitmap).unwrap();
        let uniforms_screen = uniform![
            tex: Sampler::new(&screen_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            matrix: self.screen_matrix,
        ];
        let mut target = display.draw();
        target.draw(&self.screen_vb,
                    &self.screen_idx,
                    &self.shader,
                    &uniforms_screen,
                    &Default::default())
              .unwrap();
        target.finish().unwrap();
    }
}
