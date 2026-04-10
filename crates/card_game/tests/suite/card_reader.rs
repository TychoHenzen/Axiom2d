#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::EventBus;
use engine_core::prelude::Transform2D;
use engine_core::scale_spring::ScaleSpring;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_physics::prelude::{PhysicsCommand, RigidBody};
use glam::Vec2;

use card_game::card::component::{Card, CardZone};
use card_game::card::identity::signature::CardSignature;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::interaction::pick::{DRAGGED_COLLISION_FILTER, DRAGGED_COLLISION_GROUP};
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::{
    CardReader, READER_CARD_SCALE, ReaderDragInfo, ReaderDragState, SIGNATURE_SPACE_RADIUS,
    SignatureSpace, card_overlaps_reader, card_reader_eject_system, card_reader_insert_system,
    on_reader_clicked, reader_drag_system, reader_release_system, reader_rotation_lock_system,
    signature_radius,
};
use crate::test_helpers::spawn_entity;

fn run_rotation_lock(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(reader_rotation_lock_system);
    schedule.run(world);
}

fn trigger_reader_click(world: &mut World, entity: Entity, cursor: Vec2) {
    use card_game::card::interaction::click_resolve::ClickedEntity;
    world.entity_mut(entity).observe(on_reader_clicked);
    world.flush();
    world.trigger_targets(
        ClickedEntity {
            world_cursor: cursor,
        },
        entity,
    );
}

fn run_reader_release(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(reader_release_system);
    schedule.run(world);
}

fn run_insert(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_reader_insert_system);
    schedule.run(world);
}

