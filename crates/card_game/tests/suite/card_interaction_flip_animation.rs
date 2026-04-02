#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::{DeltaTime, Seconds, TextureId, Transform2D};
use glam::Vec2;

use card_game::card::component::Card;
use card_game::card::identity::signature::CardSignature;
use card_game::card::interaction::flip_animation::{FlipAnimation, flip_animation_system};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(flip_animation_system);
    schedule.run(world);
}

fn default_card(face_up: bool) -> Card {
    Card {
        face_texture: TextureId(1),
        back_texture: TextureId(2),
        face_up,
        signature: CardSignature::default(),
    }
}

#[test]
fn when_no_entities_have_flip_animation_then_system_runs_without_panic() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.016)));

    // Act
    run_system(&mut world);
}

/// @doc: Progress accumulates as dt / duration—controls animation playback speed independent of framerate
#[test]
fn when_animation_advances_then_progress_increases_by_dt_over_duration() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.1)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(0.4),
                progress: 0.0,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let anim = world.entity(entity).get::<FlipAnimation>().unwrap();
    assert!((anim.progress - 0.25).abs() < 1e-5);
}

/// @doc: First half of flip shrinks scale.x toward zero—card appears to turn away from camera
#[test]
fn when_animation_in_first_half_then_scale_x_shrinks_proportionally() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.0)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.25,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let scale_x = world.entity(entity).get::<Transform2D>().unwrap().scale.x;
    assert!((scale_x - 0.5).abs() < 1e-5);
}

/// @doc: Midpoint shrinkage = 0—card is edge-on at flip transition for hidden face swap
#[test]
fn when_animation_at_midpoint_then_scale_x_is_zero() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.0)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.5,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let scale_x = world.entity(entity).get::<Transform2D>().unwrap().scale.x;
    assert!(scale_x.abs() < 1e-5, "scale.x should be 0.0 at midpoint");
}

/// @doc: Second half of flip expands scale.x from zero—card appears to turn toward camera with new face
#[test]
fn when_animation_in_second_half_then_scale_x_grows_proportionally() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.0)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.75,
                target_face_up: true,
            },
            default_card(true),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let scale_x = world.entity(entity).get::<Transform2D>().unwrap().scale.x;
    assert!((scale_x - 0.5).abs() < 1e-5);
}

/// @doc: Face swap happens at midpoint—ensures card texture matches shrunk edge state
#[test]
fn when_animation_crosses_midpoint_then_face_up_toggled_to_target() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.05)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.49,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.entity(entity).get::<Card>().unwrap().face_up);
}

#[test]
fn when_face_up_already_matches_target_past_midpoint_then_face_up_unchanged() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.05)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.6,
                target_face_up: true,
            },
            default_card(true),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    assert!(world.entity(entity).get::<Card>().unwrap().face_up);
}

#[test]
fn when_face_up_card_flips_through_midpoint_then_face_up_becomes_false() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.05)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.49,
                target_face_up: false,
            },
            default_card(true),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(!world.entity(entity).get::<Card>().unwrap().face_up);
}

/// @doc: `FlipAnimation` removed at completion—prevent re-triggering same flip state twice
#[test]
fn when_animation_completes_then_component_removed() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.1)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.95,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.entity(entity).get::<FlipAnimation>().is_none());
}

/// @doc: Scale.x reset to normal after animation—prevents lingering visual shrinkage
#[test]
fn when_animation_completes_then_scale_x_restored_to_one() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.1)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.95,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::new(0.1, 1.0),
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let scale_x = world.entity(entity).get::<Transform2D>().unwrap().scale.x;
    assert!((scale_x - 1.0).abs() < 1e-5);
}

#[test]
fn when_animation_completes_then_face_up_set_to_target() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.1)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.95,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.entity(entity).get::<Card>().unwrap().face_up);
}

#[test]
fn when_animation_completes_then_face_up_matches_target_and_scale_restored() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.1)));
    let root = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.95,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::new(0.1, 1.0),
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.entity(root).get::<FlipAnimation>().is_none());
    let scale_x = world.entity(root).get::<Transform2D>().unwrap().scale.x;
    assert!((scale_x - 1.0).abs() < 1e-5);
    assert!(world.entity(root).get::<Card>().unwrap().face_up);
}

/// @doc: Midpoint boundary condition >= 0.5—ensures swap happens at exact midpoint
#[test]
fn when_progress_exactly_at_midpoint_then_face_up_changes_to_target() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.0)));
    let entity = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.5,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    assert!(world.entity(entity).get::<Card>().unwrap().face_up);
}

#[test]
fn when_multiple_cards_animating_then_each_progresses_independently() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.1)));
    let card_a = world
        .spawn((
            FlipAnimation {
                duration: Seconds(1.0),
                progress: 0.1,
                target_face_up: true,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let card_b = world
        .spawn((
            FlipAnimation {
                duration: Seconds(0.5),
                progress: 0.6,
                target_face_up: false,
            },
            default_card(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let anim_a = world.entity(card_a).get::<FlipAnimation>().unwrap();
    assert!((anim_a.progress - 0.2).abs() < 1e-5);
    let anim_b = world.entity(card_b).get::<FlipAnimation>().unwrap();
    assert!((anim_b.progress - 0.8).abs() < 1e-5);

    let scale_a = world.entity(card_a).get::<Transform2D>().unwrap().scale.x;
    assert!((scale_a - 0.6).abs() < 1e-5);
    let scale_b = world.entity(card_b).get::<Transform2D>().unwrap().scale.x;
    assert!((scale_b - 0.6).abs() < 1e-5);
}
