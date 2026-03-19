use bevy_ecs::prelude::{Entity, Query, Res, ResMut, Resource};
use engine_core::prelude::Color;
use engine_input::prelude::{InputState, KeyCode, MouseState};
use engine_render::prelude::{BlendMode, Camera2D, RendererRes, ShaderHandle, screen_to_world};
use glam::Vec2;

use crate::card_art_shader::CardArtShader;
use crate::card_pick::find_stash_slot_at;
use crate::drag_state::DragState;
use crate::spawn_table_card::{CARD_HEIGHT, CARD_WIDTH};
use crate::stash_grid::StashGrid;
use crate::stash_render::{GRID_MARGIN, SLOT_STRIDE_H};
use crate::stash_toggle::StashVisible;

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

fn rect_vertices(x: f32, y: f32, w: f32, h: f32) -> [[f32; 2]; 4] {
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

const RECT_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

fn scale_translate_model(sx: f32, sy: f32, tx: f32, ty: f32) -> [[f32; 4]; 4] {
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx, ty, 0.0, 1.0],
    ]
}

struct CardFaceDef {
    offset_y_frac: f32,
    half_w_frac: f32,
    half_h_abs: f32,
    color: Color,
    use_art_shader: bool,
}

const CARD_FACE_DEFS: [CardFaceDef; 4] = [
    // Border (full card)
    CardFaceDef {
        offset_y_frac: 0.0,
        half_w_frac: 0.5,
        half_h_abs: 0.5, // special: fraction of h, not absolute
        color: Color::WHITE,
        use_art_shader: false,
    },
    // Name strip
    CardFaceDef {
        offset_y_frac: -0.5, // relative to top, adjusted by half_h
        half_w_frac: 0.45,
        half_h_abs: 7.5,
        color: Color {
            r: 0.863,
            g: 0.863,
            b: 0.863,
            a: 1.0,
        },
        use_art_shader: false,
    },
    // Art area
    CardFaceDef {
        offset_y_frac: -0.1,
        half_w_frac: 0.45,
        half_h_abs: 22.5,
        color: Color {
            r: 0.706,
            g: 0.784,
            b: 0.902,
            a: 1.0,
        },
        use_art_shader: true,
    },
    // Description strip
    CardFaceDef {
        offset_y_frac: 0.5, // relative to bottom, adjusted by half_h
        half_w_frac: 0.45,
        half_h_abs: 15.0,
        color: Color {
            r: 0.941,
            g: 0.941,
            b: 0.784,
            a: 1.0,
        },
        use_art_shader: false,
    },
];

#[allow(clippy::too_many_arguments)]
pub fn stash_hover_preview_render_system(
    hover_preview: Res<StashHoverPreview>,
    grid: Res<StashGrid>,
    art_shader: Option<Res<CardArtShader>>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    if hover_preview.hovered_entity.is_none() {
        return;
    }

    let (vw, vh) = renderer.viewport_size();
    if vw == 0 || vh == 0 {
        return;
    }
    let vw = vw as f32;
    let vh = vh as f32;

    let camera = camera_query.iter().next().copied().unwrap_or_default();

    let grid_screen_h = f32::from(grid.height()) * SLOT_STRIDE_H;
    let preview_screen_h = grid_screen_h;
    let preview_screen_w = preview_screen_h * (CARD_WIDTH / CARD_HEIGHT);

    let preview_center_screen = Vec2::new(
        vw - GRID_MARGIN - preview_screen_w * 0.5,
        GRID_MARGIN + preview_screen_h * 0.5,
    );
    let preview_center = screen_to_world(preview_center_screen, &camera, vw, vh);

    let world_w = preview_screen_w / camera.zoom;
    let world_h = preview_screen_h / camera.zoom;

    // Scale factor for child element heights (original card is 90px tall)
    let h_scale = preview_screen_h / CARD_HEIGHT;

    renderer.set_shader(ShaderHandle(0));
    renderer.set_blend_mode(BlendMode::Alpha);

    let art = art_shader.map(|s| s.0);

    // Shader-space half-extents (must match stash_render's local_slot_verts)
    const SHADER_HALF_W: f32 = 27.0;
    const SHADER_HALF_H: f32 = 22.5;

    let local_verts = rect_vertices(
        -SHADER_HALF_W,
        -SHADER_HALF_H,
        SHADER_HALF_W * 2.0,
        SHADER_HALF_H * 2.0,
    );

    for def in &CARD_FACE_DEFS {
        let half_w = world_w * def.half_w_frac;
        let half_h = if def.offset_y_frac.abs() < 0.01 && (def.half_h_abs - 0.5).abs() < 0.01 {
            world_h * 0.5
        } else {
            def.half_h_abs * h_scale / camera.zoom
        };

        let offset_y = if (def.offset_y_frac + 0.5).abs() < 0.01 {
            -(world_h * 0.5 - half_h)
        } else if (def.offset_y_frac - 0.5).abs() < 0.01 {
            world_h * 0.5 - half_h
        } else {
            world_h * def.offset_y_frac
        };

        let center_x = preview_center.x;
        let center_y = preview_center.y + offset_y;

        if def.use_art_shader {
            if let Some(shader) = art {
                renderer.set_shader(shader);
            }
            let sx = half_w / SHADER_HALF_W;
            let sy = half_h / SHADER_HALF_H;
            renderer.draw_shape(
                &local_verts,
                &RECT_INDICES,
                def.color,
                scale_translate_model(sx, sy, center_x, center_y),
            );
            renderer.set_shader(ShaderHandle(0));
        } else {
            let verts = rect_vertices(-half_w, -half_h, half_w * 2.0, half_h * 2.0);
            renderer.draw_shape(
                &verts,
                &RECT_INDICES,
                def.color,
                scale_translate_model(1.0, 1.0, center_x, center_y),
            );
        }
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
    use crate::drag_state::DragState;
    use crate::stash_grid::StashGrid;
    use crate::stash_toggle::StashVisible;

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
        mouse.set_screen_pos(Vec2::new(45.0 + crate::stash_render::SLOT_STRIDE_W, 45.0));
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
                origin_zone: crate::card_zone::CardZone::Table,
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
}
