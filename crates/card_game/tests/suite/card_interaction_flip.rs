#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::*;
use engine_core::prelude::TextureId;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::{Affine2, Vec2};

use card_game::card::component::{Card, CardZone};
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::interaction::flip::card_flip_system;
use card_game::card::interaction::flip_animation::{FLIP_DURATION, FlipAnimation};
use card_game::test_helpers::make_test_card;

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
                signature: card_game::card::identity::signature::CardSignature::default(),
            },
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(pos)),
            SortOrder::new(sort),
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

/// @doc: Flip animation targets toggled state—animates to opposite of current `face_up` value
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

/// @doc: Hand cards immune to flip—only table cards can be flipped with right-click
#[test]
fn when_right_click_on_hand_card_then_no_animation_inserted() {
    // Arrange
    let mut world = World::new();
    let card = world
        .spawn((
            make_test_card(),
            CardZone::Hand(0),
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ))
        .id();
    setup_mouse_right_click(&mut world, Vec2::ZERO);
    world.insert_resource(DragState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.entity(card).get::<FlipAnimation>().is_none());
}

/// @doc: Right-click ignored during drag—prevents flip intent while manipulating cards
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
            make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
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
