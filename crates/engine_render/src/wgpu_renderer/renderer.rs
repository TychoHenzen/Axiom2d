use std::collections::HashMap;
use std::sync::Arc;

use winit::window::Window;

use engine_core::color::Color;

use crate::shader::ShaderHandle;
use crate::window::WindowConfig;

use super::bloom::PostProcessResources;
use super::gpu_init;
use super::types::{Instance, ShapeBatch};

pub(super) struct ShapeDrawRecord {
    pub(super) blend_mode: crate::material::BlendMode,
    pub(super) shader_handle: ShaderHandle,
    pub(super) index_offset: u32,
    pub(super) model: [[f32; 4]; 4],
}

pub struct WgpuRenderer {
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) quad_pipelines: [wgpu::RenderPipeline; 3],
    pub(super) quad_vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) texture_bind_group: wgpu::BindGroup,
    pub(super) camera_uniform_buffer: wgpu::Buffer,
    pub(super) camera_bind_group: wgpu::BindGroup,
    pub(super) clear_color: Color,
    pub(super) pending_instances: Vec<Instance>,
    pub(super) instance_blend_modes: Vec<crate::material::BlendMode>,
    pub(super) current_blend_mode: crate::material::BlendMode,
    pub(super) shape_batch: ShapeBatch,
    pub(super) shape_draws: Vec<ShapeDrawRecord>,
    pub(super) shape_pipelines: [wgpu::RenderPipeline; 3],
    pub(super) shape_pipeline_layout: wgpu::PipelineLayout,
    pub(super) model_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) model_uniform_align: u32,
    pub(super) shader_cache: HashMap<ShaderHandle, [wgpu::RenderPipeline; 3]>,
    pub(super) active_shader: ShaderHandle,
    pub(super) surface_format: wgpu::TextureFormat,
    pub(super) post_process: Option<PostProcessResources>,
    pub(super) post_process_pending: bool,
    pub(super) bloom_threshold: f32,
    pub(super) bloom_intensity: f32,
    pub(super) glyph_cache: crate::font::GlyphCache,
    pub(super) msaa_view: wgpu::TextureView,
    pub(super) sample_count: u32,
}

impl WgpuRenderer {
    pub fn new(window: Arc<Window>, config: &WindowConfig) -> Self {
        Self::from_parts(gpu_init::build_renderer_parts(window, config))
    }

    pub(super) fn create_msaa_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa_texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn from_parts(p: gpu_init::RendererParts) -> Self {
        let msaa_view = Self::create_msaa_texture(
            &p.gpu.device,
            p.gpu_format,
            p.gpu.config.width,
            p.gpu.config.height,
            p.sample_count,
        );
        Self {
            surface: p.gpu.surface,
            device: p.gpu.device,
            queue: p.gpu.queue,
            config: p.gpu.config,
            quad_pipelines: p.quad_pipelines,
            quad_vertex_buffer: p.quad_vertex_buffer,
            index_buffer: p.index_buffer,
            texture_bind_group_layout: p.tex_layout,
            texture_bind_group: p.texture_bind_group,
            camera_uniform_buffer: p.cam.uniform_buffer,
            camera_bind_group: p.cam.bind_group,
            shape_pipelines: p.shape.pipelines,
            shape_pipeline_layout: p.shape.pipeline_layout,
            model_bind_group_layout: p.shape.model_bind_group_layout,
            model_uniform_align: p.shape.model_uniform_align,
            surface_format: p.gpu_format,
            clear_color: Color::BLACK,
            pending_instances: Vec::new(),
            instance_blend_modes: Vec::new(),
            current_blend_mode: crate::material::BlendMode::Alpha,
            shape_batch: ShapeBatch::new(),
            shape_draws: Vec::new(),
            shader_cache: HashMap::new(),
            active_shader: ShaderHandle(0),
            post_process: None,
            post_process_pending: false,
            bloom_threshold: 0.8,
            bloom_intensity: 0.3,
            glyph_cache: crate::font::GlyphCache::new(),
            msaa_view,
            sample_count: p.sample_count,
        }
    }

    pub(super) fn reset_frame_state(&mut self) {
        self.pending_instances.clear();
        self.instance_blend_modes.clear();
        self.current_blend_mode = crate::material::BlendMode::Alpha;
        self.active_shader = ShaderHandle(0);
        self.shape_batch.clear();
        self.shape_draws.clear();
    }

    pub(super) fn ensure_post_process(&mut self) {
        if self.post_process.is_none() {
            self.post_process = Some(PostProcessResources::new(
                &self.device,
                &self.bloom_config(),
            ));
        }
    }

    pub(super) fn bloom_config(&self) -> super::bloom::BloomConfig {
        super::bloom::BloomConfig {
            format: self.config.format,
            width: self.config.width,
            height: self.config.height,
            threshold: self.bloom_threshold,
            intensity: self.bloom_intensity,
        }
    }
}
