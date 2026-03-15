use bevy_ecs::prelude::{Entity, Query, Res, ResMut};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::Vec2;

use crate::card::Card;
use crate::card_zone::CardZone;
use crate::drag_state::{DragInfo, DragState};

fn collider_half_extents(collider: &Collider) -> Option<Vec2> {
    match collider {
        Collider::Aabb(half) => Some(*half),
        _ => None,
    }
}

fn local_space_hit(cursor_local: Vec2, half: Vec2) -> bool {
    cursor_local.x.abs() <= half.x && cursor_local.y.abs() <= half.y
}

pub fn card_pick_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &mut SortOrder,
    )>,
) {
    if drag_state.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let cursor = mouse.world_pos();

    let max_sort = query
        .iter()
        .filter(|(_, _, zone, _, _, _)| **zone == CardZone::Table)
        .map(|(_, _, _, _, _, sort)| sort.0)
        .max()
        .unwrap_or(0);

    let best = query
        .iter()
        .filter(|(_, _, zone, _, _, _)| **zone == CardZone::Table)
        .filter(|(_, _, _, transform, collider, _)| {
            let Some(half) = collider_half_extents(collider) else {
                return false;
            };
            let cursor_local = transform.0.inverse().transform_point2(cursor);
            local_space_hit(cursor_local, half)
        })
        .max_by_key(|(_, _, _, _, _, sort)| sort.0)
        .map(|(entity, _, zone, transform, _, _)| {
            let cursor_delta = cursor - transform.0.translation;
            let local_grab_offset = transform.0.matrix2.inverse().mul_vec2(cursor_delta);
            (entity, *zone, local_grab_offset)
        });

    if let Some((entity, zone, local_grab_offset)) = best {
        drag_state.dragging = Some(DragInfo {
            entity,
            local_grab_offset,
            origin_zone: zone,
        });
        if let Ok((_, _, _, _, _, mut sort)) = query.get_mut(entity) {
            sort.0 = max_sort + 1;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::TextureId;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::Collider;
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::{Affine2, Vec2};

    use super::card_pick_system;
    use crate::card::Card;
    use crate::card_zone::CardZone;
    use crate::drag_state::DragState;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_pick_system);
        schedule.run(world);
    }

    fn default_collider() -> Collider {
        Collider::Aabb(Vec2::new(30.0, 45.0))
    }

    #[test]
    fn when_left_click_on_single_table_card_then_drag_state_contains_that_entity() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.entity), Some(card_entity));
    }

    #[test]
    fn when_left_click_at_card_center_then_drag_state_is_some() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(3), TextureId(4)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_left_click_on_table_card_then_drag_info_records_table_origin_zone() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(5), TextureId(6)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.origin_zone), Some(CardZone::Table));
    }

    #[test]
    fn when_left_click_outside_all_cards_then_drag_state_remains_none() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(200.0, 200.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_left_click_with_no_table_cards_then_drag_state_remains_none() {
        // Arrange
        let mut world = World::new();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_two_overlapping_cards_then_picks_highest_sort_order() {
        // Arrange
        let mut world = World::new();
        let _card_a = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        let card_b = world
            .spawn((
                Card::face_down(TextureId(3), TextureId(4)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(5),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.entity), Some(card_b));
    }

    #[test]
    fn when_card_picked_then_sort_order_bumped_above_all_others() {
        // Arrange
        let mut world = World::new();
        let card_a = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        world.spawn((
            Card::face_down(TextureId(3), TextureId(4)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 0.0))),
            SortOrder(7),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let picked_sort = world.entity(card_a).get::<SortOrder>().unwrap().0;
        assert!(
            picked_sort > 7,
            "picked card sort {picked_sort} should be > 7"
        );
    }

    #[test]
    fn when_already_dragging_then_new_click_does_not_replace_drag() {
        // Arrange
        let mut world = World::new();
        let card_a = world.spawn_empty().id();
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState {
            dragging: Some(crate::drag_state::DragInfo {
                entity: card_a,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert_eq!(drag.entity, card_a);
    }

    #[test]
    fn when_card_picked_at_offset_then_local_grab_offset_is_inverse_rotated() {
        // Arrange
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        let transform =
            Affine2::from_scale_angle_translation(Vec2::ONE, angle, Vec2::new(100.0, 50.0));
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(transform),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(110.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let offset = world
            .resource::<DragState>()
            .dragging
            .unwrap()
            .local_grab_offset;
        let expected_x = 10.0_f32 * angle.cos();
        let expected_y = -10.0_f32 * angle.sin();
        assert!(
            (offset.x - expected_x).abs() < 1e-4,
            "offset.x={} expected ~{expected_x}",
            offset.x
        );
        assert!(
            (offset.y - expected_y).abs() < 1e-4,
            "offset.y={} expected ~{expected_y}",
            offset.y
        );
    }

    #[test]
    fn when_card_picked_at_center_then_local_grab_offset_is_zero() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let offset = world
            .resource::<DragState>()
            .dragging
            .unwrap()
            .local_grab_offset;
        assert!(
            offset.length() < 1e-6,
            "offset should be ~zero, got {offset}"
        );
    }

    #[test]
    fn when_cursor_on_edge_of_card_then_card_is_picked() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(30.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_rotated_card_clicked_inside_obb_then_picked() {
        // Arrange — card at origin, rotated 45 degrees, half extents (30, 45)
        // Click at (20, 20) — inside the rotated rectangle but outside the AABB
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(20.0, 20.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_rotated_card_clicked_outside_obb_then_not_picked() {
        // Arrange — card at origin, rotated 45 degrees, half extents (30, 45)
        // Click at (50, 0) — outside the rotated rectangle
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(50.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }
}
