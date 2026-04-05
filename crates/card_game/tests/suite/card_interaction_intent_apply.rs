#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::*;
use engine_core::prelude::EventBus;
use engine_core::scale_spring::ScaleSpring;
use engine_physics::prelude::PhysicsCommand;
use engine_scene::prelude::{GlobalTransform2D, LocalSortOrder, SortOrder};
use glam::{Affine2, Vec2};

use card_game::card::component::CardZone;
use card_game::card::interaction::apply::interaction_apply_system;
use card_game::card::interaction::drag_state::DragState;
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::interaction::pick::DRAG_SCALE;

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

/// @doc: When a `PickCard` intent for a Table card is consumed, `DragState` must be populated with the
/// correct entity and zone so that the drag system can track the active drag. This is the primary
/// output contract of `interaction_apply_system`: the intent bus is the input, `DragState` is the
/// output, and `origin_zone` must faithfully reflect where the card came from.
#[test]
fn when_pick_card_table_intent_applied_then_drag_state_set_with_correct_entity_and_zone() {
    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
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
            grab_offset: Vec2::new(5.0, 3.0),
        });

    // Act
    run_system(&mut world);

    // Assert
    let drag = world
        .resource::<DragState>()
        .dragging
        .expect("DragState should be Some after applying PickCard intent");
    assert_eq!(drag.entity, card_entity);
    assert_eq!(drag.origin_zone, CardZone::Table);
    assert!(!drag.stash_cursor_follow);
}

/// @doc: A `PickCard` intent for a Table card must issue a `SetCollisionGroup` command with both
/// membership and filter set to 0 (`DRAGGED_COLLISION_GROUP` / `DRAGGED_COLLISION_FILTER`), removing
/// the card from all collision layers so it does not interfere with physics queries while being
/// dragged. This verifies the physics side-effect, not merely the `DragState`.
#[test]
fn when_pick_card_table_intent_applied_then_physics_bus_contains_set_collision_group_zero() {
    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
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

    // Assert
    let commands: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let set_group = commands.iter().find(|cmd| {
        matches!(
            cmd,
            PhysicsCommand::SetCollisionGroup {
                entity,
                membership: 0,
                filter: 0,
            } if *entity == card_entity
        )
    });
    assert!(
        set_group.is_some(),
        "Expected SetCollisionGroup(membership=0, filter=0) for the picked entity"
    );
}

/// @doc: After `interaction_apply_system` processes a `PickCard` intent, the intent bus must be empty.
/// Leaving unconsumed intents would cause the next frame's apply pass to double-apply the same
/// transition. This test also verifies that the entity receives `LocalSortOrder` and `ScaleSpring`
/// components, which are the visual side effects of a pick (sort bump + scale-up animation).
#[test]
fn when_pick_card_intent_applied_then_intent_bus_drained_and_entity_has_sort_and_scale_components()
{
    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
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

    // Assert — intent bus is empty
    assert!(
        world.resource::<EventBus<InteractionIntent>>().is_empty(),
        "InteractionIntent bus must be drained after apply"
    );

    // Assert — entity has LocalSortOrder and ScaleSpring with target=DRAG_SCALE
    let scale_spring = world.entity(card_entity).get::<ScaleSpring>();
    assert!(
        scale_spring.is_some_and(|s| (s.target - DRAG_SCALE).abs() < f32::EPSILON),
        "Entity must have ScaleSpring with target=DRAG_SCALE ({DRAG_SCALE})"
    );
    assert!(
        world.entity(card_entity).get::<LocalSortOrder>().is_some(),
        "Entity must have LocalSortOrder inserted on pick"
    );
}

/// @doc: A `PickFromStash` intent causes the applier to vacate the stash slot and set up
/// `DragState` with `stash_cursor_follow=true` so the card tracks the cursor without physics.
/// If the slot were not vacated here, the stash grid would show a ghost card in a slot
/// that the player already picked up, and another card could be placed on top of it.
#[test]
fn when_pick_from_stash_intent_applied_then_drag_state_set_and_slot_vacated() {
    use card_game::stash::grid::StashGrid;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Stash {
                page: 0,
                col: 2,
                row: 3,
            },
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ))
        .id();
    let mut grid = StashGrid::new(4, 5, 1);
    grid.place(0, 2, 3, card_entity).unwrap();
    world.insert_resource(grid);
    insert_apply_resources(&mut world);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::PickFromStash {
            entity: card_entity,
            page: 0,
            col: 2,
            row: 3,
        });

    // Act
    run_system(&mut world);

    // Assert
    let drag = world
        .resource::<DragState>()
        .dragging
        .expect("DragState should be Some after applying PickFromStash");
    assert_eq!(drag.entity, card_entity);
    assert_eq!(
        drag.origin_zone,
        CardZone::Stash {
            page: 0,
            col: 2,
            row: 3,
        }
    );
    assert!(
        drag.stash_cursor_follow,
        "stash picks use cursor-follow mode, not physics"
    );
    assert!(
        world.resource::<StashGrid>().get(0, 2, 3).is_none(),
        "stash slot should be vacated by the applier"
    );
    assert!(
        world.resource::<EventBus<PhysicsCommand>>().is_empty(),
        "stash picks should not emit physics commands"
    );
}

