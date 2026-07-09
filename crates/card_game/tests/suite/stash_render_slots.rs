#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::{Schedule, World};
use card_game::card::component::{Card, CardLabel};
use card_game::card::identity::signature::CardSignature;
use card_game::card::identity::signature_profile::SignatureProfile;
use card_game::card::identity::visual_params::generate_card_visuals;
use card_game::card::rendering::art_shader::CardArtShader;
use card_game::card::rendering::bake::{bake_back_face, bake_front_face};
use card_game::card::rendering::baked_mesh::BakedCardMesh;
use card_game::stash::constants::{
    GRID_MARGIN, SLOT_COLOR, SLOT_HEIGHT, SLOT_HIGHLIGHT_COLOR, SLOT_STRIDE_H, SLOT_STRIDE_W,
    SLOT_WIDTH,
};
use card_game::stash::grid::StashGrid;
use card_game::stash::render::stash_render_system;
use card_game::stash::toggle::StashVisible;
use engine_render::prelude::{Camera2D, RendererRes, ShaderHandle, screen_to_world};
use engine_render::testing::{ColoredMeshCallLog, ShapeCallLog};
use glam::Vec2;
use std::sync::{Arc, Mutex};

fn make_world_with_spy(grid: StashGrid, visible: bool) -> (World, ShapeCallLog) {
    let mut world = World::new();
    world.insert_resource(grid);
    world.insert_resource(StashVisible(visible));
    world.insert_resource(card_game::card::interaction::drag_state::DragState::default());
    world.insert_resource(engine_input::prelude::MouseState::default());

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));

    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    (world, shape_calls)
}

fn run_system(world: &mut World) {
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_render_system);
    schedule.run(world);
    // Drain the DrawQueue through unified_render to produce spy draw calls
    let mut render_schedule = Schedule::default();
    render_schedule.add_systems(engine_ui::unified_render::unified_render_system);
    render_schedule.run(world);
}

// --- Slot count by grid dimensions ---

/// @doc: A 1x1 grid draws exactly one slot shape — verifies minimal grid produces exactly two
/// shape calls (background + one slot).
#[test]
fn when_grid_1x1_then_one_slot_shape_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 1, 1), true);

    // Act
    run_system(&mut world);

    // Assert — calls[0] = background, calls[1] = the only slot
    let calls = shape_calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        2,
        "expected 1 bg + 1 slot = 2, got {}",
        calls.len()
    );
}

/// @doc: A 4x2 grid produces exactly 8 slot shapes — verifies wide grid slot count.
#[test]
fn when_grid_4x2_then_eight_slot_shapes() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(4, 2, 1), true);

    // Act
    run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        9,
        "expected 1 bg + 4*2 slots = 9, got {}",
        calls.len()
    );
}

/// @doc: A 2x4 grid produces exactly 8 slot shapes — verifies tall grid slot count.
#[test]
fn when_grid_2x4_then_eight_slot_shapes() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 4, 1), true);

    // Act
    run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        9,
        "expected 1 bg + 2*4 slots = 9, got {}",
        calls.len()
    );
}

// --- Occupied slot rendering ---

