//! Module with glium-related types and functions for rendering screen
//! contains `ZXScreenRenderer`

use glium::{Surface, VertexBuffer, Program};
use glium::uniforms::*;
use glium::texture::RawImage2d;
use glium::texture::texture2d::Texture2d;
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::backend::glutin_backend::GlutinFacade;

use cgmath::{Vector3, Matrix4, ortho};
use cgmath::prelude::One;

use zx::constants::*;

/// Custom vertex type for glium
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coord);

/// Constructs new vertex buffer
fn construct_rect_vb(x: f32, y: f32, width: f32, height: f32) -> [Vertex; 6] {
    [
        Vertex { position: [ x,  y], tex_coord: [0.0 , 1.0] },
        Vertex { position: [width + x,  y], tex_coord: [1.0 , 1.0] },
        Vertex { position: [ x, height + y], tex_coord: [0.0 , 0.0] },
        Vertex { position: [ x, height + y], tex_coord: [0.0 , 0.0] },
        Vertex { position: [width + x, y], tex_coord: [1.0 , 1.0] },
        Vertex { position: [width + x, height + y], tex_coord: [1.0 , 0.0] },
    ]
}

lazy_static! {
    static ref CANVAS_MATRIX: Matrix4<f32> = {
        Matrix4::one()
    };
    static ref SCREEN_MATRIX: Matrix4<f32> = {
        Matrix4::one()
    };
}

/// Renderer object
pub struct ZXScreenRenderer {
    screen_vb: VertexBuffer<Vertex>,
    canvas_vb: VertexBuffer<Vertex>,
    rect_idx: NoIndices,
    shader: Program,
    view_matrix: Matrix4<f32>,
    corection_matrix: Matrix4<f32>,
}

impl ZXScreenRenderer {
    /// Returns new Renderer based on glium backend object (`Facade`)
    pub fn new<F: Facade>(display: &F) -> ZXScreenRenderer {
        let vb_arr_screen = construct_rect_vb(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        let vb_arr_canvas = construct_rect_vb(CANVAS_X as f32, CANVAS_Y as f32,
                                              CANVAS_WIDTH as f32, CANVAS_HEIGHT as f32);
        let vert_shader = include_str!("shaders/vert.glsl");
        let frag_shader = include_str!("shaders/frag.glsl");
        let program = Program::from_source(display, vert_shader, frag_shader, None).unwrap();
        ZXScreenRenderer {
            screen_vb: VertexBuffer::new(display, &vb_arr_screen).unwrap(),
            canvas_vb: VertexBuffer::new(display, &vb_arr_canvas).unwrap(),
            rect_idx: NoIndices(PrimitiveType::TrianglesList),
            shader: program,
            view_matrix: ortho(0.0, SCREEN_WIDTH as f32,  0.0, SCREEN_HEIGHT as f32, -1.0, 1.0),
            corection_matrix: Matrix4::one(),
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
        // uniforms for screen
        let combo_matrix = self.view_matrix * self.corection_matrix;
        let mat_view: &[[f32; 4]; 4] = combo_matrix.as_ref();
        let mat_border: &[[f32; 4]; 4] = SCREEN_MATRIX.as_ref();
        let mat_canvas: &[[f32; 4]; 4] = CANVAS_MATRIX.as_ref();
        let uniforms_border = uniform![
            tex: Sampler::new(&border_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            model_matrix: *mat_border,
            view_matrix: *mat_view,
        ];
        let uniforms_canvas = uniform![
            tex: Sampler::new(&canvas_tex).magnify_filter(MagnifySamplerFilter::Nearest),
            model_matrix: *mat_canvas,
            view_matrix: *mat_view,
        ];
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&self.screen_vb,
                    &self.rect_idx,
                    &self.shader,
                    &uniforms_border,
                    &Default::default())
              .unwrap();
        target.draw(&self.canvas_vb,
                    &self.rect_idx,
                    &self.shader,
                    &uniforms_canvas,
                    &Default::default())
            .unwrap();
        target.finish().unwrap();
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        let base_aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT  as f32;
        if width as f32 >= height as f32 * base_aspect {
            // find scale
            let window_aspect = height as f32 / width as f32;
            let base_aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT  as f32;
            let scale = window_aspect * base_aspect;
            let scale_matrix = Matrix4::from_nonuniform_scale(scale, 1.0, 1.0);
            // find x shift
            let width_scaled = scale * SCREEN_WIDTH as f32;
            let trans_matrix = Matrix4::from_translation(
                Vector3::new((SCREEN_WIDTH as f32 - width_scaled) / 2.0, 0.0 , 0.0));
            // save matrix
            self.corection_matrix = trans_matrix * scale_matrix;
        } else {
            // find scale
            let window_aspect = width as f32 / height as f32;
            let base_aspect = SCREEN_HEIGHT as f32 / SCREEN_WIDTH  as f32;
            let scale = window_aspect * base_aspect;
            let scale_matrix = Matrix4::from_nonuniform_scale(1.0, scale, 1.0);
            // find y shift
            let height_scaled = scale * SCREEN_HEIGHT as f32;
            let trans_matrix = Matrix4::from_translation(
                Vector3::new(0.0, (SCREEN_HEIGHT as f32 - height_scaled) / 2.0, 0.0));
            // save matrix
            self.corection_matrix = trans_matrix * scale_matrix;
        }
    }

}