/// @doc: Picking a Hand card transitions it to the table — the applier must remove it from
/// the Hand resource and emit `AddBody` physics commands so the card becomes a physics-driven
/// table entity. If the Hand removal is skipped, the hand layout system will still position
/// the card in the hand while the drag system moves it on the table, causing a visual tug-of-war.
#[test]
fn when_pick_card_hand_intent_applied_then_removed_from_hand_and_physics_body_added() {
    use card_game::hand::cards::Hand;
    use engine_scene::prelude::RenderLayer;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Hand(0),
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 200.0))),
            SortOrder::new(0),
            RenderLayer::UI,
        ))
        .id();
    let mut hand = Hand::new(10);
    hand.add(card_entity).unwrap();
    world.insert_resource(hand);
    insert_apply_resources(&mut world);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::PickCard {
            entity: card_entity,
            zone: CardZone::Hand(0),
            collider: default_card_collider(),
            grab_offset: Vec2::ZERO,
        });

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        !world.resource::<Hand>().cards().contains(&card_entity),
        "card must be removed from Hand resource"
    );
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let has_add_body = physics
        .iter()
        .any(|cmd| matches!(cmd, PhysicsCommand::AddBody { entity, .. } if *entity == card_entity));
    assert!(has_add_body, "must emit AddBody for the picked hand card");
}

// ── Release applier tests ───────────────────────────────────────

fn setup_active_drag(world: &mut World, entity: bevy_ecs::prelude::Entity) {
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

/// @doc: After a `ReleaseOnTable` intent, the applier must clear `DragState` and re-add the
/// card's physics body with normal collision groups so it participates in table collisions
/// again. If `DragState` is not cleared, the drag system will keep moving the card on
/// subsequent frames even though the player has released the mouse button.
#[test]
fn when_release_on_table_intent_applied_then_drag_state_cleared_and_physics_restored() {
    use engine_core::prelude::Transform2D;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
            SortOrder::new(0),
            Transform2D {
                position: Vec2::new(100.0, 200.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
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

    // Assert
    assert!(
        world.resource::<DragState>().dragging.is_none(),
        "DragState must be cleared after release"
    );
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let has_add_body = physics
        .iter()
        .any(|cmd| matches!(cmd, PhysicsCommand::AddBody { entity, .. } if *entity == card_entity));
    assert!(has_add_body, "must re-add physics body on table release");
}

/// @doc: When `snap_back` is true, the card must be teleported to its origin position before
/// re-adding physics. This handles the case where a stash drop failed (occupied slot) and
/// the card needs to return to where it was before the drag started. Without the position
/// reset, the card would remain at the cursor's last position with no valid zone.
#[test]
fn when_release_on_table_with_snap_back_then_position_restored() {
    use engine_core::prelude::Transform2D;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(300.0, 400.0))),
            SortOrder::new(0),
            Transform2D {
                position: Vec2::new(300.0, 400.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    insert_apply_resources(&mut world);
    setup_active_drag(&mut world, card_entity);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::ReleaseOnTable {
            entity: card_entity,
            snap_back: true,
        });

    // Act
    run_system(&mut world);
    world.flush();

    // Assert
    assert!(world.resource::<DragState>().dragging.is_none());
    let transform = world.entity(card_entity).get::<Transform2D>().unwrap();
    assert_eq!(
        transform.position,
        Vec2::new(50.0, 75.0),
        "position must be restored to origin_position from DragInfo"
    );
}

/// @doc: Releasing a card into the hand adds it to the Hand resource, removes its physics
/// body, and switches it to the UI render layer. If the card keeps its physics body after
/// entering the hand, table collisions will knock it around despite being in the hand
/// inventory, breaking the spatial layout.
#[test]
fn when_release_on_hand_intent_applied_then_card_added_to_hand_and_physics_removed() {
    use card_game::hand::cards::Hand;
    use engine_scene::prelude::RenderLayer;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
            RenderLayer::World,
        ))
        .id();
    world.insert_resource(Hand::new(10));
    insert_apply_resources(&mut world);
    setup_active_drag(&mut world, card_entity);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::ReleaseOnHand {
            entity: card_entity,
            face_up: true,
            origin_position: Vec2::ZERO,
        });

    // Act
    run_system(&mut world);
    world.flush();

    // Assert
    assert!(world.resource::<DragState>().dragging.is_none());
    assert!(world.resource::<Hand>().cards().contains(&card_entity));
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let has_remove = physics
        .iter()
        .any(|cmd| matches!(cmd, PhysicsCommand::RemoveBody { entity } if *entity == card_entity));
    assert!(has_remove, "must remove physics body when entering hand");
}

