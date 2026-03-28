use bevy_ecs::prelude::{Commands, Entity, Has, Query, Res};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::hit_test::{collider_half_extents, local_space_hit};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};

use crate::card::component::Card;
use crate::card::component::CardZone;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::flip_animation::FlipAnimation;

#[allow(clippy::type_complexity)]
pub fn card_flip_system(
    mut commands: Commands,
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
        Has<FlipAnimation>,
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
        .filter(|(_, _, zone, _, _, _, has_anim)| **zone == CardZone::Table && !has_anim)
        .filter(|(_, _, _, transform, collider, _, _)| {
            let Some(half) = collider_half_extents(collider) else {
                return false;
            };
            let cursor_local = transform.0.inverse().transform_point2(cursor);
            local_space_hit(cursor_local, half)
        })
        .max_by_key(|(_, _, _, _, _, sort, _)| sort.0)
        .map(|(entity, card, _, _, _, _, _)| (entity, card.face_up));

    if let Some((entity, face_up)) = best {
        commands
            .entity(entity)
            .insert(FlipAnimation::start(!face_up));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::TextureId;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::Collider;
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::{Affine2, Vec2};

    use super::card_flip_system;
    use crate::card::component::Card;
    use crate::card::component::CardZone;
    use crate::card::interaction::drag_state::{DragInfo, DragState};
    use crate::card::interaction::flip_animation::{FLIP_DURATION, FlipAnimation};

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
                    signature: crate::card::identity::signature::CardSignature::default(),
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

    /// @doc: Flip state unchanged until animation completes—prevents flashing mid-flip
    #[test]
    fn when_flip_triggered_then_face_up_unchanged_and_animation_inserted() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let entity_ref = world.entity(card);
        assert!(
            !entity_ref.get::<Card>().unwrap().face_up,
            "face_up must stay false until the flip animation completes"
        );
        let anim = entity_ref.get::<FlipAnimation>().unwrap();
        assert_eq!(anim.duration, FLIP_DURATION);
        assert_eq!(anim.progress, 0.0);
    }

    #[test]
    fn when_right_click_hits_table_card_then_flip_animation_targets_face_up() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert — animation targets the toggled state, not an immediate write
        let anim = world.entity(card).get::<FlipAnimation>().unwrap();
        assert!(
            anim.target_face_up,
            "flip from face-down should target face-up"
        );
    }

    #[test]
    fn when_right_click_hits_face_up_card_then_flip_animation_targets_face_down() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, true, 0);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let anim = world.entity(card).get::<FlipAnimation>().unwrap();
        assert!(
            !anim.target_face_up,
            "flip from face-up should target face-down"
        );
    }

    #[test]
    fn when_right_click_misses_all_cards_then_no_animation_inserted() {
        // Arrange
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        setup_mouse_right_click(&mut world, Vec2::new(200.0, 200.0));
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.entity(card).get::<FlipAnimation>().is_none());
    }

    #[test]
    fn when_right_click_on_hand_card_then_no_animation_inserted() {
        // Arrange
        let mut world = World::new();
        let card = world
            .spawn((
                crate::test_helpers::make_test_card(),
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
        assert!(world.entity(card).get::<FlipAnimation>().is_none());
    }

    #[test]
    fn when_right_click_during_drag_then_no_animation_inserted() {
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
                stash_cursor_follow: false,
                origin_position: Vec2::ZERO,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.entity(card).get::<FlipAnimation>().is_none());
    }

    /// @doc: Sort order selects highest card at click position—prevents flipping obscured cards
    #[test]
    fn when_right_click_overlapping_cards_then_only_topmost_gets_animation() {
        // Arrange
        let mut world = World::new();
        let card_a = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        let card_b = spawn_table_card_at(&mut world, Vec2::ZERO, false, 5);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.entity(card_a).get::<FlipAnimation>().is_none());
        assert!(world.entity(card_b).get::<FlipAnimation>().is_some());
    }

    /// @doc: Don't interrupt active flip animation—prevents competing flip directions mid-play
    #[test]
    fn when_flip_triggered_while_animation_active_then_animation_unchanged() {
        // Arrange — card already mid-animation
        let mut world = World::new();
        let card = spawn_table_card_at(&mut world, Vec2::ZERO, false, 0);
        let original_anim = FlipAnimation {
            duration: FLIP_DURATION,
            progress: 0.2,
            target_face_up: true,
        };
        world.entity_mut(card).insert(original_anim);
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert — animation unchanged, no restart
        let anim = world.entity(card).get::<FlipAnimation>().unwrap();
        assert_eq!(*anim, original_anim);
    }

    #[test]
    fn when_flip_triggered_then_face_up_unchanged_until_animation_completes() {
        // Arrange — card face-down
        let mut world = World::new();
        let root = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        setup_mouse_right_click(&mut world, Vec2::ZERO);
        world.insert_resource(DragState::default());

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(card_flip_system);
        schedule.run(&mut world);

        // Assert — face_up stays false, animation inserted
        assert!(
            !world.entity(root).get::<Card>().unwrap().face_up,
            "face_up must not change until animation completes"
        );
        assert!(world.entity(root).get::<FlipAnimation>().is_some());
    }
}
