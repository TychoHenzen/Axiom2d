use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_render::prelude::{Camera2D, RendererRes, Shape, screen_to_world};
use glam::Vec2;

use crate::stash_grid::StashGrid;
use crate::stash_toggle::StashVisible;

pub const SLOT_SIZE: f32 = 50.0;
pub const SLOT_GAP: f32 = 4.0;
pub const SLOT_STRIDE: f32 = SLOT_SIZE + SLOT_GAP;
pub const SLOT_COLOR: Color = Color {
    r: 0.25,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
pub const GRID_MARGIN: f32 = 20.0;

fn rect_vertices(x: f32, y: f32, w: f32, h: f32) -> [[f32; 2]; 4] {
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

const RECT_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub fn stash_render_system(
    grid: Res<StashGrid>,
    visible: Res<StashVisible>,
    card_shapes: Query<&Shape>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    if !visible.0 {
        return;
    }

    let (vw, vh) = renderer.viewport_size();
    if vw == 0 || vh == 0 {
        return;
    }
    let vw = vw as f32;
    let vh = vh as f32;

    let camera = camera_query
        .iter()
        .next()
        .copied()
        .unwrap_or(Camera2D::default());
    let world_slot_size = SLOT_SIZE / camera.zoom;

    let page = grid.current_page();
    for col in 0..grid.width() {
        for row in 0..grid.height() {
            let screen_x = GRID_MARGIN + f32::from(col) * SLOT_STRIDE;
            let screen_y = GRID_MARGIN + f32::from(row) * SLOT_STRIDE;
            let wp = screen_to_world(Vec2::new(screen_x, screen_y), &camera, vw, vh);

            let color = grid
                .get(page, col, row)
                .and_then(|&entity| card_shapes.get(entity).ok())
                .map_or(SLOT_COLOR, |shape| shape.color);

            let verts = rect_vertices(wp.x, wp.y, world_slot_size, world_slot_size);
            renderer.draw_shape(&verts, &RECT_INDICES, color);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};
    use engine_render::testing::{ShapeCallLog, SpyRenderer};
    use std::sync::{Arc, Mutex};

    fn make_world_with_spy(grid: StashGrid, visible: bool) -> (World, ShapeCallLog) {
        let mut world = World::new();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(visible));

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

        // Assert
        assert_eq!(shape_calls.lock().unwrap().len(), 12);
    }

    #[test]
    fn when_visible_and_empty_slot_then_drawn_with_slot_color() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 1, 1), true);

        // Act
        run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[0].2, SLOT_COLOR);
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
        let entity = world
            .spawn(Shape {
                variant: engine_render::prelude::ShapeVariant::Circle { radius: 1.0 },
                color: card_color,
            })
            .id();

        let mut grid = StashGrid::new(1, 1, 1);
        grid.place(0, 0, 0, entity).unwrap();
        world.insert_resource(grid);

        // Act
        run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[0].2, card_color);
    }

    #[test]
    fn when_visible_then_adjacent_columns_differ_by_slot_stride_in_x() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(2, 1, 1), true);

        // Act
        run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        let x0 = calls[0].0[0][0];
        let x1 = calls[1].0[0][0];
        let dx = x1 - x0;
        assert!(
            (dx - SLOT_STRIDE).abs() < 0.01,
            "expected stride {SLOT_STRIDE}, got {dx}"
        );
    }

    #[test]
    fn when_visible_then_adjacent_rows_differ_by_slot_stride_in_y() {
        // Arrange
        let (mut world, shape_calls) = make_world_with_spy(StashGrid::new(1, 2, 1), true);

        // Act
        run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        let y0 = calls[0].0[0][1];
        let y1 = calls[1].0[0][1];
        let dy = y1 - y0;
        assert!(
            (dy - SLOT_STRIDE).abs() < 0.01,
            "expected stride {SLOT_STRIDE}, got {dy}"
        );
    }
}