/// @doc: An occupied slot without baked mesh but with the card art shader resource draws `ART_QUAD`
/// vertices at the slot position — verifies the art-shader fallback path produces correct vertex
/// geometry (24.0 x 22.5 half-extents) and the card's art color.
#[test]
fn when_occupied_slot_without_baked_mesh_and_art_shader_then_art_quad_vertices() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(1, 1, 1));
    world.insert_resource(StashVisible(true));
    world.insert_resource(card_game::card::interaction::drag_state::DragState::default());
    world.insert_resource(engine_input::prelude::MouseState::default());
    world.insert_resource(CardArtShader(ShaderHandle(0)));

    let signature = CardSignature::default();
    let expected_color = {
        let profile = SignatureProfile::without_archetype(&signature);
        generate_card_visuals(&signature, &profile).art_color
    };
    let entity = world
        .spawn(Card {
            face_texture: engine_core::prelude::TextureId(0),
            back_texture: engine_core::prelude::TextureId(0),
            face_up: true,
            signature,
        })
        .id();

    let mut grid = StashGrid::new(1, 1, 1);
    grid.place(0, 0, 0, entity).unwrap();
    world.insert_resource(grid);

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — ART_QUAD half-extents: 24.0 x 22.5
    let calls = shape_calls.lock().unwrap();
    let slot_verts = &calls[1].0;
    assert!(
        (slot_verts[0][0] + 24.0).abs() < 1e-3,
        "expected ART_QUAD x min -24.0, got {}",
        slot_verts[0][0]
    );
    assert!(
        (slot_verts[1][0] - 24.0).abs() < 1e-3,
        "expected ART_QUAD x max 24.0, got {}",
        slot_verts[1][0]
    );
    assert!(
        (slot_verts[0][1] + 22.5).abs() < 1e-3,
        "expected ART_QUAD y min -22.5, got {}",
        slot_verts[0][1]
    );
    assert!(
        (slot_verts[3][1] - 22.5).abs() < 1e-3,
        "expected ART_QUAD y max 22.5, got {}",
        slot_verts[3][1]
    );
    assert_eq!(
        calls[1].2, expected_color,
        "occupied slot shape must use card's art color, not SLOT_COLOR"
    );
}

/// @doc: A slot holding an entity without the Card component falls through to `SLOT_COLOR` since
/// the entity won't be present in the `icon_colors` map built by `stash_render_system`.
#[test]
fn when_slot_holds_entity_without_card_then_fallback_to_slot_color() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(1, 1, 1));
    world.insert_resource(StashVisible(true));
    world.insert_resource(card_game::card::interaction::drag_state::DragState::default());
    world.insert_resource(engine_input::prelude::MouseState::default());

    let entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(1, 1, 1);
    grid.place(0, 0, 0, entity).unwrap();
    world.insert_resource(grid);

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — entity without Card won't appear in icon_colors, so SLOT_COLOR is used
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls[1].2, SLOT_COLOR);
}

/// @doc: Multiple occupied slots in a mixed-occupancy grid each render with their card's unique
/// art color while empty slots use `SLOT_COLOR` — verifies per-slot color independence.
#[test]
fn when_mixed_occupancy_grid_then_each_slot_has_correct_color() {
    // Arrange — 2x2 grid, three occupied and one empty
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 2, 1), true);

    let sig_a = CardSignature::new([0.1; 8]);
    let color_a = {
        let profile = SignatureProfile::without_archetype(&sig_a);
        generate_card_visuals(&sig_a, &profile).art_color
    };
    let e_a = world
        .spawn(Card {
            face_texture: engine_core::prelude::TextureId(0),
            back_texture: engine_core::prelude::TextureId(0),
            face_up: true,
            signature: sig_a,
        })
        .id();

    let sig_b = CardSignature::new([0.7; 8]);
    let color_b = {
        let profile = SignatureProfile::without_archetype(&sig_b);
        generate_card_visuals(&sig_b, &profile).art_color
    };
    let e_b = world
        .spawn(Card {
            face_texture: engine_core::prelude::TextureId(0),
            back_texture: engine_core::prelude::TextureId(0),
            face_up: true,
            signature: sig_b,
        })
        .id();

    let sig_c = CardSignature::new([-0.4; 8]);
    let color_c = {
        let profile = SignatureProfile::without_archetype(&sig_c);
        generate_card_visuals(&sig_c, &profile).art_color
    };
    let e_c = world
        .spawn(Card {
            face_texture: engine_core::prelude::TextureId(0),
            back_texture: engine_core::prelude::TextureId(0),
            face_up: true,
            signature: sig_c,
        })
        .id();

    let mut grid = StashGrid::new(2, 2, 1);
    grid.place(0, 0, 0, e_a).unwrap();
    grid.place(0, 0, 1, e_b).unwrap();
    grid.place(0, 1, 0, e_c).unwrap();
    // slot (1,1) stays empty
    world.insert_resource(grid);

    // Act
    run_system(&mut world);

    // Assert
    // Layout: calls[0]=bg, [1]=(0,0), [2]=(0,1), [3]=(1,0), [4]=(1,1)
    let calls = shape_calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        5,
        "expected 1 bg + 4 slots = 5, got {}",
        calls.len()
    );
    assert_eq!(
        calls[1].2, color_a,
        "slot (0,0) must show card A's art color"
    );
    assert_eq!(
        calls[2].2, color_b,
        "slot (0,1) must show card B's art color"
    );
    assert_eq!(
        calls[3].2, color_c,
        "slot (1,0) must show card C's art color"
    );
    assert_eq!(
        calls[4].2, SLOT_COLOR,
        "empty slot (1,1) must use SLOT_COLOR"
    );
}

