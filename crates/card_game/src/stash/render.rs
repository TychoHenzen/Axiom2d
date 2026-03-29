use std::collections::HashMap;

use bevy_ecs::prelude::{Entity, Query, Res, ResMut};
use bevy_ecs::system::SystemParam;
use engine_core::color::Color;
use engine_render::prelude::{
    BlendMode, Camera2D, QUAD_INDICES, RendererRes, ShaderHandle, UNIT_QUAD, rect_vertices,
    screen_to_world, unit_quad_model,
};
use glam::Vec2;

use crate::card::component::Card;
use crate::card::identity::signature_profile::SignatureProfile;
use crate::card::identity::visual_params::generate_card_visuals;
use crate::card::rendering::baked_mesh::BakedCardMesh;
use crate::card::rendering::geometry::{
    ART_QUAD, TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH, art_quad_model,
};
use crate::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_COLOR, SLOT_GAP, SLOT_HEIGHT, SLOT_HIGHLIGHT_COLOR,
    SLOT_STRIDE_H, SLOT_STRIDE_W, SLOT_WIDTH,
};
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;
use engine_render::prelude::resolve_viewport_camera;

#[derive(SystemParam)]
pub struct StashRenderParams<'w> {
    grid: Res<'w, StashGrid>,
    visible: Res<'w, StashVisible>,
    drag_state: Res<'w, crate::card::interaction::drag_state::DragState>,
    mouse: Res<'w, engine_input::prelude::MouseState>,
    art_shader: Option<Res<'w, crate::card::rendering::art_shader::CardArtShader>>,
}

pub(crate) fn reset_default_shader(renderer: &mut dyn engine_render::prelude::Renderer) {
    renderer.set_shader(ShaderHandle(0));
    renderer.set_blend_mode(BlendMode::Alpha);
}

