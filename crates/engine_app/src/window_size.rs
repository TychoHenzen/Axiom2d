// EVOLVE-BLOCK-START
use bevy_ecs::prelude::Resource;
use engine_core::types::Pixels;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct WindowSize {
    pub width: Pixels,
    pub height: Pixels,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: Pixels(0.0),
            height: Pixels(0.0),
        }
    }
}
// EVOLVE-BLOCK-END
