use bevy_ecs::prelude::{Entity, Query, Res, ResMut, Resource};
use engine_input::prelude::{InputState, KeyCode, MouseState};
use engine_render::prelude::{Camera2D, RendererRes, resolve_viewport_camera, screen_to_world};
use glam::Vec2;

use crate::card::interaction::drag_state::DragState;
use crate::card::rendering::baked_mesh::BakedCardMesh;
use crate::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
use crate::stash::constants::{GRID_MARGIN, SLOT_STRIDE_H};
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;
use engine_render::material::apply_material;
use engine_render::shape::MeshOverlays;

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

/// Renders a scaled preview of a hovered stash card using its baked front mesh.
pub fn stash_hover_preview_render_system(
    hover_preview: Res<StashHoverPreview>,
    grid: Res<StashGrid>,
    camera_query: Query<&Camera2D>,
    baked_query: Query<(&BakedCardMesh, Option<&MeshOverlays>)>,
    mut renderer: ResMut<RendererRes>,
) {
    let Some(hovered_entity) = hover_preview.hovered_entity else {
        return;
    };

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let Ok((baked, overlays)) = baked_query.get(hovered_entity) else {
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

    if !baked.front.is_empty() {
        let model = [
            [scale_x, 0.0, 0.0, 0.0],
            [0.0, scale_y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [preview_center.x, preview_center.y, 0.0, 1.0],
        ];
        renderer.set_shader(engine_render::prelude::ShaderHandle(0));
        renderer.draw_colored_mesh(&baked.front.vertices, &baked.front.indices, model);
        if let Some(overlays) = overlays {
            for entry in &overlays.0 {
                apply_material(&mut **renderer, Some(&entry.material), &mut None, &mut None);
                renderer.draw_shape(&entry.vertices, &entry.indices, entry.color, model);
            }
            renderer.set_shader(engine_render::prelude::ShaderHandle(0));
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
    use engine_render::testing::{ColoredMeshCallLog, SpyRenderer};
    use glam::Vec2;

    use super::{StashHoverPreview, stash_hover_preview_render_system, stash_hover_preview_system};
    use crate::card::interaction::drag_state::DragState;
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

    fn make_world_with_spy(grid: StashGrid) -> (World, ColoredMeshCallLog) {
        let mut world = World::new();
        let mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(mesh_log.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });
        world.insert_resource(grid);
        world.insert_resource(DragState::default());
        world.insert_resource(StashHoverPreview::default());
        (world, mesh_log)
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
            dragging: Some(crate::card::interaction::drag_state::DragInfo {
                entity: card_entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: crate::card::component::CardZone::Table,
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

    use crate::card::identity::definition::{
        CardAbilities, CardDefinition, CardType, art_descriptor_default,
    };
    use crate::card::rendering::spawn_table_card::spawn_visual_card;

    fn make_test_def() -> CardDefinition {
        CardDefinition {
            card_type: CardType::Spell,
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
        use crate::card::identity::signature::CardSignature;
        spawn_visual_card(
            world,
            &make_test_def(),
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            true,
            CardSignature::default(),
        )
    }

    #[test]
    fn when_hovered_entity_none_then_no_colored_mesh_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        let (mut world, mesh_log) = make_world_with_spy(grid);

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = mesh_log.lock().unwrap();
        assert!(
            calls.is_empty(),
            "no mesh should be drawn when not hovering"
        );
    }

    #[test]
    fn when_hovered_entity_set_then_baked_front_mesh_drawn() {
        // Arrange
        let grid = StashGrid::new(10, 10, 1);
        let (mut world, mesh_log) = make_world_with_spy(grid);
        let card = spawn_card_in_world(&mut world);
        world
            .resource_mut::<StashGrid>()
            .place(0, 0, 0, card)
            .unwrap();
        world.resource_mut::<StashHoverPreview>().hovered_entity = Some(card);

        // Act
        run_render_system(&mut world);

        // Assert — one draw_colored_mesh call for the baked front face
        let calls = mesh_log.lock().unwrap();
        assert_eq!(
            calls.len(),
            1,
            "expected exactly 1 draw_colored_mesh call for baked card"
        );
    }

    #[test]
    fn when_viewport_zero_then_no_mesh_drawn() {
        // Arrange
        let mut world = World::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_colored_mesh_capture(mesh_log.clone())
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
        let calls = mesh_log.lock().unwrap();
        assert!(calls.is_empty());
    }
}
