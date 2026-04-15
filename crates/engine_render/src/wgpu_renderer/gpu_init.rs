// EVOLVE-BLOCK-START
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::window::WindowConfig;

use super::shaders::{SHADER_SRC, SHAPE_SHADER_SRC};
use super::types::{
    Instance, QUAD_INDICES, QUAD_VERTICES, QuadVertex, ShapeVertex, TextureData,
    blend_mode_to_blend_state, create_texture_bind_group,
};

const SAMPLE_COUNT: u32 = 4;
const WHITE_TEXTURE: [u8; 4] = [255, 255, 255, 255];

pub(super) struct GpuContext {
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
}

pub(super) struct CameraResources {
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
}

fn uniform_bind_group_layout(
    device: &wgpu::Device,
    visibility: wgpu::ShaderStages,
    has_dynamic_offset: bool,
    min_binding_size: Option<wgpu::BufferSize>,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset,
                min_binding_size,
            },
            count: None,
        }],
    })
}

pub(super) struct ShapeResources {
    pub(super) pipelines: [wgpu::RenderPipeline; 3],
    pub(super) pipeline_layout: wgpu::PipelineLayout,
    pub(super) model_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) material_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) model_uniform_align: u32,
}

fn request_adapter_and_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    // INVARIANT: Adapter request fails only when no GPU supports the
    // required surface format. The renderer cannot function without a GPU.
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(surface),
        ..Default::default()
    }))
    .expect("no compatible GPU adapter found");
    // INVARIANT: Device creation fails only on hardware/driver errors.
    // Without a device, no rendering is possible.
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .expect("failed to create GPU device");
    (adapter, device, queue)
}

pub(super) fn init_gpu(window: Arc<Window>, config: &WindowConfig) -> GpuContext {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    // INVARIANT: Surface creation fails only on incompatible window handles.
    // The window was just created by winit, so this is unreachable.
    let surface = instance
        .create_surface(window.clone())
        .expect("failed to create surface");
    let (adapter, device, queue) = request_adapter_and_device(&instance, &surface);
    let surface_config = configure_surface(&surface, &adapter, config, &window);
    surface.configure(&device, &surface_config);
    GpuContext {
        surface,
        device,
        queue,
        config: surface_config,
    }
}

fn configure_surface(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    config: &WindowConfig,
    window: &Window,
) -> wgpu::SurfaceConfiguration {
    let size = window.inner_size();
    let caps = surface.get_capabilities(adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .unwrap_or_else(|| {
            caps.formats
                .first()
                .copied()
                .expect("surface reported no supported formats")
        });
    let alpha_mode = caps
        .alpha_modes
        .first()
        .copied()
        .expect("surface reported no alpha modes");
    let desired_present_mode = if config.vsync {
        wgpu::PresentMode::AutoVsync
    } else {
        wgpu::PresentMode::AutoNoVsync
    };
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: desired_present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

pub(super) fn create_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    })
}

fn camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    uniform_bind_group_layout(device, wgpu::ShaderStages::VERTEX, false, None)
}

pub(super) fn create_camera_resources(
    device: &wgpu::Device,
) -> (CameraResources, wgpu::BindGroupLayout) {
    let layout = camera_bind_group_layout(device);
    let identity = glam::Mat4::IDENTITY.to_cols_array_2d();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&identity),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    (
        CameraResources {
            uniform_buffer,
            bind_group,
        },
        layout,
    )
}

const QUAD_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 0,
    shader_location: 0,
}];

const INSTANCE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
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
];

const SHAPE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
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
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 24,
        shader_location: 2,
    },
];

fn quad_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<QuadVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &QUAD_VERTEX_ATTRIBUTES,
    }
}

fn instance_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<Instance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &INSTANCE_VERTEX_ATTRIBUTES,
    }
}

fn shape_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<ShapeVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &SHAPE_VERTEX_ATTRIBUTES,
    }
}

pub(super) fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    vs_entry: &str,
    fs_entry: &str,
    format: wgpu::TextureFormat,
    blend: wgpu::BlendState,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
    sample_count: u32,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(vs_entry),
            buffers: vertex_layouts,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fs_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn create_pipeline_set<'a>(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    sample_count: u32,
    vertex_layouts: &'a [wgpu::VertexBufferLayout<'a>],
    vs_entry: &'static str,
    fs_entry: &'static str,
) -> [wgpu::RenderPipeline; 3] {
    crate::material::BlendMode::ALL.map(|mode| {
        create_pipeline(
            device,
            layout,
            shader,
            vs_entry,
            fs_entry,
            format,
            blend_mode_to_blend_state(mode),
            vertex_layouts,
            sample_count,
        )
    })
}

