#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsCommand};
use engine_render::prelude::RendererRes;
use engine_render::testing::SpyRenderer;
use glam::Vec2;

use crate::test_helpers::make_test_card;
use card_game::card::component::CardZone;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::interaction::release::card_release_system;
use card_game::hand::cards::Hand;
use card_game::stash::grid::StashGrid;
use card_game::stash::toggle::StashVisible;

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_release_system);
    schedule.run(world);
}

/// Helper: builds a minimal world with a dragged entity in the given state.
fn build_world(
    screen_pos: Vec2,
    viewport_h: u32,
    stash_visible: bool,
    origin_zone: CardZone,
    grid: StashGrid,
) -> (World, Entity) {
    let mut world = World::new();
    world.insert_resource(EventBus::<InteractionIntent>::default());
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    let entity = world
        .spawn((
            make_test_card(),
            origin_zone,
            Collider::Aabb(Vec2::new(30.0, 45.0)),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Mouse with a recent left-button release at the given screen position.
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.release(MouseButton::Left);
    mouse.set_screen_pos(screen_pos);
    world.insert_resource(mouse);

    // Spy renderer for viewport dimensions.
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_viewport(800, viewport_h);
    world.insert_resource(RendererRes::new(Box::new(spy)));

    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });

    world.insert_resource(grid);
    world.insert_resource(StashVisible(stash_visible));
    world.insert_resource(Hand::new(10));

    (world, entity)
}

// ── Tests ──────────────────────────────────────────────────────────────────

/// @doc: When the stash is on the store page (page 0) and the cursor is inside the stash UI
/// region, the system emits a snap-back intent. Cards cannot be dropped into store-page
/// slots; they must return to their origin.
#[test]
fn when_store_page_and_cursor_in_stash_ui_then_snap_back_intent() {
    // Arrange — store page (page 0), cursor well inside stash UI bounds
    let mut grid = StashGrid::new(10, 10, 1);
    grid.set_current_page(0);
    let (mut world, entity) = build_world(
        Vec2::new(400.0, 400.0),
        600,
        true,
        CardZone::Table,
        grid,
    );

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one intent");
    match &intents[0] {
        InteractionIntent::ReleaseOnTable {
            entity: e,
            snap_back,
        } => {
            assert_eq!(*e, entity, "snap-back should reference the dragged entity");
            assert!(*snap_back, "store-page drop must snap back");
        }
        other => panic!("expected ReleaseOnTable with snap_back, got {other:?}"),
    }
}

/// @doc: When the stash is on the store page but the cursor is outside the stash UI
/// region, the store-page short-circuit does not trigger. The cursor position falls through
/// to the hand-zone / table check, producing a normal ReleaseOnTable (no snap-back).
#[test]
fn when_store_page_and_cursor_outside_stash_ui_then_table_intent() {
    // Arrange — store page, cursor at (10, 10) which is left of GRID_MARGIN (20)
    let mut grid = StashGrid::new(10, 10, 1);
    grid.set_current_page(0);
    let (mut world, _entity) = build_world(
        Vec2::new(10.0, 10.0),
        600,
        true,
        CardZone::Table,
        grid,
    );

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one intent");
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnTable { snap_back: false, .. }),
        "expected ReleaseOnTable (no snap-back) when cursor outside stash UI on store page, got {:?}",
        intents[0],
    );
}

/// @doc: When the stash is visible but the cursor is outside the grid slot boundaries,
/// `find_stash_slot_at` returns None. The system skips stash target resolution and falls
/// through to the hand-zone / table check, emitting a ReleaseOnTable.
#[test]
fn when_stash_visible_and_cursor_outside_grid_then_table_intent() {
    // Arrange — stash visible, cursor at x=600 which is past grid right edge (556)
    let (mut world, _entity) = build_world(
        Vec2::new(600.0, 100.0),
        600,
        true,
        CardZone::Table,
        StashGrid::new(10, 10, 2),
    );

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one intent");
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnTable { .. }),
        "expected ReleaseOnTable when cursor outside stash grid bounds, got {:?}",
        intents[0],
    );
}

/// @doc: When the cursor targets an occupied stash slot but the card's origin stash slot
/// is now empty, the system emits `ReleaseOnStash` back to the origin slot. This lets the
/// card return to its original grid position instead of snapping back to the table.
#[test]
fn when_target_slot_occupied_but_origin_slot_freed_then_stash_to_origin() {
    // Arrange — origin: Stash(0,1,1), cursor at slot (0,0,0) which is occupied
    let (mut world, entity) = build_world(
        Vec2::new(45.0, 57.0),
        600,
        true,
        CardZone::Stash {
            page: 0,
            col: 1,
            row: 1,
        },
        StashGrid::new(10, 10, 2),
    );
    // Occupy target slot (0,0,0) — origin slot (0,1,1) stays empty
    let blocker = world.spawn_empty().id();
    world
        .resource_mut::<StashGrid>()
        .place(0, 0, 0, blocker)
        .unwrap();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one intent");
    match &intents[0] {
        InteractionIntent::ReleaseOnStash {
            entity: e,
            page,
            col,
            row,
            ..
        } => {
            assert_eq!(*e, entity, "release should reference dragged entity");
            assert_eq!(*page, 0, "should release to origin page");
            assert_eq!(*col, 1, "should release to origin col");
            assert_eq!(*row, 1, "should release to origin row");
        }
        other => panic!(
            "expected ReleaseOnStash to origin slot (page=0,col=1,row=1), got {other:?}",
        ),
    }
}

/// @doc: The hand drop zone uses `>=` comparison: screen_y >= viewport_height - 120.
/// At the exact boundary, the cursor is inside the hand zone and the system must emit
/// `ReleaseOnHand`.
#[test]
fn when_released_at_exact_hand_zone_boundary_then_hand_intent() {
    // Arrange — hand zone starts at 600 - 120 = 480
    let viewport_h: u32 = 600;
    let threshold_y = viewport_h as f32 - 120.0;

    let (mut world, entity) = build_world(
        Vec2::new(400.0, threshold_y),
        viewport_h,
        false,
        CardZone::Table,
        StashGrid::new(10, 10, 1),
    );

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one intent");
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnHand { entity: e, .. } if *e == entity),
        "expected ReleaseOnHand at exact boundary y={threshold_y:.1}, got {:?}",
        intents[0],
    );
}

/// @doc: When the cursor is just above the hand-zone threshold (screen_y < viewport_height - 120),
/// the cursor is not in the hand drop zone. The system emits `ReleaseOnTable`.
#[test]
fn when_released_just_above_hand_zone_boundary_then_table_intent() {
    // Arrange — 0.1px above hand zone threshold
    let viewport_h: u32 = 600;
    let just_above = viewport_h as f32 - 120.0 - 0.1;

    let (mut world, _entity) = build_world(
        Vec2::new(400.0, just_above),
        viewport_h,
        false,
        CardZone::Table,
        StashGrid::new(10, 10, 1),
    );

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one intent");
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnTable { .. }),
        "expected ReleaseOnTable just above hand zone (y={just_above:.1}), got {:?}",
        intents[0],
    );
}
