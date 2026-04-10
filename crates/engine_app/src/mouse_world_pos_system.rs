// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_input::mouse::MouseState;
use engine_render::camera::{Camera2D, screen_to_world};

use crate::window_size::WindowSize;

pub fn mouse_world_pos_system(
    window_size: Res<WindowSize>,
    camera_query: Query<&Camera2D>,
    mut mouse: ResMut<MouseState>,
) {
    let camera = camera_query.iter().next().copied().unwrap_or_default();
    let world_pos = screen_to_world(
        mouse.screen_pos(),
        &camera,
        window_size.width.0,
        window_size.height.0,
    );
    mouse.set_world_pos(world_pos);
}
// EVOLVE-BLOCK-END
