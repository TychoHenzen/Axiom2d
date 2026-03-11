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

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> camera: CameraUniform;

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
    let world_pos = vec4<f32>(x, y, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
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

const BLOOM_SHADER_SRC: &str = "
struct FullscreenOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BloomParams {
    threshold: f32,
    intensity: f32,
    direction: vec2<f32>,
    texel_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

@vertex
fn vs_fullscreen(@location(0) position: vec2<f32>) -> FullscreenOutput {
    var out: FullscreenOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, -position.y * 0.5 + 0.5);
    return out;
}

@fragment
fn fs_brightness(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_input, s_input, in.uv);
    let luminance = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (luminance > params.threshold) {
        return vec4<f32>(color.rgb, 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_blur(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let offset = params.direction * params.texel_size;
    var result = textureSample(t_input, s_input, in.uv) * 0.227027;
    result += textureSample(t_input, s_input, in.uv + offset) * 0.1945946;
    result += textureSample(t_input, s_input, in.uv - offset) * 0.1945946;
    result += textureSample(t_input, s_input, in.uv + offset * 2.0) * 0.1216216;
    result += textureSample(t_input, s_input, in.uv - offset * 2.0) * 0.1216216;
    result += textureSample(t_input, s_input, in.uv + offset * 3.0) * 0.054054;
    result += textureSample(t_input, s_input, in.uv - offset * 3.0) * 0.054054;
    result += textureSample(t_input, s_input, in.uv + offset * 4.0) * 0.016216;
    result += textureSample(t_input, s_input, in.uv - offset * 4.0) * 0.016216;
    return result;
}

@fragment
fn fs_blit(in: FullscreenOutput) -> @location(0) vec4<f32> {
    return textureSample(t_input, s_input, in.uv);
}
";

const COMPOSITE_SHADER_SRC: &str = "
struct FullscreenOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BloomParams {
    threshold: f32,
    intensity: f32,
    direction: vec2<f32>,
    texel_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var t_scene: texture_2d<f32>;
@group(0) @binding(1) var t_bloom: texture_2d<f32>;
@group(0) @binding(2) var s_composite: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

@vertex
fn vs_fullscreen(@location(0) position: vec2<f32>) -> FullscreenOutput {
    var out: FullscreenOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, -position.y * 0.5 + 0.5);
    return out;
}

@fragment
fn fs_composite(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(t_scene, s_composite, in.uv);
    let bloom = textureSample(t_bloom, s_composite, in.uv);
    return vec4<f32>(scene.rgb + bloom.rgb * params.intensity, scene.a);
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

pub(crate) const FULLSCREEN_QUAD_VERTICES: [QuadVertex; 4] = [
    QuadVertex {
        position: [-1.0, -1.0],
    },
    QuadVertex {
        position: [1.0, -1.0],
    },
    QuadVertex {
        position: [1.0, 1.0],
    },
    QuadVertex {
        position: [-1.0, 1.0],
    },
];

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    pub(crate) ndc_rect: [f32; 4],
    pub(crate) uv_rect: [f32; 4],
    pub(crate) color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BloomParamsUniform {
    threshold: f32,
    intensity: f32,
    direction: [f32; 2],
    texel_size: [f32; 2],
    _pad: [f32; 2],
}

pub(crate) fn rect_to_instance(rect: &Rect) -> Instance {
    let Pixels(x) = rect.x;
    let Pixels(y) = rect.y;
    let Pixels(w) = rect.width;
    let Pixels(h) = rect.height;

    Instance {
        ndc_rect: [x, y, w, h],
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        color: [rect.color.r, rect.color.g, rect.color.b, rect.color.a],
    }
}

struct PostProcessResources {
    scene_view: wgpu::TextureView,
    ping_view: wgpu::TextureView,
    pong_view: wgpu::TextureView,

    scene_bg: wgpu::BindGroup,
    ping_bg: wgpu::BindGroup,
    pong_bg: wgpu::BindGroup,
    composite_bg: wgpu::BindGroup,

    brightness_params: (wgpu::Buffer, wgpu::BindGroup),
    h_blur_params: (wgpu::Buffer, wgpu::BindGroup),
    v_blur_params: (wgpu::Buffer, wgpu::BindGroup),
    composite_params: (wgpu::Buffer, wgpu::BindGroup),

    brightness_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,

    fs_vertex_buffer: wgpu::Buffer,
}

impl PostProcessResources {
    fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        threshold: f32,
        intensity: f32,
    ) -> Self {
        let half_w = (width / 2).max(1);
        let half_h = (height / 2).max(1);

        let single_tex_layout =
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

        let dual_tex_layout =
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

        let params_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let brightness_params =
            create_params_buffer_and_bg(device, &params_layout, &BloomParamsUniform {
                threshold,
                intensity: 0.0,
                direction: [0.0, 0.0],
                texel_size: scene_texel,
                _pad: [0.0; 2],
            });
        let h_blur_params =
            create_params_buffer_and_bg(device, &params_layout, &BloomParamsUniform {
                threshold: 0.0,
                intensity: 0.0,
                direction: [1.0, 0.0],
                texel_size: half_texel,
                _pad: [0.0; 2],
            });
        let v_blur_params =
            create_params_buffer_and_bg(device, &params_layout, &BloomParamsUniform {
                threshold: 0.0,
                intensity: 0.0,
                direction: [0.0, 1.0],
                texel_size: half_texel,
                _pad: [0.0; 2],
            });
        let composite_params =
            create_params_buffer_and_bg(device, &params_layout, &BloomParamsUniform {
                threshold: 0.0,
                intensity,
                direction: [0.0, 0.0],
                texel_size: [0.0; 2],
                _pad: [0.0; 2],
            });

        let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(BLOOM_SHADER_SRC.into()),
        });
        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(COMPOSITE_SHADER_SRC.into()),
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

        let dual_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&dual_tex_layout, &params_layout],
                push_constant_ranges: &[],
            });

        let brightness_pipeline =
            create_fullscreen_pipeline(device, &single_pipeline_layout, &bloom_shader, "fs_brightness", format, &fs_vertex_layout);
        let blur_pipeline =
            create_fullscreen_pipeline(device, &single_pipeline_layout, &bloom_shader, "fs_blur", format, &fs_vertex_layout);
        let composite_pipeline =
            create_fullscreen_pipeline(device, &dual_pipeline_layout, &composite_shader, "fs_composite", format, &fs_vertex_layout);

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

    fn resize(
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
            buffers: &[vertex_layout.clone()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
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
    })
}

