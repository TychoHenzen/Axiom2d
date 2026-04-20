#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsRes;
use engine_scene::prelude::GlobalTransform2D;
use engine_ui::draw_command::DrawQueue;
use glam::Affine2;

use card_game::card::component::CardZone;
use card_game::card::rendering::debug_sleep_indicator::debug_sleep_indicator_system;

use crate::test_helpers::{SpyPhysicsBackend, make_test_card};

fn make_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(debug_sleep_indicator_system);
    schedule
}

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(InputState::default());
    world.insert_resource(DrawQueue::default());
    world
}

/// @doc: When F9 toggles the overlay on, sleeping table cards must produce draw
/// commands so the renderer can tint them red. Without visible output, developers
/// have no way to verify the sleep system during gameplay.
#[test]
fn when_debug_key_toggles_on_then_sleeping_cards_produce_draw_commands() {
    // Arrange
    let mut world = setup_world();
    let sleeping_entity = world
        .spawn((
            make_test_card(),
            CardZone::Table,
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(100.0, 200.0))),
        ))
        .id();
    let awake_entity = world
        .spawn((
            make_test_card(),
            CardZone::Table,
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(300.0, 200.0))),
        ))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_body_sleeping(sleeping_entity, true)
        .with_body_sleeping(awake_entity, false);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.resource_mut::<InputState>().press(KeyCode::F9);
    let mut schedule = make_schedule();

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.resource::<DrawQueue>().len(),
        1,
        "only sleeping card should get tint overlay"
    );
}

/// @doc: When F9 is not pressed, no draw commands should be produced. The overlay
/// is opt-in — it must never activate on its own.
#[test]
fn when_debug_key_not_pressed_then_no_draw_commands() {
    // Arrange
    let mut world = setup_world();
    let entity = world
        .spawn((
            make_test_card(),
            CardZone::Table,
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(100.0, 200.0))),
        ))
        .id();
    let spy = SpyPhysicsBackend::new().with_body_sleeping(entity, true);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    let mut schedule = make_schedule();

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.resource::<DrawQueue>().len(),
        0,
        "no commands when overlay disabled"
    );
}

/// @doc: Pressing F9 twice must toggle the overlay off. The second press disables
/// the overlay so developers can return to normal rendering without restarting.
#[test]
fn when_f9_pressed_twice_then_overlay_toggles_off() {
    // Arrange
    let mut world = setup_world();
    let entity = world
        .spawn((
            make_test_card(),
            CardZone::Table,
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(100.0, 200.0))),
        ))
        .id();
    let spy = SpyPhysicsBackend::new().with_body_sleeping(entity, true);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    let mut schedule = make_schedule();

    // First frame: press F9 to enable
    world.resource_mut::<InputState>().press(KeyCode::F9);
    schedule.run(&mut world);
    assert_eq!(
        world.resource::<DrawQueue>().len(),
        1,
        "overlay should be on"
    );

    // Clear frame state, then press F9 again to disable
    world.resource_mut::<InputState>().clear_frame_state();
    world.resource_mut::<InputState>().press(KeyCode::F9);
    schedule.run(&mut world);

    // Queue still has the 1 command from first run (not drained) but no new ones
    assert_eq!(
        world.resource::<DrawQueue>().len(),
        1,
        "no new commands after toggle off — total stays at 1"
    );
}
