use bevy_ecs::prelude::{Entity, Local, Query, Res, ResMut, Resource};
use engine_core::prelude::Color;
use engine_input::prelude::{InputState, KeyCode, MouseState};
use engine_render::font::{GlyphCache, measure_text, render_text_transformed};
use engine_render::prelude::{Camera2D, RendererRes, ShaderHandle, screen_to_world};
use engine_render::shape::affine2_to_mat4;
use glam::{Affine2, Vec2};

use crate::card::art_shader::CardArtShader;
use crate::card::definition::{CardDefinition, rarity_border_color};
use crate::card::face_layout::FRONT_FACE_REGIONS;
use crate::card::geometry::{
    ART_QUAD, QUAD_INDICES, TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH, UNIT_QUAD, art_quad_model,
    unit_quad_model,
};
use crate::card::label::CardLabel;
use crate::drag_state::DragState;
use crate::stash::constants::{GRID_MARGIN, SLOT_STRIDE_H};
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::render::reset_default_shader;
use crate::stash::toggle::StashVisible;
use crate::viewport_camera::resolve_viewport_camera;

#[derive(Resource, Debug, Default)]
pub struct StashHoverPreview {
    pub hovered_entity: Option<Entity>,
}

pub fn stash_hover_preview_system(
    stash_visible: Res<StashVisible>,
    input: Res<InputState>,
    mouse: Res<MouseState>,
    grid: Res<StashGrid>,
    drag_state: Res<DragState>,
    mut hover_preview: ResMut<StashHoverPreview>,
) {
    let ctrl_held = input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);

    let hovered = stash_visible
        .0
        .then_some(())
        .filter(|()| ctrl_held)
        .filter(|()| drag_state.dragging.is_none())
        .and_then(|()| find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height()))
        .and_then(|(col, row)| grid.get(grid.current_page(), col, row).copied());

    hover_preview.hovered_entity = hovered;
}

#[allow(clippy::too_many_arguments)]
pub fn stash_hover_preview_render_system(
    hover_preview: Res<StashHoverPreview>,
    grid: Res<StashGrid>,
    art_shader: Option<Res<CardArtShader>>,
    camera_query: Query<&Camera2D>,
    label_query: Query<&CardLabel>,
    def_query: Query<&CardDefinition>,
    mut renderer: ResMut<RendererRes>,
    mut glyph_cache: Local<GlyphCache>,
) {
    let Some(hovered_entity) = hover_preview.hovered_entity else {
        return;
    };

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let grid_screen_h = f32::from(grid.height()) * SLOT_STRIDE_H;
    let preview_screen_h = grid_screen_h;
    let preview_screen_w = preview_screen_h * (TABLE_CARD_WIDTH / TABLE_CARD_HEIGHT);

    let preview_center_screen = Vec2::new(
        vw - GRID_MARGIN - preview_screen_w * 0.5,
        GRID_MARGIN + preview_screen_h * 0.5,
    );
    let preview_center = screen_to_world(preview_center_screen, &camera, vw, vh);

    reset_default_shader(&mut **renderer);

    let art = art_shader.map(|s| s.0);
    let border_color = def_query
        .get(hovered_entity)
        .map(|d| rarity_border_color(d.rarity))
        .unwrap_or(Color::WHITE);

    for (i, region) in FRONT_FACE_REGIONS.iter().enumerate() {
        let (half_w_card, half_h_card, offset_y_card) =
            region.resolve(preview_screen_w, preview_screen_h);
        let width = half_w_card * 2.0 / camera.zoom;
        let height = half_h_card * 2.0 / camera.zoom;
        let offset_y = offset_y_card / camera.zoom;

        let center_x = preview_center.x;
        let center_y = preview_center.y + offset_y;
        let color = if i == 0 { border_color } else { region.color };

        if region.use_art_shader {
            if let Some(shader) = art {
                renderer.set_shader(shader);
            }
            renderer.draw_shape(
                &ART_QUAD,
                &QUAD_INDICES,
                color,
                art_quad_model(width, height, center_x, center_y),
            );
            renderer.set_shader(ShaderHandle(0));
        } else {
            renderer.draw_shape(
                &UNIT_QUAD,
                &QUAD_INDICES,
                color,
                unit_quad_model(width, height, center_x, center_y),
            );
        }
    }

    if let Ok(label) = label_query.get(hovered_entity) {
        render_preview_text(
            &mut **renderer,
            &mut glyph_cache,
            label,
            preview_center,
            preview_screen_w,
            preview_screen_h,
            &camera,
        );
    }
}

