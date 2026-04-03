use std::collections::HashMap;
use std::sync::Arc;

use winit::window::Window;

use engine_core::color::Color;
use engine_core::types::TextureId;

use crate::renderer::GpuMeshHandle;
use crate::shader::ShaderHandle;
use crate::window::WindowConfig;

use super::bloom::PostProcessResources;
use super::gpu_init;
use super::types::{Instance, ShapeBatch};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedTextureBinding {
    pub texture_id: u32,
    pub binding: u32,
    pub uv_rect: [f32; 4],
    pub _pad: [u32; 2],
}

pub(super) struct PersistentMesh {
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) index_count: u32,
}

pub(super) enum MeshSource {
    Batched { index_start: u32, index_count: u32 },
    Persistent { handle: GpuMeshHandle },
}

pub(super) struct ShapeDrawRecord {
    pub(super) blend_mode: crate::material::BlendMode,
    pub(super) shader_handle: ShaderHandle,
    pub(super) source: MeshSource,
    pub(super) model: [[f32; 4]; 4],
    pub(super) material_uniforms: Vec<u8>,
    #[allow(dead_code)]
    pub(super) material_textures: Vec<(TextureId, u32)>,
}

#[derive(Default)]
pub struct PendingMaterialBindings {
    uniforms: Vec<u8>,
    textures: Vec<(TextureId, u32)>,
}

impl PendingMaterialBindings {
    pub(super) fn clear(&mut self) {
        self.uniforms.clear();
        self.textures.clear();
    }

    pub fn set_uniforms(&mut self, data: &[u8]) {
        self.uniforms = data.to_vec();
    }

    pub fn bind_texture(&mut self, texture: TextureId, binding: u32) {
        self.textures.push((texture, binding));
    }

    pub fn take_uniforms(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.uniforms)
    }

    pub fn take_textures(&mut self) -> Vec<(TextureId, u32)> {
        std::mem::take(&mut self.textures)
    }
}

pub(super) struct UploadBuffer {
    pub(super) buffer: wgpu::Buffer,
    capacity: usize,
    usage: wgpu::BufferUsages,
}

impl UploadBuffer {
    pub(super) fn new(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        min_capacity: usize,
    ) -> Self {
        let capacity = next_upload_capacity(min_capacity);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: capacity as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            capacity,
            usage,
        }
    }

    pub(super) fn ensure_capacity(&mut self, device: &wgpu::Device, min_capacity: usize) {
        if min_capacity <= self.capacity {
            return;
        }
        *self = Self::new(device, self.usage, min_capacity);
    }
}

pub(super) struct UploadBindGroupBuffer {
    pub(super) storage: UploadBuffer,
    pub(super) bind_group: wgpu::BindGroup,
    binding_size: u64,
}

impl UploadBindGroupBuffer {
    pub(super) fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        binding_size: u64,
        min_capacity: usize,
    ) -> Self {
        let storage = UploadBuffer::new(device, wgpu::BufferUsages::UNIFORM, min_capacity);
        let bind_group = create_uniform_bind_group(device, layout, &storage.buffer, binding_size);
        Self {
            storage,
            bind_group,
            binding_size,
        }
    }

    pub(super) fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        binding_size: u64,
        min_capacity: usize,
    ) {
        if min_capacity <= self.storage.capacity && binding_size == self.binding_size {
            return;
        }
        *self = Self::new(device, layout, binding_size, min_capacity);
    }
}

fn next_upload_capacity(min_capacity: usize) -> usize {
    min_capacity.max(64).next_power_of_two()
}

fn create_uniform_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    binding_size: u64,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(binding_size),
            }),
        }],
    })
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
    pub(super) persistent_meshes: HashMap<GpuMeshHandle, PersistentMesh>,
    pub(super) next_persistent_id: u32,
    pub(super) shape_pipelines: [wgpu::RenderPipeline; 3],
    pub(super) shape_pipeline_layout: wgpu::PipelineLayout,
    pub(super) model_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) material_bind_group_layout: wgpu::BindGroupLayout,
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
    pub(super) instance_upload_buffer: Option<UploadBuffer>,
    pub(super) shape_vertex_upload_buffer: Option<UploadBuffer>,
    pub(super) shape_index_upload_buffer: Option<UploadBuffer>,
    pub(super) model_upload_buffer: Option<UploadBindGroupBuffer>,
    pub(super) material_upload_buffer: Option<UploadBindGroupBuffer>,
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
            texture_lookups: HashMap::new(),
            camera_uniform_buffer: p.cam.uniform_buffer,
            camera_bind_group: p.cam.bind_group,
            shape_pipelines: p.shape.pipelines,
            shape_pipeline_layout: p.shape.pipeline_layout,
            model_bind_group_layout: p.shape.model_bind_group_layout,
            material_bind_group_layout: p.shape.material_bind_group_layout,
            model_uniform_align: p.shape.model_uniform_align,
            surface_format: p.gpu_format,
            clear_color: Color::BLACK,
            pending_instances: Vec::new(),
            instance_blend_modes: Vec::new(),
            current_blend_mode: crate::material::BlendMode::Alpha,
            shape_batch: ShapeBatch::new(),
            shape_draws: Vec::new(),
            persistent_meshes: HashMap::new(),
            next_persistent_id: 1,
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
            instance_upload_buffer: None,
            shape_vertex_upload_buffer: None,
            shape_index_upload_buffer: None,
            model_upload_buffer: None,
            material_upload_buffer: None,
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

