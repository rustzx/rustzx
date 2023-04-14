//! TODO
//! UNDER CONSTRUCTION
//! This is a draft and will be refactored & split into multiple files

use crate::app::video::{Palette, PALETTE_SIZE, ColorRgba, ColorIndexed};

use wgpu::{
    Instance, Surface, Adapter, Device, Queue, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features, Limits, util::DeviceExt,
};
use winit::window::Window;
use core::mem::size_of;
use bytemuck::{Pod, Zeroable};

use rustzx_core::zx::constants::{SCREEN_WIDTH, SCREEN_HEIGHT, CANVAS_WIDTH, CANVAS_HEIGHT, CANVAS_X, CANVAS_Y};
use std::num::NonZeroU32;

/// Atlas layout
/// +-----------------------+
/// |              |        |
/// |    SCREEN    |        |
/// |   320x240    |        |
/// |---------------        |
/// |   CANVAS  |           |
/// |  256x192  |           |
/// |------------           |
/// |                       |
/// +-----------------------+
const ATLAS_TEXTURE_SIZE: u32 = 512;


fn zx_screen_rect() -> Rect {
    Rect {
        x: zx_to_screen_x(0),
        y: zx_to_screen_y(0),
        width: zx_to_screen_x(SCREEN_WIDTH as u16),
        height: zx_to_screen_y(SCREEN_HEIGHT as u16),
    }
}

fn zx_canvas_rect() -> Rect {
    Rect {
        x: zx_to_screen_x(CANVAS_X as u16),
        y: zx_to_screen_y(CANVAS_Y as u16),
        width: zx_to_screen_x(CANVAS_WIDTH as u16),
        height: zx_to_screen_y(CANVAS_HEIGHT as u16),
    }
}

fn zx_screen_tex_rect() -> Rect {
    Rect {
        x: texture_x(0 as u16),
        y: texture_y(0 as u16),
        width: texture_x(SCREEN_WIDTH as u16),
        height: texture_y(SCREEN_HEIGHT as u16),
    }
}

fn zx_canvas_tex_rect() -> Rect {
    Rect {
        x: texture_x(0 as u16),
        y: texture_y(SCREEN_HEIGHT as u16),
        width: texture_x(CANVAS_WIDTH as u16),
        height: texture_y(CANVAS_HEIGHT as u16),
    }
}

fn zx_to_screen_x(x: u16) -> f32 {
    (x as f32) / (SCREEN_WIDTH as f32)
}

fn zx_to_screen_y(y: u16) -> f32 {
    (y as f32) / (SCREEN_HEIGHT as f32)
}

fn texture_x(x: u16) -> f32 {
    (x as f32) / (ATLAS_TEXTURE_SIZE as f32)
}

fn texture_y(y: u16) -> f32 {
    (y as f32) / (ATLAS_TEXTURE_SIZE as f32)
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error(transparent)]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error(transparent)]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    SurfaceError(#[from] wgpu::SurfaceError),

    #[error("Failed to find an appropriate video adapter!")]
    NoAdapter,

    #[error("Surface configuration is not supported by the adapter!")]
    NotSupportedSurface,
}

pub type RenderResult<T> = Result<T, RenderError>;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ShaderGlobals {
    // Always aligned to 16 bytes regardless of PALETTE_SIZE value. Usually when PALETTE_SIZE is 16,
    // size of the field is 16 * 16 = 256 bytes
    palette: [[f32; 4]; PALETTE_SIZE],

    content_aspect_ratio: f32,
    screen_aspect_ratio: f32,

    texture_atlas_size: [f32; 2],

    // NOTE: Align to 16 bytes if adding new fields
}