const PREVIEW_TEXT_COLOR: Color = Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

fn render_preview_text(
    renderer: &mut dyn engine_render::prelude::Renderer,
    cache: &mut GlyphCache,
    label: &CardLabel,
    preview_center: Vec2,
    preview_w: f32,
    preview_h: f32,
    camera: &Camera2D,
) {
    let (_, _, name_offset_y) = FRONT_FACE_REGIONS[1].resolve(preview_w, preview_h);
    let name_font_size = preview_h / (12.0 * camera.zoom);
    let name_y = preview_center.y + name_offset_y / camera.zoom;

    let name_width = measure_text(&label.name, name_font_size);
    let name_transform =
        Affine2::from_translation(Vec2::new(preview_center.x - name_width * 0.5, name_y));
    let name_model = affine2_to_mat4(&name_transform);
    render_text_transformed(
        renderer,
        cache,
        &label.name,
        &name_model,
        name_font_size,
        PREVIEW_TEXT_COLOR,
    );

    let (_, _, desc_offset_y) = FRONT_FACE_REGIONS[3].resolve(preview_w, preview_h);
    let desc_font_size = preview_h / (16.0 * camera.zoom);
    let desc_y = preview_center.y + desc_offset_y / camera.zoom;

    let desc_width = measure_text(&label.description, desc_font_size);
    let desc_transform =
        Affine2::from_translation(Vec2::new(preview_center.x - desc_width * 0.5, desc_y));
    let desc_model = affine2_to_mat4(&desc_transform);
    render_text_transformed(
        renderer,
        cache,
        &label.description,
        &desc_model,
        desc_font_size,
        PREVIEW_TEXT_COLOR,
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_input::prelude::{InputState, KeyCode, MouseState};
    use engine_render::prelude::{Camera2D, RendererRes};
    use engine_render::testing::{ShapeCallLog, SpyRenderer};
    use glam::Vec2;

    use super::{StashHoverPreview, stash_hover_preview_render_system, stash_hover_preview_system};
    use crate::drag_state::DragState;
    use crate::stash::grid::StashGrid;
    use crate::stash::toggle::StashVisible;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_hover_preview_system);
        schedule.run(world);
    }

    fn run_render_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_hover_preview_render_system);
        schedule.run(world);
    }

    fn make_world_with_occupied_slot() -> (World, Entity) {
        let mut world = World::new();

        let card_entity = world.spawn_empty().id();
        let mut grid = StashGrid::new(10, 10, 1);
        grid.place(0, 0, 0, card_entity).unwrap();

        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        world.insert_resource(grid);
        world.insert_resource(DragState::default());
        world.insert_resource(StashHoverPreview::default());

        (world, card_entity)
    }

    fn make_world_with_spy(grid: StashGrid) -> (World, ShapeCallLog) {
        let mut world = World::new();
        let shape_log: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_log.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });
        world.insert_resource(grid);
        world.insert_resource(DragState::default());
        world.insert_resource(StashHoverPreview::default());
        (world, shape_log)
    }

    fn all_conditions_met(world: &mut World) {
        let mut input = InputState::default();
        input.press(KeyCode::ControlLeft);
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(Vec2::new(45.0, 45.0));
        world.insert_resource(StashVisible(true));
        world.insert_resource(input);
        world.insert_resource(mouse);
    }

    // -- Trigger condition tests (update system) --------------------------

    #[test]
    fn when_ctrl_held_and_cursor_over_occupied_slot_and_no_drag_then_hovered_entity_set() {
        // Arrange
        let (mut world, card_entity) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let preview = world.resource::<StashHoverPreview>();
        assert_eq!(preview.hovered_entity, Some(card_entity));
    }

    #[test]
    fn when_stash_hidden_then_hovered_entity_none() {
        // Arrange
        let (mut world, _) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        world.insert_resource(StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<StashHoverPreview>()
                .hovered_entity
                .is_none()
        );
    }

    #[test]
    fn when_no_ctrl_pressed_then_hovered_entity_none() {
        // Arrange
        let (mut world, _) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        world.insert_resource(InputState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<StashHoverPreview>()
                .hovered_entity
                .is_none()
        );
    }

    #[test]
    fn when_ctrl_right_pressed_then_hovered_entity_set() {
        // Arrange
        let (mut world, card_entity) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        let mut input = InputState::default();
        input.press(KeyCode::ControlRight);
        world.insert_resource(input);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashHoverPreview>().hovered_entity,
            Some(card_entity)
        );
    }

    #[test]
    fn when_cursor_over_empty_slot_then_hovered_entity_none() {
        // Arrange
        let (mut world, _) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(Vec2::new(
            45.0 + crate::stash::constants::SLOT_STRIDE_W,
            45.0,
        ));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<StashHoverPreview>()
                .hovered_entity
                .is_none()
        );
    }

    #[test]
    fn when_cursor_outside_stash_area_then_hovered_entity_none() {
        // Arrange
        let (mut world, _) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(Vec2::new(800.0, 400.0));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<StashHoverPreview>()
                .hovered_entity
                .is_none()
        );
    }

    #[test]
    fn when_drag_active_then_hovered_entity_none() {
        // Arrange
        let (mut world, card_entity) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        world.insert_resource(DragState {
            dragging: Some(crate::drag_state::DragInfo {
                entity: card_entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: crate::card::zone::CardZone::Table,
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<StashHoverPreview>()
                .hovered_entity
                .is_none()
        );
    }

    #[test]
    fn when_conditions_fail_after_hovering_then_hovered_entity_cleared() {
        // Arrange — first frame: hover
        let (mut world, card_entity) = make_world_with_occupied_slot();
        all_conditions_met(&mut world);
        run_system(&mut world);
        assert_eq!(
            world.resource::<StashHoverPreview>().hovered_entity,
            Some(card_entity)
        );

        // Arrange — second frame: release Ctrl
        world.insert_resource(InputState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<StashHoverPreview>()
                .hovered_entity
                .is_none()
        );
    }

    // -- Render system tests ----------------------------------------------

    #[test]
    fn when_hovered_entity_none_then_no_shapes_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        // Don't place any card — hovered_entity stays None
        let (mut world, shape_log) = make_world_with_spy(grid);

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_log.lock().unwrap();
        assert!(
            calls.is_empty(),
            "no shapes should be drawn when not hovering"
        );
    }

    #[test]
    fn when_hovered_entity_set_then_four_shapes_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        let (mut world, shape_log) = make_world_with_spy(grid);
        let card = world.spawn_empty().id();
        world
            .resource_mut::<StashGrid>()
            .place(0, 0, 0, card)
            .unwrap();
        world.resource_mut::<StashHoverPreview>().hovered_entity = Some(card);

        // Act
        run_render_system(&mut world);

        // Assert — 4 face shapes (border, name strip, art area, desc strip)
        let calls = shape_log.lock().unwrap();
        assert_eq!(
            calls.len(),
            4,
            "expected 4 card face shapes, got {}",
            calls.len()
        );
    }

    #[test]
    fn when_viewport_zero_then_no_shapes_drawn() {
        // Arrange
        let mut world = World::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_log: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_log.clone())
            .with_viewport(0, 0);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D::default());
        world.insert_resource(StashGrid::new(10, 10, 1));
        world.insert_resource(DragState::default());
        let card = world.spawn_empty().id();
        world.insert_resource(StashHoverPreview {
            hovered_entity: Some(card),
        });

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_log.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[test]
    fn when_hovered_entity_has_card_label_then_text_shapes_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        let (mut world, shape_log) = make_world_with_spy(grid);
        let card = world
            .spawn(crate::card::label::CardLabel {
                name: "Test".to_owned(),
                description: "Desc".to_owned(),
            })
            .id();
        world
            .resource_mut::<StashGrid>()
            .place(0, 0, 0, card)
            .unwrap();
        world.resource_mut::<StashHoverPreview>().hovered_entity = Some(card);

        // Act
        run_render_system(&mut world);

        // Assert — 4 face shapes + text glyph shapes
        let calls = shape_log.lock().unwrap();
        assert!(
            calls.len() > 4,
            "expected more than 4 shapes (4 face + text glyphs), got {}",
            calls.len()
        );
    }
}
