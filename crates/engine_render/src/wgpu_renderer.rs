use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;

use engine_core::color::Color;
use engine_core::types::Pixels;

use crate::rect::Rect;
use crate::renderer::Renderer;
use crate::window::WindowConfig;

const SHADER_SRC: &str = "
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(
    @location(0) quad_pos: vec2<f32>,
    @location(1) ndc_rect: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let x = quad_pos.x * ndc_rect.z + ndc_rect.x;
    let y = quad_pos.y * ndc_rect.w + ndc_rect.y;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = color;
    out.uv = vec2<f32>(
        mix(uv_rect.x, uv_rect.z, quad_pos.x),
        mix(uv_rect.y, uv_rect.w, quad_pos.y),
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex_color * in.color;
}
";

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct QuadVertex {
    pub(crate) position: [f32; 2],
}

pub(crate) const QUAD_VERTICES: [QuadVertex; 4] = [
    QuadVertex {
        position: [0.0, 0.0],
    },
    QuadVertex {
        position: [1.0, 0.0],
    },
    QuadVertex {
        position: [1.0, 1.0],
    },
    QuadVertex {
        position: [0.0, 1.0],
    },
];

pub(crate) const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    pub(crate) ndc_rect: [f32; 4],
    pub(crate) uv_rect: [f32; 4],
    pub(crate) color: [f32; 4],
}

pub(crate) fn rect_to_instance(rect: &Rect, vw: f32, vh: f32) -> Instance {
    let Pixels(x) = rect.x;
    let Pixels(y) = rect.y;
    let Pixels(w) = rect.width;
    let Pixels(h) = rect.height;

    let [x0, y0, x1, y1] = rect_to_ndc(x, y, w, h, vw, vh);

    Instance {
        ndc_rect: [x0, y1, x1 - x0, y0 - y1],
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        color: [rect.color.r, rect.color.g, rect.color.b, rect.color.a],
    }
}

pub struct WgpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    clear_color: Color,
    pending_instances: Vec<Instance>,
}

impl WgpuRenderer {
    pub fn new(window: Arc<Window>, config: &WindowConfig) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("no compatible GPU adapter found");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to create GPU device");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let present_mode = if config.vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<QuadVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<Instance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 3,
                            },
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_bind_group =
            create_texture_bind_group(&device, &queue, &texture_bind_group_layout, 1, 1, &[255; 4]);

        Self {
            surface,
            device,
            queue,
            config: surface_config,
            pipeline,
            quad_vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            texture_bind_group,
            clear_color: Color::BLACK,
            pending_instances: Vec::new(),
        }
    }

    pub fn upload_atlas(&mut self, atlas: &crate::atlas::TextureAtlas) {
        self.texture_bind_group = create_texture_bind_group(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            atlas.width,
            atlas.height,
            &atlas.data,
        );
    }
}

fn create_texture_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    width: u32,
    height: u32,
    data: &[u8],
) -> wgpu::BindGroup {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}

/// Converts pixel-space rect bounds to NDC coordinates [x0, y0, x1, y1].
pub(crate) fn rect_to_ndc(x: f32, y: f32, w: f32, h: f32, vw: f32, vh: f32) -> [f32; 4] {
    let x0 = x / vw * 2.0 - 1.0;
    let y0 = 1.0 - y / vh * 2.0;
    let x1 = (x + w) / vw * 2.0 - 1.0;
    let y1 = 1.0 - (y + h) / vh * 2.0;
    [x0, y0, x1, y1]
}

