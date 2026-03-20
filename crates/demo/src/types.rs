use axiom2d::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct FrameCount(pub u64);

#[derive(Component)]
pub(crate) struct Sun;

#[derive(Component)]
pub(crate) struct Moon;

#[derive(Component)]
pub(crate) struct OrbitalSpeed(pub f32);

pub(crate) const SUN_POSITION: Vec2 = Vec2::ZERO;
pub(crate) const SUN_COLOR: Color = Color {
    r: 1.0,
    g: 0.85,
    b: 0.0,
    a: 1.0,
};
pub(crate) const CAMERA_PAN_SPEED: f32 = 300.0;
pub(crate) const CAMERA_ZOOM_SPEED: f32 = 1.0;
pub(crate) const ZOOM_MIN: f32 = 0.1;

pub(crate) mod action {
    pub const MOVE_RIGHT: &str = "move_right";
    pub const MOVE_LEFT: &str = "move_left";
    pub const MOVE_UP: &str = "move_up";
    pub const MOVE_DOWN: &str = "move_down";
    pub const ZOOM_IN: &str = "zoom_in";
    pub const ZOOM_OUT: &str = "zoom_out";
}

pub(crate) struct CelestialDef {
    pub orbit_radius: f32,
    pub speed: f32,
    pub color: Color,
    pub size: f32,
    pub moon: Option<MoonDef>,
}

pub(crate) struct MoonDef {
    pub orbit_radius: f32,
    pub speed: f32,
    pub color: Color,
    pub size: f32,
}
