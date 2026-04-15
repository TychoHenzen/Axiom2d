use engine_core::color::Color;
use engine_core::types::Pixels;

use crate::material::BlendMode;
use crate::rect::Rect;

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
    pub(crate) uv: [f32; 2],
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
pub(super) struct BloomParamsUniform {
    pub(super) threshold: f32,
    pub(super) intensity: f32,
    pub(super) direction: [f32; 2],
    pub(super) texel_size: [f32; 2],
    pub(super) _pad: [f32; 2],
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

    fn append_indices(&mut self, indices: &[u32], base: u32) {
        self.indices.reserve(indices.len());
        self.indices.extend(indices.iter().map(|index| {
            base.checked_add(*index)
                .expect("shape index exceeds u32 range")
        }));
    }

    fn push_indexed_vertices(
        &mut self,
        vertices: impl ExactSizeIterator<Item = ShapeVertex>,
        indices: &[u32],
    ) {
        let base =
            u32::try_from(self.vertices.len()).expect("shape vertex count exceeds u32 range");
        self.vertices.reserve(vertices.len());
        self.vertices.extend(vertices);
        self.append_indices(indices, base);
    }

    pub(crate) fn push(&mut self, positions: &[[f32; 2]], indices: &[u32], color: Color) {
        let color = [color.r, color.g, color.b, color.a];
        self.push_indexed_vertices(
            positions.iter().copied().map(|position| ShapeVertex {
                position,
                color,
                uv: [0.0, 0.0],
            }),
            indices,
        );
    }

    pub(crate) fn push_colored(&mut self, vertices: &[ShapeVertex], indices: &[u32]) {
        self.push_indexed_vertices(vertices.iter().copied(), indices);
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
        self.indices.is_empty()
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

pub(crate) fn compute_batch_ranges(modes: &[BlendMode]) -> Vec<(BlendMode, std::ops::Range<u32>)> {
    let mut batches = Vec::with_capacity(modes.len());
    let Some((&first_mode, _)) = modes.split_first() else {
        return batches;
    };

    let to_u32 = |value: usize| u32::try_from(value).expect("batch index exceeds u32 range");
    let mut batch_mode = first_mode;
    let mut batch_start = 0usize;

    for (index, pair) in modes.windows(2).enumerate() {
        if pair[0] != pair[1] {
            let end = index + 1;
            batches.push((batch_mode, to_u32(batch_start)..to_u32(end)));
            batch_mode = pair[1];
            batch_start = end;
        }
    }

    batches.push((batch_mode, to_u32(batch_start)..to_u32(modes.len())));
    batches
}

pub(crate) struct TextureData<'a> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) data: &'a [u8],
}

pub(crate) fn create_texture_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    tex: TextureData<'_>,
) -> wgpu::BindGroup {
    let view = upload_texture(device, queue, &tex);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
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

fn rgba_texture_descriptor(tex: &TextureData<'_>) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: tex.width,
            height: tex.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    }
}

fn upload_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex: &TextureData<'_>,
) -> wgpu::TextureView {
    let desc = rgba_texture_descriptor(tex);
    let size = desc.size;
    let texture = device.create_texture(&desc);
    let bytes_per_row = tex
        .width
        .checked_mul(4)
        .expect("texture row size exceeds u32 range");
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        tex.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(tex.height),
        },
        size,
    );
    texture.create_view(&wgpu::TextureViewDescriptor::default())
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

pub(super) struct FullscreenPass<'a> {
    pub(super) target: &'a wgpu::TextureView,
    pub(super) pipeline: &'a wgpu::RenderPipeline,
    pub(super) tex_bg: &'a wgpu::BindGroup,
    pub(super) params_bg: &'a wgpu::BindGroup,
}

pub(super) fn run_fullscreen_pass(
    encoder: &mut wgpu::CommandEncoder,
    buffers: &FullscreenBuffers<'_>,
    desc: &FullscreenPass<'_>,
) {
    let quad_index_count =
        u32::try_from(QUAD_INDICES.len()).expect("quad index count exceeds u32 range");

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: desc.target,
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
    pass.set_pipeline(desc.pipeline);
    pass.set_bind_group(0, desc.tex_bg, &[]);
    pass.set_bind_group(1, desc.params_bg, &[]);
    pass.set_vertex_buffer(0, buffers.vertex.slice(..));
    pass.set_index_buffer(buffers.index.slice(..), wgpu::IndexFormat::Uint16);
    pass.draw_indexed(0..quad_index_count, 0, 0..1);
}

pub(super) struct FullscreenBuffers<'a> {
    pub(super) vertex: &'a wgpu::Buffer,
    pub(super) index: &'a wgpu::Buffer,
}
