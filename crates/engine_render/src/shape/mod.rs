mod components;
mod geometry;
mod path;
mod render;
mod tessellate;

pub use components::{Shape, ShapeVariant, Stroke, TessellatedMesh};
pub use geometry::{QUAD_INDICES, UNIT_QUAD, rect_polygon, rect_vertices, unit_quad_model};
pub use path::{
    PathCommand, resolve_commands, reverse_path, sample_cubic, sample_quadratic, split_contours,
};
pub use render::{affine2_to_mat4, shape_render_system};
pub use tessellate::{tessellate, tessellate_stroke};
