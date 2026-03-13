use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;

use engine_core::color::Color;
use engine_core::types::Pixels;

use crate::rect::Rect;
use crate::renderer::Renderer;
use crate::window::WindowConfig;

pub(crate) const SHADER_SRC: &str = "
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
    @location(1) world_rect: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let x = quad_pos.x * world_rect.z + world_rect.x;
    let y = quad_pos.y * world_rect.w + world_rect.y;
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

pub(crate) const SHAPE_SHADER_SRC: &str = "
struct ShapeOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_shape(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> ShapeOutput {
    var out: ShapeOutput;
    let world_pos = vec4<f32>(position, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    return out;
}

@fragment
fn fs_shape(in: ShapeOutput) -> @location(0) vec4<f32> {
    return in.color;
}
";

const BLOOM_PREAMBLE: &str = "
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

@vertex
fn vs_fullscreen(@location(0) position: vec2<f32>) -> FullscreenOutput {
    var out: FullscreenOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, -position.y * 0.5 + 0.5);
    return out;
}
";

const BLOOM_SHADER_FRAG: &str = "
@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

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
";

const COMPOSITE_SHADER_FRAG: &str = "
@group(0) @binding(0) var t_scene: texture_2d<f32>;
@group(0) @binding(1) var t_bloom: texture_2d<f32>;
@group(0) @binding(2) var s_composite: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

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
pub(crate) struct ShapeVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    pub(crate) world_rect: [f32; 4],
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

pub(crate) struct ShapeBatch {
    vertices: Vec<ShapeVertex>,
    indices: Vec<u32>,
}

impl ShapeBatch {
    pub(crate) fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, positions: &[[f32; 2]], indices: &[u32], color: Color) {
        let base = self.vertices.len() as u32;
        let color = [color.r, color.g, color.b, color.a];
        self.vertices.extend(
            positions
                .iter()
                .map(|&position| ShapeVertex { position, color }),
        );
        self.indices.extend(indices.iter().map(|&i| i + base));
    }

    #[cfg(test)]
    pub(crate) fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub(crate) fn vertices(&self) -> &[ShapeVertex] {
        &self.vertices
    }

    pub(crate) fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

pub(crate) fn rect_to_instance(rect: &Rect) -> Instance {
    let Pixels(x) = rect.x;
    let Pixels(y) = rect.y;
    let Pixels(w) = rect.width;
    let Pixels(h) = rect.height;

    Instance {
        world_rect: [x, y, w, h],
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
    #[allow(clippy::similar_names, clippy::too_many_lines)]
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
    quad_pipelines: [wgpu::RenderPipeline; 3],
    quad_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    clear_color: Color,
    pending_instances: Vec<Instance>,
    instance_blend_modes: Vec<crate::material::BlendMode>,
    current_blend_mode: crate::material::BlendMode,
    shape_batch: ShapeBatch,
    shape_blend_modes: Vec<crate::material::BlendMode>,
    shape_index_offsets: Vec<u32>,
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

        let identity: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
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
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
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
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
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
            shape_blend_modes: Vec::new(),
            shape_index_offsets: Vec::new(),
            shape_pipelines,
            post_process: None,
            post_process_pending: false,
            bloom_threshold: 0.8,
            bloom_intensity: 0.3,
        }
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

            let total_indices = self.shape_batch.index_count() as u32;
            for (mode, shape_range) in compute_batch_ranges(&self.shape_blend_modes) {
                let idx_start = self.shape_index_offsets[shape_range.start as usize];
                let idx_end = if (shape_range.end as usize) < self.shape_index_offsets.len() {
                    self.shape_index_offsets[shape_range.end as usize]
                } else {
                    total_indices
                };
                pass.set_pipeline(&self.shape_pipelines[mode.index()]);
                pass.draw_indexed(idx_start..idx_end, 0, 0..1);
            }
        }
    }

    #[allow(clippy::too_many_lines)]
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

pub(crate) fn create_texture_bind_group(
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
        self.instance_blend_modes.clear();
        self.current_blend_mode = crate::material::BlendMode::Alpha;
        self.shape_batch.clear();
        self.shape_blend_modes.clear();
        self.shape_index_offsets.clear();
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
        self.shape_blend_modes.push(self.current_blend_mode);
        #[allow(clippy::cast_possible_truncation)]
        self.shape_index_offsets
            .push(self.shape_batch.index_count() as u32);
        self.shape_batch.push(vertices, indices, color);
    }

    fn set_blend_mode(&mut self, mode: crate::material::BlendMode) {
        self.current_blend_mode = mode;
    }

    fn set_shader(&mut self, _shader: crate::material::ShaderHandle) {}

    fn set_material_uniforms(&mut self, _data: &[u8]) {}

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

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn compute_batch_ranges(
    modes: &[crate::material::BlendMode],
) -> Vec<(crate::material::BlendMode, std::ops::Range<u32>)> {
    let mut batches = Vec::new();
    let Some(&first) = modes.first() else {
        return batches;
    };
    let mut current_mode = first;
    let mut start = 0u32;
    for (i, &mode) in modes.iter().enumerate().skip(1) {
        if mode != current_mode {
            batches.push((current_mode, start..i as u32));
            current_mode = mode;
            start = i as u32;
        }
    }
    batches.push((current_mode, start..modes.len() as u32));
    batches
}

