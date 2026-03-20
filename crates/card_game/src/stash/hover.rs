use bevy_ecs::prelude::{Entity, Local, Query, Res, ResMut, Resource};
use engine_input::prelude::{InputState, KeyCode, MouseState};
use engine_render::font::{GlyphCache, measure_text, render_text_transformed, wrap_text};
use engine_render::prelude::{
    Camera2D, Material2d, RendererRes, ShaderHandle, Shape, resolve_viewport_camera,
    screen_to_world, tessellate,
};
use engine_render::shape::affine2_to_mat4;
use engine_scene::prelude::ChildOf;
use engine_scene::sort_propagation::LocalSortOrder;
use engine_ui::prelude::Text;
use glam::{Affine2, Vec2};

use crate::card::drag_state::DragState;
use crate::card::face_side::CardFaceSide;
use engine_render::prelude::QUAD_INDICES;

use crate::card::geometry::{ART_QUAD, TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
use crate::stash::constants::{GRID_MARGIN, SLOT_STRIDE_H};
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::render::reset_default_shader;
use crate::stash::toggle::StashVisible;

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

pub fn stash_hover_preview_render_system(
    hover_preview: Res<StashHoverPreview>,
    grid: Res<StashGrid>,
    camera_query: Query<&Camera2D>,
    children_query: Query<(
        &ChildOf,
        &CardFaceSide,
        &LocalSortOrder,
        &engine_core::prelude::Transform2D,
        Option<&Shape>,
        Option<&Text>,
        Option<&Material2d>,
    )>,
    mut renderer: ResMut<RendererRes>,
    mut glyph_cache: Local<GlyphCache>,
) {
    let Some(hovered_entity) = hover_preview.hovered_entity else {
        return;
    };

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let preview_screen_h = f32::from(grid.height()) * SLOT_STRIDE_H;
    let preview_screen_w = preview_screen_h * (TABLE_CARD_WIDTH / TABLE_CARD_HEIGHT);

    let preview_center_screen = Vec2::new(
        vw - GRID_MARGIN - preview_screen_w * 0.5,
        GRID_MARGIN + preview_screen_h * 0.5,
    );
    let preview_center = screen_to_world(preview_center_screen, &camera, vw, vh);

    let scale_x = (preview_screen_w / camera.zoom) / TABLE_CARD_WIDTH;
    let scale_y = (preview_screen_h / camera.zoom) / TABLE_CARD_HEIGHT;

    reset_default_shader(&mut **renderer);

    let mut front_children: Vec<_> = children_query
        .iter()
        .filter(|(parent, side, _, _, _, _, _)| {
            parent.0 == hovered_entity && **side == CardFaceSide::Front
        })
        .collect();
    front_children.sort_by_key(|(_, _, sort, _, _, _, _)| sort.0);

    for (_, _, _, child_transform, shape, text, material) in &front_children {
        let child_x = child_transform.position.x * scale_x;
        let child_y = child_transform.position.y * scale_y;
        let cx = preview_center.x + child_x;
        let cy = preview_center.y + child_y;

        if let Some(shape) = shape {
            let has_art_shader = material.is_some_and(|m| m.shader != ShaderHandle(0));
            let model = [
                [scale_x, 0.0, 0.0, 0.0],
                [0.0, scale_y, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [cx, cy, 0.0, 1.0],
            ];

            if has_art_shader {
                if let Some(m) = material {
                    renderer.set_shader(m.shader);
                }
                renderer.draw_shape(&ART_QUAD, &QUAD_INDICES, shape.color, model);
                renderer.set_shader(ShaderHandle(0));
            } else if let Ok(mesh) = tessellate(&shape.variant) {
                renderer.draw_shape(&mesh.vertices, &mesh.indices, shape.color, model);
            }
        }

        if let Some(text) = text {
            let base = Affine2::from_scale_angle_translation(
                Vec2::new(scale_x, scale_y),
                0.0,
                Vec2::new(cx, cy),
            );
            render_preview_text_component(&mut **renderer, &mut glyph_cache, text, &base);
        }
    }
}

const LINE_HEIGHT_FACTOR: f32 = 1.3;

fn render_preview_text_component(
    renderer: &mut dyn engine_render::prelude::Renderer,
    cache: &mut GlyphCache,
    text: &Text,
    base_transform: &Affine2,
) {
    if let Some(max_width) = text.max_width {
        let lines = wrap_text(&text.content, text.font_size, max_width);
        let line_height = text.font_size * LINE_HEIGHT_FACTOR;
        let total_height = (lines.len() as f32 - 1.0) * line_height;
        let start_y = -total_height * 0.5;
        for (i, line) in lines.iter().enumerate() {
            let line_width = measure_text(line, text.font_size);
            let y_offset = start_y + i as f32 * line_height;
            let offset = Affine2::from_translation(Vec2::new(-line_width * 0.5, y_offset));
            let line_transform = *base_transform * offset;
            let model = affine2_to_mat4(&line_transform);
            render_text_transformed(renderer, cache, line, &model, text.font_size, text.color);
        }
    } else {
        let text_width = measure_text(&text.content, text.font_size);
        let center_offset = Affine2::from_translation(Vec2::new(-text_width * 0.5, 0.0));
        let centered = *base_transform * center_offset;
        let model = affine2_to_mat4(&centered);
        render_text_transformed(
            renderer,
            cache,
            &text.content,
            &model,
            text.font_size,
            text.color,
        );
    }
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
    use crate::card::drag_state::DragState;
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
            dragging: Some(crate::card::drag_state::DragInfo {
                entity: card_entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: crate::card::zone::CardZone::Table,
                stash_cursor_follow: false,
                origin_position: Vec2::ZERO,
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

    use crate::card::definition::{
        CardAbilities, CardDefinition, CardType, Rarity, art_descriptor_default,
    };
    use crate::card::spawn_table_card::spawn_visual_card;

    fn make_test_def() -> CardDefinition {
        CardDefinition {
            card_type: CardType::Spell,
            rarity: Rarity::Common,
            name: "Fireball".to_owned(),
            stats: None,
            abilities: CardAbilities {
                keywords: vec![],
                text: "Deal 3 damage".to_owned(),
            },
            art: art_descriptor_default(CardType::Spell),
        }
    }

    fn spawn_card_in_world(world: &mut World) -> Entity {
        spawn_visual_card(
            world,
            &make_test_def(),
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            true,
        )
    }

    #[test]
    fn when_hovered_entity_none_then_no_shapes_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
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
    fn when_hovered_entity_set_then_front_face_shapes_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        let (mut world, shape_log) = make_world_with_spy(grid);
        let card = spawn_card_in_world(&mut world);
        world
            .resource_mut::<StashGrid>()
            .place(0, 0, 0, card)
            .unwrap();
        world.resource_mut::<StashHoverPreview>().hovered_entity = Some(card);

        // Act
        run_render_system(&mut world);

        // Assert — 4 front face shapes + text glyph shapes
        let calls = shape_log.lock().unwrap();
        assert!(
            calls.len() >= 4,
            "expected at least 4 card face shapes, got {}",
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
        let card = spawn_card_in_world(&mut world);
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
    fn when_hovered_entity_has_text_children_then_text_shapes_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        let (mut world, shape_log) = make_world_with_spy(grid);
        let card = spawn_card_in_world(&mut world);
        world
            .resource_mut::<StashGrid>()
            .place(0, 0, 0, card)
            .unwrap();
        world.resource_mut::<StashHoverPreview>().hovered_entity = Some(card);

        // Act
        run_render_system(&mut world);

        // Assert — 4 face shapes + text glyph shapes (name + description)
        let calls = shape_log.lock().unwrap();
        assert!(
            calls.len() > 4,
            "expected more than 4 shapes (4 face + text glyphs), got {}",
            calls.len()
        );
    }
}
