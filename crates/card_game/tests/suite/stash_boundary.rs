#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use engine_input::prelude::MouseState;
use engine_physics::prelude::{Collider, PhysicsBackend, PhysicsRes};
use engine_scene::prelude::GlobalTransform2D;
use glam::{Affine2, Vec2};

use card_game::card::component::CardZone;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::interaction::pick::DRAG_SCALE;
use card_game::card::rendering::geometry::TABLE_CARD_WIDTH as CARD_WIDTH;
use card_game::stash::boundary::stash_boundary_system;
use card_game::stash::constants::SLOT_WIDTH;
use card_game::stash::grid::StashGrid;
use card_game::stash::toggle::StashVisible;
use card_game::test_helpers::{AddBodyLog, RemoveBodyLog, SpyPhysicsBackend};
use engine_core::scale_spring::ScaleSpring;

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_boundary_system);
    schedule.run(world);
}

fn make_spy_physics() -> (
    Box<dyn PhysicsBackend + Send + Sync>,
    AddBodyLog,
    RemoveBodyLog,
) {
    let add_log: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
    let remove_log: RemoveBodyLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyPhysicsBackend::new()
        .with_add_body_log(add_log.clone())
        .with_remove_body_log(remove_log.clone());
    (Box::new(spy), add_log, remove_log)
}

fn make_drag_info(entity: Entity, stash_cursor_follow: bool) -> DragState {
    DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            stash_cursor_follow,
            origin_position: Vec2::ZERO,
        }),
    }
}

fn mouse_at_screen(pos: Vec2) -> MouseState {
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(pos);
    mouse
}

#[test]
fn when_no_drag_then_no_physics_calls() {
    // Arrange
    let mut world = World::new();
    let (spy, add_log, remove_log) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(DragState::default());
    world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5)));
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    assert!(add_log.lock().unwrap().is_empty());
    assert!(remove_log.lock().unwrap().is_empty());
}

#[test]
fn when_stash_follow_and_cursor_in_stash_then_no_physics_calls() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, add_log, remove_log) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, true));
    world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5))); // inside slot (0,0)
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    assert!(add_log.lock().unwrap().is_empty());
    assert!(remove_log.lock().unwrap().is_empty());
    assert!(
        world
            .resource::<DragState>()
            .dragging
            .unwrap()
            .stash_cursor_follow,
        "stash_cursor_follow should remain true"
    );
}

/// @doc: Exiting stash boundary re-adds physics — card transitions from cursor-follow mode back to physics drag
#[test]
fn when_stash_follow_and_cursor_exits_stash_then_physics_body_added() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, add_log, _) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, true));
    world.insert_resource(mouse_at_screen(Vec2::new(800.0, 400.0))); // outside stash
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    let calls = add_log.lock().unwrap();
    assert_eq!(calls.len(), 1, "add_body should be called once");
    assert_eq!(calls[0].0, entity);
}

#[test]
fn when_stash_follow_and_cursor_exits_stash_then_follow_set_false() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, _, _) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, true));
    world.insert_resource(mouse_at_screen(Vec2::new(800.0, 400.0)));
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    let drag = world.resource::<DragState>().dragging.unwrap();
    assert!(
        !drag.stash_cursor_follow,
        "stash_cursor_follow should be false after exit"
    );
}

#[test]
fn when_stash_follow_and_cursor_exits_stash_then_scale_spring_targets_drag_elevation() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, _, _) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, true));
    world.insert_resource(mouse_at_screen(Vec2::new(800.0, 400.0)));
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    let spring = world
        .get::<ScaleSpring>(entity)
        .expect("ScaleSpring should be inserted");
    assert!(
        (spring.target - DRAG_SCALE).abs() < 1e-4,
        "ScaleSpring target should be {DRAG_SCALE} (drag elevation), got {}",
        spring.target
    );
}

/// @doc: Entering stash strips physics — card switches to direct cursor tracking for precise slot placement
#[test]
fn when_physics_drag_and_cursor_enters_stash_then_physics_body_removed() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, _, remove_log) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, false));
    world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5))); // inside slot (0,0)
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    let calls = remove_log.lock().unwrap();
    assert_eq!(calls.len(), 1, "remove_body should be called once");
    assert_eq!(calls[0], entity);
}

#[test]
fn when_physics_drag_and_cursor_enters_stash_then_follow_set_true() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, _, _) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, false));
    world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5)));
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    let drag = world.resource::<DragState>().dragging.unwrap();
    assert!(
        drag.stash_cursor_follow,
        "stash_cursor_follow should be true after entry"
    );
}

#[test]
fn when_physics_drag_and_cursor_enters_stash_then_scale_spring_targets_slot_scale() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, _, _) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, false));
    world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5)));
    world.insert_resource(StashVisible(true));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    let expected = SLOT_WIDTH / CARD_WIDTH;
    let spring = world
        .get::<ScaleSpring>(entity)
        .expect("ScaleSpring should be inserted");
    assert!(
        (spring.target - expected).abs() < 1e-4,
        "ScaleSpring target should be {expected}, got {}",
        spring.target
    );
}

/// @doc: Hiding stash while dragging over it triggers exit transition — card returns to physics mode immediately
#[test]
fn when_stash_hidden_and_follow_true_then_physics_body_added() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            GlobalTransform2D(Affine2::IDENTITY),
            Collider::Aabb(Vec2::new(30.0, 45.0)),
        ))
        .id();
    let (spy, add_log, _) = make_spy_physics();
    world.insert_resource(PhysicsRes::new(spy));
    world.insert_resource(make_drag_info(entity, true));
    world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5))); // would be in stash, but stash hidden
    world.insert_resource(StashVisible(false));
    world.insert_resource(StashGrid::new(10, 10, 1));

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(
        add_log.lock().unwrap().len(),
        1,
        "stash hidden → over_stash=false → exit transition should fire"
    );
}
