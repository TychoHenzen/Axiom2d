use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;

use engine_core::color::Color;
use engine_core::types::TextureId;

use crate::shader::ShaderHandle;
use crate::window::WindowConfig;

use super::bloom::PostProcessResources;
use super::gpu_init;
use super::types::{Instance, ShapeBatch};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PackedTextureBinding {
    texture_id: u32,
    binding: u32,
    uv_rect: [f32; 4],
    _pad: [u32; 2],
}

pub(super) struct ShapeDrawRecord {
    pub(super) blend_mode: crate::material::BlendMode,
    pub(super) shader_handle: ShaderHandle,
    pub(super) index_offset: u32,
    pub(super) model: [[f32; 4]; 4],
    pub(super) material_uniforms: Vec<u8>,
    #[allow(dead_code)]
    pub(super) material_textures: Vec<(TextureId, u32)>,
}

#[derive(Default)]
pub(super) struct PendingMaterialBindings {
    uniforms: Vec<u8>,
    textures: Vec<(TextureId, u32)>,
}

impl PendingMaterialBindings {
    pub(super) fn clear(&mut self) {
        self.uniforms.clear();
        self.textures.clear();
    }

    pub(super) fn set_uniforms(&mut self, data: &[u8]) {
        self.uniforms = data.to_vec();
    }

    pub(super) fn bind_texture(&mut self, texture: TextureId, binding: u32) {
        self.textures.push((texture, binding));
    }

    pub(super) fn take_uniforms(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.uniforms)
    }

    pub(super) fn take_textures(&mut self) -> Vec<(TextureId, u32)> {
        std::mem::take(&mut self.textures)
    }
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
    pub(super) texture_lookups: HashMap<engine_core::types::TextureId, [f32; 4]>,
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
    pub(super) material_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) default_material_bind_group: wgpu::BindGroup,
    pub(super) model_uniform_align: u32,
    pub(super) shader_cache: HashMap<ShaderHandle, [wgpu::RenderPipeline; 3]>,
    pub(super) active_shader: ShaderHandle,
    pub(super) surface_format: wgpu::TextureFormat,
    pub(super) post_process: Option<PostProcessResources>,
    pub(super) post_process_pending: bool,
    pub(super) pending_material: PendingMaterialBindings,
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
        let default_material_buffer =
            p.gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &[0u8; 32],
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let default_material_bind_group =
            p.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &p.shape.material_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: default_material_buffer.as_entire_binding(),
                }],
            });
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
            texture_lookups: HashMap::new(),
            camera_uniform_buffer: p.cam.uniform_buffer,
            camera_bind_group: p.cam.bind_group,
            shape_pipelines: p.shape.pipelines,
            shape_pipeline_layout: p.shape.pipeline_layout,
            model_bind_group_layout: p.shape.model_bind_group_layout,
            material_bind_group_layout: p.shape.material_bind_group_layout,
            default_material_bind_group,
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
            pending_material: PendingMaterialBindings::default(),
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
        self.pending_material.clear();
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

pub(super) fn pack_material_bindings(
    uniforms: &[u8],
    textures: &[(TextureId, u32)],
    lookups: &HashMap<TextureId, [f32; 4]>,
) -> Vec<u8> {
    let mut packed = uniforms.to_vec();
    if packed.len() < 32 {
        packed.resize(32, 0);
    }
    for &(texture_id, binding) in textures {
        let binding_data = PackedTextureBinding {
            texture_id: texture_id.0,
            binding,
            uv_rect: lookups.get(&texture_id).copied().unwrap_or([0.0; 4]),
            _pad: [0; 2],
        };
        packed.extend_from_slice(bytemuck::bytes_of(&binding_data));
    }
    packed
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{PackedTextureBinding, PendingMaterialBindings, pack_material_bindings};
    use engine_core::types::TextureId;
    use std::collections::HashMap;

    #[test]
    fn when_material_bindings_recorded_then_take_returns_uniforms_and_textures_in_order() {
        let mut pending = PendingMaterialBindings::default();
        pending.set_uniforms(&[1, 2, 3]);
        pending.bind_texture(TextureId(7), 0);
        pending.bind_texture(TextureId(9), 2);

        let uniforms = pending.take_uniforms();
        let textures = pending.take_textures();

        assert_eq!(uniforms, vec![1, 2, 3]);
        assert_eq!(textures, vec![(TextureId(7), 0), (TextureId(9), 2)]);
    }

    #[test]
    fn when_material_bindings_taken_then_subsequent_take_is_empty() {
        let mut pending = PendingMaterialBindings::default();
        pending.set_uniforms(&[4, 5]);
        pending.bind_texture(TextureId(11), 1);

        let _ = pending.take_uniforms();
        let _ = pending.take_textures();

        assert!(pending.take_uniforms().is_empty());
        assert!(pending.take_textures().is_empty());
    }

    #[test]
    fn when_material_bindings_packed_then_texture_lookup_data_appended_after_uniforms() {
        let mut lookups = HashMap::new();
        lookups.insert(TextureId(7), [0.1, 0.2, 0.3, 0.4]);
        let packed = pack_material_bindings(&[9, 8, 7], &[(TextureId(7), 2)], &lookups);

        let tex = bytemuck::from_bytes::<PackedTextureBinding>(&packed[32..64]);
        assert_eq!(tex.texture_id, 7);
        assert_eq!(tex.binding, 2);
        assert_eq!(tex.uv_rect, [0.1, 0.2, 0.3, 0.4]);
    }
}
