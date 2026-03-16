use wgpu::util::DeviceExt;

use super::gpu_init::{PipelineDesc, create_pipeline};
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

pub(super) struct BloomConfig {
    pub(super) format: wgpu::TextureFormat,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) threshold: f32,
    pub(super) intensity: f32,
}

impl PostProcessResources {
    #[allow(clippy::similar_names)]
    pub(super) fn new(device: &wgpu::Device, cfg: &BloomConfig) -> Self {
        Self::build(device, cfg)
    }

    pub(super) fn resize(&mut self, device: &wgpu::Device, cfg: &BloomConfig) {
        *self = Self::build(device, cfg);
    }

    #[allow(clippy::similar_names)]
    fn build(device: &wgpu::Device, cfg: &BloomConfig) -> Self {
        let layouts = create_bloom_layouts(device);
        let sampler = create_linear_sampler(device);
        let textures = create_bloom_textures(device, cfg);
        let bgs = create_bloom_bind_groups(device, &layouts, &textures, &sampler);
        let params = create_bloom_params(device, &layouts.params, cfg);
        let pipelines = create_bloom_pipelines(device, &layouts, cfg.format);
        let fs_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&FULLSCREEN_QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            scene_view: textures.scene_view,
            ping_view: textures.ping_view,
            pong_view: textures.pong_view,
            scene_bg: bgs.scene,
            ping_bg: bgs.ping,
            pong_bg: bgs.pong,
            composite_bg: bgs.composite,
            brightness_params: params.brightness,
            h_blur_params: params.h_blur,
            v_blur_params: params.v_blur,
            composite_params: params.composite,
            brightness_pipeline: pipelines.brightness,
            blur_pipeline: pipelines.blur,
            composite_pipeline: pipelines.composite,
            fs_vertex_buffer,
        }
    }
}

struct BloomLayouts {
    single_tex: wgpu::BindGroupLayout,
    dual_tex: wgpu::BindGroupLayout,
    params: wgpu::BindGroupLayout,
}
struct BloomTextures {
    scene_view: wgpu::TextureView,
    ping_view: wgpu::TextureView,
    pong_view: wgpu::TextureView,
}
struct BloomBindGroups {
    scene: wgpu::BindGroup,
    ping: wgpu::BindGroup,
    pong: wgpu::BindGroup,
    composite: wgpu::BindGroup,
}
struct BloomParams {
    brightness: (wgpu::Buffer, wgpu::BindGroup),
    h_blur: (wgpu::Buffer, wgpu::BindGroup),
    v_blur: (wgpu::Buffer, wgpu::BindGroup),
    composite: (wgpu::Buffer, wgpu::BindGroup),
}
struct BloomPipelines {
    brightness: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
}

fn create_linear_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

fn tex_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

fn sampler_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn single_tex_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[tex_layout_entry(0), sampler_layout_entry(1)],
    })
}

fn dual_tex_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            tex_layout_entry(0),
            tex_layout_entry(1),
            sampler_layout_entry(2),
        ],
    })
}

fn params_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    })
}

fn create_bloom_layouts(device: &wgpu::Device) -> BloomLayouts {
    BloomLayouts {
        single_tex: single_tex_layout(device),
        dual_tex: dual_tex_layout(device),
        params: params_layout(device),
    }
}

fn create_bloom_textures(device: &wgpu::Device, cfg: &BloomConfig) -> BloomTextures {
    let half_w = (cfg.width / 2).max(1);
    let half_h = (cfg.height / 2).max(1);
    let scene = create_render_texture(device, cfg.format, cfg.width, cfg.height);
    let ping = create_render_texture(device, cfg.format, half_w, half_h);
    let pong = create_render_texture(device, cfg.format, half_w, half_h);
    BloomTextures {
        scene_view: scene.create_view(&wgpu::TextureViewDescriptor::default()),
        ping_view: ping.create_view(&wgpu::TextureViewDescriptor::default()),
        pong_view: pong.create_view(&wgpu::TextureViewDescriptor::default()),
    }
}

fn create_single_tex_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