impl ShaderGlobals {
    fn new(screen_width: u32, screen_height: u32) -> Self {
        let to_shader_color = |color: u32| {
            let r = ((color >> 24) & 0xFF) as f32 / 255.0;
            let g = ((color >> 16) & 0xFF) as f32 / 255.0;
            let b = ((color >> 8) & 0xFF) as f32 / 255.0;
            let a = ((color >> 0) & 0xFF) as f32 / 255.0;
            [r, g, b, a]
        };

        let default_palette = Palette::default();

        // Init palette colors
        let mut palette = [[0.0f32; 4]; PALETTE_SIZE];
        for (idx, color) in palette.iter_mut().enumerate() {
            *color = to_shader_color(default_palette.get_color(idx as ColorIndexed));
        }

        let content_aspect_ratio = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
        let screen_aspect_ratio = screen_width as f32 / screen_height as f32;

        Self {
            palette,
            content_aspect_ratio,
            screen_aspect_ratio,
            texture_atlas_size: [ATLAS_TEXTURE_SIZE as f32; 2]
        }
    }

    fn update_screen_size(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_aspect_ratio = screen_width as f32 / screen_height as f32;
    }

    fn update_palette(&mut self, palette: &Palette) {
        self.palette = Self::convert_palete(palette);
    }

    fn convert_palete(palette: &Palette) -> [[f32; 4]; PALETTE_SIZE] {
        let to_shader_color = |color: u32| {
            let r = ((color >> 24) & 0xFF) as f32 / 255.0;
            let g = ((color >> 16) & 0xFF) as f32 / 255.0;
            let b = ((color >> 8) & 0xFF) as f32 / 255.0;
            let a = ((color >> 0) & 0xFF) as f32 / 255.0;
            [r, g, b, a]
        };

        let mut palette_colors = [[0.0f32; 4]; PALETTE_SIZE];
        for (idx, color) in palette_colors.iter_mut().enumerate() {
            *color = to_shader_color(palette.get_color(idx as ColorIndexed));
        }

        palette_colors
    }
}



// Enforce alignment to 4 bytes for vertexes
#[repr(C, align(4))]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

pub struct ScreenParams {}

pub struct Screen {
    surface_config: wgpu::SurfaceConfiguration,

    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    atlas_texture: wgpu::Texture,
    globals_buffer: wgpu::Buffer,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    /// Number of vertices to draw in each frame (not equal to the number of vertices in the buffer)
    vertex_count: u32,

    render_pipeline: wgpu::RenderPipeline,

    bind_group: wgpu::BindGroup,

    atlas_texture_data: Vec<u8>,
    shader_globals_data: ShaderGlobals,
}

struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}


/// Helper struct for building vertexes and their indices for rendering
#[derive(Default)]
struct VertexBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl VertexBuilder {
    /// Add a quad to the vertex buffer
    pub fn add_quad(mut self, rect: Rect, tex_coords: Rect) -> Self {
        let vertices = [
            Vertex {
                position: [rect.x, rect.y],
                tex_coords: [tex_coords.x, tex_coords.y],
            },
            Vertex {
                position: [rect.x, rect.y + rect.height],
                tex_coords: [tex_coords.x, tex_coords.y + tex_coords.height],
            },
            Vertex {
                position: [rect.x + rect.width, rect.y + rect.height],
                tex_coords: [tex_coords.x + tex_coords.width, tex_coords.y + tex_coords.height],
            },
            Vertex {
                position: [rect.x + rect.width, rect.y],
                tex_coords: [tex_coords.x + tex_coords.width, tex_coords.y],
            },
        ];

        let index_base = self.vertices.len() as u16;

        // Build indices for quad as two triangles
        let indices = [
            index_base,
            index_base + 1,
            index_base + 2,
            index_base,
            index_base + 2,
            index_base + 3,
        ];

        self.vertices.extend_from_slice(&vertices);
        self.indices.extend_from_slice(&indices);

        self
    }
}



