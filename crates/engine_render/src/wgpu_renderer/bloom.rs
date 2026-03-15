use wgpu::util::DeviceExt;

use super::shaders::{BLOOM_PREAMBLE, BLOOM_SHADER_FRAG, COMPOSITE_SHADER_FRAG};
use super::types::{BloomParamsUniform, FULLSCREEN_QUAD_VERTICES, QuadVertex};

pub(super) struct PostProcessResources {
    pub(super) scene_view: wgpu::TextureView,
    pub(super) ping_view: wgpu::TextureView,
    pub(super) pong_view: wgpu::TextureView,

    pub(super) scene_bg: wgpu::BindGroup,
    pub(super) ping_bg: wgpu::BindGroup,
    pub(super) pong_bg: wgpu::BindGroup,
    pub(super) composite_bg: wgpu::BindGroup,

    pub(super) brightness_params: (wgpu::Buffer, wgpu::BindGroup),
    pub(super) h_blur_params: (wgpu::Buffer, wgpu::BindGroup),
    pub(super) v_blur_params: (wgpu::Buffer, wgpu::BindGroup),
    pub(super) composite_params: (wgpu::Buffer, wgpu::BindGroup),

    pub(super) brightness_pipeline: wgpu::RenderPipeline,
    pub(super) blur_pipeline: wgpu::RenderPipeline,
    pub(super) composite_pipeline: wgpu::RenderPipeline,

    pub(super) fs_vertex_buffer: wgpu::Buffer,
}

impl PostProcessResources {
    #[allow(clippy::similar_names, clippy::too_many_lines)]
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        threshold: f32,
        intensity: f32,
    ) -> Self {
        let half_w = (width / 2).max(1);
        let half_h = (height / 2).max(1);

        let single_tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let dual_tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let scene_texture = create_render_texture(device, format, width, height);
        let scene_view = scene_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ping_texture = create_render_texture(device, format, half_w, half_h);
        let ping_view = ping_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let pong_texture = create_render_texture(device, format, half_w, half_h);
        let pong_view = pong_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let scene_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &single_tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let ping_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &single_tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&ping_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pong_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &single_tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&pong_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let composite_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &dual_tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&ping_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let scene_texel = [1.0 / width as f32, 1.0 / height as f32];
        let half_texel = [1.0 / half_w as f32, 1.0 / half_h as f32];

        let brightness_params = create_params_buffer_and_bg(
            device,
            &params_layout,
            &BloomParamsUniform {
                threshold,
                intensity: 0.0,
                direction: [0.0, 0.0],
                texel_size: scene_texel,
                _pad: [0.0; 2],
            },
        );
        let h_blur_params = create_params_buffer_and_bg(
            device,
            &params_layout,
            &BloomParamsUniform {
                threshold: 0.0,
                intensity: 0.0,
                direction: [1.0, 0.0],
                texel_size: half_texel,
                _pad: [0.0; 2],
            },
        );
        let v_blur_params = create_params_buffer_and_bg(
            device,
            &params_layout,
            &BloomParamsUniform {
                threshold: 0.0,
                intensity: 0.0,
                direction: [0.0, 1.0],
                texel_size: half_texel,
                _pad: [0.0; 2],
            },
        );
        let composite_params = create_params_buffer_and_bg(
            device,
            &params_layout,
            &BloomParamsUniform {
                threshold: 0.0,
                intensity,
                direction: [0.0, 0.0],
                texel_size: [0.0; 2],
                _pad: [0.0; 2],
            },
        );

        let bloom_src = format!("{BLOOM_PREAMBLE}{BLOOM_SHADER_FRAG}");
        let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(bloom_src.into()),
        });
        let composite_src = format!("{BLOOM_PREAMBLE}{COMPOSITE_SHADER_FRAG}");
        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(composite_src.into()),
        });

        let fs_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let single_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&single_tex_layout, &params_layout],
                push_constant_ranges: &[],
            });

        let dual_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&dual_tex_layout, &params_layout],
            push_constant_ranges: &[],
        });

        let brightness_pipeline = create_fullscreen_pipeline(
            device,
            &single_pipeline_layout,
            &bloom_shader,
            "fs_brightness",
            format,
            &fs_vertex_layout,
        );
        let blur_pipeline = create_fullscreen_pipeline(
            device,
            &single_pipeline_layout,
            &bloom_shader,
            "fs_blur",
            format,
            &fs_vertex_layout,
        );
        let composite_pipeline = create_fullscreen_pipeline(
            device,
            &dual_pipeline_layout,
            &composite_shader,
            "fs_composite",
            format,
            &fs_vertex_layout,
        );

        let fs_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&FULLSCREEN_QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            scene_view,
            ping_view,
            pong_view,
            scene_bg,
            ping_bg,
            pong_bg,
            composite_bg,
            brightness_params,
            h_blur_params,
            v_blur_params,
            composite_params,
            brightness_pipeline,
            blur_pipeline,
            composite_pipeline,
            fs_vertex_buffer,
        }
    }

    pub(super) fn resize(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        threshold: f32,
        intensity: f32,
    ) {
        *self = Self::new(device, format, width, height, threshold, intensity);
    }
}

fn create_render_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_params_buffer_and_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    data: &BloomParamsUniform,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (buffer, bind_group)
}

fn create_fullscreen_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    fragment_entry: &str,
    format: wgpu::TextureFormat,
    vertex_layout: &wgpu::VertexBufferLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_fullscreen"),
            buffers: std::slice::from_ref(vertex_layout),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
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
}