#[allow(clippy::similar_names)]
fn create_bloom_bind_groups(
    device: &wgpu::Device,
    layouts: &BloomLayouts,
    textures: &BloomTextures,
    sampler: &wgpu::Sampler,
) -> BloomBindGroups {
    let scene = create_single_tex_bg(device, &layouts.single_tex, &textures.scene_view, sampler);
    let ping = create_single_tex_bg(device, &layouts.single_tex, &textures.ping_view, sampler);
    let pong = create_single_tex_bg(device, &layouts.single_tex, &textures.pong_view, sampler);
    let composite = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layouts.dual_tex,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&textures.scene_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&textures.ping_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    BloomBindGroups {
        scene,
        ping,
        pong,
        composite,
    }
}

#[allow(clippy::cast_possible_truncation)]
fn create_bloom_params(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    cfg: &BloomConfig,
) -> BloomParams {
    let scene_texel = [1.0 / cfg.width as f32, 1.0 / cfg.height as f32];
    let half_w = (cfg.width / 2).max(1) as f32;
    let half_h = (cfg.height / 2).max(1) as f32;
    let half_texel = [1.0 / half_w, 1.0 / half_h];
    BloomParams {
        brightness: create_params_buf(device, layout, cfg.threshold, [0.0, 0.0], scene_texel),
        h_blur: create_params_buf(device, layout, 0.0, [1.0, 0.0], half_texel),
        v_blur: create_params_buf(device, layout, 0.0, [0.0, 1.0], half_texel),
        composite: create_params_buf_raw(
            device,
            layout,
            &BloomParamsUniform {
                threshold: 0.0,
                intensity: cfg.intensity,
                direction: [0.0, 0.0],
                texel_size: [0.0; 2],
                _pad: [0.0; 2],
            },
        ),
    }
}

fn create_params_buf(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    threshold: f32,
    direction: [f32; 2],
    texel_size: [f32; 2],
) -> (wgpu::Buffer, wgpu::BindGroup) {
    create_params_buf_raw(
        device,
        layout,
        &BloomParamsUniform {
            threshold,
            intensity: 0.0,
            direction,
            texel_size,
            _pad: [0.0; 2],
        },
    )
}

fn create_params_buf_raw(
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

fn fs_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<QuadVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    }
}

fn load_bloom_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let bloom = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(format!("{BLOOM_PREAMBLE}{BLOOM_SHADER_FRAG}").into()),
    });
    let composite = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(format!("{BLOOM_PREAMBLE}{COMPOSITE_SHADER_FRAG}").into()),
    });
    (bloom, composite)
}

fn create_bloom_pipelines(
    device: &wgpu::Device,
    layouts: &BloomLayouts,
    format: wgpu::TextureFormat,
) -> BloomPipelines {
    let (bloom_shader, composite_shader) = load_bloom_shaders(device);
    let single_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&layouts.single_tex, &layouts.params],
        push_constant_ranges: &[],
    });
    let dual_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&layouts.dual_tex, &layouts.params],
        push_constant_ranges: &[],
    });
    let vl = [fs_vertex_layout()];
    BloomPipelines {
        brightness: create_pipeline(
            device,
            &single_pl,
            &bloom_desc(&bloom_shader, "fs_brightness", format, &vl),
        ),
        blur: create_pipeline(
            device,
            &single_pl,
            &bloom_desc(&bloom_shader, "fs_blur", format, &vl),
        ),
        composite: create_pipeline(
            device,
            &dual_pl,
            &bloom_desc(&composite_shader, "fs_composite", format, &vl),
        ),
    }
}

fn bloom_desc<'a>(
    shader: &'a wgpu::ShaderModule,
    fs_entry: &'a str,
    format: wgpu::TextureFormat,
    vertex_layouts: &'a [wgpu::VertexBufferLayout<'a>],
) -> PipelineDesc<'a> {
    PipelineDesc {
        shader,
        vs_entry: "vs_fullscreen",
        fs_entry,
        format,
        blend: wgpu::BlendState::REPLACE,
        vertex_layouts,
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
