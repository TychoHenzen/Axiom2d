#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::*;
use engine_core::prelude::EventBus;
use engine_core::scale_spring::ScaleSpring;
use engine_physics::prelude::PhysicsCommand;
use engine_scene::prelude::{GlobalTransform2D, LocalSortOrder, RenderLayer, SortOrder};
use glam::{Affine2, Vec2};

use card_game::card::component::{CardItemForm, CardZone};
use card_game::card::interaction::apply::interaction_apply_system;
use card_game::card::interaction::drag_state::DragState;
use card_game::card::interaction::intent::InteractionIntent;

use super::helpers::default_card_collider;

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(interaction_apply_system);
    schedule.run(world);
}

fn insert_apply_resources(world: &mut World) {
    world.insert_resource(DragState::default());
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
}

fn setup_active_drag(world: &mut World, entity: Entity) {
    use card_game::card::interaction::drag_state::DragInfo;
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::new(50.0, 75.0),
        }),
    });
}

/// @doc: A `ReleaseOnTable` intent must transition the card to `CardZone::Table`, insert
/// `RigidBody::Dynamic` and `RenderLayer::World`, remove `CardItemForm`, and reset scale to
/// 1.0. Without the zone transition the card would remain in its previous zone (e.g. Stash or
/// Hand) causing layout systems to still track it in the wrong container after drop.
#[test]
fn when_intent_apply_to_table_then_card_moved_to_table_zone() {
    use engine_core::prelude::Transform2D;
    use engine_physics::prelude::RigidBody;

    // Arrange — spawn a card in the stash, then release it on the table
    let mut world = World::new();
    let card_entity = world
        .spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 1,
            },
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
            SortOrder::new(0),
            Transform2D {
                position: Vec2::new(100.0, 200.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            CardItemForm,
            RenderLayer::UI,
        ))
        .id();
    insert_apply_resources(&mut world);
    setup_active_drag(&mut world, card_entity);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::ReleaseOnTable {
            entity: card_entity,
            snap_back: false,
        });

    // Act
    run_system(&mut world);
    world.flush();

    // Assert
    assert_eq!(
        *world
            .entity(card_entity)
            .get::<CardZone>()
            .expect("card must have CardZone"),
        CardZone::Table,
        "card must be moved to Table zone on ReleaseOnTable"
    );
    assert!(
        world.entity(card_entity).get::<RigidBody>().is_some(),
        "table card must have RigidBody"
    );
    assert_eq!(
        *world
            .entity(card_entity)
            .get::<RenderLayer>()
            .expect("card must have RenderLayer"),
        RenderLayer::World,
        "table card must have World render layer"
    );
    let scale = world
        .entity(card_entity)
        .get::<ScaleSpring>()
        .expect("card must have ScaleSpring");
    assert!(
        (scale.target - 1.0).abs() < f32::EPSILON,
        "table card scale must be 1.0, got {}",
        scale.target
    );
    assert!(
        world.entity(card_entity).get::<CardItemForm>().is_none(),
        "CardItemForm must be removed when on table"
    );
}

