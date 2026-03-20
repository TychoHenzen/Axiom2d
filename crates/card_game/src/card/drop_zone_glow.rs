use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_render::prelude::{
    Camera2D, QUAD_INDICES, RendererRes, UNIT_QUAD, resolve_viewport_camera, screen_to_world,
    unit_quad_model,
};

use glam::Vec2;

use crate::card::drag_state::DragState;

pub(crate) const HAND_DROP_ZONE_HEIGHT: f32 = 120.0;

pub(crate) const HAND_ZONE_GLOW_COLOR: Color = Color {
    r: 0.3,
    g: 0.5,
    b: 0.8,
    a: 0.25,
};

pub fn hand_drop_zone_render_system(
    drag_state: Res<DragState>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    if drag_state.dragging.is_none() {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let top_left = screen_to_world(Vec2::new(0.0, vh - HAND_DROP_ZONE_HEIGHT), &camera, vw, vh);
    let bottom_right = screen_to_world(Vec2::new(vw, vh), &camera, vw, vh);
    let width = bottom_right.x - top_left.x;
    let height = bottom_right.y - top_left.y;
    let cx = (top_left.x + bottom_right.x) * 0.5;
    let cy = (top_left.y + bottom_right.y) * 0.5;

    let model = unit_quad_model(width, height, cx, cy);
    renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, HAND_ZONE_GLOW_COLOR, model);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_render::prelude::{Camera2D, RendererRes};
    use engine_render::testing::{ShapeCallLog, SpyRenderer};
    use glam::Vec2;

    use super::*;
    use crate::card::drag_state::{DragInfo, DragState};
    use crate::card::zone::CardZone;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(hand_drop_zone_render_system);
        schedule.run(world);
    }

    fn make_world_with_spy(viewport_w: u32, viewport_h: u32) -> (World, ShapeCallLog) {
        let mut world = World::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(viewport_w, viewport_h);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });
        (world, shape_calls)
    }

    fn make_drag_state(world: &mut World) {
        let entity = world.spawn_empty().id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
                origin_position: Vec2::ZERO,
            }),
        });
    }

    #[test]
    fn when_drag_active_then_glow_rect_drawn() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(800, 600);
        make_drag_state(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "exactly one shape call for glow rect");
        assert!(
            calls[0].2.a < 1.0,
            "glow rect should be translucent, got alpha={}",
            calls[0].2.a
        );
    }

    #[test]
    fn when_no_drag_then_no_glow_rect_drawn() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(800, 600);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn when_viewport_zero_then_no_glow_rect_and_no_panic() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(0, 0);
        make_drag_state(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }
}
