//! Module with glium-related types and functions for rendering screen
//! contains `ZXScreenRenderer`
//std
use std::ops::Range;
// glium
use glium::{Surface, VertexBuffer, Program};
use glium::uniforms::*;
use glium::texture::RawImage2d;
use glium::texture::texture2d::Texture2d;
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::backend::glutin_backend::GlutinFacade;
// gc math section
use cgmath::{Vector3, Matrix4, ortho};
use cgmath::prelude::One;
// emulator section
use zx::constants::*;

/// Custom vertex type for glium
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coord);

/// Constructs new vertex buffer as triangle fan
#[cfg_attr(rustfmt, rustfmt_skip)]
fn construct_rect_vb(x: f32, y: f32, width: f32, height: f32) -> [Vertex; 4] {
    [
        Vertex { position: [x        , y         ], tex_coord: [0.0 , 0.0] },
        Vertex { position: [width + x, y         ], tex_coord: [1.0 , 0.0] },
        Vertex { position: [width + x, height + y], tex_coord: [1.0 , 1.0] },
        Vertex { position: [x        , height + y], tex_coord: [0.0 , 1.0] },
    ]
}

/// Builds 2d ortho projection
fn ortho2d(width: usize, height: usize) -> Matrix4<f32> {
    ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
}

// Ranges of vertex indices in vb
const SCREEN_VB_RANGE: Range<usize> = 0..4;
const CANVAS_VB_RANGE: Range<usize> = 4..8;

/// Renderer object
pub struct ZXScreenRenderer {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,
    shader: Program,
    view_matrix: Matrix4<f32>,
    correction_matrix: Matrix4<f32>,
}

impl ZXScreenRenderer {
    /// Returns new Renderer based on glium backend object (`Facade`)
    pub fn new<F: Facade>(display: &F) -> ZXScreenRenderer {
        // build vertices for primitives
        let vb_scr = construct_rect_vb(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        let vb_canvas = construct_rect_vb(CANVAS_X as f32, CANVAS_Y as f32,
                                          CANVAS_WIDTH as f32, CANVAS_HEIGHT as f32);
        // merge them
        let mut vb = vb_scr.to_vec();
        vb.extend_from_slice(&vb_canvas);
        let program = Program::from_source(display,
                                           include_str!("shaders/vert.glsl"),
                                           include_str!("shaders/frag.glsl"),
                                           None).unwrap();
        ZXScreenRenderer {
            vertex_buffer: VertexBuffer::new(display, &vb).unwrap(),
            index_buffer: NoIndices(PrimitiveType::TriangleFan),
            shader: program,
            // standard gl ortho projection
            view_matrix: ortho2d(SCREEN_WIDTH, SCREEN_HEIGHT),
            // no correction, initial screen is normalized
            correction_matrix: Matrix4::one(),
        }
    }

    /// Main screen rendering function
    pub fn draw_screen(&self, display: &GlutinFacade, border: &[u8], canvas: &[u8]) {
        // generate screen tex
        let canvas_bitmap = RawImage2d::from_raw_rgba(canvas.to_vec(),
                                                      (CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32));
        let canvas_tex = Texture2d::new(display, canvas_bitmap).unwrap();
        // generate border tex
        let border_bitmap = RawImage2d::from_raw_rgba(border.to_vec(),
                                                      (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32));
        let border_tex = Texture2d::new(display, border_bitmap).unwrap();
        // make aspect correction
        let view_matrix = self.view_matrix * self.correction_matrix;
        // get matrix array for glium
        let raw_view_matrix: &[[f32; 4]; 4] = view_matrix.as_ref();
        // build uniforms
        let uniforms_border = uniform![
            tex: Sampler::new(&border_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            view_matrix: *raw_view_matrix,
        ];
        let uniforms_canvas = uniform![
            tex: Sampler::new(&canvas_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            view_matrix: *raw_view_matrix,
        ];
        // start rendering
        let mut target = display.draw();
        // fill screen with black color
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        // render border on whole Screen rect
        target.draw(self.vertex_buffer.slice(SCREEN_VB_RANGE).unwrap(),
                    &self.index_buffer,
                    &self.shader,
                    &uniforms_border,
                    &Default::default())
              .unwrap();
        //  render canvas only on canvas rect
        target.draw(self.vertex_buffer.slice(CANVAS_VB_RANGE).unwrap(),
                    &self.index_buffer,
                    &self.shader,
                    &uniforms_canvas,
                    &Default::default())
            .unwrap();
        target.finish().unwrap();
    }

    /// Makes viewport aspect ratio correction
    /// `with` and `height` - size of new screen
    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        let base_aspect_width = SCREEN_WIDTH as f32 / SCREEN_HEIGHT  as f32;
        let base_aspect_height = SCREEN_HEIGHT as f32 / SCREEN_WIDTH  as f32;
        let (trans_matrix, scale_matrix);
        if width as f32 >= height as f32 * base_aspect_width {
            // find x scale
            let scale_x = (height as f32 / width as f32) * base_aspect_width;
            scale_matrix = Matrix4::from_nonuniform_scale(scale_x, 1.0, 1.0);
            // find x shift
            let width_scaled = scale_x * SCREEN_WIDTH as f32;
            trans_matrix = Matrix4::from_translation(
                Vector3::new((SCREEN_WIDTH as f32 - width_scaled) / 2.0, 0.0 , 0.0));
        } else {
            let scale_y = (width as f32 / height as f32) * base_aspect_height;
            scale_matrix = Matrix4::from_nonuniform_scale(1.0, scale_y, 1.0);
            // find y shift
            let height_scaled = scale_y * SCREEN_HEIGHT as f32;
            trans_matrix = Matrix4::from_translation(
                Vector3::new(0.0, (SCREEN_HEIGHT as f32 - height_scaled) / 2.0, 0.0));
        }
        // save matrix
        self.correction_matrix = trans_matrix * scale_matrix;
    }

}