fn miniature_card_model(zoom: f32, center_x: f32, center_y: f32) -> [[f32; 4]; 4] {
    let scale_x = (SLOT_WIDTH / zoom) / TABLE_CARD_WIDTH;
    let scale_y = (SLOT_HEIGHT / zoom) / TABLE_CARD_HEIGHT;
    [
        [scale_x, 0.0, 0.0, 0.0],
        [0.0, scale_y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [center_x, center_y, 0.0, 1.0],
    ]
}

pub fn stash_render_system(
    params: StashRenderParams,
    card_query: Query<(Entity, &Card, Option<&BakedCardMesh>)>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    let renderer_art_shader = params.art_shader.map(|s| s.0);
    if !params.visible.0 {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    reset_default_shader(&mut **renderer);
    let world_slot_w = SLOT_WIDTH / camera.zoom;
    let world_slot_h = SLOT_HEIGHT / camera.zoom;

    let bg_screen_w = f32::from(params.grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
    let bg_screen_h = f32::from(params.grid.height()) * SLOT_STRIDE_H - SLOT_GAP;
    let bg_origin = screen_to_world(Vec2::new(GRID_MARGIN, GRID_MARGIN), &camera, vw, vh);
    let bg_verts = rect_vertices(
        bg_origin.x,
        bg_origin.y,
        bg_screen_w / camera.zoom,
        bg_screen_h / camera.zoom,
    );
    renderer.draw_shape(
        &bg_verts,
        &QUAD_INDICES,
        BACKGROUND_COLOR,
        engine_render::prelude::IDENTITY_MODEL,
    );

    let icon_colors: HashMap<Entity, Color> = card_query
        .iter()
        .map(|(entity, card, _baked)| {
            let profile = SignatureProfile::without_archetype(&card.signature);
            (
                entity,
                generate_card_visuals(&card.signature, &profile).art_color,
            )
        })
        .collect();

    let baked_meshes: HashMap<Entity, &BakedCardMesh> = card_query
        .iter()
        .filter_map(|(entity, _card, baked)| baked.map(|b| (entity, b)))
        .collect();

    let page = params.grid.current_page();

    let highlight_slot = if params.drag_state.dragging.is_some() {
        find_stash_slot_at(
            params.mouse.screen_pos(),
            params.grid.width(),
            params.grid.height(),
        )
    } else {
        None
    };

    for col in 0..params.grid.width() {
        for row in 0..params.grid.height() {
            let screen_x = GRID_MARGIN + f32::from(col) * SLOT_STRIDE_W;
            let screen_y = GRID_MARGIN + f32::from(row) * SLOT_STRIDE_H;
            let center = screen_to_world(
                Vec2::new(screen_x + SLOT_WIDTH * 0.5, screen_y + SLOT_HEIGHT * 0.5),
                &camera,
                vw,
                vh,
            );

            if let Some(&entity) = params.grid.get(page, col, row) {
                if let Some(baked) = baked_meshes.get(&entity) {
                    let model = miniature_card_model(camera.zoom, center.x, center.y);
                    renderer.set_shader(ShaderHandle(0));
                    renderer.draw_colored_mesh(&baked.front.vertices, &baked.front.indices, model);
                } else {
                    let color = icon_colors.get(&entity).copied().unwrap_or(SLOT_COLOR);
                    if let Some(art) = renderer_art_shader {
                        renderer.set_shader(art);
                        let model = art_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                        renderer.draw_shape(&ART_QUAD, &QUAD_INDICES, color, model);
                    } else {
                        let model = unit_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                        renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, color, model);
                    }
                    renderer.set_shader(ShaderHandle(0));
                }
            } else {
                let slot_color = if highlight_slot == Some((col, row)) {
                    SLOT_HIGHLIGHT_COLOR
                } else {
                    SLOT_COLOR
                };
                let model = unit_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, slot_color, model);
            }
        }
    }

    // Draw the dragged card's icon on top of the stash grid at the cursor position
    if let Some(info) = params.drag_state.dragging {
        let screen = params.mouse.screen_pos();
        let bg_x_max = GRID_MARGIN + f32::from(params.grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
        let bg_y_max = GRID_MARGIN + f32::from(params.grid.height()) * SLOT_STRIDE_H - SLOT_GAP;
        let over_stash_area = screen.x >= GRID_MARGIN
            && screen.x < bg_x_max
            && screen.y >= GRID_MARGIN
            && screen.y < bg_y_max;

        if over_stash_area {
            let cursor_world = params.mouse.world_pos();
            if let Some(baked) = baked_meshes.get(&info.entity) {
                let model = miniature_card_model(camera.zoom, cursor_world.x, cursor_world.y);
                renderer.set_shader(ShaderHandle(0));
                renderer.draw_colored_mesh(&baked.front.vertices, &baked.front.indices, model);
            } else {
                let color = icon_colors.get(&info.entity).copied().unwrap_or(SLOT_COLOR);
                if let Some(art) = renderer_art_shader {
                    renderer.set_shader(art);
                    let model =
                        art_quad_model(world_slot_w, world_slot_h, cursor_world.x, cursor_world.y);
                    renderer.draw_shape(&ART_QUAD, &QUAD_INDICES, color, model);
                } else {
                    let model =
                        unit_quad_model(world_slot_w, world_slot_h, cursor_world.x, cursor_world.y);
                    renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, color, model);
                }
                renderer.set_shader(ShaderHandle(0));
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};
    use engine_render::testing::{ShapeCallLog, SpyRenderer};
    use std::sync::{Arc, Mutex};

    fn make_world_with_spy(grid: StashGrid, visible: bool) -> (World, ShapeCallLog) {
        let mut world = World::new();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(visible));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
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
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_render_system);
        schedule.run(world);
    }

    #[test]
    fn when_hidden_then_no_shapes_drawn() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(3, 3, 1), false);

        // Act
        run_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn when_visible_and_empty_grid_then_draws_width_times_height_shapes() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(4, 3, 1), true);

        // Act
        run_system(&mut world);

        // Assert — 1 background + 4*3 slots = 13
        assert_eq!(shape_calls.lock().unwrap().len(), 13);
    }

    #[test]
    fn when_visible_and_empty_slot_then_drawn_with_slot_color() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 1, 1), true);

        // Act
        run_system(&mut world);

        // Assert — calls[0] is background, calls[1] is the first (only) slot
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[1].2, SLOT_COLOR);
    }

    #[test]
    fn when_visible_and_occupied_slot_then_drawn_with_card_art_color() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 1, 1), true);

        let signature = crate::card::identity::signature::CardSignature::default();
        let expected_color = {
            let profile =
                crate::card::identity::signature_profile::SignatureProfile::without_archetype(
                    &signature,
                );
            crate::card::identity::visual_params::generate_card_visuals(&signature, &profile)
                .art_color
        };
        let root = world
            .spawn(crate::card::component::Card {
                face_texture: engine_core::prelude::TextureId(0),
                back_texture: engine_core::prelude::TextureId(0),
                face_up: false,
                signature,
            })
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, root).unwrap();
        world.insert_resource(grid);

        // Act
        run_system(&mut world);

        // Assert — calls[0] is background, calls[1] is the only slot
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[1].2, expected_color);
    }

    /// @doc: Column spacing must be precisely `SLOT_STRIDE_W` — grid visual layout alignment depends on correct X offsets.
    #[test]
    fn when_visible_then_adjacent_columns_differ_by_slot_stride_in_x() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 1, 1), true);

        // Act
        run_system(&mut world);

        // Assert — calls[0] is background; calls[1] and calls[2] are the two column slots
        // Stride is measured via model matrix translation (tx = model[3][0])
        let calls = shape_calls.lock().unwrap();
        let tx0 = calls[1].3[3][0];
        let tx1 = calls[2].3[3][0];
        let dx = tx1 - tx0;
        assert!(
            (dx - SLOT_STRIDE_W).abs() < 0.01,
            "expected stride {SLOT_STRIDE_W}, got {dx}"
        );
    }

    /// @doc: Row spacing must be precisely `SLOT_STRIDE_H` — ensures grid rows stack without overlap or gaps.
    #[test]
    fn when_visible_then_adjacent_rows_differ_by_slot_stride_in_y() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 2, 1), true);

        // Act
        run_system(&mut world);

        // Assert — calls[0] is background; calls[1] and calls[2] are the two row slots
        // Stride is measured via model matrix translation (ty = model[3][1])
        let calls = shape_calls.lock().unwrap();
        let ty0 = calls[1].3[3][1];
        let ty1 = calls[2].3[3][1];
        let dy = ty1 - ty0;
        assert!(
            (dy - SLOT_STRIDE_H).abs() < 0.01,
            "expected stride {SLOT_STRIDE_H}, got {dy}"
        );
    }

    /// @doc: Background shape must span the entire grid extent — prevents visual gaps between grid background and slot edges.
    #[test]
    fn when_visible_then_first_shape_covers_full_grid_area() {
        // Arrange — 2x2 grid so grid bounds are deterministic
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 2, 1), true);

        // Act
        run_system(&mut world);

        // Assert — calls[0] is the background; its width must span the entire slot grid
        let calls = shape_calls.lock().unwrap();
        assert!(
            !calls.is_empty(),
            "expected at least one draw call for the background"
        );
        let bg_verts = &calls[0].0;
        // vertex layout: [0]=top-left, [1]=top-right
        let bg_width_world = bg_verts[1][0] - bg_verts[0][0];
        let grid_span_screen = 2.0 * SLOT_STRIDE_W - SLOT_GAP;
        assert!(
            bg_width_world >= grid_span_screen,
            "background width {bg_width_world} should cover grid span {grid_span_screen}"
        );
    }

    #[test]
    fn when_slot_strides_then_equal_slot_dimension_plus_gap() {
        // Assert — SLOT_STRIDE_W and SLOT_STRIDE_H must be sums, not differences or products
        assert!(
            (SLOT_STRIDE_W - (SLOT_WIDTH + SLOT_GAP)).abs() < 1e-6,
            "SLOT_STRIDE_W={SLOT_STRIDE_W}, expected {}",
            SLOT_WIDTH + SLOT_GAP
        );
        assert!(
            (SLOT_STRIDE_H - (SLOT_HEIGHT + SLOT_GAP)).abs() < 1e-6,
            "SLOT_STRIDE_H={SLOT_STRIDE_H}, expected {}",
            SLOT_HEIGHT + SLOT_GAP
        );
    }

    #[test]
    fn when_viewport_width_zero_then_no_shapes_drawn() {
        // Arrange — viewport (0, 768) should trigger early return via || guard
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(0, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn when_viewport_height_zero_then_no_shapes_drawn() {
        // Arrange — viewport (1024, 0) should trigger early return via || guard
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 0);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }

    fn slot_vertex_span(zoom: f32) -> (f32, f32) {
        let mut world = World::new();
        world.insert_resource(StashGrid::new(1, 1, 1));
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom,
        });

        run_system(&mut world);

        let calls = shape_calls.lock().unwrap();
        let verts = &calls[1].0; // calls[0] = background, calls[1] = slot
        let width = verts[1][0] - verts[0][0];
        let height = verts[3][1] - verts[0][1];
        (width, height)
    }

    #[test]
    fn when_stash_rendered_at_any_zoom_then_slot_vertices_are_unit_quad() {
        // Assert — at all zoom levels, local vertices span 1.0 (normalized unit quad)
        for zoom in [1.0, 2.0, 0.5] {
            let (w, h) = slot_vertex_span(zoom);
            assert!(
                (w - 1.0).abs() < 1e-4,
                "zoom={zoom}: vertex width={w}, expected 1.0"
            );
            assert!(
                (h - 1.0).abs() < 1e-4,
                "zoom={zoom}: vertex height={h}, expected 1.0"
            );
        }
    }

    #[test]
    fn when_stash_rendered_then_model_matrix_scale_equals_slot_size_over_zoom() {
        // Arrange
        let zoom = 2.0_f32;
        let mut world = World::new();
        world.insert_resource(StashGrid::new(1, 1, 1));
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom,
        });

        // Act
        run_system(&mut world);

        // Assert — model scale is directly world_slot_size (unit quad normalized)
        let calls = shape_calls.lock().unwrap();
        let model = &calls[1].3;
        let expected_sx = SLOT_WIDTH / zoom;
        let expected_sy = SLOT_HEIGHT / zoom;
        assert!(
            (model[0][0] - expected_sx).abs() < 1e-4,
            "scale x: got {}, expected {expected_sx}",
            model[0][0]
        );
        assert!(
            (model[1][1] - expected_sy).abs() < 1e-4,
            "scale y: got {}, expected {expected_sy}",
            model[1][1]
        );
    }

    /// @doc: Slot translation must map screen-space center to world coords — validates camera integration in grid rendering.
    #[test]
    fn when_stash_rendered_then_model_translation_matches_slot_center() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 1, 1), true);

        // Act
        run_system(&mut world);

        // Assert — slot center in screen space is (GRID_MARGIN + SLOT_WIDTH/2, GRID_MARGIN + SLOT_HEIGHT/2)
        // At zoom=1 camera at origin, screen_to_world maps that to world coords
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
            "tx: got {}, expected {}",
            model[3][0],
            expected.x
        );
        assert!(
            (model[3][1] - expected.y).abs() < 0.01,
            "ty: got {}, expected {}",
            model[3][1],
            expected.y
        );
    }

    /// @doc: Drop target slots highlight when dragging — visual feedback shows where card will land.
    #[test]
    fn when_dragging_over_empty_slot_then_slot_drawn_with_highlight_color() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));

        let entity = world.spawn_empty().id();
        let drag_info = crate::card::interaction::drag_state::DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: crate::card::component::CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        };
        world.insert_resource(crate::card::interaction::drag_state::DragState {
            dragging: Some(drag_info),
        });

        // Position mouse over slot (0,0) center: (GRID_MARGIN + SLOT_WIDTH/2, GRID_MARGIN + SLOT_HEIGHT/2)
        let mut mouse = engine_input::prelude::MouseState::default();
        mouse.set_screen_pos(Vec2::new(
            GRID_MARGIN + SLOT_WIDTH * 0.5,
            GRID_MARGIN + SLOT_HEIGHT * 0.5,
        ));
        world.insert_resource(mouse);

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert — calls[0] = background, calls[1] = slot (0,0) which should be highlighted
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[1].2, SLOT_HIGHLIGHT_COLOR);
    }

    /// @doc: Highlight only renders during drag — ensures slots stay neutral-colored at rest even with cursor over them.
    #[test]
    fn when_no_drag_and_cursor_over_slot_then_slot_drawn_with_normal_color() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());

        // Position mouse over slot (0,0) even though no drag is active
        let mut mouse = engine_input::prelude::MouseState::default();
        mouse.set_screen_pos(Vec2::new(
            GRID_MARGIN + SLOT_WIDTH * 0.5,
            GRID_MARGIN + SLOT_HEIGHT * 0.5,
        ));
        world.insert_resource(mouse);

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert — slot (0,0) should use normal SLOT_COLOR, not highlight
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[1].2, SLOT_COLOR);
    }

    /// @doc: When a stash slot holds a card with baked geometry, the system must submit
    /// that pre-tessellated mesh via `draw_colored_mesh` rather than a flat colored quad.
    /// This lets the player see a miniature thumbnail of the card's actual art — border,
    /// gems, and name strip — instead of an opaque colored rectangle that conveys no
    /// card identity at a glance.
    #[test]
    fn when_occupied_slot_has_baked_mesh_then_drawn_via_colored_mesh_not_shape() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let sig = crate::card::identity::signature::CardSignature::default();
        let label = crate::card::component::CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = glam::Vec2::new(60.0, 90.0);
        let baked = crate::card::rendering::baked_mesh::BakedCardMesh {
            front: crate::card::rendering::bake::bake_front_face(&sig, size, &label, None),
            back: crate::card::rendering::bake::bake_back_face(size),
        };
        let card_entity = world
            .spawn((
                crate::card::component::Card {
                    face_texture: engine_core::prelude::TextureId(0),
                    back_texture: engine_core::prelude::TextureId(0),
                    face_up: true,
                    signature: sig,
                },
                baked,
            ))
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
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

        // Assert — occupied slot must use draw_colored_mesh, not draw_shape
        assert!(
            !colored_mesh_calls.lock().unwrap().is_empty(),
            "expected draw_colored_mesh for occupied slot with BakedCardMesh, got none"
        );
        assert_eq!(
            shape_calls.lock().unwrap().len(),
            1,
            "expected exactly 1 draw_shape (background only); occupied slot must not draw a flat quad"
        );
    }

    /// @doc: The miniature must use the card's actual baked front-face geometry — not a
    /// stand-in four-vertex quad — so that the player sees real card art (border, gems, name strip)
    /// in the stash grid. A vertex count mismatch would mean the system substituted a generic shape.
    #[test]
    fn when_occupied_slot_rendered_then_vertices_match_baked_front() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let sig = crate::card::identity::signature::CardSignature::default();
        let label = crate::card::component::CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = glam::Vec2::new(60.0, 90.0);
        let front_mesh = crate::card::rendering::bake::bake_front_face(&sig, size, &label, None);
        let expected_vertex_count = front_mesh.vertices.len();
        let baked = crate::card::rendering::baked_mesh::BakedCardMesh {
            front: front_mesh,
            back: crate::card::rendering::bake::bake_back_face(size),
        };
        let card_entity = world
            .spawn((
                crate::card::component::Card {
                    face_texture: engine_core::prelude::TextureId(0),
                    back_texture: engine_core::prelude::TextureId(0),
                    face_up: true,
                    signature: sig,
                },
                baked,
            ))
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);

        let log = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(colored_mesh_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert
        let calls = colored_mesh_calls.lock().unwrap();
        assert_eq!(
            calls[0].0.len(),
            expected_vertex_count,
            "expected front-face vertex count {expected_vertex_count}, got {}",
            calls[0].0.len()
        );
    }

    /// @doc: The model matrix must scale the card's natural size (60×90) down to slot dimensions
    /// (SLOT_WIDTH×SLOT_HEIGHT) so the miniature fits precisely. Without correct scaling, the card
    /// art would either overflow the slot bounds or appear as a tiny dot in the center.
    #[test]
    fn when_occupied_slot_rendered_then_model_scales_card_to_slot() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let sig = crate::card::identity::signature::CardSignature::default();
        let label = crate::card::component::CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = glam::Vec2::new(60.0, 90.0);
        let baked = crate::card::rendering::baked_mesh::BakedCardMesh {
            front: crate::card::rendering::bake::bake_front_face(&sig, size, &label, None),
            back: crate::card::rendering::bake::bake_back_face(size),
        };
        let card_entity = world
            .spawn((
                crate::card::component::Card {
                    face_texture: engine_core::prelude::TextureId(0),
                    back_texture: engine_core::prelude::TextureId(0),
                    face_up: true,
                    signature: sig,
                },
                baked,
            ))
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);

        let log = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(colored_mesh_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert — scale factors: SLOT_WIDTH / TABLE_CARD_WIDTH and SLOT_HEIGHT / TABLE_CARD_HEIGHT
        let calls = colored_mesh_calls.lock().unwrap();
        let model = &calls[0].2;
        let expected_sx = SLOT_WIDTH / TABLE_CARD_WIDTH;
        let expected_sy = SLOT_HEIGHT / TABLE_CARD_HEIGHT;
        assert!(
            (model[0][0] - expected_sx).abs() < 1e-4,
            "scale x: got {}, expected {expected_sx}",
            model[0][0]
        );
        assert!(
            (model[1][1] - expected_sy).abs() < 1e-4,
            "scale y: got {}, expected {expected_sy}",
            model[1][1]
        );
    }

    /// @doc: The miniature model translation must place the card art at the slot center in world
    /// coords. Without correct translation, miniatures would cluster at the world origin instead
    /// of appearing inside their grid cells.
    #[test]
    fn when_occupied_slot_rendered_then_model_translates_to_slot_center() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let sig = crate::card::identity::signature::CardSignature::default();
        let label = crate::card::component::CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = glam::Vec2::new(60.0, 90.0);
        let baked = crate::card::rendering::baked_mesh::BakedCardMesh {
            front: crate::card::rendering::bake::bake_front_face(&sig, size, &label, None),
            back: crate::card::rendering::bake::bake_back_face(size),
        };
        let card_entity = world
            .spawn((
                crate::card::component::Card {
                    face_texture: engine_core::prelude::TextureId(0),
                    back_texture: engine_core::prelude::TextureId(0),
                    face_up: true,
                    signature: sig,
                },
                baked,
            ))
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);

        let log = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(colored_mesh_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert — slot (0,0) center in screen space → world coords
        let expected = screen_to_world(
            Vec2::new(
                GRID_MARGIN + SLOT_WIDTH * 0.5,
                GRID_MARGIN + SLOT_HEIGHT * 0.5,
            ),
            &Camera2D::default(),
            1024.0,
            768.0,
        );
        let calls = colored_mesh_calls.lock().unwrap();
        let model = &calls[0].2;
        assert!(
            (model[3][0] - expected.x).abs() < 0.01,
            "tx: got {}, expected {}",
            model[3][0],
            expected.x
        );
        assert!(
            (model[3][1] - expected.y).abs() < 0.01,
            "ty: got {}, expected {}",
            model[3][1],
            expected.y
        );
    }

    /// @doc: Camera zoom must inversely scale the miniature model matrix, matching how empty
    /// slots already scale. Without zoom adjustment, miniatures would appear at different sizes
    /// than their surrounding slot borders when the player zooms in/out.
    #[test]
    fn when_occupied_slot_at_zoom2_then_scale_halved() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let sig = crate::card::identity::signature::CardSignature::default();
        let label = crate::card::component::CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = glam::Vec2::new(60.0, 90.0);
        let baked = crate::card::rendering::baked_mesh::BakedCardMesh {
            front: crate::card::rendering::bake::bake_front_face(&sig, size, &label, None),
            back: crate::card::rendering::bake::bake_back_face(size),
        };
        let card_entity = world
            .spawn((
                crate::card::component::Card {
                    face_texture: engine_core::prelude::TextureId(0),
                    back_texture: engine_core::prelude::TextureId(0),
                    face_up: true,
                    signature: sig,
                },
                baked,
            ))
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);

        let log = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(colored_mesh_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 2.0,
        });

        // Act
        run_system(&mut world);

        // Assert — at zoom=2, scale is halved
        let calls = colored_mesh_calls.lock().unwrap();
        let model = &calls[0].2;
        let expected_sx = (SLOT_WIDTH / TABLE_CARD_WIDTH) / 2.0;
        let expected_sy = (SLOT_HEIGHT / TABLE_CARD_HEIGHT) / 2.0;
        assert!(
            (model[0][0] - expected_sx).abs() < 1e-4,
            "scale x at zoom=2: got {}, expected {expected_sx}",
            model[0][0]
        );
        assert!(
            (model[1][1] - expected_sy).abs() < 1e-4,
            "scale y at zoom=2: got {}, expected {expected_sy}",
            model[1][1]
        );
    }

    /// @doc: Empty stash slots must continue using `draw_shape` with `SLOT_COLOR` — the miniature
    /// card art path only applies to occupied slots. Without this separation, empty slots would
    /// either crash (no `BakedCardMesh` to draw) or render invisible.
    #[test]
    fn when_empty_slots_rendered_then_still_draw_shape_with_slot_color() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange — 2x2 grid, all slots empty
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));
        world.insert_resource(crate::card::interaction::drag_state::DragState::default());
        world.insert_resource(engine_input::prelude::MouseState::default());

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
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

        // Assert — 1 background + 4 empty slots = 5 shape calls, 0 colored mesh calls
        assert_eq!(shape_calls.lock().unwrap().len(), 5);
        assert!(
            colored_mesh_calls.lock().unwrap().is_empty(),
            "empty slots must not use draw_colored_mesh"
        );
    }

    /// @doc: The drag preview ghost that follows the cursor over the stash must also render the
    /// card's baked front mesh, not a flat colored quad. Without this, the preview looks like a
    /// featureless square while the slots show detailed miniatures — an inconsistency that breaks
    /// the player's visual expectation of what the dragged card looks like.
    #[test]
    fn when_dragged_card_with_baked_mesh_over_stash_then_preview_uses_colored_mesh() {
        use engine_render::testing::ColoredMeshCallLog;

        // Arrange
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));

        let sig = crate::card::identity::signature::CardSignature::default();
        let label = crate::card::component::CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = glam::Vec2::new(60.0, 90.0);
        let front_mesh = crate::card::rendering::bake::bake_front_face(&sig, size, &label, None);
        let expected_vertex_count = front_mesh.vertices.len();
        let baked = crate::card::rendering::baked_mesh::BakedCardMesh {
            front: front_mesh,
            back: crate::card::rendering::bake::bake_back_face(size),
        };
        let entity = world
            .spawn((
                crate::card::component::Card {
                    face_texture: engine_core::prelude::TextureId(0),
                    back_texture: engine_core::prelude::TextureId(0),
                    face_up: true,
                    signature: sig,
                },
                baked,
            ))
            .id();
        let drag_info = crate::card::interaction::drag_state::DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: crate::card::component::CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        };
        world.insert_resource(crate::card::interaction::drag_state::DragState {
            dragging: Some(drag_info),
        });

        let mut mouse = engine_input::prelude::MouseState::default();
        mouse.set_screen_pos(Vec2::new(GRID_MARGIN + 10.0, GRID_MARGIN + 10.0));
        world.insert_resource(mouse);

        let log = Arc::new(Mutex::new(Vec::new()));
        let colored_mesh_calls: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(colored_mesh_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert — at least one colored mesh call for the drag preview
        let calls = colored_mesh_calls.lock().unwrap();
        assert!(
            !calls.is_empty(),
            "drag preview must use draw_colored_mesh when card has BakedCardMesh"
        );
        let last = calls.last().unwrap();
        assert_eq!(
            last.0.len(),
            expected_vertex_count,
            "drag preview must use front-face vertices"
        );
    }

    #[test]
    fn when_dragged_card_over_stash_then_drag_preview_uses_unit_quad() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));

        let entity = world.spawn_empty().id();
        let drag_info = crate::card::interaction::drag_state::DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: crate::card::component::CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        };
        world.insert_resource(crate::card::interaction::drag_state::DragState {
            dragging: Some(drag_info),
        });

        let mut mouse = engine_input::prelude::MouseState::default();
        mouse.set_screen_pos(Vec2::new(GRID_MARGIN + 10.0, GRID_MARGIN + 10.0));
        world.insert_resource(mouse);

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        // Act
        run_system(&mut world);

        // Assert — last draw call is the drag preview; vertices should be normalized unit quad
        let calls = shape_calls.lock().unwrap();
        let last = calls.last().expect("should have draw calls");
        let verts = &last.0;
        let w = verts[1][0] - verts[0][0];
        let h = verts[3][1] - verts[0][1];
        assert!(
            (w - 1.0).abs() < 1e-4,
            "drag preview vertex width={w}, expected 1.0"
        );
        assert!(
            (h - 1.0).abs() < 1e-4,
            "drag preview vertex height={h}, expected 1.0"
        );
    }
}