/// @doc: Empty slots use `UNIT_QUAD` vertices (+/-0.5) even when an art shader resource exists,
/// because empty slots have no card body to apply the art shader to — the art shader path only
/// activates for occupied slots without baked meshes.
#[test]
fn when_empty_slots_with_art_shader_then_still_use_unit_quad() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(1, 1, 1));
    world.insert_resource(StashVisible(true));
    world.insert_resource(card_game::card::interaction::drag_state::DragState::default());
    world.insert_resource(engine_input::prelude::MouseState::default());
    world.insert_resource(CardArtShader(ShaderHandle(0)));

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — empty slots always use UNIT_QUAD regardless of art shader presence
    let calls = shape_calls.lock().unwrap();
    let slot_verts = &calls[1].0;
    assert!(
        (slot_verts[0][0] + 0.5).abs() < 1e-4,
        "expected UNIT_QUAD x min -0.5, got {}",
        slot_verts[0][0]
    );
    assert!(
        (slot_verts[1][0] - 0.5).abs() < 1e-4,
        "expected UNIT_QUAD x max 0.5, got {}",
        slot_verts[1][0]
    );
    assert_eq!(calls[1].2, SLOT_COLOR, "empty slot must use SLOT_COLOR");
}

// --- Baked mesh priority ---

/// @doc: When a slot holds a card with `BakedCardMesh` AND the card art shader resource is
/// available, the baked mesh path takes priority — only a colored mesh draw call is emitted,
/// not a shape draw call.
#[test]
fn when_baked_mesh_and_art_shader_both_present_then_colored_mesh_used() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(1, 1, 1));
    world.insert_resource(StashVisible(true));
    world.insert_resource(card_game::card::interaction::drag_state::DragState::default());
    world.insert_resource(engine_input::prelude::MouseState::default());
    world.insert_resource(CardArtShader(ShaderHandle(0)));

    let sig = CardSignature::default();
    let label = CardLabel {
        name: "Test".to_owned(),
        description: "Desc".to_owned(),
    };
    let size = Vec2::new(60.0, 90.0);
    let baked = BakedCardMesh {
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
            baked,
        ))
        .id();

    let mut grid = StashGrid::new(1, 1, 1);
    grid.place(0, 0, 0, entity).unwrap();
    world.insert_resource(grid);

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_colored_mesh_capture(colored_mesh_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — baked mesh path produces a colored mesh call, not a shape call for the slot
    assert!(
        !colored_mesh_calls.lock().unwrap().is_empty(),
        "expected draw_colored_mesh for occupied slot with BakedCardMesh"
    );
    // Only shape call is the background
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        1,
        "expected exactly 1 shape (background), slot must use colored mesh"
    );
}

// --- Highlight behavior ---

/// @doc: During a drag operation, empty slots under the cursor receive `SLOT_HIGHLIGHT_COLOR`
/// instead of `SLOT_COLOR` — provides visual feedback for where the card will land.
#[test]
fn when_dragging_over_empty_slot_then_highlight_color_applied() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(2, 2, 1));
    world.insert_resource(StashVisible(true));

    let entity = world.spawn_empty().id();
    world.insert_resource(card_game::card::interaction::drag_state::DragState {
        dragging: Some(card_game::card::interaction::drag_state::DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: card_game::card::component::CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });

    let mut mouse = engine_input::prelude::MouseState::default();
    mouse.set_screen_pos(Vec2::new(
        GRID_MARGIN + SLOT_WIDTH * 0.5,
        GRID_MARGIN + SLOT_HEIGHT * 0.5,
    ));
    world.insert_resource(mouse);

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — slot (0,0) has highlight color
    let calls = shape_calls.lock().unwrap();
    // calls[0]=bg, [1]=slot(0,0), [2]=slot(1,0), [3]=slot(0,1), [4]=slot(1,1)
    assert_eq!(calls[1].2, SLOT_HIGHLIGHT_COLOR);
}

