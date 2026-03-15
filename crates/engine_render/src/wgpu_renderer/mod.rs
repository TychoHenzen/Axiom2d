mod bloom;
mod renderer;
mod shaders;
mod types;

pub use renderer::WgpuRenderer;

#[cfg(any(test, feature = "testing"))]
pub(crate) use shaders::{SHADER_SRC, SHAPE_SHADER_SRC};
#[cfg(any(test, feature = "testing"))]
pub(crate) use types::{
    Instance, QUAD_INDICES, QUAD_VERTICES, QuadVertex, ShapeBatch, ShapeVertex,
    blend_mode_to_blend_state, compute_batch_ranges, create_texture_bind_group, rect_to_instance,
};
