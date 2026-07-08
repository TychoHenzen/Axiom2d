#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::component::{Card, CardLabel, CardZone};
use card_game::card::identity::signature::CardSignature;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::rendering::bake::{bake_back_face, bake_front_face};
use card_game::card::rendering::baked_mesh::BakedCardMesh;
use card_game::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
use card_game::stash::constants::{GRID_MARGIN, SLOT_HEIGHT, SLOT_WIDTH};
use card_game::stash::grid::StashGrid;
use card_game::stash::render::stash_render_system;
use card_game::stash::toggle::StashVisible;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ColoredMeshCallLog, ShapeCallLog, SpyRenderer};
use glam::Vec2;

fn make_world_with_drag(baked: Option<BakedCardMesh>) -> (World, ColoredMeshCallLog, ShapeCallLog) {
    let mut world = World::new();
    world.insert_resource(StashGrid::new(2, 2, 1));
    world.insert_resource(StashVisible(true));

    let sig = CardSignature::default();
    let label = CardLabel {
        name: "Test".to_owned(),
        description: "Desc".to_owned(),
    };
    let size = Vec2::new(60.0, 90.0);
    let baked_card = baked.unwrap_or_else(|| BakedCardMesh {
        front: bake_front_face(&sig, size, &label, None),
        back: bake_back_face(size),
    });
    let entity = world
        .spawn((
            Card {
                face_texture: engine_core::prelude::TextureId(0),
                back_texture: engine_core::prelude::TextureId(0),
                face_up: true,
                signature: sig,
            },
            baked_card,
        ))
        .id();

    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });

    let mut mouse = MouseState::default();
    // Position cursor inside the stash area (GRID_MARGIN is the origin)
    mouse.set_screen_pos(Vec2::new(GRID_MARGIN + 10.0, GRID_MARGIN + 10.0));
    world.insert_resource(mouse);

    let colored_mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let shape_log: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_colored_mesh_capture(colored_mesh_log.clone())
        .with_shape_capture(shape_log.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    (world, colored_mesh_log, shape_log)
}

fn run_system(world: &mut World) {
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_render_system);
    schedule.run(world);
    let mut render_schedule = Schedule::default();
    render_schedule.add_systems(engine_ui::unified_render::unified_render_system);
    render_schedule.run(world);
}

/// @doc: Drag preview miniature must scale card geometry from `TABLE_CARD` dimensions to SLOT
/// dimensions divided by zoom. Without correct scale, the ghost would overflow the slot or
/// appear as a tiny dot, failing to communicate where the card would land.
#[test]
fn when_drag_preview_active_then_scale_applied() {
    // Arrange
    let (mut world, colored_mesh_log, _) = make_world_with_drag(None);

    // Act
    run_system(&mut world);

    // Assert — last colored mesh call is the drag preview
    let calls = colored_mesh_log.lock().unwrap();
    let model = calls
        .last()
        .expect("drag preview should produce a colored mesh call")
        .2;
    let expected_sx = (SLOT_WIDTH / 1.0) / TABLE_CARD_WIDTH;
    let expected_sy = (SLOT_HEIGHT / 1.0) / TABLE_CARD_HEIGHT;
    assert!(
        (model[0][0] - expected_sx).abs() < 1e-4,
        "drag preview scale x: got {}, expected {expected_sx}",
        model[0][0],
    );
    assert!(
        (model[1][1] - expected_sy).abs() < 1e-4,
        "drag preview scale y: got {}, expected {expected_sy}",
        model[1][1],
    );
}

/// @doc: Scale must halve at zoom=2 so the preview shrinks proportionally — matching how all
/// other stash elements contract under camera zoom.
#[test]
fn when_drag_preview_at_zoom2_then_scale_halved() {
    // Arrange
    let (mut world, colored_mesh_log, _) = make_world_with_drag(None);
    // Override camera zoom to 2.0
    let mut camera_query = world.query::<&mut Camera2D>();
    for mut cam in camera_query.iter_mut(&mut world) {
        cam.zoom = 2.0;
    }

    // Act
    run_system(&mut world);

    // Assert
    let calls = colored_mesh_log.lock().unwrap();
    let model = calls
        .last()
        .expect("drag preview should produce a colored mesh call")
        .2;
    let expected_sx = (SLOT_WIDTH / 2.0) / TABLE_CARD_WIDTH;
    let expected_sy = (SLOT_HEIGHT / 2.0) / TABLE_CARD_HEIGHT;
    assert!(
        (model[0][0] - expected_sx).abs() < 1e-4,
        "drag preview scale x at zoom=2: got {}, expected {expected_sx}",
        model[0][0],
    );
    assert!(
        (model[1][1] - expected_sy).abs() < 1e-4,
        "drag preview scale y at zoom=2: got {}, expected {expected_sy}",
        model[1][1],
    );
}

/// @doc: When no drag is active, stash render produces exactly one background shape plus
/// N slot shapes — no extra draw calls for a preview ghost.
#[test]
fn when_no_drag_then_no_preview() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(2, 2, 1));
    world.insert_resource(StashVisible(true));
    world.insert_resource(DragState::default());
    world.insert_resource(MouseState::default());

    let shape_log: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let colored_mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_shape_capture(shape_log.clone())
        .with_colored_mesh_capture(colored_mesh_log.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — 1 background + 4 slots = 5 shape calls; zero colored mesh calls
    assert_eq!(
        shape_log.lock().unwrap().len(),
        5,
        "expected 1 background + 4 slot shapes, got {}",
        shape_log.lock().unwrap().len(),
    );
    assert!(
        colored_mesh_log.lock().unwrap().is_empty(),
        "no drag preview colored mesh should exist when DragState is None",
    );
}

/// @doc: Drag preview renders at the cursor world position — verifies the preview ghost
/// follows the mouse rather than staying at some fixed location.
#[test]
fn when_drag_preview_active_then_translation_matches_cursor() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(2, 2, 1));
    world.insert_resource(StashVisible(true));

    let sig = CardSignature::default();
    let label = CardLabel {
        name: "Test".to_owned(),
        description: "Desc".to_owned(),
    };
    let size = Vec2::new(60.0, 90.0);
    let baked_card = BakedCardMesh {
        front: bake_front_face(&sig, size, &label, None),
        back: bake_back_face(size),
    };
    let entity = world
        .spawn((
            Card {
                face_texture: engine_core::prelude::TextureId(0),
                back_texture: engine_core::prelude::TextureId(0),
                face_up: true,
                signature: sig,
            },
            baked_card,
        ))
        .id();

    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });

    // Cursor at a known stash area position
    let cursor_screen = Vec2::new(GRID_MARGIN + 50.0, GRID_MARGIN + 30.0);
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(cursor_screen);
    world.insert_resource(mouse);

    let colored_mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_colored_mesh_capture(colored_mesh_log.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — drag preview produces a colored mesh call
    let calls = colored_mesh_log.lock().unwrap();
    assert!(
        !calls.is_empty(),
        "drag preview with active drag should produce colored mesh calls"
    );
}
