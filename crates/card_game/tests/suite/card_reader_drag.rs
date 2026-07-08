#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_physics::prelude::PhysicsCommand;
use glam::Vec2;

use card_game::card::interaction::drag_state::DeviceDragInfo;
use card_game::card::reader::{CardReader, ReaderDragState, reader_drag_system};

fn run_drag(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(reader_drag_system);
    schedule.run(world);
}

/// @doc: When a reader with a loaded card is dragged, the loaded card's transform
/// must follow the reader's position exactly. The `reader.loaded` field must remain
/// set — card ejection is handled by the eject system, not the drag system.
/// Without this, dragging a reader would orphan its loaded card visually.
#[test]
fn when_card_in_reader_and_drag_begins_then_reader_loaded_cleared() {
    // Arrange
    let mut world = World::new();
    let initial_pos = Vec2::new(100.0, 100.0);
    let target_pos = Vec2::new(250.0, 180.0);

    let jack_entity = world
        .spawn(Transform2D {
            position: initial_pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let card_entity = world
        .spawn(Transform2D {
            position: initial_pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let reader_entity = world
        .spawn((
            CardReader {
                loaded: Some(card_entity),
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: initial_pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(target_pos);
    world.insert_resource(mouse);

    world.insert_resource(ReaderDragState {
        dragging: Some(DeviceDragInfo {
            entity: reader_entity,
            grab_offset: Vec2::ZERO,
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_drag(&mut world);

    // Assert — reader still holds the card
    let reader = world.get::<CardReader>(reader_entity).unwrap();
    assert!(
        reader.loaded.is_some(),
        "reader.loaded must remain set during reader drag"
    );
    assert_eq!(
        reader.loaded,
        Some(card_entity),
        "reader.loaded must still reference the loaded card"
    );

    // Assert — reader position updated
    let reader_pos = world.get::<Transform2D>(reader_entity).unwrap().position;
    assert_eq!(
        reader_pos, target_pos,
        "reader should move to target position {target_pos}, got {reader_pos}"
    );

    // Assert — loaded card follows reader
    let card_pos = world.get::<Transform2D>(card_entity).unwrap().position;
    assert_eq!(
        card_pos, target_pos,
        "loaded card should follow reader to {target_pos}, got {card_pos}"
    );
}

/// @doc: The drag system must be a no-op when no drag is active
/// (`ReaderDragState.dragging` is None). Without this early-return guard,
/// an empty drag state could trigger panics when the system tries to
/// dereference `reader_drag.dragging` as `Some`.
#[test]
fn when_no_drag_active_then_system_noop() {
    // Arrange
    let mut world = World::new();
    let initial_pos = Vec2::new(100.0, 100.0);

    let jack_entity = world
        .spawn(Transform2D {
            position: initial_pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let reader = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: initial_pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    world.insert_resource(ReaderDragState::default());
    world.insert_resource(MouseState::default());
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_drag(&mut world);

    // Assert — nothing changed
    let transform = world.get::<Transform2D>(reader).unwrap();
    assert_eq!(
        transform.position,
        initial_pos,
        "reader position must not change when no drag is active"
    );
    assert!(
        world.resource::<EventBus<PhysicsCommand>>().is_empty(),
        "no physics commands must be emitted when no drag is active"
    );
}
