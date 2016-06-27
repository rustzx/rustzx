//! Module with glium-related types and functions for rendering screen
//! contains `ZXScreenRenderer`

use glium::{Surface, VertexBuffer, Program};
use glium::uniforms::*;
use glium::texture::RawImage2d;
use glium::texture::texture2d::Texture2d;
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::backend::glutin_backend::GlutinFacade;

use zx::constants::*;

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
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]];
        ZXScreenRenderer {
            screen_vb: vb,
            screen_idx: idx,
            shader: program,
            screen_matrix: mat,
        }
    }

    /// Main screen rendering function
    pub fn draw_screen(&self, display: &GlutinFacade, border: &[u8], screen: &[u8]) {
        // generate screen tex
        let screen_bitmap = RawImage2d::from_raw_rgba(screen.to_vec(),
                                                      (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32));
        let screen_tex = Texture2d::new(display, screen_bitmap).unwrap();
        // generate border tex
        let border_bitmap = RawImage2d::from_raw_rgba(border.to_vec(),
                                                      (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32));
        let border_tex = Texture2d::new(display, border_bitmap).unwrap();
        // uniforms for screen
        let uniforms = uniform![
            tex_screen: Sampler::new(&screen_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            tex_border: Sampler::new(&border_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            matrix: self.screen_matrix,
        ];
        let mut target = display.draw();
        target.draw(&self.screen_vb,
                    &self.screen_idx,
                    &self.shader,
                    &uniforms,
                    &Default::default())
              .unwrap();
        target.finish().unwrap();
    }
}
