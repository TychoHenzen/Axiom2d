use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;

use crate::card_geometry::{QUAD_INDICES, rect_vertices};
use crate::stash_grid::StashGrid;
use crate::stash_render::{BACKGROUND_COLOR, GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_H, SLOT_STRIDE_W};
use crate::stash_toggle::StashVisible;
use crate::viewport_camera::resolve_viewport_camera;

pub const TAB_WIDTH: f32 = 30.0;
pub const TAB_HEIGHT: f32 = 16.0;
pub const TAB_GAP: f32 = 4.0;
pub const TAB_MARGIN_TOP: f32 = 0.0;

pub const TAB_ACTIVE: Color = Color {
    r: 0.35,
    g: 0.35,
    b: 0.35,
    a: 1.0,
};
pub const TAB_INACTIVE: Color = BACKGROUND_COLOR;

/// Returns the screen-space Y position of the top edge of the tab row.
pub fn tab_row_top_y(grid_height: u8) -> f32 {
    GRID_MARGIN + f32::from(grid_height) * SLOT_STRIDE_H - SLOT_GAP + TAB_MARGIN_TOP
}

/// Returns the screen-space X position of the left edge of tab `i` in a row of `page_count` tabs,
/// centered under a grid of width `grid_width`.
pub fn tab_left_x(grid_width: u8, page_count: u8, tab_index: u8) -> f32 {
    let tab_stride = TAB_WIDTH + TAB_GAP;
    let total_width = f32::from(page_count) * tab_stride - TAB_GAP;
    let grid_screen_w = f32::from(grid_width) * SLOT_STRIDE_W - SLOT_GAP;
    let grid_center_x = GRID_MARGIN + grid_screen_w / 2.0;
    let tabs_start_x = grid_center_x - total_width / 2.0;
    tabs_start_x + f32::from(tab_index) * tab_stride
}

pub fn stash_tab_click_system(
    mouse: Res<MouseState>,
    visible: Res<StashVisible>,
    mut grid: ResMut<StashGrid>,
) {
    if !visible.0 {
        return;
    }
    if !mouse.just_pressed(engine_input::prelude::MouseButton::Left) {
        return;
    }

    let screen = mouse.screen_pos();
    let top_y = tab_row_top_y(grid.height());
    let bottom_y = top_y + TAB_HEIGHT;

    if screen.y < top_y || screen.y > bottom_y {
        return;
    }

    let page_count = grid.page_count();
    for i in 0..page_count {
        let left = tab_left_x(grid.width(), page_count, i);
        let right = left + TAB_WIDTH;
        if screen.x >= left && screen.x <= right {
            grid.set_current_page(i);
            return;
        }
    }
}

