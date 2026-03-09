use bevy_ecs::prelude::Component;

use crate::types::Pixels;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: Pixels,
    pub y: Pixels,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    pub dx: Pixels,
    pub dy: Pixels,
}
