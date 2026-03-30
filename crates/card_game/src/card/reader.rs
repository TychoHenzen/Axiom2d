use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut, Resource, With, Without};
use engine_core::prelude::{ScaleSpring, Transform2D};
use engine_input::prelude::MouseState;
use engine_physics::prelude::{PhysicsRes, RigidBody};
use glam::Vec2;

use crate::card::component::{Card, CardZone};
use crate::card::identity::signature::CardSignature;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};

pub const READER_CARD_SCALE: f32 = 0.6;
pub const READER_COLLISION_GROUP: u32 = 0b0010;
pub const READER_COLLISION_FILTER: u32 = 0b0001;

#[derive(Component, Debug, Clone)]
pub struct CardReader {
    pub loaded: Option<Entity>,
    pub half_extents: Vec2,
    pub jack_entity: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct OutputJack {
    pub data: Option<CardSignature>,
}

pub fn card_overlaps_reader(card_pos: Vec2, reader_pos: Vec2, reader_half: Vec2) -> bool {
    let delta = (card_pos - reader_pos).abs();
    delta.x <= reader_half.x && delta.y <= reader_half.y
}

#[derive(Resource, Debug, Default)]
pub struct ReaderDragState {
    pub dragging: Option<ReaderDragInfo>,
}

#[derive(Debug, Clone)]
pub struct ReaderDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
}

pub fn reader_rotation_lock_system(
    query: Query<Entity, With<CardReader>>,
    mut physics: ResMut<PhysicsRes>,
) {
    for entity in &query {
        let _ = physics.set_angular_velocity(entity, 0.0);
    }
}

pub fn reader_pick_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    mut reader_drag: ResMut<ReaderDragState>,
    readers: Query<(Entity, &Transform2D, &CardReader)>,
) {
    use engine_input::mouse_button::MouseButton;

    if drag_state.dragging.is_some() || reader_drag.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let cursor = mouse.world_pos();
    for (entity, transform, reader) in &readers {
        let delta = (cursor - transform.position).abs();
        if delta.x <= reader.half_extents.x && delta.y <= reader.half_extents.y {
            reader_drag.dragging = Some(ReaderDragInfo {
                entity,
                grab_offset: cursor - transform.position,
            });
            return;
        }
    }
}

pub fn reader_drag_system(
    mouse: Res<MouseState>,
    reader_drag: Res<ReaderDragState>,
    mut transforms: Query<&mut Transform2D>,
    mut physics: ResMut<PhysicsRes>,
) {
    use engine_input::mouse_button::MouseButton;

    let Some(info) = &reader_drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = transforms.get_mut(info.entity) {
        transform.position = target;
    }
    let _ = physics.set_body_position(info.entity, target);
}

pub fn reader_release_system(mouse: Res<MouseState>, mut reader_drag: ResMut<ReaderDragState>) {
    use engine_input::mouse_button::MouseButton;

    if reader_drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        reader_drag.dragging = None;
    }
}

pub fn card_reader_insert_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut readers: Query<(Entity, &Transform2D, &mut CardReader)>,
    mut cards: Query<(&mut Transform2D, &Card, &mut CardZone), Without<CardReader>>,
    mut jacks: Query<&mut OutputJack>,
    mut physics: ResMut<PhysicsRes>,
    mut commands: Commands,
) {
    use engine_input::mouse_button::MouseButton;

    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(drag_info) = drag_state.dragging.as_ref() else {
        return;
    };
    let card_entity = drag_info.entity;

    let Ok((mut card_transform, card, mut card_zone)) = cards.get_mut(card_entity) else {
        return;
    };
    let card_pos = card_transform.position;

    for (reader_entity, reader_transform, mut reader) in &mut readers {
        if reader.loaded.is_some() {
            continue;
        }
        if !card_overlaps_reader(card_pos, reader_transform.position, reader.half_extents) {
            continue;
        }

        // Snap card to reader position and zero rotation
        card_transform.position = reader_transform.position;
        card_transform.rotation = 0.0;

        // Scale down
        commands
            .entity(card_entity)
            .insert(ScaleSpring::new(READER_CARD_SCALE));

        // Remove physics body
        let _ = physics.remove_body(card_entity);
        commands.entity(card_entity).remove::<RigidBody>();

        // Update zone
        *card_zone = CardZone::Reader(reader_entity);

        // Store in reader
        reader.loaded = Some(card_entity);

        // Update output jack
        if let Ok(mut jack) = jacks.get_mut(reader.jack_entity) {
            jack.data = Some(card.signature);
        }

        // Clear drag
        drag_state.dragging = None;
        return;
    }
}