impl Screen {
    /// Video context initialization code, which is called before event handling loop starts
    pub async fn init(params: ScreenParams, window: &Window) -> RenderResult<Self> {
        let instance = Instance::default();
        // Creating surface is 99.9% safe when using winit
        let surface = unsafe { instance.create_surface(window)? };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // ZX Spectrum emulator is not a graphics-intensive application
                // TODO: Make selectable via env/config/args
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                // TODO: Check how software render will work
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    // We need to support as many platforms as possible, especially WebGL2
                    limits: Limits::downlevel_webgl2_defaults(),
                },
                None, // TODO: Add tracing here
            )
            .await?;

        let size = window.inner_size();

        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or(RenderError::NotSupportedSurface)?;

        let caps = surface.get_capabilities(&adapter);

        // Disable VSync if available
        if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            surface_config.present_mode = wgpu::PresentMode::Immediate;
        }

        surface.configure(&device, &surface_config);

        // Buffer to store palette
        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rustzx_buffer_palette"),
            size: size_of::<ShaderGlobals>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        // Use single POT texture to store both screen and border data. Note that texture is
        // using 8-bit
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rustzx_texture_atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_TEXTURE_SIZE,
                height: ATLAS_TEXTURE_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rustzx_shader_main"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rustzx_bind_group_layout"),
            entries: &[
                // Shader globals
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(size_of::<ShaderGlobals>() as _),
                    },
                    count: None,
                },
                // Atlas texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let atlas_texture_view = atlas_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rustzx_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                // Shader globals
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                // Atlas texture
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture_view),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rustzx_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let VertexBuilder {
            vertices,
            indices,
        } = VertexBuilder::default()
            .add_quad(zx_screen_rect(), zx_screen_tex_rect())
            .add_quad(zx_canvas_rect(), zx_canvas_tex_rect());

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("rustzx_buffer_vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("rustzx_buffer_indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        // Vertex Buffer Layout
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 2 * size_of::<f32>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        // Create pipeliene
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rustzx_pipeline_main"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });


        let atlas_texture_data = vec![0u8; (ATLAS_TEXTURE_SIZE * ATLAS_TEXTURE_SIZE) as usize];
        let shader_globals_data = ShaderGlobals::new(
            size.width,
            size.height,
        );
        queue.write_buffer(&globals_buffer, 0, bytemuck::bytes_of(&shader_globals_data));

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            globals_buffer,
            atlas_texture,
            surface_config,
            index_buffer,
            vertex_buffer,
            render_pipeline,
            bind_group,
            vertex_count: indices.len() as u32,
            atlas_texture_data,
            shader_globals_data,
        })
    }

    /// Resize surface to match new window size
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        // Shader will receive new aspect ratio
        self.shader_globals_data.update_screen_size(width, height);

        // Resize surface to match window size
        self.surface.configure(&self.device, &self.surface_config);
        self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&self.shader_globals_data));
    }

    /// Render frame
    pub fn render(&self) -> Result<(), RenderError> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                // Reconfigure if lost
                // TODO: More precise error handling
                self.surface.configure(&self.device, &self.surface_config);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rustzx_cmd_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rustzx_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw_indexed(0..self.vertex_count, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn update_canvas(&mut self, canvas: &[u8]) {
        for row_idx in 0..CANVAS_HEIGHT {
            let row_offset = (SCREEN_HEIGHT + row_idx) * ATLAS_TEXTURE_SIZE as usize;
            self.atlas_texture_data[row_offset..row_offset + CANVAS_WIDTH]
                .copy_from_slice(&canvas[row_idx * CANVAS_WIDTH..(row_idx + 1) * CANVAS_WIDTH]);
        }

        self.send_texture_atlas_to_gpu();
    }

    pub fn update_screen(&mut self, screen: &[u8]) {
        for row_idx in 0..SCREEN_HEIGHT {
            let row_offset = row_idx * ATLAS_TEXTURE_SIZE as usize;
            self.atlas_texture_data[row_offset..row_offset + SCREEN_WIDTH]
                .copy_from_slice(&screen[row_idx * SCREEN_WIDTH..(row_idx + 1) * SCREEN_WIDTH]);
        }

        self.send_texture_atlas_to_gpu();
    }

    fn send_texture_atlas_to_gpu(&self) {
        self.queue.write_texture(self.atlas_texture.as_image_copy(), &self.atlas_texture_data, wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(ATLAS_TEXTURE_SIZE).unwrap()),
            rows_per_image: None,
        }, wgpu::Extent3d {
            width: ATLAS_TEXTURE_SIZE,
            height: ATLAS_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        });
    }
}