pub fn pack_material_bindings(
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

pub(super) fn align_to_uniform_offset(size: usize, alignment: usize) -> usize {
    let alignment = alignment.max(1);
    size.div_ceil(alignment) * alignment
}

pub(super) fn pack_material_frame_data(
    draws: &[ShapeDrawRecord],
    lookups: &HashMap<TextureId, [f32; 4]>,
    alignment: usize,
) -> (usize, Vec<u8>) {
    let packed_materials: Vec<Vec<u8>> = draws
        .iter()
        .map(|draw| {
            pack_material_bindings(&draw.material_uniforms, &draw.material_textures, lookups)
        })
        .collect();
    let max_len = packed_materials
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(32)
        .max(32);
    let slot_size = align_to_uniform_offset(max_len, alignment);
    let mut frame_data = vec![0u8; packed_materials.len() * slot_size];
    for (i, packed) in packed_materials.iter().enumerate() {
        let offset = i * slot_size;
        frame_data[offset..offset + packed.len()].copy_from_slice(packed);
    }
    (slot_size, frame_data)
}

#[cfg(test)]
mod tests {
    use super::{
        MeshSource, PackedTextureBinding, ShapeDrawRecord, align_to_uniform_offset,
        next_upload_capacity, pack_material_frame_data,
    };
    use crate::shader::ShaderHandle;
    use engine_core::types::TextureId;
    use std::collections::HashMap;

    #[test]
    fn when_upload_capacity_requested_then_growth_rounds_up_once() {
        assert_eq!(next_upload_capacity(1), 64);
        assert_eq!(next_upload_capacity(64), 64);
        assert_eq!(next_upload_capacity(65), 128);
        assert_eq!(next_upload_capacity(300), 512);
    }

    #[test]
    fn when_uniform_slot_size_calculated_then_rounds_up_to_alignment() {
        assert_eq!(align_to_uniform_offset(32, 256), 256);
        assert_eq!(align_to_uniform_offset(300, 256), 512);
        assert_eq!(align_to_uniform_offset(64, 1), 64);
    }

    #[test]
    fn when_material_frame_data_packed_then_each_draw_uses_same_aligned_slot() {
        let mut lookups = HashMap::new();
        lookups.insert(TextureId(9), [0.1, 0.2, 0.3, 0.4]);
        let draws = vec![
            ShapeDrawRecord {
                blend_mode: crate::material::BlendMode::Alpha,
                shader_handle: ShaderHandle(0),
                source: MeshSource::Batched {
                    index_start: 0,
                    index_count: 6,
                },
                model: glam::Mat4::IDENTITY.to_cols_array_2d(),
                material_uniforms: vec![1, 2, 3],
                material_textures: Vec::new(),
            },
            ShapeDrawRecord {
                blend_mode: crate::material::BlendMode::Alpha,
                shader_handle: ShaderHandle(0),
                source: MeshSource::Batched {
                    index_start: 6,
                    index_count: 6,
                },
                model: glam::Mat4::IDENTITY.to_cols_array_2d(),
                material_uniforms: vec![4; 32],
                material_textures: vec![(TextureId(9), 2)],
            },
        ];

        let (slot_size, packed) = pack_material_frame_data(&draws, &lookups, 16);

        assert_eq!(slot_size, 64);
        assert_eq!(packed.len(), 128);
        assert_eq!(&packed[..3], &[1, 2, 3]);
        let tex = bytemuck::from_bytes::<PackedTextureBinding>(&packed[96..128]);
        assert_eq!(tex.texture_id, 9);
        assert_eq!(tex.binding, 2);
        assert_eq!(tex.uv_rect, [0.1, 0.2, 0.3, 0.4]);
    }
}