pub fn card_reader_eject_system(
    drag_state: Res<DragState>,
    mut readers: Query<&mut CardReader>,
    mut cards: Query<&mut CardZone, With<Card>>,
    mut jacks: Query<&mut OutputJack>,
    mut physics: ResMut<PhysicsRes>,
    transforms: Query<&Transform2D>,
    colliders: Query<&engine_physics::prelude::Collider>,
    mut commands: Commands,
) {
    let Some(drag_info) = drag_state.dragging.as_ref() else {
        return;
    };
    let card_entity = drag_info.entity;

    let Ok(mut card_zone) = cards.get_mut(card_entity) else {
        return;
    };
    let CardZone::Reader(reader_entity) = *card_zone else {
        return;
    };

    // Restore card to table
    *card_zone = CardZone::Table;
    commands
        .entity(card_entity)
        .insert(ScaleSpring::new(1.0))
        .insert(RigidBody::Dynamic);

    // Re-add physics body with correct collision groups
    if let Ok(transform) = transforms.get(card_entity) {
        if let Ok(collider) = colliders.get(card_entity) {
            activate_physics_body(
                card_entity,
                transform.position,
                collider,
                &mut physics,
                CARD_COLLISION_GROUP,
                CARD_COLLISION_FILTER,
            );
        }
    }

    // Clear reader
    if let Ok(mut reader) = readers.get_mut(reader_entity) {
        reader.loaded = None;
        if let Ok(mut jack) = jacks.get_mut(reader.jack_entity) {
            jack.data = None;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::{ScaleSpring, Transform2D};
    use engine_input::mouse_button::MouseButton;
    use engine_input::prelude::MouseState;
    use engine_physics::prelude::{PhysicsRes, RigidBody};
    use glam::Vec2;

    use super::*;
    use crate::card::component::{Card, CardZone};
    use crate::card::identity::signature::CardSignature;
    use crate::card::interaction::drag_state::{DragInfo, DragState};
    use crate::test_helpers::{AngularVelocityLog, RemoveBodyLog, SpyPhysicsBackend, spawn_entity};

    fn run_rotation_lock(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(reader_rotation_lock_system);
        schedule.run(world);
    }

    fn run_insert(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_reader_insert_system);
        schedule.run(world);
    }

    struct InsertTestSetup {
        card_entity: Entity,
        reader_entity: Entity,
        jack_entity: Entity,
    }

    fn setup_insert_scenario(world: &mut World) -> InsertTestSetup {
        let sig = CardSignature::new([0.5, -0.3, 0.8, 0.1, -0.6, 0.2, 0.4, -0.1]);
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

        let jack_entity = world.spawn(OutputJack { data: None }).id();

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

        let spy = SpyPhysicsBackend::new();
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        InsertTestSetup {
            card_entity,
            reader_entity,
            jack_entity,
        }
    }

    // --- Overlap function tests ---

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
            "card at {card_pos} should NOT overlap reader at {reader_pos} ± {reader_half}"
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

    /// @doc: Verifies the basic hit case: a card whose center lies strictly inside the
    /// reader's AABB is detected as overlapping. Without this, a reader would never
    /// trigger even when a card is positioned directly over it, breaking the core
    /// card-scanning mechanic entirely.
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
            "card at {card_pos} should overlap reader at {reader_pos} ± {reader_half}"
        );
    }

    // --- Rotation lock tests ---

    /// @doc: Readers must stay axis-aligned on the table so their card slot and
    /// jack positions remain predictable. The physics engine has no rotation-lock
    /// API, so we zero angular velocity every frame. Without this, a reader hit
    /// by a sliding card would spin freely, making it impossible to aim card drops.
    #[test]
    fn when_reader_has_angular_velocity_then_zeroed() {
        // Arrange
        let ang_log: AngularVelocityLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let jack = spawn_entity();
        let reader = world
            .spawn(CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 30.0),
                jack_entity: jack,
            })
            .id();
        let spy = SpyPhysicsBackend::new()
            .with_angular_velocity(reader, 5.0)
            .with_angular_velocity_log(ang_log.clone());
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_rotation_lock(&mut world);

        // Assert
        let calls = ang_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, reader);
        assert!(
            (calls[0].1).abs() < 1e-6,
            "angular velocity should be zeroed"
        );
    }

    /// @doc: The rotation lock must only affect `CardReader` entities — if it
    /// accidentally queried all physics bodies, every card on the table would
    /// stop spinning, breaking the flick-to-spin interaction.
    #[test]
    fn when_no_readers_then_no_angular_velocity_calls() {
        // Arrange
        let ang_log: AngularVelocityLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let spy = SpyPhysicsBackend::new().with_angular_velocity_log(ang_log.clone());
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_rotation_lock(&mut world);

        // Assert
        assert!(ang_log.lock().unwrap().is_empty());
    }

    // --- Insertion tests (TC009-TC015) ---

    /// @doc: When a dragged card is released over a reader, it snaps to the reader's
    /// exact position. This provides clear visual feedback that the card is "locked in"
    /// rather than floating loosely near the reader. Without snapping, players couldn't
    /// tell whether a card was properly inserted or just happened to land nearby.
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

    /// @doc: Inserted cards lose their physics body to prevent them from being
    /// knocked out of the reader by table collisions. A card that retains its
    /// body could be launched out of the reader by another card sliding into it,
    /// which would break the reader's slot guarantee.
    #[test]
    fn when_card_released_over_reader_then_physics_body_removed() {
        // Arrange
        let mut world = World::new();
        let remove_log: RemoveBodyLog = Arc::new(Mutex::new(Vec::new()));
        let setup = setup_insert_scenario(&mut world);
        // Re-insert physics with remove log
        let spy = SpyPhysicsBackend::new().with_remove_body_log(remove_log.clone());
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_insert(&mut world);

        // Assert
        let removed = remove_log.lock().unwrap();
        assert!(
            removed.contains(&setup.card_entity),
            "physics body should be removed for card"
        );
        assert!(
            world.get::<RigidBody>(setup.card_entity).is_none(),
            "RigidBody component should be removed"
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

    /// @doc: The output jack immediately reflects the loaded card's signature so
    /// downstream consumers (future cable connections) get data as soon as a card
    /// is inserted. A stale or empty jack after insertion would break any device
    /// chain that depends on the reader's output.
    #[test]
    fn when_card_released_over_reader_then_jack_has_signature() {
        // Arrange
        let mut world = World::new();
        let setup = setup_insert_scenario(&mut world);

        // Act
        run_insert(&mut world);

        // Assert
        let jack = world.get::<OutputJack>(setup.jack_entity).unwrap();
        let expected = CardSignature::new([0.5, -0.3, 0.8, 0.1, -0.6, 0.2, 0.4, -0.1]);
        assert_eq!(
            jack.data,
            Some(expected),
            "output jack should contain card signature"
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
        let jack_entity = world.spawn(OutputJack { data: None }).id();
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
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));

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
        let jack_entity = world.spawn(OutputJack { data: None }).id();
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
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));

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

    // --- Ejection tests (TC018-TC021) ---

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
        let sig = CardSignature::new([0.5, -0.3, 0.8, 0.1, -0.6, 0.2, 0.4, -0.1]);

        let jack_entity = world.spawn(OutputJack { data: Some(sig) }).id();

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

        // Card is in Reader zone
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

        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));

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
        let jack = world.get::<OutputJack>(setup.jack_entity).unwrap();
        assert!(jack.data.is_none(), "output jack data should be cleared");
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
        let jack_a = world.spawn(OutputJack { data: None }).id();
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
        let jack_b = world.spawn(OutputJack { data: None }).id();
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
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));

        // Act
        run_insert(&mut world);

        // Assert — exactly one reader should have the card
        let a_loaded = world.get::<CardReader>(reader_a).unwrap().loaded;
        let b_loaded = world.get::<CardReader>(reader_b).unwrap().loaded;
        let total_loaded = a_loaded.iter().count() + b_loaded.iter().count();
        assert_eq!(
            total_loaded, 1,
            "exactly one reader should claim the card, got a={a_loaded:?} b={b_loaded:?}"
        );

        // Card should be in exactly one reader zone
        let zone = world.get::<CardZone>(card_entity).unwrap();
        assert!(
            matches!(zone, CardZone::Reader(_)),
            "card should be in a reader zone"
        );
    }
}
