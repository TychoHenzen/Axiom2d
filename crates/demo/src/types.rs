// EVOLVE-BLOCK-START
use axiom2d::prelude::*;

#[derive(Resource, Default)]
pub struct FrameCount(pub u64);

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct Earth;

#[derive(Component)]
pub struct Moon;

#[derive(Component)]
pub struct SynodicFrame;

#[derive(Component)]
pub struct OrbitalSpeed(pub f32);

pub const EARTH_POSITION: Vec2 = Vec2::ZERO;
pub const MOON_POSITION: Vec2 = EARTH_POSITION;
pub const SUN_POSITION: Vec2 = EARTH_POSITION;
pub const EARTH_COLOR: Color = Color {
    r: 0.20,
    g: 0.52,
    b: 0.86,
    a: 1.0,
};
pub const MOON_COLOR: Color = Color {
    r: 0.83,
    g: 0.83,
    b: 0.86,
    a: 1.0,
};
pub const SUN_COLOR: Color = Color {
    r: 1.0,
    g: 0.85,
    b: 0.0,
    a: 1.0,
};
pub const CAMERA_PAN_SPEED: f32 = 300.0;
pub const CAMERA_ZOOM_SPEED: f32 = 1.0;
pub const ZOOM_MIN: f32 = 0.1;

pub mod action {
    pub const MOVE_RIGHT: &str = "move_right";
    pub const MOVE_LEFT: &str = "move_left";
    pub const MOVE_UP: &str = "move_up";
    pub const MOVE_DOWN: &str = "move_down";
    pub const ZOOM_IN: &str = "zoom_in";
    pub const ZOOM_OUT: &str = "zoom_out";
}
// EVOLVE-BLOCK-END
