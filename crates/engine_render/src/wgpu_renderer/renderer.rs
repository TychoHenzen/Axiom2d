use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;

use engine_core::color::Color;

use crate::rect::Rect;
use crate::renderer::Renderer;
use crate::window::WindowConfig;

use super::bloom::PostProcessResources;
use super::shaders::{SHADER_SRC, SHAPE_SHADER_SRC};
use super::types::{
    BloomParamsUniform, Instance, QUAD_INDICES, QUAD_VERTICES, QuadVertex, ShapeBatch, ShapeVertex,
    blend_mode_to_blend_state, compute_batch_ranges, create_texture_bind_group, rect_to_instance,
    run_fullscreen_pass,
};

struct ShapeDrawRecord {
    blend_mode: crate::material::BlendMode,
    index_offset: u32,
}

pub struct WgpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    quad_pipelines: [wgpu::RenderPipeline; 3],
    quad_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    clear_color: Color,
    pending_instances: Vec<Instance>,
    instance_blend_modes: Vec<crate::material::BlendMode>,
    current_blend_mode: crate::material::BlendMode,
    shape_batch: ShapeBatch,
    shape_draws: Vec<ShapeDrawRecord>,
    shape_pipelines: [wgpu::RenderPipeline; 3],
    post_process: Option<PostProcessResources>,
    post_process_pending: bool,
    bloom_threshold: f32,
    bloom_intensity: f32,
}

impl WgpuRenderer {
    #[allow(clippy::too_many_lines)]
    pub fn new(window: Arc<Window>, config: &WindowConfig) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");

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

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let identity = glam::Mat4::IDENTITY.to_cols_array_2d();
        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&identity),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let quad_pipelines = crate::material::BlendMode::ALL.map(|mode| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend_mode_to_blend_state(mode)),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
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

        let shape_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHAPE_SHADER_SRC.into()),
        });

        let shape_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shape_pipelines = crate::material::BlendMode::ALL.map(|mode| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&shape_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shape_shader,
                    entry_point: Some("vs_shape"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: size_of::<ShapeVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    }],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shape_shader,
                    entry_point: Some("fs_shape"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend_mode_to_blend_state(mode)),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        });

        Self {
            surface,
            device,
            queue,
            config: surface_config,
            quad_pipelines,
            quad_vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            texture_bind_group,
            camera_uniform_buffer,
            camera_bind_group,
            clear_color: Color::BLACK,
            pending_instances: Vec::new(),
            instance_blend_modes: Vec::new(),
            current_blend_mode: crate::material::BlendMode::Alpha,
            shape_batch: ShapeBatch::new(),
            shape_draws: Vec::new(),
            shape_pipelines,
            post_process: None,
            post_process_pending: false,
            bloom_threshold: 0.8,
            bloom_intensity: 0.3,
        }
    }

    fn reset_frame_state(&mut self) {
        self.pending_instances.clear();
        self.instance_blend_modes.clear();
        self.current_blend_mode = crate::material::BlendMode::Alpha;
        self.shape_batch.clear();
        self.shape_draws.clear();
    }

    fn ensure_post_process(&mut self) {
        if self.post_process.is_none() {
            self.post_process = Some(PostProcessResources::new(
                &self.device,
                self.config.format,
                self.config.width,
                self.config.height,
                self.bloom_threshold,
                self.bloom_intensity,
            ));
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw_scene_to(&self, encoder: &mut wgpu::CommandEncoder, target_view: &wgpu::TextureView) {
        let clear_color = wgpu::Color {
            r: f64::from(self.clear_color.r),
            g: f64::from(self.clear_color.g),
            b: f64::from(self.clear_color.b),
            a: f64::from(self.clear_color.a),
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
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

            pass.set_bind_group(0, &self.texture_bind_group, &[]);
            pass.set_bind_group(1, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for (mode, range) in compute_batch_ranges(&self.instance_blend_modes) {
                pass.set_pipeline(&self.quad_pipelines[mode.index()]);
                pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, range);
            }
        }

        if !self.shape_batch.is_empty() {
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(self.shape_batch.vertices()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(self.shape_batch.indices()),
                    usage: wgpu::BufferUsages::INDEX,
                });

            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            let blend_modes: Vec<_> = self.shape_draws.iter().map(|d| d.blend_mode).collect();
            let total_indices = self.shape_batch.index_count() as u32;
            for (mode, shape_range) in compute_batch_ranges(&blend_modes) {
                let idx_start = self.shape_draws[shape_range.start as usize].index_offset;
                let idx_end = if (shape_range.end as usize) < self.shape_draws.len() {
                    self.shape_draws[shape_range.end as usize].index_offset
                } else {
                    total_indices
                };
                pass.set_pipeline(&self.shape_pipelines[mode.index()]);
                pass.draw_indexed(idx_start..idx_end, 0, 0..1);
            }
        }
    }

    fn execute_bloom(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        swapchain_view: &wgpu::TextureView,
    ) {
        let pp = self
            .post_process
            .as_ref()
            .expect("post_process resources not initialized");

        self.queue.write_buffer(
            &pp.brightness_params.0,
            0,
            bytemuck::bytes_of(&BloomParamsUniform {
                threshold: self.bloom_threshold,
                intensity: 0.0,
                direction: [0.0, 0.0],
                texel_size: [
                    1.0 / self.config.width as f32,
                    1.0 / self.config.height as f32,
                ],
                _pad: [0.0; 2],
            }),
        );

        let half_w = (self.config.width / 2).max(1) as f32;
        let half_h = (self.config.height / 2).max(1) as f32;

        self.queue.write_buffer(
            &pp.h_blur_params.0,
            0,
            bytemuck::bytes_of(&BloomParamsUniform {
                threshold: 0.0,
                intensity: 0.0,
                direction: [1.0, 0.0],
                texel_size: [1.0 / half_w, 1.0 / half_h],
                _pad: [0.0; 2],
            }),
        );

        self.queue.write_buffer(
            &pp.v_blur_params.0,
            0,
            bytemuck::bytes_of(&BloomParamsUniform {
                threshold: 0.0,
                intensity: 0.0,
                direction: [0.0, 1.0],
                texel_size: [1.0 / half_w, 1.0 / half_h],
                _pad: [0.0; 2],
            }),
        );

        self.queue.write_buffer(
            &pp.composite_params.0,
            0,
            bytemuck::bytes_of(&BloomParamsUniform {
                threshold: 0.0,
                intensity: self.bloom_intensity,
                direction: [0.0, 0.0],
                texel_size: [0.0; 2],
                _pad: [0.0; 2],
            }),
        );

        let ib = &self.index_buffer;
        let vb = &pp.fs_vertex_buffer;
        run_fullscreen_pass(
            encoder,
            &pp.ping_view,
            &pp.brightness_pipeline,
            &pp.scene_bg,
            &pp.brightness_params.1,
            vb,
            ib,
        );
        run_fullscreen_pass(
            encoder,
            &pp.pong_view,
            &pp.blur_pipeline,
            &pp.ping_bg,
            &pp.h_blur_params.1,
            vb,
            ib,
        );
        run_fullscreen_pass(
            encoder,
            &pp.ping_view,
            &pp.blur_pipeline,
            &pp.pong_bg,
            &pp.v_blur_params.1,
            vb,
            ib,
        );
        run_fullscreen_pass(
            encoder,
            swapchain_view,
            &pp.composite_pipeline,
            &pp.composite_bg,
            &pp.composite_params.1,
            vb,
            ib,
        );
    }
}