pub(crate) fn blend_mode_to_blend_state(mode: crate::material::BlendMode) -> wgpu::BlendState {
    use crate::material::BlendMode;
    match mode {
        BlendMode::Alpha => wgpu::BlendState::ALPHA_BLENDING,
        BlendMode::Additive => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        },
        BlendMode::Multiply => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        },
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
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
        assert_eq!(instance.world_rect, [0.0, 0.0, 800.0, 600.0]);
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
        assert_eq!(instance.world_rect, [200.0, 150.0, 400.0, 300.0]);
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
        assert_eq!(instance.world_rect, [400.0, 300.0, 0.0, 0.0]);
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
        assert_eq!(instance.world_rect, [400.0, 300.0, -100.0, -50.0]);
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
    fn when_shape_vertex_size_checked_then_exactly_24_bytes() {
        // Act
        let size = std::mem::size_of::<ShapeVertex>();

        // Assert
        assert_eq!(size, 24);
    }

    #[test]
    fn when_shape_vertices_cast_to_bytes_then_no_panic() {
        // Arrange
        let vertices = [
            ShapeVertex {
                position: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            ShapeVertex {
                position: [1.0, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        // Act
        let bytes: &[u8] = bytemuck::cast_slice(&vertices);

        // Assert
        assert_eq!(bytes.len(), 48);
    }

    #[test]
    fn when_single_shape_pushed_then_vertex_and_index_counts_match_input() {
        // Arrange
        let vertices = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0_u32, 1, 2];
        let mut batch = ShapeBatch::new();

        // Act
        batch.push(&vertices, &indices, Color::RED);

        // Assert
        assert_eq!(batch.vertex_count(), 3);
        assert_eq!(batch.index_count(), 3);
    }

    #[test]
    fn when_two_shapes_pushed_then_second_indices_are_offset_by_first_vertex_count() {
        // Arrange
        let tri_verts = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let tri_indices = [0_u32, 1, 2];
        let quad_verts = [[2.0_f32, 0.0], [3.0, 0.0], [3.0, 1.0], [2.0, 1.0]];
        let quad_indices = [0_u32, 1, 2, 0, 2, 3];
        let mut batch = ShapeBatch::new();
        batch.push(&tri_verts, &tri_indices, Color::RED);

        // Act
        batch.push(&quad_verts, &quad_indices, Color::BLUE);

        // Assert
        assert_eq!(&batch.indices()[3..], &[3_u32, 4, 5, 3, 5, 6]);
    }

    #[test]
    fn when_batch_cleared_then_vertex_and_index_counts_are_zero() {
        // Arrange
        let vertices = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0_u32, 1, 2];
        let mut batch = ShapeBatch::new();
        batch.push(&vertices, &indices, Color::RED);

        // Act
        batch.clear();

        // Assert
        assert_eq!(batch.vertex_count(), 0);
        assert_eq!(batch.index_count(), 0);
    }

    #[test]
    fn when_batch_is_empty_then_is_empty_returns_true() {
        // Act
        let batch = ShapeBatch::new();

        // Assert
        assert!(batch.is_empty());
    }

    #[test]
    fn when_triangle_pushed_then_vertices_returns_three_and_is_empty_false() {
        // Arrange
        let positions = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0_u32, 1, 2];
        let mut batch = ShapeBatch::new();

        // Act
        batch.push(&positions, &indices, Color::RED);

        // Assert
        assert!(!batch.is_empty());
        assert_eq!(batch.vertices().len(), 3);
        assert_eq!(batch.vertices()[0].position, [0.0, 0.0]);
        assert_eq!(batch.vertices()[1].position, [1.0, 0.0]);
        assert_eq!(batch.vertices()[2].position, [0.5, 1.0]);
    }

    #[test]
    fn when_shape_shader_parsed_then_no_error() {
        // Act
        let result = naga::front::wgsl::parse_str(SHAPE_SHADER_SRC);

        // Assert
        assert!(result.is_ok(), "WGSL parse error: {result:?}");
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

    #[test]
    fn when_blend_mode_alpha_then_blend_state_is_alpha_blending() {
        // Act
        let result = blend_mode_to_blend_state(crate::material::BlendMode::Alpha);

        // Assert
        assert_eq!(result, wgpu::BlendState::ALPHA_BLENDING);
    }

    #[test]
    fn when_blend_mode_additive_then_blend_state_uses_src_alpha_one() {
        // Act
        let result = blend_mode_to_blend_state(crate::material::BlendMode::Additive);

        // Assert
        let expected = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn when_blend_mode_multiply_then_blend_state_uses_dst_zero() {
        // Act
        let result = blend_mode_to_blend_state(crate::material::BlendMode::Multiply);

        // Assert
        let expected = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn when_mixed_blend_modes_then_batches_split_at_boundaries() {
        use crate::material::BlendMode;

        // Arrange
        let modes = [
            BlendMode::Alpha,
            BlendMode::Alpha,
            BlendMode::Additive,
            BlendMode::Alpha,
        ];

        // Act
        let batches = compute_batch_ranges(&modes);

        // Assert
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], (BlendMode::Alpha, 0..2));
        assert_eq!(batches[1], (BlendMode::Additive, 2..3));
        assert_eq!(batches[2], (BlendMode::Alpha, 3..4));
    }

    #[test]
    fn when_all_same_blend_mode_then_single_batch() {
        use crate::material::BlendMode;

        // Arrange
        let modes = [BlendMode::Alpha, BlendMode::Alpha, BlendMode::Alpha];

        // Act
        let batches = compute_batch_ranges(&modes);

        // Assert
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], (BlendMode::Alpha, 0..3));
    }

    #[test]
    fn when_no_items_then_empty_batches() {
        // Act
        let batches = compute_batch_ranges(&[]);

        // Assert
        assert!(batches.is_empty());
    }
}
