#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::Color;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_render::shape::{ColorMesh, TessellatedColorMesh};
use engine_render::testing::{
    ColoredMeshCallLog, ShapeCallLog, insert_spy_with_colored_mesh_capture,
    insert_spy_with_shape_capture,
};
use engine_scene::prelude::{EffectiveVisibility, RenderLayer, SortOrder};
use engine_scene::transform_propagation::GlobalTransform2D;
use engine_ui::prelude::*;
use engine_ui::unified_render::unified_render_system;
use glam::{Affine2, Vec2};

fn run_system(world: &mut World) -> ShapeCallLog {
    let shape_calls = insert_spy_with_shape_capture(world);
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(world);
    shape_calls
}

#[test]
fn when_shape_and_text_then_both_drawn() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
    ));
    world.spawn((
        Text {
            content: "A".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
            max_width: None,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(1),
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(
        calls.len() >= 2,
        "should have draw calls for both shape and text, got {}",
        calls.len()
    );
}

#[test]
fn when_text_has_lower_sort_order_then_drawn_before_shape() {
    // Arrange
    let mut world = World::new();
    let shape_y = 100.0;
    let text_y = -100.0;
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(0.0, shape_y))),
        SortOrder::new(5),
        RenderLayer::World,
    ));
    world.spawn((
        Text {
            content: "A".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
            max_width: None,
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(0.0, text_y))),
        SortOrder::new(1),
        RenderLayer::World,
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert — text (SortOrder 1) should draw before shape (SortOrder 5).
    // Text glyphs have y near text_y, shape vertices have y near shape_y.
    let calls = shape_calls.lock().unwrap();
    assert!(calls.len() >= 2, "expected at least 2 draw calls");

    // Find first call from text (model translation y near text_y) and
    // first call from shape (model translation y near shape_y).
    let first_text_idx = calls.iter().position(|c| (c.3[3][1] - text_y).abs() < 50.0);
    let first_shape_idx = calls
        .iter()
        .position(|c| (c.3[3][1] - shape_y).abs() < 50.0);
    assert!(
        first_text_idx.is_some() && first_shape_idx.is_some(),
        "should find both text and shape draw calls"
    );
    assert!(
        first_text_idx.unwrap() < first_shape_idx.unwrap(),
        "text (SortOrder 1) should draw before shape (SortOrder 5)"
    );
}

fn run_system_colored(world: &mut World) -> ColoredMeshCallLog {
    let calls = insert_spy_with_colored_mesh_capture(world);
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(world);
    calls
}

#[test]
fn when_color_mesh_entity_then_draw_colored_mesh_called() {
    // Arrange
    let mut world = World::new();
    let mut mesh = TessellatedColorMesh::new();
    mesh.push_vertices(
        &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0, 1, 2],
        [1.0, 0.0, 0.0, 1.0],
    );
    world.spawn((
        ColorMesh(mesh),
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
    ));

    // Act
    let calls = run_system_colored(&mut world);

    // Assert
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0[0].color, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn when_invisible_then_not_drawn() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
        EffectiveVisibility(false),
    ));
    world.spawn((
        Text {
            content: "A".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
            max_width: None,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(1),
        EffectiveVisibility(false),
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(calls.is_empty());
}

/// @doc: When `CachedMesh` is present, `unified_render_system` must use the cached
/// vertices instead of re-tessellating. This avoids redundant lyon tessellation on
/// every frame for the card game's ~200 shapes.
#[test]
fn when_shape_has_cached_mesh_then_unified_render_uses_cached_vertices() {
    use engine_render::shape::{CachedMesh, TessellatedMesh};

    // Arrange — a fake single-triangle mesh.
    let fake_vertices = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let fake_indices = vec![0_u32, 1, 2];
    let cached = CachedMesh(TessellatedMesh {
        vertices: fake_vertices.clone(),
        indices: fake_indices.clone(),
    });

    let mut world = World::new();
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 30.0 },
            color: Color::WHITE,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
        cached,
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "draw_shape should be called once");
    assert_eq!(
        calls[0].0, fake_vertices,
        "vertices must come from CachedMesh, not tessellate()"
    );
    assert_eq!(
        calls[0].1, fake_indices,
        "indices must come from CachedMesh, not tessellate()"
    );
}

/// @doc: When `CachedMesh` is absent, `unified_render_system` falls back to inline
/// tessellation. This ensures shapes render correctly before `mesh_cache_system` has
/// populated the cache, and in tests that skip caching.
#[test]
fn when_shape_has_no_cached_mesh_then_unified_render_falls_back_to_tessellate() {
    // Arrange — no CachedMesh.
    let mut world = World::new();
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 30.0 },
            color: Color::WHITE,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].0.len() > 3,
        "fallback tessellate path should produce circle vertices, got {} verts",
        calls[0].0.len()
    );
}
