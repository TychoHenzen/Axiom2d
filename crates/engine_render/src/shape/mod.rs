mod components;
mod geometry;
mod path;
mod render;
mod tessellate;

pub use components::{ColorVertex, Shape, ShapeVariant, Stroke, TessellatedColorMesh, TessellatedMesh};
pub use geometry::{
    QUAD_INDICES, UNIT_QUAD, rect_polygon, rect_vertices, rounded_rect_path, unit_quad_model,
};
pub use path::{
    PathCommand, resolve_commands, reverse_path, sample_cubic, sample_quadratic, split_contours,
};
pub use render::{ShapeRenderDisabled, affine2_to_mat4, shape_render_system};
pub use tessellate::{TessellateError, shape_aabb, tessellate, tessellate_stroke};
