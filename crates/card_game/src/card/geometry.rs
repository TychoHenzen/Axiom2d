use glam::Vec2;

pub const TABLE_CARD_WIDTH: f32 = 60.0;
pub const TABLE_CARD_HEIGHT: f32 = 90.0;
pub const TABLE_CARD_SIZE: Vec2 = Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT);

/// Half-extents of the art shader's fixed coordinate space.
/// Vertices passed to `draw_shape` with the art shader MUST be in
/// `[-ART_HALF_W, ART_HALF_W] × [-ART_HALF_H, ART_HALF_H]`.
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
