// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Query, Res, ResMut, Resource};
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::prelude::Camera2D;
use glam::Vec2;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct CameraDragState {
    pub anchor_screen_pos: Option<Vec2>,
}

const ZOOM_SPEED: f32 = 0.1;
pub const ZOOM_MIN: f32 = 0.1;

pub fn camera_drag_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<CameraDragState>,
    mut query: Query<&mut Camera2D>,
) {
    if mouse.just_released(MouseButton::Right) {
        drag_state.anchor_screen_pos = None;
        return;
    }

    if mouse.just_pressed(MouseButton::Right) {
        drag_state.anchor_screen_pos = Some(mouse.screen_pos());
        return;
    }

    if mouse.pressed(MouseButton::Right)
        && let Some(anchor) = drag_state.anchor_screen_pos
    {
        let delta = mouse.screen_pos() - anchor;
        if let Ok(mut camera) = query.single_mut() {
            let zoom = camera.zoom;
            camera.position -= delta / zoom;
        }
        drag_state.anchor_screen_pos = Some(mouse.screen_pos());
    }
}

pub fn camera_zoom_system(mouse: Res<MouseState>, mut query: Query<&mut Camera2D>) {
    let scroll = mouse.scroll_delta().y;
    if scroll == 0.0 {
        return;
    }
    if let Ok(mut camera) = query.single_mut() {
        camera.zoom = (camera.zoom + ZOOM_SPEED * scroll).max(ZOOM_MIN);
    }
}
// EVOLVE-BLOCK-END