impl Renderer for WgpuRenderer {
    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.reset_frame_state();
    }

    fn draw_rect(&mut self, rect: Rect) {
        self.pending_instances.push(rect_to_instance(&rect));
        self.instance_blend_modes.push(self.current_blend_mode);
    }

    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]) {
        let mut instance = rect_to_instance(&rect);
        instance.uv_rect = uv_rect;
        self.pending_instances.push(instance);
        self.instance_blend_modes.push(self.current_blend_mode);
    }

    fn draw_shape(&mut self, vertices: &[[f32; 2]], indices: &[u32], color: Color) {
        #[allow(clippy::cast_possible_truncation)]
        self.shape_draws.push(ShapeDrawRecord {
            blend_mode: self.current_blend_mode,
            index_offset: self.shape_batch.index_count() as u32,
        });
        self.shape_batch.push(vertices, indices, color);
    }

    fn set_blend_mode(&mut self, mode: crate::material::BlendMode) {
        self.current_blend_mode = mode;
    }

    // TODO(shader-cache): wire to GPU pipeline cache when shader variant compilation is implemented
    fn set_shader(&mut self, _shader: crate::shader::ShaderHandle) {}

    // TODO(shader-cache): upload uniform buffer per-material when GPU pipeline cache exists
    fn set_material_uniforms(&mut self, _data: &[u8]) {}

    // TODO(shader-cache): bind texture to material slot when GPU pipeline cache exists
    fn bind_material_texture(&mut self, _texture: engine_core::types::TextureId, _binding: u32) {}

    fn upload_atlas(&mut self, atlas: &crate::atlas::TextureAtlas) {
        self.texture_bind_group = create_texture_bind_group(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            atlas.width,
            atlas.height,
            &atlas.data,
        );
    }

    fn set_view_projection(&mut self, matrix: [[f32; 4]; 4]) {
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&matrix),
        );
    }

    fn viewport_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn apply_post_process(&mut self) {
        self.ensure_post_process();
        self.post_process_pending = true;
    }

    fn present(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to get current texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        if self.post_process_pending
            && let Some(pp) = self.post_process.as_ref()
        {
            self.draw_scene_to(&mut encoder, &pp.scene_view);
            self.execute_bloom(&mut encoder, &view);
            self.post_process_pending = false;
        } else {
            self.draw_scene_to(&mut encoder, &view);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);

        if let Some(pp) = &mut self.post_process {
            pp.resize(
                &self.device,
                self.config.format,
                self.config.width,
                self.config.height,
                self.bloom_threshold,
                self.bloom_intensity,
            );
        }
    }
}
