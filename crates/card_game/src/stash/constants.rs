use engine_core::color::Color;

pub(crate) const SLOT_WIDTH: f32 = 50.0;
pub(crate) const SLOT_HEIGHT: f32 = 75.0;
pub(crate) const SLOT_GAP: f32 = 4.0;
pub(crate) const SLOT_STRIDE_W: f32 = SLOT_WIDTH + SLOT_GAP;
pub(crate) const SLOT_STRIDE_H: f32 = SLOT_HEIGHT + SLOT_GAP;
pub(crate) const SLOT_COLOR: Color = Color {
    r: 0.25,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
pub(crate) const SLOT_HIGHLIGHT_COLOR: Color = Color {
    r: 0.45,
    g: 0.45,
    b: 0.55,
    a: 1.0,
};
pub(crate) const GRID_MARGIN: f32 = 20.0;
pub(crate) const BACKGROUND_COLOR: Color = Color {
    r: 0.15,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};