impl Renderer for WgpuRenderer {
    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.pending_instances.clear();
    }

    fn draw_rect(&mut self, rect: Rect) {
        let vw = self.config.width as f32;
        let vh = self.config.height as f32;
        self.pending_instances.push(rect_to_instance(&rect, vw, vh));
    }

    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]) {
        let vw = self.config.width as f32;
        let vh = self.config.height as f32;
        let mut instance = rect_to_instance(&rect, vw, vh);
        instance.uv_rect = uv_rect;
        self.pending_instances.push(instance);
    }

    fn present(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let clear_color = wgpu::Color {
            r: self.clear_color.r as f64,
            g: self.clear_color.g as f64,
            b: self.clear_color.b as f64,
            a: self.clear_color.a as f64,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !self.pending_instances.is_empty() {
                let instance_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&self.pending_instances),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.texture_bind_group, &[]);
                pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, instance_buffer.slice(..));
                pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(
                    0..QUAD_INDICES.len() as u32,
                    0,
                    0..self.pending_instances.len() as u32,
                );
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_rect_fills_viewport_then_ndc_spans_full_clip_space() {
        // Act
        let [x0, y0, x1, y1] = rect_to_ndc(0.0, 0.0, 800.0, 600.0, 800.0, 600.0);

        // Assert
        assert_eq!(x0, -1.0);
        assert_eq!(y0, 1.0);
        assert_eq!(x1, 1.0);
        assert_eq!(y1, -1.0);
    }

    #[test]
    fn when_rect_offset_within_viewport_then_ndc_reflects_position() {
        // Act
        let [x0, y0, x1, y1] = rect_to_ndc(200.0, 150.0, 400.0, 300.0, 800.0, 600.0);

        // Assert
        assert_eq!(x0, -0.5);
        assert_eq!(y0, 0.5);
        assert_eq!(x1, 0.5);
        assert_eq!(y1, -0.5);
    }

    #[test]
    fn when_rect_at_viewport_center_then_ndc_is_origin() {
        // Act
        let [x0, y0, _, _] = rect_to_ndc(400.0, 300.0, 0.0, 0.0, 800.0, 600.0);

        // Assert
        assert_eq!(x0, 0.0);
        assert_eq!(y0, 0.0);
    }

    #[test]
    fn when_quad_indices_used_then_two_triangles_cover_unit_square() {
        // Act
        let tri0: [[f32; 2]; 3] = [
            QUAD_VERTICES[QUAD_INDICES[0] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[1] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[2] as usize].position,
        ];
        let tri1: [[f32; 2]; 3] = [
            QUAD_VERTICES[QUAD_INDICES[3] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[4] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[5] as usize].position,
        ];

        // Assert
        assert_eq!(tri0, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
        assert_eq!(tri1, [[0.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    }

    #[test]
    fn when_full_viewport_rect_then_ndc_rect_spans_clip_space() {
        // Arrange
        let rect = Rect {
            x: Pixels(0.0),
            y: Pixels(0.0),
            width: Pixels(800.0),
            height: Pixels(600.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect, 800.0, 600.0);

        // Assert
        assert_eq!(instance.ndc_rect, [-1.0, -1.0, 2.0, 2.0]);
    }

    #[test]
    fn when_offset_rect_then_ndc_rect_reflects_position() {
        // Arrange
        let rect = Rect {
            x: Pixels(200.0),
            y: Pixels(150.0),
            width: Pixels(400.0),
            height: Pixels(300.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect, 800.0, 600.0);

        // Assert
        assert_eq!(instance.ndc_rect, [-0.5, -0.5, 1.0, 1.0]);
    }

    #[test]
    fn when_any_rect_then_uv_rect_is_full_texture() {
        // Arrange
        let rect = Rect {
            x: Pixels(100.0),
            y: Pixels(100.0),
            width: Pixels(50.0),
            height: Pixels(50.0),
            color: Color::RED,
        };

        // Act
        let instance = rect_to_instance(&rect, 800.0, 600.0);

        // Assert
        assert_eq!(instance.uv_rect, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn when_colored_rect_then_instance_color_matches() {
        // Arrange
        let rect = Rect {
            x: Pixels(0.0),
            y: Pixels(0.0),
            width: Pixels(100.0),
            height: Pixels(100.0),
            color: Color::RED,
        };

        // Act
        let instance = rect_to_instance(&rect, 800.0, 600.0);

        // Assert
        assert_eq!(instance.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn when_zero_size_rect_then_no_panic_and_zero_ndc_dims() {
        // Arrange
        let rect = Rect {
            x: Pixels(400.0),
            y: Pixels(300.0),
            width: Pixels(0.0),
            height: Pixels(0.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect, 800.0, 600.0);

        // Assert
        assert_eq!(instance.ndc_rect[2], 0.0);
        assert_eq!(instance.ndc_rect[3], 0.0);
    }

    #[test]
    fn when_negative_dimensions_then_stored_without_clamping() {
        // Arrange
        let rect = Rect {
            x: Pixels(400.0),
            y: Pixels(300.0),
            width: Pixels(-100.0),
            height: Pixels(-50.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect, 800.0, 600.0);

        // Assert
        assert!(instance.ndc_rect[2] < 0.0);
        assert!(instance.ndc_rect[3] < 0.0);
    }
}
