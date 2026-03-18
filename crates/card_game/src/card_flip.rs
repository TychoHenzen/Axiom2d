use bevy_ecs::prelude::{Commands, Entity, Has, Query, Res};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};

use crate::card::Card;
use crate::card_pick::{collider_half_extents, local_space_hit};
use crate::card_zone::CardZone;
use crate::drag_state::DragState;
use crate::flip_animation::FlipAnimation;

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
    use crate::card::Card;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};
    use crate::flip_animation::{FLIP_DURATION, FlipAnimation};

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
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.entity(card).get::<FlipAnimation>().is_none());
    }

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
    fn when_flip_then_visibility_sync_does_not_change_before_animation_completes() {
        use crate::card_face_side::CardFaceSide;
        use crate::card_face_visibility::card_face_visibility_sync_system;
        use engine_scene::prelude::{ChildOf, Children, Visible};

        // Arrange — card face-down: front hidden, back visible
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

        // Act — flip system runs, then visibility sync runs
        let mut schedule = Schedule::default();
        schedule.add_systems((card_flip_system, card_face_visibility_sync_system).chain());
        schedule.run(&mut world);

        // Assert — face_up unchanged, so visibility stays as-is
        assert!(
            !world.entity(root).get::<Card>().unwrap().face_up,
            "face_up must not change until animation completes"
        );
        assert!(
            !world.entity(front).get::<Visible>().unwrap().0,
            "front stays hidden while face_up is still false"
        );
        assert!(
            world.entity(back).get::<Visible>().unwrap().0,
            "back stays visible while face_up is still false"
        );
        // Animation was inserted
        assert!(world.entity(root).get::<FlipAnimation>().is_some());
    }
}