pub(super) fn create_quad_pipeline_set(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> [wgpu::RenderPipeline; 3] {
    create_pipeline_set(
        device,
        layout,
        shader,
        format,
        sample_count,
        &[quad_vertex_layout(), instance_vertex_layout()],
        "vs_main",
        "fs_main",
    )
}

pub(super) fn create_shape_pipeline_set(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> [wgpu::RenderPipeline; 3] {
    create_pipeline_set(
        device,
        layout,
        shader,
        format,
        sample_count,
        &[shape_vertex_layout()],
        "vs_shape",
        "fs_shape",
    )
}

fn model_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    uniform_bind_group_layout(
        device,
        wgpu::ShaderStages::VERTEX,
        true,
        wgpu::BufferSize::new(64),
    )
}

fn material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    uniform_bind_group_layout(
        device,
        wgpu::ShaderStages::FRAGMENT,
        true,
        wgpu::BufferSize::new(32),
    )
}

pub(super) fn create_shape_resources(
    device: &wgpu::Device,
    camera_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> ShapeResources {
    let shape_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(SHAPE_SHADER_SRC.into()),
    });
    let mbl = model_bind_group_layout(device);
    let mat_bl = material_bind_group_layout(device);
    let model_uniform_align = device.limits().min_uniform_buffer_offset_alignment;
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[camera_layout, &mbl, &mat_bl, texture_layout],
        push_constant_ranges: &[],
    });
    let pipelines = create_shape_pipeline_set(
        device,
        &pipeline_layout,
        &shape_shader,
        format,
        sample_count,
    );
    ShapeResources {
        pipelines,
        pipeline_layout,
        model_bind_group_layout: mbl,
        material_bind_group_layout: mat_bl,
        model_uniform_align,
    }
}

pub(super) fn create_static_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
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
    (quad_vertex_buffer, index_buffer)
}

pub(super) fn create_quad_pipeline_layout(
    device: &wgpu::Device,
    texture_layout: &wgpu::BindGroupLayout,
    camera_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[texture_layout, camera_layout],
        push_constant_ranges: &[],
    })
}

pub(super) fn load_quad_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
    })
}

pub(super) struct RendererParts {
    pub(super) gpu: GpuContext,
    pub(super) tex_layout: wgpu::BindGroupLayout,
    pub(super) cam: CameraResources,
    pub(super) quad_pipelines: [wgpu::RenderPipeline; 3],
    pub(super) quad_vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) texture_bind_group: wgpu::BindGroup,
    pub(super) shape: ShapeResources,
    pub(super) sample_count: u32,
}

pub(super) fn build_renderer_parts(window: Arc<Window>, config: &WindowConfig) -> RendererParts {
    let gpu = init_gpu(window, config);
    let tex_layout = create_texture_layout(&gpu.device);
    let (cam, cam_layout) = create_camera_resources(&gpu.device);
    let shader = load_quad_shader(&gpu.device);
    let pl = create_quad_pipeline_layout(&gpu.device, &tex_layout, &cam_layout);
    let sample_count = SAMPLE_COUNT;
    let quad_pipelines =
        create_quad_pipeline_set(&gpu.device, &pl, &shader, gpu.config.format, sample_count);
    let (quad_vertex_buffer, index_buffer) = create_static_buffers(&gpu.device);
    let tex_bg = create_texture_bind_group(
        &gpu.device,
        &gpu.queue,
        &tex_layout,
        TextureData {
            width: 1,
            height: 1,
            data: &WHITE_TEXTURE,
        },
    );
    let shape = create_shape_resources(
        &gpu.device,
        &cam_layout,
        &tex_layout,
        gpu.config.format,
        sample_count,
    );
    RendererParts {
        gpu,
        tex_layout,
        cam,
        quad_pipelines,
        quad_vertex_buffer,
        index_buffer,
        texture_bind_group: tex_bg,
        shape,
        sample_count,
    }
}
// EVOLVE-BLOCK-END
