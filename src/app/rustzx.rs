use std::borrow::Cow;

use glium::glutin::WindowBuilder;
use glium::glutin::{Event, VirtualKeyCode as VKey};
use glium::DisplayBuild;
use glium::Surface;
use glium::texture::texture2d::Texture2d;
use glium::texture::RawImage2d;
use glium::texture::ClientFormat;
use glium::VertexBuffer;
use glium::index;
use glium::Program;
use glium::uniforms::*;

use z80::Z80;
use zx::*;

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coord);

pub struct RustZXApp {
    red: f32,
    zx: ZXController,
}

impl RustZXApp {
    pub fn new() -> RustZXApp {
        let mut computer = ZXController::new();
        computer.atach_memory(ZXMemory::new(RomType::K16, RamType::K48));
        computer.attach_cpu(Z80::new());
        RustZXApp {
            red: 0.0,
            zx: computer,
        }
    }
    pub fn start(&mut self) -> Result<(), ()> {
        // build new glium window
        let display = WindowBuilder::new().build_glium().unwrap();
        // make texture

        let tex_vec: Vec<f32> = vec![
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 1.0,
            0.0, 0.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
        ];
        let raw = RawImage2d {
            data: Cow::Owned(tex_vec),
            width: 2,
            height: 2,
            format: ClientFormat::F32F32F32F32,
        };
        let tex = Texture2d::new(&display, raw).unwrap();
        // make square
        let square = vec![
            Vertex { position: [-0.5, -0.5], tex_coord: [0.0 , 0.0] },
            Vertex { position: [ 0.5, -0.5], tex_coord: [1.0 , 0.0] },
            Vertex { position: [-0.5,  0.5], tex_coord: [0.0 , 1.0] },
            Vertex { position: [-0.5,  0.5], tex_coord: [0.0 , 1.0] },
            Vertex { position: [ 0.5, -0.5], tex_coord: [1.0 , 0.0] },
            Vertex { position: [ 0.5,  0.5], tex_coord: [1.0 , 1.0] },
        ];
        let vb = VertexBuffer::new(&display, &square).unwrap();
        let idx = index::NoIndices(index::PrimitiveType::TrianglesList);
        let vert_shader = include_str!("shaders/vert.glsl");
        let frag_shader = include_str!("shaders/frag.glsl");
        let program = Program::from_source(
            &display, vert_shader, frag_shader, None).unwrap();

        let uniforms = uniform![
            tex: Sampler::new(&tex).magnify_filter(MagnifySamplerFilter::Nearest),
        ];
        loop {
            for event in display.poll_events() {
                match event {
                    Event::Closed => {
                        return Ok(())
                    }
                    Event::KeyboardInput(_, _, Some(key_code)) => {
                        match key_code {
                            VKey::Tab => {
                                self.red += 0.1;
                                if self.red > 1.0 {
                                    self.red -= 1.0;
                                };
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            //start frame draw and clear screen
            let mut target = display.draw();
            target.clear_color(self.red , 0.0, 0.0, 0.1);
            target.draw(&vb, &idx, &program, &uniforms, &Default::default())
                .unwrap();


            //TODO: Copy data from internal screen
            target.finish().unwrap();
        }
        Ok(())
    }
}