#[test]
fn when_reader_clicked_then_starts_reader_drag() {
    // Arrange
    let mut world = World::new();
    let jack = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let reader = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity: jack,
            },
            Transform2D {
                position: Vec2::new(100.0, 100.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    world.insert_resource(ReaderDragState::default());

    let cursor = Vec2::new(110.0, 95.0);

    // Act — trigger the observer directly
    trigger_reader_click(&mut world, reader, cursor);

    // Assert
    let dragging = world.resource::<ReaderDragState>().dragging.clone();
    assert_eq!(
        dragging,
        Some(ReaderDragInfo {
            entity: reader,
            grab_offset: Vec2::new(10.0, -5.0),
        })
    );
}

#[test]
fn when_mouse_released_while_reader_dragging_then_clears_reader_drag() {
    // Arrange
    let mut world = World::new();
    let reader = world.spawn_empty().id();
    world.insert_resource(ReaderDragState {
        dragging: Some(ReaderDragInfo {
            entity: reader,
            grab_offset: Vec2::new(2.0, 3.0),
        }),
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.release(MouseButton::Left);
    world.insert_resource(mouse);
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_reader_release(&mut world);

    // Assert
    assert!(world.resource::<ReaderDragState>().dragging.is_none());
}

// Canonical signature used in insert/eject scenario helpers — shared so
// assertion-level comparisons (e.g. SignatureSpace::center) stay in sync.
const INSERT_SCENARIO_SIG: [f32; 8] = [0.5, -0.3, 0.8, 0.1, -0.6, 0.2, 0.4, -0.1];

struct InsertTestSetup {
    card_entity: Entity,
    reader_entity: Entity,
    jack_entity: Entity,
}

fn setup_insert_scenario(world: &mut World) -> InsertTestSetup {
    let sig = CardSignature::new(INSERT_SCENARIO_SIG);
    let card_entity = world
        .spawn((
            Card {
                face_texture: engine_core::prelude::TextureId(0),
                back_texture: engine_core::prelude::TextureId(0),
                face_up: true,
                signature: sig,
            },
            CardZone::Table,
            Transform2D {
                position: Vec2::new(120.0, 90.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
        ))
        .id();

    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();

    let reader_entity = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: Vec2::new(100.0, 100.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.clear_frame_state();
    mouse.release(MouseButton::Left);
    world.insert_resource(mouse);

    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: card_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::new(120.0, 90.0),
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    InsertTestSetup {
        card_entity,
        reader_entity,
        jack_entity,
    }
}

/// @doc: Cards positioned outside the reader's bounding box must be rejected.
/// A false positive here would cause cards dropped far from a reader to
/// teleport into it, confusing the player and breaking spatial reasoning
/// about where readers are on the table.
#[test]
fn when_card_outside_reader_aabb_then_returns_false() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(200.0, 100.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(
        !result,
        "card at {card_pos} should NOT overlap reader at {reader_pos} +/- {reader_half}"
    );
}

/// @doc: The overlap boundary is inclusive (<=) so that a card positioned exactly
/// on the reader's edge is accepted. An exclusive boundary would create a
/// frustrating pixel-perfect dead zone where cards visually touch the reader
/// but fail to insert.
#[test]
fn when_card_exactly_on_reader_boundary_then_returns_true() {
    // Arrange
    let reader_pos = Vec2::new(0.0, 0.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(40.0, 0.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(result, "card exactly on boundary should be accepted");
}

/// @doc: A card whose center lies inside the reader's AABB must be detected as
/// overlapping. Without this, a reader would never trigger insertion even when a
/// card is positioned directly over it, breaking the core card-scanning mechanic.
#[test]
fn when_card_inside_reader_aabb_then_returns_true() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(120.0, 80.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(
        result,
        "card at {card_pos} should overlap reader at {reader_pos} +/- {reader_half}"
    );
}

/// @doc: Readers must stay axis-aligned on the table so their card slot and
/// jack positions remain predictable. The physics engine has no rotation-lock
/// API, so we zero angular velocity every frame. Without this, a reader hit
/// by a sliding card would spin freely, making it impossible to aim card drops.
/// @doc: The rotation lock must only affect `CardReader` entities — if it
/// accidentally queried all physics bodies, every card on the table would
/// stop spinning, breaking the flick-to-spin interaction.
#[test]
fn when_no_readers_then_no_angular_velocity_commands() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_rotation_lock(&mut world);

    // Assert
    assert!(world.resource::<EventBus<PhysicsCommand>>().is_empty());
}

/// @doc: After migration to the physics command layer, the rotation-lock system
/// must queue `SetAngularVelocity` commands into the `EventBus` rather than
/// calling `PhysicsRes` directly. This test pins both entity identity and the
/// zero-velocity payload so a refactor that targets the wrong entity or leaves
/// the velocity non-zero is caught immediately.
#[test]
fn when_reader_present_then_queues_set_angular_velocity_command() {
    // Arrange
    let mut world = World::new();
    let jack = spawn_entity();
    let reader = world
        .spawn(CardReader {
            loaded: None,
            half_extents: Vec2::new(40.0, 30.0),
            jack_entity: jack,
        })
        .id();
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_rotation_lock(&mut world);

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    assert_eq!(commands.len(), 1, "expected exactly one command queued");
    assert!(
        matches!(
            &commands[0],
            PhysicsCommand::SetAngularVelocity { entity, angular_velocity }
                if *entity == reader && angular_velocity.abs() < 1e-6
        ),
        "expected SetAngularVelocity for reader {reader:?} with 0.0, got {:?}",
        commands[0]
    );
}

/// @doc: Cards must teleport to the reader's center on insertion so their
/// visual position matches the slot. A card left at its drop position would
/// appear offset inside the reader frame, breaking the visual affordance that
/// the card is locked into place.
#[test]
fn when_card_released_over_reader_then_snaps_to_reader_position() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(setup.card_entity).unwrap();
    assert_eq!(
        transform.position,
        Vec2::new(100.0, 100.0),
        "card should snap to reader position"
    );
}

/// @doc: Inserted cards scale down to 60% to visually fit within the reader's
/// frame, distinguishing them from free table cards. The `ScaleSpring` component
/// provides smooth animation. Without scaling, inserted cards would overlap the
/// reader borders and obscure adjacent UI elements.
#[test]
fn when_card_released_over_reader_then_scale_spring_inserted() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    let spring = world.get::<ScaleSpring>(setup.card_entity).unwrap();
    assert!(
        (spring.target - READER_CARD_SCALE).abs() < 1e-6,
        "ScaleSpring target should be {READER_CARD_SCALE}, got {}",
        spring.target
    );
}

/// @doc: Inserted cards lose their `RigidBody` ECS component so zone-aware
/// systems correctly identify them as non-physical reader cards. A stale
/// `RigidBody` would cause the damping system to attempt physics operations
/// on an entity whose body was already removed by the command layer.
#[test]
fn when_card_released_over_reader_then_rigid_body_component_removed() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    assert!(
        world.get::<RigidBody>(setup.card_entity).is_none(),
        "RigidBody component should be removed"
    );
}

/// @doc: When a card is inserted into a reader, the system must queue a
/// `RemoveBody` command so the physics layer tears down the card's rigid body.
/// A card with an active physics body inside a reader would be affected by
/// table collisions, potentially getting knocked out of position while the
/// reader expects it to stay fixed.
#[test]
fn when_card_inserted_then_queues_remove_body_command() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_insert(&mut world);

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    assert!(
        commands.iter().any(
            |c| matches!(c, PhysicsCommand::RemoveBody { entity } if *entity == setup.card_entity)
        ),
        "expected RemoveBody command for card {:?}, got {:?}",
        setup.card_entity,
        commands
    );
}

/// @doc: The card's zone changes to Reader(entity) so all zone-aware systems
/// (damping, rendering, pick) correctly identify it as reader-loaded rather
/// than a free table card. Without this, the damping system would try to apply
/// drag to a card with no physics body, causing errors.
#[test]
fn when_card_released_over_reader_then_zone_set_to_reader() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    let zone = world.get::<CardZone>(setup.card_entity).unwrap();
    assert_eq!(
        *zone,
        CardZone::Reader(setup.reader_entity),
        "card zone should be Reader"
    );
}

/// @doc: The reader tracks the loaded card entity so the ejection system can
/// look it up directly without scanning all cards. This also lets the
/// full-reader guard (`is_some` check) reject additional card drops.
#[test]
fn when_card_released_over_reader_then_reader_loaded_stores_card() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    let reader = world.get::<CardReader>(setup.reader_entity).unwrap();
    assert_eq!(
        reader.loaded,
        Some(setup.card_entity),
        "reader loaded slot should store card entity"
    );
}

/// @doc: The reader's output jack must emit a `SignatureSpace` — a spherical region in
/// 8D signature space centered on the inserted card's signature with radius 0.2 — rather
/// than a raw `CardSignature`. This allows downstream cable-connected devices to reason
/// about a zone of signatures rather than a single point, enabling signature-matching
/// logic that can tolerate slight variations between cards.
#[test]
fn when_card_inserted_then_jack_emits_signature_space() {
    // Arrange
    let mut world = World::new();
    let setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    let jack = world
        .get::<Jack<SignatureSpace>>(setup.jack_entity)
        .unwrap();
    let expected_sig = CardSignature::new(INSERT_SCENARIO_SIG);
    assert_eq!(
        jack.data,
        Some(SignatureSpace::from_single(
            expected_sig,
            signature_radius(&expected_sig),
            setup.card_entity,
        )),
        "jack must emit SignatureSpace centered on the inserted card's signature"
    );
}

/// @doc: `DragState` must be cleared after insertion so the card isn't still
/// considered "being dragged" on the next frame. A stale `DragState` would
/// cause the drag system to apply velocity to a card that no longer has
/// a physics body, triggering a panic.
#[test]
fn when_card_released_over_reader_then_drag_state_cleared() {
    // Arrange
    let mut world = World::new();
    let _setup = setup_insert_scenario(&mut world);

    // Act
    run_insert(&mut world);

    // Assert
    let drag = world.resource::<DragState>();
    assert!(drag.dragging.is_none(), "drag state should be cleared");
}

/// @doc: Cards released away from any reader must fall to the table normally.
/// The reader insertion check must be a non-destructive probe — if no reader
/// overlaps, the card's state must remain completely untouched so the existing
/// release system can handle it.
#[test]
fn when_card_released_not_over_reader_then_no_insertion() {
    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            Card::face_down(
                engine_core::prelude::TextureId(0),
                engine_core::prelude::TextureId(0),
            ),
            CardZone::Table,
            Transform2D {
                position: Vec2::new(500.0, 300.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
        ))
        .id();
    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let reader_entity = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.clear_frame_state();
    mouse.release(MouseButton::Left);
    world.insert_resource(mouse);
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: card_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::new(500.0, 300.0),
        }),
    });
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_insert(&mut world);

    // Assert
    let zone = world.get::<CardZone>(card_entity).unwrap();
    assert_eq!(*zone, CardZone::Table, "card should remain on table");
    let reader = world.get::<CardReader>(reader_entity).unwrap();
    assert!(reader.loaded.is_none(), "reader should remain empty");
    let drag = world.resource::<DragState>();
    assert!(
        drag.dragging.is_some(),
        "drag state should NOT be cleared when no reader matched"
    );
}

