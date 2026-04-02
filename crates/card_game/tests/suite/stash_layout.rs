#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Query, Res, Schedule, World};
use card_game::card::component::CardZone;
use card_game::stash::constants::{
    GRID_MARGIN, SLOT_HEIGHT, SLOT_STRIDE_H, SLOT_STRIDE_W, SLOT_WIDTH,
};
use card_game::stash::grid::StashGrid;
use card_game::stash::layout::stash_layout_system;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use engine_render::testing::SpyRenderer;
use glam::Vec2;
use std::sync::{Arc, Mutex};

fn make_world(viewport_w: u32, viewport_h: u32) -> World {
    let mut world = World::new();
    world.insert_resource(StashGrid::new(5, 5, 2));

    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_viewport(viewport_w, viewport_h);
    world.insert_resource(RendererRes::new(Box::new(spy)));

    world
}

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_layout_system);
    schedule.run(world);
}

#[test]
fn when_two_stash_cards_in_same_row_then_x_positions_differ_by_slot_stride() {
    // Arrange
    let mut world = make_world(1024, 768);
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    let card_a = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let card_b = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 0,
            },
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
    let xa = world.get::<Transform2D>(card_a).unwrap().position.x;
    let xb = world.get::<Transform2D>(card_b).unwrap().position.x;
    let dx = xb - xa;
    assert!(
        (dx - SLOT_STRIDE_W).abs() < 0.01,
        "expected x gap≈{SLOT_STRIDE_W}, got {dx}"
    );
}

#[test]
fn when_card_zone_is_table_then_transform_position_unchanged() {
    // Arrange
    let mut world = make_world(1024, 768);
    world.spawn(Camera2D::default());
    let sentinel = Vec2::new(123.0, 456.0);
    let table_card = world
        .spawn((
            CardZone::Table,
            Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    // Control: a Stash card that the system must move (proves the system ran)
    let stash_card = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: Vec2::new(999.0, 999.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let stash_t = world.get::<Transform2D>(stash_card).unwrap();
    assert_ne!(
        stash_t.position,
        Vec2::new(999.0, 999.0),
        "stash card should have been repositioned (proves system ran)"
    );
    let table_t = world.get::<Transform2D>(table_card).unwrap();
    assert_eq!(
        table_t.position, sentinel,
        "Table card must not be repositioned by stash_layout_system"
    );
}

#[test]
fn when_card_zone_is_hand_then_transform_position_unchanged() {
    // Arrange
    let mut world = make_world(1024, 768);
    world.spawn(Camera2D::default());
    let sentinel = Vec2::new(77.0, 88.0);
    let hand_card = world
        .spawn((
            CardZone::Hand(0),
            Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    // Control: a Stash card that the system must move (proves the system ran)
    let stash_card = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: Vec2::new(999.0, 999.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let stash_t = world.get::<Transform2D>(stash_card).unwrap();
    assert_ne!(
        stash_t.position,
        Vec2::new(999.0, 999.0),
        "stash card should have been repositioned (proves system ran)"
    );
    let hand_t = world.get::<Transform2D>(hand_card).unwrap();
    assert_eq!(
        hand_t.position, sentinel,
        "Hand card must not be repositioned by stash_layout_system"
    );
}

#[test]
fn when_no_camera_entity_then_system_does_not_panic_and_position_is_set() {
    // Arrange — no Camera2D spawned
    let mut world = make_world(1024, 768);
    let sentinel = Vec2::new(999.0, 999.0);
    let card = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let t = world.get::<Transform2D>(card).unwrap();
    let default_camera = Camera2D::default();
    let screen_x = GRID_MARGIN + SLOT_WIDTH * 0.5;
    let screen_y = GRID_MARGIN + SLOT_HEIGHT * 0.5;
    let expected = screen_to_world(
        Vec2::new(screen_x, screen_y),
        &default_camera,
        1024.0,
        768.0,
    );
    assert!(
        (t.position.x - expected.x).abs() < 0.01,
        "expected position.x≈{} with default camera, got {}",
        expected.x,
        t.position.x
    );
}

#[test]
fn when_viewport_is_zero_then_stash_card_position_is_not_mutated() {
    // Arrange — non-zero viewport world to confirm the system does move cards
    let mut normal_world = make_world(1024, 768);
    normal_world.spawn(Camera2D::default());
    let normal_card = normal_world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: Vec2::new(999.0, 999.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    run_system(&mut normal_world);
    let normal_pos = normal_world
        .get::<Transform2D>(normal_card)
        .unwrap()
        .position;

    // Arrange — zero viewport world
    let mut zero_world = make_world(0, 0);
    zero_world.spawn(Camera2D::default());
    let sentinel = Vec2::new(999.0, 999.0);
    let zero_card = zero_world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut zero_world);

    // Assert
    assert_ne!(
        normal_pos,
        Vec2::new(999.0, 999.0),
        "stash card must be moved in a normal viewport (proves system works)"
    );
    let zero_t = zero_world.get::<Transform2D>(zero_card).unwrap();
    assert_eq!(
        zero_t.position, sentinel,
        "zero viewport must not mutate stash card transform"
    );
}

#[test]
fn when_stash_card_at_col1_row2_then_transform_position_is_slot_center() {
    // Arrange
    let mut world = make_world(1024, 768);
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    let card = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 2,
            },
            Transform2D {
                position: Vec2::new(999.0, 999.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let camera = Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    };
    let center_screen = Vec2::new(
        GRID_MARGIN + SLOT_STRIDE_W + SLOT_WIDTH * 0.5,
        GRID_MARGIN + 2.0 * SLOT_STRIDE_H + SLOT_HEIGHT * 0.5,
    );
    let expected = screen_to_world(center_screen, &camera, 1024.0, 768.0);
    let t = world.get::<Transform2D>(card).unwrap();
    assert!(
        (t.position.x - expected.x).abs() < 0.01,
        "expected slot-center position.x≈{}, got {}",
        expected.x,
        t.position.x
    );
    assert!(
        (t.position.y - expected.y).abs() < 0.01,
        "expected slot-center position.y≈{}, got {}",
        expected.y,
        t.position.y
    );
}

#[test]
fn when_card_page_differs_from_current_page_then_transform_position_unchanged() {
    // Arrange — grid with 2 pages, current page stays at 0
    let mut world = make_world(1024, 768);
    world.spawn(Camera2D::default());

    let grid = StashGrid::new(5, 5, 2);
    world.insert_resource(grid);

    let sentinel = Vec2::new(33.0, 44.0);
    let page1_card = world
        .spawn((
            CardZone::Stash {
                page: 1,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    // Control: a page-0 card that the system must move (proves the system ran)
    let page0_card = world
        .spawn((
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Transform2D {
                position: Vec2::new(999.0, 999.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let p0_t = world.get::<Transform2D>(page0_card).unwrap();
    assert_ne!(
        p0_t.position,
        Vec2::new(999.0, 999.0),
        "page-0 card should have been repositioned (proves system ran)"
    );
    let p1_t = world.get::<Transform2D>(page1_card).unwrap();
    assert_eq!(
        p1_t.position, sentinel,
        "card on page 1 must not be repositioned when current_page is 0"
    );
}
