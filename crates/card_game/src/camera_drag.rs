use bevy_ecs::prelude::{Query, Res, ResMut, Resource};
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::prelude::Camera2D;
use glam::Vec2;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct CameraDragState {
    pub anchor_screen_pos: Option<Vec2>,
}

pub const ZOOM_SPEED: f32 = 0.1;
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

    if mouse.pressed(MouseButton::Right) {
        if let Some(anchor) = drag_state.anchor_screen_pos {
            let delta = mouse.screen_pos() - anchor;
            if let Ok(mut camera) = query.single_mut() {
                let zoom = camera.zoom;
                camera.position -= delta / zoom;
            }
            drag_state.anchor_screen_pos = Some(mouse.screen_pos());
        }
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_render::prelude::Camera2D;
    use glam::Vec2;

    use super::{CameraDragState, ZOOM_MIN, camera_drag_system, camera_zoom_system};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(camera_drag_system);
        schedule.run(world);
    }

    #[test]
    fn when_rmb_not_pressed_then_drag_state_anchor_remains_none() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        world.insert_resource(MouseState::default());
        world.insert_resource(CameraDragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.resource::<CameraDragState>().anchor_screen_pos, None);
    }

    #[test]
    fn when_rmb_just_pressed_then_drag_state_anchor_set_to_screen_pos() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.set_screen_pos(Vec2::new(100.0, 200.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<CameraDragState>().anchor_screen_pos,
            Some(Vec2::new(100.0, 200.0)),
        );
    }

    #[test]
    fn when_rmb_held_and_mouse_moved_then_camera_moves_inversely() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.clear_frame_state();
        mouse.set_screen_pos(Vec2::new(110.0, 220.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
        });

        // Act
        run_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert_eq!(camera.position, Vec2::new(-10.0, -20.0));
    }

    #[test]
    fn when_rmb_just_pressed_then_camera_position_unchanged() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D {
            position: Vec2::new(50.0, 50.0),
            zoom: 1.0,
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.set_screen_pos(Vec2::new(150.0, 150.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert_eq!(camera.position, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn when_rmb_held_then_anchor_updated_to_current_screen_pos() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.clear_frame_state();
        mouse.set_screen_pos(Vec2::new(110.0, 220.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<CameraDragState>().anchor_screen_pos,
            Some(Vec2::new(110.0, 220.0)),
        );
    }

    #[test]
    fn when_zoomed_in_then_same_screen_drag_moves_camera_less() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 2.0,
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.clear_frame_state();
        mouse.set_screen_pos(Vec2::new(110.0, 100.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 100.0)),
        });

        // Act
        run_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert_eq!(camera.position.x, -5.0);
    }

    #[test]
    fn when_rmb_released_then_drag_state_becomes_none() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D {
            position: Vec2::new(50.0, 50.0),
            zoom: 1.0,
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.release(MouseButton::Right);
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.resource::<CameraDragState>().anchor_screen_pos, None);
    }

    #[test]
    fn when_rmb_released_then_camera_position_unchanged() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D {
            position: Vec2::new(50.0, 50.0),
            zoom: 1.0,
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.release(MouseButton::Right);
        mouse.set_screen_pos(Vec2::new(110.0, 210.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
        });

        // Act
        run_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert_eq!(camera.position, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn when_no_camera_entity_then_drag_system_does_not_panic() {
        // Arrange
        let mut world = World::new();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.clear_frame_state();
        mouse.set_screen_pos(Vec2::new(110.0, 110.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 100.0)),
        });

        // Act + Assert (no panic)
        run_system(&mut world);
    }

    #[test]
    fn when_card_drag_active_then_rmb_pan_still_works() {
        // Arrange
        use crate::drag_state::{DragInfo, DragState};
        use crate::card_zone::CardZone;

        let mut world = World::new();
        world.spawn(Camera2D::default());
        let entity = world.spawn_empty().id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.clear_frame_state();
        mouse.set_screen_pos(Vec2::new(110.0, 100.0));
        world.insert_resource(mouse);
        world.insert_resource(CameraDragState {
            anchor_screen_pos: Some(Vec2::new(100.0, 100.0)),
        });
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert_eq!(camera.position.x, -10.0);
    }

    fn run_zoom_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(camera_zoom_system);
        schedule.run(world);
    }

    #[test]
    fn when_scroll_up_then_zoom_increases() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        let mut mouse = MouseState::default();
        mouse.add_scroll_delta(Vec2::new(0.0, 1.0));
        world.insert_resource(mouse);

        // Act
        run_zoom_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert!(camera.zoom > 1.0);
    }

    #[test]
    fn when_scroll_down_then_zoom_decreases() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        let mut mouse = MouseState::default();
        mouse.add_scroll_delta(Vec2::new(0.0, -1.0));
        world.insert_resource(mouse);

        // Act
        run_zoom_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert!(camera.zoom < 1.0);
    }

    #[test]
    fn when_zoom_at_floor_and_scroll_down_then_zoom_stays_at_minimum() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: ZOOM_MIN,
        });
        let mut mouse = MouseState::default();
        mouse.add_scroll_delta(Vec2::new(0.0, -5.0));
        world.insert_resource(mouse);

        // Act
        run_zoom_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert!(camera.zoom >= ZOOM_MIN);
    }

    #[test]
    fn when_zoom_above_floor_and_scroll_down_then_zoom_can_decrease() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D::default());
        let mut mouse = MouseState::default();
        mouse.add_scroll_delta(Vec2::new(0.0, -1.0));
        world.insert_resource(mouse);

        // Act
        run_zoom_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert!(camera.zoom > ZOOM_MIN);
        assert!(camera.zoom < 1.0);
    }

    #[test]
    fn when_zero_scroll_delta_then_zoom_unchanged() {
        // Arrange
        let mut world = World::new();
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.5,
        });
        world.insert_resource(MouseState::default());

        // Act
        run_zoom_system(&mut world);

        // Assert
        let camera = world.query::<&Camera2D>().single(&world).unwrap();
        assert_eq!(camera.zoom, 1.5);
    }

    #[test]
    fn when_no_camera_entity_then_zoom_system_does_not_panic() {
        // Arrange
        let mut world = World::new();
        let mut mouse = MouseState::default();
        mouse.add_scroll_delta(Vec2::new(0.0, 1.0));
        world.insert_resource(mouse);

        // Act + Assert (no panic)
        run_zoom_system(&mut world);
    }
}