/// @doc: Occupied slots ignore the drag highlight — the highlight-only-empty-slots logic
/// prevents the user from mistakenly thinking they can drop onto an already-occupied cell.
#[test]
fn when_dragging_over_occupied_slot_then_no_highlight() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashGrid::new(1, 1, 1));
    world.insert_resource(StashVisible(true));

    let drag_entity = world.spawn_empty().id();
    world.insert_resource(card_game::card::interaction::drag_state::DragState {
        dragging: Some(card_game::card::interaction::drag_state::DragInfo {
            entity: drag_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: card_game::card::component::CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });

    let mut mouse = engine_input::prelude::MouseState::default();
    mouse.set_screen_pos(Vec2::new(
        GRID_MARGIN + SLOT_WIDTH * 0.5,
        GRID_MARGIN + SLOT_HEIGHT * 0.5,
    ));
    world.insert_resource(mouse);

    // Place a card in slot (0,0) so it's occupied
    let slot_entity = world
        .spawn(Card {
            face_texture: engine_core::prelude::TextureId(0),
            back_texture: engine_core::prelude::TextureId(0),
            face_up: true,
            signature: CardSignature::default(),
        })
        .id();
    let mut grid = StashGrid::new(1, 1, 1);
    grid.place(0, 0, 0, slot_entity).unwrap();
    world.insert_resource(grid);

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Act
    run_system(&mut world);

    // Assert — occupied slot does NOT use highlight color
    let calls = shape_calls.lock().unwrap();
    assert!(
        calls[1].2 != SLOT_HIGHLIGHT_COLOR,
        "occupied slot must not show highlight color"
    );
}

// --- Slot position / layout ---

/// @doc: Slot (col=0, row=0) center world position matches `GRID_MARGIN` + half-slot offset
/// translated through `screen_to_world` — verifies the grid origin slot is placed at the
/// correct screen position.
#[test]
fn when_slot_zero_zero_then_world_position_matches_grid_origin() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 2, 1), true);

    // Act
    run_system(&mut world);

    // Assert — slot (0,0) center: (GRID_MARGIN + SLOT_WIDTH/2, GRID_MARGIN + SLOT_HEIGHT/2)
    let expected = screen_to_world(
        Vec2::new(
            GRID_MARGIN + SLOT_WIDTH * 0.5,
            GRID_MARGIN + SLOT_HEIGHT * 0.5,
        ),
        &Camera2D::default(),
        1024.0,
        768.0,
    );
    let calls = shape_calls.lock().unwrap();
    let model = &calls[1].3;
    assert!(
        (model[3][0] - expected.x).abs() < 0.01,
        "slot (0,0) tx: got {}, expected {}",
        model[3][0],
        expected.x
    );
    assert!(
        (model[3][1] - expected.y).abs() < 0.01,
        "slot (0,0) ty: got {}, expected {}",
        model[3][1],
        expected.y
    );
}

/// @doc: Slot (col=1, row=0) is one `SLOT_STRIDE_W` to the right of slot (0,0) — verifies
/// horizontal grid stepping is consistent across columns.
#[test]
fn when_two_slots_in_same_row_then_x_spacing_equals_stride() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 1, 1), true);

    // Act
    run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    let tx0 = calls[1].3[3][0];
    let tx1 = calls[2].3[3][0];
    let dx = tx1 - tx0;
    assert!(
        (dx - SLOT_STRIDE_W).abs() < 0.01,
        "expected horizontal stride {SLOT_STRIDE_W}, got {dx}"
    );
}

/// @doc: Slot (col=0, row=1) is one `SLOT_STRIDE_H` below slot (0,0) — verifies vertical
/// grid stepping is consistent across rows.
#[test]
fn when_two_slots_in_same_column_then_y_spacing_equals_stride() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 2, 1), true);

    // Act
    run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    let ty0 = calls[1].3[3][1];
    let ty1 = calls[2].3[3][1];
    let dy = ty1 - ty0;
    assert!(
        (dy - SLOT_STRIDE_H).abs() < 0.01,
        "expected vertical stride {SLOT_STRIDE_H}, got {dy}"
    );
}
