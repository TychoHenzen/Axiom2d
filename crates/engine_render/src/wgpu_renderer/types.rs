use engine_core::color::Color;
use engine_core::types::Pixels;

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

    pub(crate) fn push_colored(&mut self, vertices: &[ShapeVertex], indices: &[u32]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(vertices);
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
            bytes_per_row: Some(4 * tex.width),
            rows_per_image: Some(tex.height),
        },
        size,
    );
    texture.create_view(&wgpu::TextureViewDescriptor::default())
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

pub(super) struct FullscreenPass<'a> {
    pub(super) target: &'a wgpu::TextureView,
    pub(super) pipeline: &'a wgpu::RenderPipeline,
    pub(super) tex_bg: &'a wgpu::BindGroup,
    pub(super) params_bg: &'a wgpu::BindGroup,
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn run_fullscreen_pass(
    encoder: &mut wgpu::CommandEncoder,
    buffers: &FullscreenBuffers<'_>,
    desc: &FullscreenPass<'_>,
) {
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
    pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
}

pub(super) struct FullscreenBuffers<'a> {
    pub(super) vertex: &'a wgpu::Buffer,
    pub(super) index: &'a wgpu::Buffer,
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
    fn when_colored_vertices_pushed_then_colors_preserved_per_vertex() {
        // Arrange
        let mut batch = ShapeBatch::new();
        let vertices = [
            ShapeVertex {
                position: [0.0, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            ShapeVertex {
                position: [1.0, 0.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            ShapeVertex {
                position: [0.5, 1.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ];

        // Act
        batch.push_colored(&vertices, &[0, 1, 2]);

        // Assert
        assert_eq!(batch.vertex_count(), 3);
        assert_eq!(batch.vertices()[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(batch.vertices()[1].color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(batch.vertices()[2].color, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn when_color_vertices_cast_to_shape_vertices_then_values_match() {
        use crate::shape::ColorVertex;

        // Arrange
        let color_verts = [
            ColorVertex {
                position: [1.0, 2.0],
                color: [0.5, 0.6, 0.7, 1.0],
            },
            ColorVertex {
                position: [3.0, 4.0],
                color: [0.1, 0.2, 0.3, 0.4],
            },
        ];

        // Act — zero-copy cast from ColorVertex to ShapeVertex
        let shape_verts: &[ShapeVertex] = bytemuck::cast_slice(&color_verts);

        // Assert
        assert_eq!(shape_verts.len(), 2);
        assert_eq!(shape_verts[0].position, [1.0, 2.0]);
        assert_eq!(shape_verts[0].color, [0.5, 0.6, 0.7, 1.0]);
        assert_eq!(shape_verts[1].position, [3.0, 4.0]);
        assert_eq!(shape_verts[1].color, [0.1, 0.2, 0.3, 0.4]);
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
