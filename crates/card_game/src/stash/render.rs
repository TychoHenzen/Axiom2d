use std::collections::HashMap;

use bevy_ecs::prelude::{Entity, Query, Res, ResMut, With};
use engine_core::color::Color;
use engine_render::prelude::{
    BlendMode, Camera2D, QUAD_INDICES, RendererRes, ShaderHandle, Shape, UNIT_QUAD, rect_vertices,
    screen_to_world, unit_quad_model,
};
use engine_scene::prelude::ChildOf;
use glam::Vec2;

use crate::card::geometry::{ART_QUAD, art_quad_model};
use crate::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_COLOR, SLOT_GAP, SLOT_HEIGHT, SLOT_STRIDE_H, SLOT_STRIDE_W,
    SLOT_WIDTH,
};
use crate::stash::grid::StashGrid;
use crate::stash::icon::StashIcon;
use crate::stash::toggle::StashVisible;
use engine_render::prelude::resolve_viewport_camera;

pub(crate) fn reset_default_shader(renderer: &mut dyn engine_render::prelude::Renderer) {
    renderer.set_shader(ShaderHandle(0));
    renderer.set_blend_mode(BlendMode::Alpha);
}

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn stash_render_system(
    grid: Res<StashGrid>,
    visible: Res<StashVisible>,
    drag_state: Res<crate::card::drag_state::DragState>,
    mouse: Res<engine_input::prelude::MouseState>,
    art_shader: Option<Res<crate::card::art_shader::CardArtShader>>,
    stash_icons: Query<(&ChildOf, &Shape), With<StashIcon>>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    let renderer_art_shader = art_shader.map(|s| s.0);
    if !visible.0 {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    // Reset shader and blend mode — shape_render_system may have left a custom shader active
    reset_default_shader(&mut **renderer);
    let world_slot_w = SLOT_WIDTH / camera.zoom;
    let world_slot_h = SLOT_HEIGHT / camera.zoom;

    let bg_screen_w = f32::from(grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
    let bg_screen_h = f32::from(grid.height()) * SLOT_STRIDE_H - SLOT_GAP;
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

    let icon_colors: HashMap<Entity, Color> = stash_icons
        .iter()
        .map(|(parent, shape)| (parent.0, shape.color))
        .collect();

    let page = grid.current_page();

    for col in 0..grid.width() {
        for row in 0..grid.height() {
            let screen_x = GRID_MARGIN + f32::from(col) * SLOT_STRIDE_W;
            let screen_y = GRID_MARGIN + f32::from(row) * SLOT_STRIDE_H;
            let center = screen_to_world(
                Vec2::new(screen_x + SLOT_WIDTH * 0.5, screen_y + SLOT_HEIGHT * 0.5),
                &camera,
                vw,
                vh,
            );

            if let Some(&entity) = grid.get(page, col, row) {
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
            } else {
                let model = unit_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, SLOT_COLOR, model);
            }
        }
    }

    // Draw the dragged card's icon on top of the stash grid at the cursor position
    if let Some(info) = drag_state.dragging {
        let screen = mouse.screen_pos();
        let bg_x_max = GRID_MARGIN + f32::from(grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
        let bg_y_max = GRID_MARGIN + f32::from(grid.height()) * SLOT_STRIDE_H - SLOT_GAP;
        let over_stash_area = screen.x >= GRID_MARGIN
            && screen.x < bg_x_max
            && screen.y >= GRID_MARGIN
            && screen.y < bg_y_max;

        if over_stash_area {
            let color = icon_colors.get(&info.entity).copied().unwrap_or(SLOT_COLOR);
            let cursor_world = mouse.world_pos();
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
        world.insert_resource(crate::card::drag_state::DragState::default());
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
    fn when_visible_and_occupied_slot_then_drawn_with_card_shape_color() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 1, 1), true);

        let card_color = Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let root = world.spawn_empty().id();
        world.spawn((
            ChildOf(root),
            StashIcon,
            Shape {
                variant: engine_render::prelude::ShapeVariant::Circle { radius: 1.0 },
                color: card_color,
            },
        ));

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, root).unwrap();
        world.insert_resource(grid);

        // Act
        run_system(&mut world);

        // Assert — calls[0] is background, calls[1] is the only slot
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[1].2, card_color);
    }

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
        world.insert_resource(crate::card::drag_state::DragState::default());
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
        world.insert_resource(crate::card::drag_state::DragState::default());
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
        world.insert_resource(crate::card::drag_state::DragState::default());
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
        world.insert_resource(crate::card::drag_state::DragState::default());
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

    #[test]
    fn when_dragged_card_over_stash_then_drag_preview_uses_unit_quad() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashGrid::new(2, 2, 1));
        world.insert_resource(StashVisible(true));

        let entity = world.spawn_empty().id();
        let drag_info = crate::card::drag_state::DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: crate::card::zone::CardZone::Table,
            stash_cursor_follow: false,
        };
        world.insert_resource(crate::card::drag_state::DragState {
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