/// @doc: A reader with a card already loaded must reject additional drops.
/// Without this guard, a second card drop would overwrite the first card's
/// reference, orphaning it in the reader zone with no way to eject it.
#[test]
fn when_reader_full_then_second_card_not_inserted() {
    // Arrange
    let mut world = World::new();
    let existing_card = world.spawn_empty().id();
    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let reader_entity = world
        .spawn((
            CardReader {
                loaded: Some(existing_card),
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: Vec2::new(100.0, 100.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let second_card = world
        .spawn((
            Card::face_down(
                engine_core::prelude::TextureId(0),
                engine_core::prelude::TextureId(0),
            ),
            CardZone::Table,
            Transform2D {
                position: Vec2::new(110.0, 95.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
        ))
        .id();
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.clear_frame_state();
    mouse.release(MouseButton::Left);
    world.insert_resource(mouse);
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: second_card,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::new(110.0, 95.0),
        }),
    });
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_insert(&mut world);

    // Assert
    let reader = world.get::<CardReader>(reader_entity).unwrap();
    assert_eq!(
        reader.loaded,
        Some(existing_card),
        "reader should still hold original card"
    );
    let zone = world.get::<CardZone>(second_card).unwrap();
    assert_eq!(*zone, CardZone::Table, "second card should remain on table");
}

// --- Reader drag tests ---

fn run_reader_drag(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(reader_drag_system);
    schedule.run(world);
}

/// @doc: A card inside a reader must move by exactly the same displacement as the
/// reader each frame it is dragged. This invariant holds regardless of where on
/// the table the reader started — the card is rigidly attached to the reader's
/// frame. Without it, repeated drags accumulate positional drift until the card
/// is no longer visible inside the reader slot.
#[test]
fn when_reader_dragged_then_loaded_card_moves_by_same_delta() {
    // Arrange
    let mut world = World::new();
    let initial_pos = Vec2::new(50.0, 80.0);
    let target_pos = Vec2::new(250.0, 180.0);

    let card_entity = world
        .spawn((
            Card::face_down(
                engine_core::prelude::TextureId(0),
                engine_core::prelude::TextureId(0),
            ),
            Transform2D {
                position: initial_pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
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

    world
        .entity_mut(card_entity)
        .insert(CardZone::Reader(reader_entity));

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(target_pos);
    world.insert_resource(mouse);

    world.insert_resource(ReaderDragState {
        dragging: Some(ReaderDragInfo {
            entity: reader_entity,
            grab_offset: Vec2::ZERO,
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_reader_drag(&mut world);

    // Assert
    let reader_pos = world.get::<Transform2D>(reader_entity).unwrap().position;
    let card_pos = world.get::<Transform2D>(card_entity).unwrap().position;
    assert_eq!(
        card_pos, reader_pos,
        "loaded card position {card_pos} must equal reader position {reader_pos} after drag"
    );
}

/// @doc: The reader drag system must queue a `SetBodyPosition` command so the
/// physics body follows the reader's visual position. Without this, the physics
/// collider stays at the old position while the sprite moves, causing invisible
/// collisions at the wrong location and making the reader un-hittable for picks.
#[test]
fn when_reader_dragged_then_queues_set_body_position_command() {
    // Arrange
    let mut world = World::new();
    let target_pos = Vec2::new(250.0, 180.0);

    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();

    let reader_entity = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: Vec2::new(50.0, 80.0),
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
        dragging: Some(ReaderDragInfo {
            entity: reader_entity,
            grab_offset: Vec2::ZERO,
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_reader_drag(&mut world);

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    assert_eq!(
        commands.len(),
        2,
        "expected SetBodyPosition + SetCollisionGroup commands"
    );
    assert!(
        commands.iter().any(|c| matches!(
            c,
            PhysicsCommand::SetBodyPosition { entity, position }
                if *entity == reader_entity && (*position - target_pos).length() < 1e-6
        )),
        "expected SetBodyPosition for reader at {target_pos}, got {commands:?}"
    );
}

/// @doc: Dragging an empty reader must not panic or error when the card-sync
/// code finds no loaded card to update. This guards against an unwrap on
/// `CardReader.loaded` — if the Option is None, the system should simply
/// skip the card position update and move only the reader.
#[test]
fn when_empty_reader_dragged_then_no_panic() {
    // Arrange
    let mut world = World::new();
    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let reader_entity = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: Vec2::new(100.0, 100.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(200.0, 200.0));
    world.insert_resource(mouse);

    world.insert_resource(ReaderDragState {
        dragging: Some(ReaderDragInfo {
            entity: reader_entity,
            grab_offset: Vec2::ZERO,
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_reader_drag(&mut world);

    // Assert
    let reader_pos = world.get::<Transform2D>(reader_entity).unwrap().position;
    assert_eq!(
        reader_pos,
        Vec2::new(200.0, 200.0),
        "empty reader should still move normally"
    );
}

fn run_eject(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_reader_eject_system);
    schedule.run(world);
}

struct EjectTestSetup {
    card_entity: Entity,
    reader_entity: Entity,
    jack_entity: Entity,
}

fn setup_eject_scenario(world: &mut World) -> EjectTestSetup {
    let sig = CardSignature::new(INSERT_SCENARIO_SIG);

    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: Some(SignatureSpace::from_single(
                sig,
                SIGNATURE_SPACE_RADIUS,
                Entity::from_raw(0),
            )),
        })
        .id();

    let card_entity = world
        .spawn((
            Card {
                face_texture: engine_core::prelude::TextureId(0),
                back_texture: engine_core::prelude::TextureId(0),
                face_up: true,
                signature: sig,
            },
            Transform2D {
                position: Vec2::new(100.0, 100.0),
                rotation: 0.0,
                scale: Vec2::splat(READER_CARD_SCALE),
            },
            engine_physics::prelude::Collider::Aabb(Vec2::new(27.0, 22.5)),
        ))
        .id();

    let reader_entity = world
        .spawn((
            CardReader {
                loaded: Some(card_entity),
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: Vec2::new(100.0, 100.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    world
        .entity_mut(card_entity)
        .insert(CardZone::Reader(reader_entity));

    // Simulate a drag starting on the card (pick system already ran)
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: card_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Reader(reader_entity),
            stash_cursor_follow: false,
            origin_position: Vec2::new(100.0, 100.0),
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    EjectTestSetup {
        card_entity,
        reader_entity,
        jack_entity,
    }
}

/// @doc: When a player grabs a card out of a reader, the card must return to
/// Table zone and the reader must clear its loaded slot. Without this, the card
/// would remain in Reader zone forever — invisible to the hand/stash systems
/// and impossible to re-drop into a different reader.
#[test]
fn when_card_picked_from_reader_then_zone_restored_and_reader_cleared() {
    // Arrange
    let mut world = World::new();
    let setup = setup_eject_scenario(&mut world);

    // Act
    run_eject(&mut world);

    // Assert
    let zone = world.get::<CardZone>(setup.card_entity).unwrap();
    assert_eq!(
        *zone,
        CardZone::Table,
        "card zone should be restored to Table"
    );
    let reader = world.get::<CardReader>(setup.reader_entity).unwrap();
    assert!(
        reader.loaded.is_none(),
        "reader loaded slot should be cleared"
    );
}

/// @doc: Ejected cards need a physics body to participate in table collisions
/// and be draggable by the physics-based drag system. Without restoring the
/// body, the card would be a ghost — visible but unable to interact with
/// anything on the table.
#[test]
fn when_card_picked_from_reader_then_physics_body_restored() {
    // Arrange
    let mut world = World::new();
    let setup = setup_eject_scenario(&mut world);

    // Act
    run_eject(&mut world);

    // Assert
    assert!(
        world.get::<RigidBody>(setup.card_entity).is_some(),
        "RigidBody component should be re-inserted"
    );
}

/// @doc: When a card is ejected from a reader, the system must queue all four
/// physics activation commands (`AddBody`, `AddCollider`, `SetDamping`, `SetCollisionGroup`)
/// so the card becomes a physical object on the table again. Missing any one of
/// these would leave the card in a broken state — no body means invisible to
/// collisions, no collider means untouchable, no damping means infinite sliding,
/// no collision group means it clips through boundaries.
#[test]
fn when_card_ejected_then_queues_physics_activation_commands() {
    // Arrange
    let mut world = World::new();
    let setup = setup_eject_scenario(&mut world);
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_eject(&mut world);

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    assert!(
        commands.iter().any(
            |c| matches!(c, PhysicsCommand::AddBody { entity, .. } if *entity == setup.card_entity)
        ),
        "expected AddBody for card, got {commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, PhysicsCommand::AddCollider { entity, .. } if *entity == setup.card_entity)),
        "expected AddCollider for card, got {commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, PhysicsCommand::SetDamping { entity, .. } if *entity == setup.card_entity)),
        "expected SetDamping for card, got {commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, PhysicsCommand::SetCollisionGroup { entity, .. } if *entity == setup.card_entity)),
        "expected SetCollisionGroup for card, got {commands:?}"
    );
}

/// @doc: Ejected cards must start with zero-collision groups (DRAGGED) so they
/// don't physically collide with the reader they were just removed from.
/// The card starts at the reader's center position; if it were activated with
/// `CARD_COLLISION_GROUP/FILTER` (which match the reader's membership), rapier
/// would detect an immediate overlap and fire a separation impulse, causing
/// the card to fly away. Zero groups mean no collisions until the card is
/// released onto the table, at which point the release system restores the
/// proper groups.
#[test]
fn when_card_ejected_then_collision_group_is_dragged_zero() {
    // Arrange
    let mut world = World::new();
    let setup = setup_eject_scenario(&mut world);
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_eject(&mut world);

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let group_cmd = commands.iter().find(
        |c| matches!(c, PhysicsCommand::SetCollisionGroup { entity, .. } if *entity == setup.card_entity),
    );
    assert!(
        matches!(
            group_cmd,
            Some(PhysicsCommand::SetCollisionGroup {
                membership,
                filter,
                ..
            }) if *membership == DRAGGED_COLLISION_GROUP && *filter == DRAGGED_COLLISION_FILTER
        ),
        "expected SetCollisionGroup with DRAGGED groups (0, 0), got {group_cmd:?}"
    );
}

/// @doc: Ejected cards animate back to full size via ScaleSpring(1.0), matching
/// the convention used by `drop_on_table` in the release system. Without this,
/// the card would remain at 60% size on the table, looking broken.
#[test]
fn when_card_picked_from_reader_then_scale_spring_restores_full_size() {
    // Arrange
    let mut world = World::new();
    let setup = setup_eject_scenario(&mut world);

    // Act
    run_eject(&mut world);

    // Assert
    let spring = world.get::<ScaleSpring>(setup.card_entity).unwrap();
    assert!(
        (spring.target - 1.0).abs() < 1e-6,
        "ScaleSpring target should be 1.0, got {}",
        spring.target
    );
}

/// @doc: The output jack must clear when a card is ejected so downstream
/// consumers stop seeing a stale signature. A jack that retains data after
/// ejection would cause devices to act on a card that's no longer in the reader.
#[test]
fn when_card_picked_from_reader_then_jack_data_cleared() {
    // Arrange
    let mut world = World::new();
    let setup = setup_eject_scenario(&mut world);

    // Act
    run_eject(&mut world);

    // Assert
    let jack = world
        .get::<Jack<SignatureSpace>>(setup.jack_entity)
        .unwrap();
    assert!(jack.data.is_none(), "output jack data should be cleared");
}

// --- Drag-then-eject tests ---

/// @doc: Verifies the end-to-end drag-then-eject chain: `reader_drag_system` must
/// sync the card's Transform2D.position before `card_reader_eject_system` reads it
/// to place the restored physics body. Without this synchronisation, a reader
/// dragged to (300, 200) would eject its card back at the original insert position
/// (100, 100), causing the card to teleport visibly across the table.
#[test]
fn when_reader_dragged_then_ejected_card_physics_body_placed_at_new_position() {
    // Arrange
    let mut world = World::new();
    let sig = CardSignature::new([0.1, 0.2, 0.3, 0.4, -0.1, -0.2, -0.3, -0.4]);
    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: Some(SignatureSpace::from_single(
                sig,
                SIGNATURE_SPACE_RADIUS,
                Entity::from_raw(0),
            )),
        })
        .id();
    let reader_start = Vec2::new(100.0, 100.0);
    let reader_dest = Vec2::new(300.0, 200.0);

    let card_entity = world
        .spawn((
            Card {
                face_texture: engine_core::prelude::TextureId(0),
                back_texture: engine_core::prelude::TextureId(0),
                face_up: true,
                signature: sig,
            },
            Transform2D {
                position: reader_start,
                rotation: 0.0,
                scale: Vec2::splat(READER_CARD_SCALE),
            },
            engine_physics::prelude::Collider::Aabb(Vec2::new(27.0, 22.5)),
        ))
        .id();

    let reader_entity = world
        .spawn((
            CardReader {
                loaded: Some(card_entity),
                half_extents: Vec2::new(40.0, 60.0),
                jack_entity,
            },
            Transform2D {
                position: reader_start,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    world
        .entity_mut(card_entity)
        .insert(CardZone::Reader(reader_entity));

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(reader_dest);
    world.insert_resource(mouse);

    world.insert_resource(ReaderDragState {
        dragging: Some(ReaderDragInfo {
            entity: reader_entity,
            grab_offset: Vec2::ZERO,
        }),
    });

    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: card_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Reader(reader_entity),
            stash_cursor_follow: false,
            origin_position: reader_start,
        }),
    });

    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_reader_drag(&mut world);
    run_eject(&mut world);

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let body_pos = commands
        .iter()
        .find_map(|c| match c {
            PhysicsCommand::AddBody {
                entity, position, ..
            } if *entity == card_entity => Some(*position),
            _ => None,
        })
        .expect("add_body should have been called for the card entity");
    assert_eq!(
        body_pos, reader_dest,
        "physics body placed at {body_pos}, expected reader destination {reader_dest}"
    );
}

// --- Edge case tests ---

/// @doc: When two readers overlap and a card is dropped in the shared area,
/// exactly one reader must claim it. Double-insertion would corrupt the card's
/// zone (it can't be in two readers) and leave one reader with a stale
/// reference. The system uses first-match semantics — whichever reader the
/// ECS query iterates first wins.
#[test]
fn when_card_over_two_overlapping_readers_then_only_one_claims_it() {
    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            Card::face_down(
                engine_core::prelude::TextureId(0),
                engine_core::prelude::TextureId(0),
            ),
            CardZone::Table,
            Transform2D {
                position: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
        ))
        .id();
    let jack_a = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let reader_a = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 40.0),
                jack_entity: jack_a,
            },
            Transform2D {
                position: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let jack_b = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let reader_b = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 40.0),
                jack_entity: jack_b,
            },
            Transform2D {
                position: Vec2::new(60.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.clear_frame_state();
    mouse.release(MouseButton::Left);
    world.insert_resource(mouse);
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: card_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::new(50.0, 50.0),
        }),
    });
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_insert(&mut world);

    // Assert
    let a_loaded = world.get::<CardReader>(reader_a).unwrap().loaded;
    let b_loaded = world.get::<CardReader>(reader_b).unwrap().loaded;
    let total_loaded = a_loaded.iter().count() + b_loaded.iter().count();
    assert_eq!(
        total_loaded, 1,
        "exactly one reader should claim the card, got a={a_loaded:?} b={b_loaded:?}"
    );

    let zone = world.get::<CardZone>(card_entity).unwrap();
    assert!(
        matches!(zone, CardZone::Reader(_)),
        "card should be in a reader zone"
    );
}
