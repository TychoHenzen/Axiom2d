use bevy_ecs::prelude::{Entity, Query, Res};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};

use crate::card::Card;
use crate::card_pick::{collider_half_extents, local_space_hit};
use crate::card_zone::CardZone;
use crate::drag_state::DragState;

pub fn card_flip_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    mut query: Query<(
        Entity,
        &mut Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) {
    if drag_state.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let cursor = mouse.world_pos();

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
        .map(|(entity, _, _, _, _, _)| entity);

    if let Some(entity) = best
        && let Ok((_, mut card, _, _, _, _)) = query.get_mut(entity)
    {
        card.face_up = !card.face_up;
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

    use super::card_flip_system;
    use crate::card::Card;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_flip_system);
        schedule.run(world);
    }

    fn default_collider() -> Collider {
        Collider::Aabb(Vec2::new(30.0, 45.0))
    }

    fn spawn_table_card_at(world: &mut World, pos: Vec2, face_up: bool, sort: i32) -> Entity {
        world
            .spawn((
                Card {
                    face_texture: TextureId(1),
                    back_texture: TextureId(2),
                    face_up,
                },
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(pos)),
                SortOrder(sort),
            ))
            .id()
    }

    fn setup_mouse_right_click(world: &mut World, pos: Vec2) {
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Right);
        mouse.set_world_pos(pos);
        world.insert_resource(mouse);
    }

    #[test]
    fn when_right_click_hits_table_card_then_face_up_toggled() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.entity(card).get::<Card>().unwrap().face_up);
    }

    #[test]
    fn when_right_click_hits_face_up_card_then_face_up_becomes_false() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, true, 0);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.entity(card).get::<Card>().unwrap().face_up);
    }

    #[test]
    fn when_right_click_misses_all_cards_then_no_change() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        setup_mouse_right_click(&mut world, Vec2::new(200.0, 200.0));
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.entity(card).get::<Card>().unwrap().face_up);
    }

    #[test]
    fn when_right_click_on_hand_card_then_no_flip() {
        // Arrange
        let mut world = World::new();
        let card = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.entity(card).get::<Card>().unwrap().face_up);
    }

    #[test]
    fn when_right_click_during_drag_then_no_flip() {
        // Arrange
        let mut world = World::new();
        let dummy = world.spawn_empty().id();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity: dummy,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.entity(card).get::<Card>().unwrap().face_up);
    }

    #[test]
    fn when_right_click_overlapping_cards_then_only_topmost_flipped() {
        // Arrange
        let mut world = World::new();
        let card_a = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        let card_b = spawn_table_card_at(&mut world, Vec2::ZERO, false, 5);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.entity(card_a).get::<Card>().unwrap().face_up);
        assert!(world.entity(card_b).get::<Card>().unwrap().face_up);
    }

    #[test]
    fn when_flip_then_visibility_sync_updates_children() {
        use crate::card_face_side::CardFaceSide;
        use crate::card_face_visibility::card_face_visibility_sync_system;
        use engine_scene::prelude::{ChildOf, Children, Visible};

        // Arrange — card face_down with front hidden, back visible
        let mut world = World::new();
        let root = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        let front = world
            .spawn((ChildOf(root), CardFaceSide::Front, Visible(false)))
            .id();
        let back = world
            .spawn((ChildOf(root), CardFaceSide::Back, Visible(true)))
            .id();
        let mut children = vec![front, back];
        children.sort();
        world.entity_mut(root).insert(Children(children));
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act — flip, then sync visibility
        let mut schedule = Schedule::default();
        schedule.add_systems((card_flip_system, card_face_visibility_sync_system).chain());
        schedule.run(&mut world);

        // Assert — face_up is now true, so front visible, back hidden
        assert!(world.entity(root).get::<Card>().unwrap().face_up);
        assert!(world.entity(front).get::<Visible>().unwrap().0);
        assert!(!world.entity(back).get::<Visible>().unwrap().0);
    }
}