fn draw_fullscreen_quad(
    pass: &mut wgpu::RenderPass,
    pipeline: &wgpu::RenderPipeline,
    tex_bg: &wgpu::BindGroup,
    params_bg: &wgpu::BindGroup,
    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, tex_bg, &[]);
    pass.set_bind_group(1, params_bg, &[]);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
}

pub struct WgpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    clear_color: Color,
    pending_instances: Vec<Instance>,
    post_process: Option<PostProcessResources>,
    post_process_pending: bool,
    bloom_threshold: f32,
    bloom_intensity: f32,
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

        let identity: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let camera_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            camera_uniform_buffer,
            camera_bind_group,
            clear_color: Color::BLACK,
            pending_instances: Vec::new(),
            post_process: None,
            post_process_pending: false,
            bloom_threshold: 0.8,
            bloom_intensity: 0.3,
        }
    }

    #[allow(dead_code)]
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

    fn draw_scene_to(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        let clear_color = wgpu::Color {
            r: self.clear_color.r as f64,
            g: self.clear_color.g as f64,
            b: self.clear_color.b as f64,
            a: self.clear_color.a as f64,
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

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.texture_bind_group, &[]);
            pass.set_bind_group(1, &self.camera_bind_group, &[]);
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

    fn execute_bloom(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        swapchain_view: &wgpu::TextureView,
    ) {
        let pp = self.post_process.as_ref().unwrap();

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

        // Pass 1: Brightness extraction (scene → ping at half res)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pp.ping_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw_fullscreen_quad(
                &mut pass,
                &pp.brightness_pipeline,
                &pp.scene_bg,
                &pp.brightness_params.1,
                &pp.fs_vertex_buffer,
                &self.index_buffer,
            );
        }

        // Pass 2: Horizontal blur (ping → pong)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pp.pong_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw_fullscreen_quad(
                &mut pass,
                &pp.blur_pipeline,
                &pp.ping_bg,
                &pp.h_blur_params.1,
                &pp.fs_vertex_buffer,
                &self.index_buffer,
            );
        }

        // Pass 3: Vertical blur (pong → ping)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pp.ping_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw_fullscreen_quad(
                &mut pass,
                &pp.blur_pipeline,
                &pp.pong_bg,
                &pp.v_blur_params.1,
                &pp.fs_vertex_buffer,
                &self.index_buffer,
            );
        }

        // Pass 4: Composite (scene + bloom → swapchain)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: swapchain_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw_fullscreen_quad(
                &mut pass,
                &pp.composite_pipeline,
                &pp.composite_bg,
                &pp.composite_params.1,
                &pp.fs_vertex_buffer,
                &self.index_buffer,
            );
        }
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

