use engine_core::color::Color;

pub const SLOT_WIDTH: f32 = 50.0;
pub const SLOT_HEIGHT: f32 = 75.0;
pub const SLOT_GAP: f32 = 4.0;
pub const SLOT_STRIDE_W: f32 = SLOT_WIDTH + SLOT_GAP;
pub const SLOT_STRIDE_H: f32 = SLOT_HEIGHT + SLOT_GAP;
pub const SLOT_COLOR: Color = Color {
    r: 0.25,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
pub const GRID_MARGIN: f32 = 20.0;
pub const BACKGROUND_COLOR: Color = Color {
    r: 0.15,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};
