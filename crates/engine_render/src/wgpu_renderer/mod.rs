mod bloom;
mod gpu_init;
pub mod renderer;
mod renderer_trait;
mod shaders;
mod types;

pub use renderer::WgpuRenderer;

#[cfg(any(test, feature = "testing"))]
pub(crate) use shaders::{SHADER_SRC, SHAPE_SHADER_SRC};
#[cfg(any(test, feature = "testing"))]
pub(crate) use types::{
    Instance, QUAD_INDICES, QUAD_VERTICES, QuadVertex, ShapeBatch, ShapeVertex, TextureData,
    blend_mode_to_blend_state, compute_batch_ranges, create_texture_bind_group, rect_to_instance,
};

pub(crate) fn copy_texture_to_staging(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    staging: &wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}