impl Renderer for WgpuRenderer {
    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.pending_instances.clear();
    }

    fn draw_rect(&mut self, rect: Rect) {
        self.pending_instances.push(rect_to_instance(&rect));
    }

    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]) {
        let mut instance = rect_to_instance(&rect);
        instance.uv_rect = uv_rect;
        self.pending_instances.push(instance);
    }

    fn draw_shape(&mut self, _vertices: &[[f32; 2]], _indices: &[u32], _color: Color) {}

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
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        if self.post_process_pending && self.post_process.is_some() {
            let pp = self.post_process.as_ref().unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn when_rect_at_origin_then_instance_encodes_world_coordinates() {
        // Arrange
        let rect = Rect {
            x: Pixels(0.0),
            y: Pixels(0.0),
            width: Pixels(800.0),
            height: Pixels(600.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.ndc_rect, [0.0, 0.0, 800.0, 600.0]);
    }

    #[test]
    fn when_offset_rect_then_instance_encodes_position_and_size() {
        // Arrange
        let rect = Rect {
            x: Pixels(200.0),
            y: Pixels(150.0),
            width: Pixels(400.0),
            height: Pixels(300.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.ndc_rect, [200.0, 150.0, 400.0, 300.0]);
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
        let instance = rect_to_instance(&rect);

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
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn when_zero_size_rect_then_no_panic_and_zero_dimensions() {
        // Arrange
        let rect = Rect {
            x: Pixels(400.0),
            y: Pixels(300.0),
            width: Pixels(0.0),
            height: Pixels(0.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.ndc_rect, [400.0, 300.0, 0.0, 0.0]);
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
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.ndc_rect, [400.0, 300.0, -100.0, -50.0]);
    }

    #[test]
    fn when_fullscreen_quad_vertices_queried_then_four_corners_span_ndc() {
        // Act
        let positions: [[f32; 2]; 4] = [
            FULLSCREEN_QUAD_VERTICES[0].position,
            FULLSCREEN_QUAD_VERTICES[1].position,
            FULLSCREEN_QUAD_VERTICES[2].position,
            FULLSCREEN_QUAD_VERTICES[3].position,
        ];

        // Assert
        assert_eq!(
            positions,
            [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
        );
    }

    #[test]
    fn when_fullscreen_quad_indices_resolved_then_two_ccw_triangles_cover_quad() {
        // Act
        let tri0: [[f32; 2]; 3] = [
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[0] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[1] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[2] as usize].position,
        ];
        let tri1: [[f32; 2]; 3] = [
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[3] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[4] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[5] as usize].position,
        ];

        // Assert
        assert_eq!(tri0, [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0]]);
        assert_eq!(tri1, [[-1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]);
    }
}
