use bevy_ecs::prelude::{Query, Res};
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;

use crate::hand::Hand;
use crate::spawn_table_card::CARD_WIDTH;

pub const HAND_GAP: f32 = 8.0;
pub const HAND_BOTTOM_MARGIN: f32 = 60.0;

pub fn hand_layout_system(
    hand: Res<Hand>,
    camera_query: Query<&Camera2D>,
    renderer: Res<RendererRes>,
    mut cards: Query<&mut Transform2D>,
) {
    if hand.is_empty() {
        return;
    }

    let (vw, vh) = renderer.viewport_size();
    if vw == 0 || vh == 0 {
        return;
    }
    let vw = vw as f32;
    let vh = vh as f32;

    let camera = camera_query
        .iter()
        .next()
        .copied()
        .unwrap_or(Camera2D::default());

    let n = hand.len() as f32;
    let total_width = n * CARD_WIDTH + (n - 1.0) * HAND_GAP;
    let screen_y = vh - HAND_BOTTOM_MARGIN;

    for (i, &entity) in hand.cards().iter().enumerate() {
        let screen_x =
            (vw - total_width) * 0.5 + i as f32 * (CARD_WIDTH + HAND_GAP) + CARD_WIDTH * 0.5;
        let world_pos = screen_to_world(Vec2::new(screen_x, screen_y), &camera, vw, vh);

        if let Ok(mut transform) = cards.get_mut(entity) {
            transform.position = world_pos;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Entity, Schedule, World};
    use engine_render::testing::SpyRenderer;
    use std::sync::{Arc, Mutex};

    fn make_world(viewport_w: u32, viewport_h: u32) -> World {
        let mut world = World::new();
        world.insert_resource(Hand::new(10));

        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(viewport_w, viewport_h);
        world.insert_resource(RendererRes::new(Box::new(spy)));

        world
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(hand_layout_system);
        schedule.run(world);
    }

    #[test]
    fn when_hand_is_empty_then_no_transform_is_mutated() {
        // Arrange
        let mut world = make_world(800, 600);
        let sentinel = Vec2::new(999.0, 999.0);
        let entity = world
            .spawn(Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.position, sentinel);
    }

    fn add_card_to_hand(world: &mut World) -> Entity {
        let entity = world
            .spawn(Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();
        entity
    }

    #[test]
    fn when_one_card_in_hand_then_card_is_horizontally_centered() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.position.x - 0.0).abs() < 1e-3,
            "expected x≈0.0, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_one_card_in_hand_then_card_y_is_margin_above_bottom() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert — screen_to_world(_, 600 - 60, _, 800, 600) → y = (540 - 300) = 240
        let expected_y = 300.0 - HAND_BOTTOM_MARGIN;
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.position.y - expected_y).abs() < 1e-3,
            "expected y≈{expected_y}, got {}",
            t.position.y
        );
    }

    #[test]
    fn when_two_cards_then_first_is_left_of_second() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        assert!(ax < bx, "expected a.x < b.x, got {ax} vs {bx}");
    }

    #[test]
    fn when_two_cards_then_gap_equals_card_width_plus_hand_gap() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        let expected = CARD_WIDTH + HAND_GAP;
        assert!(
            (bx - ax - expected).abs() < 1e-3,
            "expected gap≈{expected}, got {}",
            bx - ax
        );
    }

    #[test]
    fn when_viewport_width_is_zero_then_card_transform_is_not_mutated() {
        // Arrange
        let mut world = make_world(0, 768);
        let sentinel = Vec2::new(42.0, 42.0);
        let entity = world
            .spawn(Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.position, sentinel);
    }

    #[test]
    fn when_viewport_height_is_zero_then_card_transform_is_not_mutated() {
        // Arrange
        let mut world = make_world(800, 0);
        let sentinel = Vec2::new(42.0, 42.0);
        let entity = world
            .spawn(Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.position, sentinel);
    }

    #[test]
    fn when_two_cards_then_row_centered_around_screen_center() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        let midpoint = (ax + bx) * 0.5;
        assert!(
            (midpoint - 0.0).abs() < 1e-3,
            "expected midpoint≈0.0, got {midpoint}"
        );
    }

    #[test]
    fn when_three_cards_then_spacing_is_uniform() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);
        let c = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        let cx = world.get::<Transform2D>(c).unwrap().position.x;
        let gap_ab = bx - ax;
        let gap_bc = cx - bx;
        assert!(
            (gap_ab - gap_bc).abs() < 1e-3,
            "expected uniform gap, got ab={gap_ab} bc={gap_bc}"
        );
    }

    #[test]
    fn when_no_camera_then_uses_default_camera() {
        // Arrange — no Camera2D entity spawned
        let mut world = make_world(800, 600);
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert — same result as with Camera2D::default() (pos=ZERO, zoom=1)
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.position.x - 0.0).abs() < 1e-3,
            "expected x≈0.0, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_camera_offset_then_positions_shift() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(100.0, 50.0),
            zoom: 1.0,
        });
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert — screen center maps to camera position
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.position.x - 100.0).abs() < 1e-3,
            "expected x≈100.0, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_camera_zoom_two_then_world_gap_halved() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 2.0,
        });
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        let expected = f32::midpoint(CARD_WIDTH, HAND_GAP);
        assert!(
            (bx - ax - expected).abs() < 1e-3,
            "expected gap≈{expected}, got {}",
            bx - ax
        );
    }

    #[test]
    fn when_multiple_cards_then_all_share_same_y() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);
        let c = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ay = world.get::<Transform2D>(a).unwrap().position.y;
        let by = world.get::<Transform2D>(b).unwrap().position.y;
        let cy = world.get::<Transform2D>(c).unwrap().position.y;
        assert!((ay - by).abs() < 1e-3, "a.y={ay} != b.y={by}");
        assert!((by - cy).abs() < 1e-3, "b.y={by} != c.y={cy}");
    }

    #[test]
    fn when_entity_has_no_transform_then_no_panic() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = world.spawn_empty().id();
        world.resource_mut::<Hand>().add(entity).unwrap();

        // Act — should not panic
        run_system(&mut world);
    }
}