/// @doc: Face-down cards get a `FlipAnimation` when entering the hand so they auto-flip to
/// face-up. The hand always shows card faces. Without this, face-down cards in the hand
/// would display their back texture, making them unidentifiable.
#[test]
fn when_release_on_hand_face_down_then_flip_animation_inserted() {
    use card_game::card::interaction::flip_animation::FlipAnimation;
    use card_game::hand::cards::Hand;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ))
        .id();
    world.insert_resource(Hand::new(10));
    insert_apply_resources(&mut world);
    setup_active_drag(&mut world, card_entity);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::ReleaseOnHand {
            entity: card_entity,
            face_up: false,
            origin_position: Vec2::ZERO,
        });

    // Act
    run_system(&mut world);
    world.flush();

    // Assert
    assert!(
        world.entity(card_entity).get::<FlipAnimation>().is_some(),
        "face-down cards must get FlipAnimation when entering hand"
    );
}

/// @doc: When the hand is at full capacity, the release-on-hand intent falls back to
/// keeping the card on the table and snapping it to its origin position. If the applier
/// blindly tried to add the card to a full hand, the `Hand::add()` call would fail and the
/// card would be in limbo — not in the hand, not properly on the table.
#[test]
fn when_release_on_hand_but_hand_full_then_card_stays_on_table() {
    use card_game::hand::cards::Hand;
    use engine_core::prelude::Transform2D;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
            Transform2D {
                position: Vec2::new(200.0, 300.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let mut hand = Hand::new(1);
    let filler = world.spawn_empty().id();
    hand.add(filler).unwrap();
    world.insert_resource(hand);
    insert_apply_resources(&mut world);
    setup_active_drag(&mut world, card_entity);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::ReleaseOnHand {
            entity: card_entity,
            face_up: false,
            origin_position: Vec2::new(30.0, 40.0),
        });

    // Act
    run_system(&mut world);
    world.flush();

    // Assert
    assert!(world.resource::<DragState>().dragging.is_none());
    assert!(
        !world.resource::<Hand>().cards().contains(&card_entity),
        "card should not be in full hand"
    );
}

/// @doc: Releasing onto a stash slot places the card in the grid, removes physics, and
/// sets the stash zone. The grid placement is authoritative — only the applier writes to
/// `StashGrid` on release, preventing double-placement if multiple systems tried to claim
/// the same slot.
#[test]
fn when_release_on_stash_intent_applied_then_card_placed_in_grid() {
    use card_game::stash::grid::StashGrid;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ))
        .id();
    world.insert_resource(StashGrid::new(10, 10, 1));
    insert_apply_resources(&mut world);
    setup_active_drag(&mut world, card_entity);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::ReleaseOnStash {
            entity: card_entity,
            page: 0,
            col: 3,
            row: 2,
            current_position: Vec2::new(100.0, 200.0),
        });

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.resource::<DragState>().dragging.is_none());
    assert_eq!(
        world.resource::<StashGrid>().get(0, 3, 2),
        Some(&card_entity)
    );
    let physics: Vec<_> = world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .collect();
    let has_remove = physics
        .iter()
        .any(|cmd| matches!(cmd, PhysicsCommand::RemoveBody { entity } if *entity == card_entity));
    assert!(has_remove, "must remove physics body when entering stash");
}

/// @doc: When picking a Table card, `max_sort` must only consider other Table cards, not Hand cards.
/// If the zone filter is inverted (`== → !=`), a Hand card with a higher sort order would inflate
/// the picked card's `LocalSortOrder`, making it sort above all hand cards instead of above all
/// table cards — this breaks the table z-ordering contract whenever the hand is non-empty.
#[test]
fn when_picking_table_card_with_higher_sort_hand_card_present_then_local_sort_is_above_table_max() {
    // Arrange — table card at sort 5, hand card at sort 10; pick the table card
    let mut world = World::new();
    let table_card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(5),
        ))
        .id();
    world.spawn((
        card_game::test_helpers::make_test_card(),
        CardZone::Hand(0),
        SortOrder::new(10),
    ));
    insert_apply_resources(&mut world);
    world
        .resource_mut::<EventBus<InteractionIntent>>()
        .push(InteractionIntent::PickCard {
            entity: table_card,
            zone: CardZone::Table,
            collider: default_card_collider(),
            grab_offset: Vec2::ZERO,
        });

    // Act
    run_system(&mut world);

    // Assert — LocalSortOrder must be table_max + 1 = 6, not hand_max + 1 = 11
    let local_sort = world
        .entity(table_card)
        .get::<LocalSortOrder>()
        .expect("LocalSortOrder must be inserted on pick");
    assert_eq!(
        local_sort.0, 6,
        "sort order must be table max (5) + 1 = 6, not hand max (10) + 1 = 11"
    );
}
