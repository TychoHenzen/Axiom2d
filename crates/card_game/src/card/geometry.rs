pub(crate) use engine_render::prelude::{
    QUAD_INDICES, UNIT_QUAD, rect_polygon, rect_vertices, unit_quad_model,
};
use glam::Vec2;

pub const TABLE_CARD_WIDTH: f32 = 60.0;
pub const TABLE_CARD_HEIGHT: f32 = 90.0;
pub const TABLE_CARD_SIZE: Vec2 = Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT);

/// Half-extents of the card art shader's fixed coordinate space.
/// The WGSL shader (`uv_gradient.wgsl`) hardcodes these values for UV mapping:
/// `uv = local_pos / (half_size * 2) + 0.5`. All vertices passed to `draw_shape`
/// when the art shader is active MUST be in this range.
pub(crate) const ART_HALF_W: f32 = 27.0;
pub(crate) const ART_HALF_H: f32 = 22.5;

/// Quad vertices matching the art shader's expected coordinate space.
/// Use this (not `UNIT_QUAD`) when the card art shader is active.
pub(crate) const ART_QUAD: [[f32; 2]; 4] = [
    [-ART_HALF_W, -ART_HALF_H],
    [ART_HALF_W, -ART_HALF_H],
    [ART_HALF_W, ART_HALF_H],
    [-ART_HALF_W, ART_HALF_H],
];

/// Build a model matrix that scales an [`ART_QUAD`] to the given world-space
/// width × height and translates to (cx, cy).
pub(crate) fn art_quad_model(world_w: f32, world_h: f32, cx: f32, cy: f32) -> [[f32; 4]; 4] {
    let sx = world_w / (ART_HALF_W * 2.0);
    let sy = world_h / (ART_HALF_H * 2.0);
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [cx, cy, 0.0, 1.0],
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_table_card_size_then_matches_width_height() {
        assert_eq!(TABLE_CARD_SIZE.x, TABLE_CARD_WIDTH);
        assert_eq!(TABLE_CARD_SIZE.y, TABLE_CARD_HEIGHT);
    }
}