pub fn stash_tab_render_system(
    grid: Res<StashGrid>,
    visible: Res<StashVisible>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    if !visible.0 {
        return;
    }
    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let page_count = grid.page_count();
    let current = grid.current_page();
    let top_y = tab_row_top_y(grid.height());

    for i in 0..page_count {
        let left_x = tab_left_x(grid.width(), page_count, i);
        let color = if i == current {
            TAB_ACTIVE
        } else {
            TAB_INACTIVE
        };
        let origin = screen_to_world(Vec2::new(left_x, top_y), &camera, vw, vh);
        let tab_w = TAB_WIDTH / camera.zoom;
        let tab_h = TAB_HEIGHT / camera.zoom;
        let verts = rect_vertices(origin.x, origin.y, tab_w, tab_h);
        renderer.draw_shape(
            &verts,
            &QUAD_INDICES,
            color,
            engine_render::prelude::IDENTITY_MODEL,
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};
    use engine_input::prelude::MouseButton;
    use engine_render::testing::{ShapeCallLog, SpyRenderer};
    use std::sync::{Arc, Mutex};

    // --- Tab click tests ---

    fn make_click_world(page_count: u8, visible: bool) -> World {
        let mut world = World::new();
        world.insert_resource(StashGrid::new(5, 5, page_count));
        world.insert_resource(StashVisible(visible));
        world.insert_resource(MouseState::default());
        world
    }

    fn run_click_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_tab_click_system);
        schedule.run(world);
    }

    /// Grid params (width=5, height=5) used by `make_click_world`.
    const GRID_W: u8 = 5;
    const GRID_H: u8 = 5;

    fn click_at(world: &mut World, x: f32, y: f32) {
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(Vec2::new(x, y));
        mouse.press(MouseButton::Left);
        world.insert_resource(mouse);
    }

    fn tab_center(tab_index: u8, page_count: u8) -> (f32, f32) {
        let left = tab_left_x(GRID_W, page_count, tab_index);
        let top = tab_row_top_y(GRID_H);
        (left + TAB_WIDTH / 2.0, top + TAB_HEIGHT / 2.0)
    }

    #[test]
    fn when_click_on_second_tab_then_switches_to_page_one() {
        // Arrange
        let mut world = make_click_world(3, true);
        let (cx, cy) = tab_center(1, 3);
        click_at(&mut world, cx, cy);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 1);
    }

    #[test]
    fn when_click_on_third_tab_then_switches_to_page_two() {
        // Arrange
        let mut world = make_click_world(3, true);
        let (cx, cy) = tab_center(2, 3);
        click_at(&mut world, cx, cy);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 2);
    }

    #[test]
    fn when_click_on_first_tab_then_stays_on_page_zero() {
        // Arrange
        let mut world = make_click_world(3, true);
        let (cx, cy) = tab_center(0, 3);
        click_at(&mut world, cx, cy);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 0);
    }

    #[test]
    fn when_click_between_tabs_then_page_unchanged() {
        // Arrange
        let mut world = make_click_world(3, true);
        let left0 = tab_left_x(GRID_W, 3, 0);
        let top = tab_row_top_y(GRID_H);
        click_at(
            &mut world,
            left0 + TAB_WIDTH + TAB_GAP / 2.0,
            top + TAB_HEIGHT / 2.0,
        );

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 0);
    }

    #[test]
    fn when_click_above_tabs_then_page_unchanged() {
        // Arrange
        let mut world = make_click_world(3, true);
        let left = tab_left_x(GRID_W, 3, 1);
        let top = tab_row_top_y(GRID_H);
        click_at(&mut world, left + TAB_WIDTH / 2.0, top - 2.0);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 0);
    }

    #[test]
    fn when_click_below_tabs_then_page_unchanged() {
        // Arrange
        let mut world = make_click_world(3, true);
        let left = tab_left_x(GRID_W, 3, 1);
        let top = tab_row_top_y(GRID_H);
        click_at(&mut world, left + TAB_WIDTH / 2.0, top + TAB_HEIGHT + 2.0);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 0);
    }

    #[test]
    fn when_stash_hidden_and_click_on_tab_then_page_unchanged() {
        // Arrange
        let mut world = make_click_world(3, false);
        let (cx, cy) = tab_center(1, 3);
        click_at(&mut world, cx, cy);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 0);
    }

    #[test]
    fn when_no_click_then_page_unchanged() {
        // Arrange
        let mut world = make_click_world(3, true);
        world.resource_mut::<StashGrid>().set_current_page(1);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 1);
    }

    #[test]
    fn when_right_click_on_tab_then_page_unchanged() {
        // Arrange
        let mut world = make_click_world(3, true);
        let (cx, cy) = tab_center(1, 3);
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(Vec2::new(cx, cy));
        mouse.press(MouseButton::Right);
        world.insert_resource(mouse);

        // Act
        run_click_system(&mut world);

        // Assert
        assert_eq!(world.resource::<StashGrid>().current_page(), 0);
    }

    // --- Tab render tests ---

    fn make_render_world(
        grid: StashGrid,
        visible: bool,
        viewport: (u32, u32),
    ) -> (World, ShapeCallLog) {
        let mut world = World::new();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(visible));

        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(shape_calls.clone())
            .with_viewport(viewport.0, viewport.1);
        world.insert_resource(RendererRes::new(Box::new(spy)));

        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        (world, shape_calls)
    }

    fn run_render_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_tab_render_system);
        schedule.run(world);
    }

    #[test]
    fn when_stash_hidden_then_no_tab_shapes_drawn() {
        // Arrange
        let (mut world, shape_calls) =
            make_render_world(StashGrid::new(5, 5, 3), false, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn when_viewport_zero_then_no_tab_shapes_drawn() {
        // Arrange
        let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 3), true, (0, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        assert!(shape_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn when_three_pages_then_three_tab_shapes_drawn() {
        // Arrange
        let (mut world, shape_calls) =
            make_render_world(StashGrid::new(5, 5, 3), true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        assert_eq!(shape_calls.lock().unwrap().len(), 3);
    }

    #[test]
    fn when_single_page_then_one_tab_shape_drawn() {
        // Arrange
        let (mut world, shape_calls) =
            make_render_world(StashGrid::new(5, 5, 1), true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        assert_eq!(shape_calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn when_on_page_zero_then_first_tab_is_active_color() {
        // Arrange
        let (mut world, shape_calls) =
            make_render_world(StashGrid::new(5, 5, 3), true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[0].2, TAB_ACTIVE);
        assert_eq!(calls[1].2, TAB_INACTIVE);
        assert_eq!(calls[2].2, TAB_INACTIVE);
    }

    #[test]
    fn when_on_page_one_then_middle_tab_is_active_color() {
        // Arrange
        let mut grid = StashGrid::new(5, 5, 3);
        grid.set_current_page(1);
        let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[0].2, TAB_INACTIVE);
        assert_eq!(calls[1].2, TAB_ACTIVE);
        assert_eq!(calls[2].2, TAB_INACTIVE);
    }

    #[test]
    fn when_on_last_page_then_last_tab_is_active_color() {
        // Arrange
        let mut grid = StashGrid::new(5, 5, 3);
        grid.set_current_page(2);
        let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[0].2, TAB_INACTIVE);
        assert_eq!(calls[1].2, TAB_INACTIVE);
        assert_eq!(calls[2].2, TAB_ACTIVE);
    }

    #[test]
    fn when_tabs_rendered_then_positioned_below_grid_bottom() {
        // Arrange
        let grid = StashGrid::new(5, 4, 3);
        let expected_top = tab_row_top_y(grid.height());
        let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        let camera = Camera2D::default();
        for (i, call) in calls.iter().enumerate() {
            let screen = engine_render::prelude::world_to_screen(
                Vec2::new(call.0[0][0], call.0[0][1]),
                &camera,
                1024.0,
                768.0,
            );
            assert!(
                (screen.y - expected_top).abs() < 1.0,
                "tab {i} screen_y={}, expected near {expected_top}",
                screen.y
            );
        }
    }

    #[test]
    fn when_tabs_rendered_then_centered_under_grid() {
        // Arrange
        let grid = StashGrid::new(5, 4, 3);
        let grid_screen_w = f32::from(grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
        let grid_center_x = GRID_MARGIN + grid_screen_w / 2.0;
        let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        let camera = Camera2D::default();
        let first_screen = engine_render::prelude::world_to_screen(
            Vec2::new(calls[0].0[0][0], calls[0].0[0][1]),
            &camera,
            1024.0,
            768.0,
        );
        let last_screen = engine_render::prelude::world_to_screen(
            Vec2::new(calls[2].0[0][0], calls[2].0[0][1]),
            &camera,
            1024.0,
            768.0,
        );
        let tab_group_mid_x = (first_screen.x + last_screen.x + TAB_WIDTH) / 2.0;
        assert!(
            (tab_group_mid_x - grid_center_x).abs() < 1.0,
            "tab midpoint {tab_group_mid_x} should be near grid center {grid_center_x}"
        );
    }

    #[test]
    fn when_tabs_rendered_then_adjacent_tabs_evenly_spaced() {
        // Arrange
        let (mut world, shape_calls) =
            make_render_world(StashGrid::new(5, 5, 3), true, (1024, 768));

        // Act
        run_render_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        let x0 = calls[0].0[0][0];
        let x1 = calls[1].0[0][0];
        let x2 = calls[2].0[0][0];
        let dx01 = x1 - x0;
        let dx12 = x2 - x1;
        assert!(
            (dx01 - dx12).abs() < 0.01,
            "spacing should be uniform: dx01={dx01}, dx12={dx12}"
        );
    }
}