/// @doc: Picking a card from a `Reader` zone must populate `DragState` with the Reader zone
/// as origin and must NOT emit physics commands (reader cards don't exist as physics bodies
/// on the table). The `origin_zone` determines valid drop targets for other systems and must
/// faithfully reflect where the card came from.
#[test]
fn when_intent_apply_from_reader_then_card_zone_updated() {
    // Arrange — spawn a separate reader entity and attach a card via CardZone::Reader
    let mut world = World::new();
    let reader_entity = world.spawn_empty().id();
    let card_entity = world
        .spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Reader(reader_entity),
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(10.0, 20.0))),
            SortOrder::new(0),
        ))
        .id();
    insert_apply_resources(&mut world);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::PickCard {
            entity: card_entity,
            zone: CardZone::Reader(reader_entity),
            collider: default_card_collider(),
            grab_offset: Vec2::new(5.0, 3.0),
        });

    // Act
    run_system(&mut world);

    // Assert
    let drag = world
        .resource::<DragState>()
        .dragging
        .expect("DragState must be set after PickCard from Reader");
    assert_eq!(
        drag.entity, card_entity,
        "DragState must track the picked card entity"
    );
    assert_eq!(
        drag.origin_zone,
        CardZone::Reader(reader_entity),
        "origin_zone must be the Reader zone"
    );
    assert!(
        !drag.stash_cursor_follow,
        "Reader pick is not a stash pick — stash_cursor_follow must be false"
    );
    // CardZone component unchanged by pick (only changed on release)
    assert_eq!(
        *world
            .entity(card_entity)
            .get::<CardZone>()
            .expect("card must have CardZone"),
        CardZone::Reader(reader_entity),
        "card zone must remain unchanged until release"
    );
    // No physics commands for reader pick (not a table or hand entity)
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    assert!(
        physics.is_empty(),
        "picking from Reader must not emit physics commands, got: {physics:?}"
    );
}

/// @doc: Picking a table card that has no existing rigid body must emit an `AddBody` physics
/// command via `activate_physics_body` and insert `RigidBody::Dynamic` on the entity. Without
/// this a card with only a collider would have no rapier body to apply drag forces to, making
/// the card unresponsive to physics-based drag movement.
#[test]
fn when_intent_apply_with_physics_then_physics_commands_emitted() {
    use engine_physics::prelude::RigidBody;

    // Arrange — spawn a Table card with a collider but NO RigidBody component
    let mut world = World::new();
    let card_entity = world
        .spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(10.0, 20.0))),
            SortOrder::new(0),
        ))
        .id();
    insert_apply_resources(&mut world);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::PickCard {
            entity: card_entity,
            zone: CardZone::Table,
            collider: default_card_collider(),
            grab_offset: Vec2::ZERO,
        });

    // Act
    run_system(&mut world);

    // Assert — AddBody must be emitted for the table card without body
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let has_add_body = physics
        .iter()
        .any(|cmd| matches!(cmd, PhysicsCommand::AddBody { entity, .. } if *entity == card_entity));
    assert!(
        has_add_body,
        "must emit AddBody for a table card without existing body; got: {physics:?}"
    );
    // RigidBody component must be inserted by the system
    assert!(
        world.entity(card_entity).get::<RigidBody>().is_some(),
        "RigidBody must be inserted for table card without existing body"
    );
}

/// @doc: When the `InteractionIntent` bus is empty, `interaction_apply_system` must be a
/// complete no-op — it must not mutate `DragState`, emit any `PhysicsCommand`, or add/remove
/// components on any entity. This guards against accidental state changes when the system is
/// ticked on frames with no player interaction.
#[test]
fn when_no_intent_pending_then_system_is_noop() {
    // Arrange — spawn a card with required components, add resources, push NO intents
    let mut world = World::new();
    let card_entity = world
        .spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(10.0, 20.0))),
            SortOrder::new(0),
        ))
        .id();
    insert_apply_resources(&mut world);
    // Intentional: no intents pushed onto the bus

    // Act
    run_system(&mut world);

    // Assert — DragState remains None
    assert!(
        world.resource::<DragState>().dragging.is_none(),
        "DragState must remain None when no intents are pending"
    );
    // No physics commands emitted
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    assert!(
        physics.is_empty(),
        "no physics commands should be emitted with empty intent bus"
    );
    // Entity unchanged — no CardZone mutation, no LocalSortOrder, no ScaleSpring
    assert_eq!(
        *world
            .entity(card_entity)
            .get::<CardZone>()
            .expect("card must have CardZone"),
        CardZone::Table,
        "card zone must be unchanged by noop system run"
    );
    assert!(
        world.entity(card_entity).get::<LocalSortOrder>().is_none(),
        "LocalSortOrder must not be inserted when no intents are processed"
    );
    assert!(
        world.entity(card_entity).get::<ScaleSpring>().is_none(),
        "ScaleSpring must not be inserted when no intents are processed"
    );
}
