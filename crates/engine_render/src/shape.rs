use bevy_ecs::prelude::{Component, Query, ResMut};
use engine_core::color::Color;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;
use lyon::math::point;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use serde::{Deserialize, Serialize};

use crate::camera::{Camera2D, aabb_intersects_view_rect, camera_view_rect};
use crate::material::{Material2d, apply_material, effective_blend_mode, effective_shader_handle};
use crate::renderer::RendererRes;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeVariant {
    Circle { radius: f32 },
    Polygon { points: Vec<Vec2> },
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub variant: ShapeVariant,
    pub color: Color,
}

pub struct TessellatedMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub fn tessellate(variant: &ShapeVariant) -> TessellatedMesh {
    let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let options = FillOptions::default();

    match variant {
        ShapeVariant::Circle { radius } => {
            tessellator
                .tessellate_circle(
                    point(0.0, 0.0),
                    *radius,
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("circle tessellation failed");
        }
        ShapeVariant::Polygon { points } => {
            if points.len() < 3 {
                return TessellatedMesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                };
            }
            let lyon_points: Vec<lyon::math::Point> =
                points.iter().map(|p| point(p.x, p.y)).collect();
            tessellator
                .tessellate_polygon(
                    lyon::path::Polygon {
                        points: &lyon_points,
                        closed: true,
                    },
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("polygon tessellation failed");
        }
    }

    TessellatedMesh {
        vertices: geometry.vertices,
        indices: geometry.indices,
    }
}

pub(crate) fn shape_aabb(variant: &ShapeVariant) -> (Vec2, Vec2) {
    match variant {
        ShapeVariant::Circle { radius } => {
            let r = *radius;
            (Vec2::new(-r, -r), Vec2::new(r, r))
        }
        ShapeVariant::Polygon { points } => {
            if points.is_empty() {
                return (Vec2::ZERO, Vec2::ZERO);
            }
            let mut min = points[0];
            let mut max = points[0];
            for &p in &points[1..] {
                min = min.min(p);
                max = max.max(p);
            }
            (min, max)
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn shape_render_system(
    query: Query<(
        &Shape,
        &GlobalTransform2D,
        Option<&RenderLayer>,
        Option<&SortOrder>,
        Option<&EffectiveVisibility>,
        Option<&Material2d>,
    )>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    let view_rect = camera_query.iter().next().map(|camera| {
        let (vw, vh) = renderer.viewport_size();
        camera_view_rect(camera, vw as f32, vh as f32)
    });

    let mut shapes: Vec<_> = query
        .iter()
        .filter(|(_, _, _, _, vis, _)| !vis.is_some_and(|v| !v.0))
        .collect();

    shapes.sort_by_key(|(_, _, layer, sort, _, mat)| {
        (
            layer.copied().unwrap_or(RenderLayer::World),
            effective_shader_handle(*mat),
            effective_blend_mode(*mat),
            sort.copied().unwrap_or_default(),
        )
    });

    let mut last_shader = None;
    let mut last_blend_mode = None;

    for (shape, transform, _, _, _, mat) in shapes {
        let pos = transform.0.translation;
        let (local_min, local_max) = shape_aabb(&shape.variant);

        if let Some((view_min, view_max)) = view_rect {
            let entity_min = Vec2::new(pos.x + local_min.x, pos.y + local_min.y);
            let entity_max = Vec2::new(pos.x + local_max.x, pos.y + local_max.y);
            if !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max) {
                continue;
            }
        }

        apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);

        let mesh = tessellate(&shape.variant);
        let offset_vertices: Vec<[f32; 2]> = mesh
            .vertices
            .iter()
            .map(|v| [v[0] + pos.x, v[1] + pos.y])
            .collect();

        renderer.draw_shape(&offset_vertices, &mesh.indices, shape.color);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use glam::Affine2;

    use super::*;
    use engine_core::types::TextureId;

    use crate::material::{BlendMode, Material2d, ShaderHandle, TextureBinding};
    use crate::testing::{
        BlendCallLog, ShaderCallLog, ShapeCallLog, SpyRenderer, TextureBindCallLog, UniformCallLog,
    };

    #[test]
    fn when_polygon_shape_variant_debug_formatted_then_snapshot_matches() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(80.0, 60.0),
                Vec2::new(20.0, 60.0),
            ],
        };

        // Act
        let debug = format!("{variant:#?}");

        // Assert
        insta::assert_snapshot!(debug);
    }

    #[test]
    fn when_shape_circle_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Circle { radius: 25.0 },
            color: Color::new(0.0, 1.0, 0.0, 1.0),
        };

        // Act
        let ron = ron::to_string(&shape).unwrap();
        let back: Shape = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(shape, back);
    }

    #[test]
    fn when_shape_polygon_serialized_to_ron_then_deserializes_with_point_order_preserved() {
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(100.0, 0.0),
                    Vec2::new(50.0, 86.6),
                ],
            },
            color: Color::RED,
        };

        // Act
        let ron = ron::to_string(&shape).unwrap();
        let back: Shape = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(shape, back);
    }

    /// @doc: Lyon `FillTessellator` generates triangle fan — all circle vertices lie at radius distance from origin
    #[test]
    fn when_tessellating_circle_then_produces_nonempty_vertices_and_indices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_circle_then_index_count_is_multiple_of_three() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_circle_then_all_indices_within_vertex_bounds() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(
                index < vertex_count,
                "index {index} out of bounds (vertex count {vertex_count})"
            );
        }
    }

    #[test]
    fn when_tessellating_zero_radius_circle_then_does_not_panic() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 0.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_larger_circle_then_more_vertices_than_smaller() {
        // Arrange
        let small = ShapeVariant::Circle { radius: 10.0 };
        let large = ShapeVariant::Circle { radius: 100.0 };

        // Act
        let small_mesh = tessellate(&small);
        let large_mesh = tessellate(&large);

        // Assert
        assert!(large_mesh.vertices.len() >= small_mesh.vertices.len());
    }

    #[test]
    fn when_tessellating_triangle_polygon_then_produces_three_vertices_and_three_indices() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(50.0, 86.6),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn when_tessellating_quad_polygon_then_valid_triangulated_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(0.0, 100.0),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
    }

    #[test]
    fn when_circle_aabb_then_width_and_height_equal_double_radius() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-50.0, -50.0));
        assert_eq!(max, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn when_polygon_aabb_then_matches_point_extents() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(-10.0, -20.0),
                Vec2::new(30.0, 40.0),
                Vec2::new(5.0, -5.0),
            ],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-10.0, -20.0));
        assert_eq!(max, Vec2::new(30.0, 40.0));
    }

    /// @doc: Degenerate polygons (< 3 vertices) produce empty mesh — no GPU draw call issued
    #[test]
    fn when_tessellating_polygon_with_fewer_than_three_points_then_returns_empty_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    fn default_shape() -> Shape {
        Shape {
            variant: ShapeVariant::Circle { radius: 30.0 },
            color: Color::WHITE,
        }
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(shape_render_system);
        schedule.run(world);
    }

    fn insert_spy(world: &mut World) -> Arc<Mutex<Vec<String>>> {
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        log
    }

    fn insert_spy_with_shape_capture(world: &mut World) -> ShapeCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_shape_capture(calls.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        calls
    }

    fn insert_spy_with_shader_capture(world: &mut World) -> ShaderCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let shader_calls: ShaderCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_shader_capture(shader_calls.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        shader_calls
    }

    fn insert_spy_with_uniform_capture(world: &mut World) -> UniformCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let uniform_calls: UniformCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_uniform_capture(uniform_calls.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        uniform_calls
    }

    fn insert_spy_with_texture_bind_capture(world: &mut World) -> TextureBindCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let texture_bind_calls: TextureBindCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_texture_bind_capture(texture_bind_calls.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        texture_bind_calls
    }

    fn insert_spy_with_blend_capture(world: &mut World) -> BlendCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let blend_calls: BlendCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_blend_capture(blend_calls.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        blend_calls
    }

    #[test]
    fn when_shape_has_no_material_then_set_blend_mode_called_with_alpha() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha]);
    }

    #[test]
    fn when_shape_has_additive_material_then_set_blend_mode_called_with_additive() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Additive]);
    }

    #[test]
    fn when_two_shapes_with_different_blend_modes_then_set_blend_mode_in_sorted_order() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Multiply,
                ..Material2d::default()
            },
        ));
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Multiply]);
    }

    fn insert_spy_with_viewport(
        world: &mut World,
        width: u32,
        height: u32,
    ) -> Arc<Mutex<Vec<String>>> {
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone()).with_viewport(width, height);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        log
    }

    #[test]
    fn when_shape_with_global_transform_then_draw_shape_called_once() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn when_shape_without_global_transform_then_draw_shape_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn(default_shape());

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn when_shape_with_effective_visibility_false_then_not_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn when_two_visible_shapes_then_draw_shape_called_twice() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn when_two_shapes_on_different_layers_then_background_drawn_before_world() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
    }

    #[test]
    fn when_two_shapes_same_layer_different_sort_order_then_lower_drawn_first() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(10),
        ));
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(1),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
    }

    #[test]
    fn when_shape_has_no_render_layer_then_treated_as_world_layer() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
    }

    #[test]
    fn when_shape_at_known_position_then_vertices_offset_by_translation() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        for vertex in &calls[0].0 {
            assert!(vertex[0] >= 100.0 - 30.0, "x={} should be >= 70", vertex[0]);
            assert!(
                vertex[1] >= 200.0 - 30.0,
                "y={} should be >= 170",
                vertex[1]
            );
        }
    }

    #[test]
    fn when_shape_with_known_color_then_draw_shape_receives_matching_color() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let color = Color::new(1.0, 0.0, 0.0, 1.0);
        world.spawn((
            Shape {
                color,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].2, color);
    }

    #[test]
    fn when_shape_fully_outside_camera_view_then_not_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(2000.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn when_shape_fully_inside_camera_view_then_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(400.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 1);
    }

    fn insert_spy_with_shape_and_viewport(
        world: &mut World,
        width: u32,
        height: u32,
    ) -> ShapeCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_shape_capture(calls.clone())
            .with_viewport(width, height);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        calls
    }

    #[test]
    fn when_shape_barely_inside_view_due_to_radius_then_drawn() {
        // Circle at (790, 300) r=30 → AABB [760, 820] overlaps view [0, 800].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(), // circle radius 30
            GlobalTransform2D(Affine2::from_translation(Vec2::new(790.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "shape at view edge should be drawn");
    }

    #[test]
    fn when_shape_barely_inside_view_due_to_y_radius_then_drawn() {
        // Circle at (400, 590) r=30 → AABB y: [560, 620] overlaps view [0, 600].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(400.0, 590.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "shape at bottom view edge should be drawn");
    }

    #[test]
    fn when_shape_near_view_min_edge_then_drawn() {
        // Circle r=30 at (5,5) → AABB [-25, 35] overlaps view [0, 100].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 100, 100);
        world.spawn(Camera2D {
            position: Vec2::new(50.0, 50.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(5.0, 5.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "shape near view min edge should be drawn");
    }

    #[test]
    fn when_shape_at_negative_pos_inside_view_then_drawn() {
        // Circle r=30 at (-20,-20) → AABB [-50, 10] edge-touches view [-50, 50].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 100, 100);
        world.spawn(Camera2D {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(-20.0, -20.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(
            calls.len(),
            1,
            "shape at negative pos inside view should be drawn"
        );
    }

    #[test]
    fn when_shape_has_material_then_set_shader_called_with_material_shader() {
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(3),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(3)]);
    }

    #[test]
    fn when_shape_has_no_material_then_set_shader_called_with_default() {
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(0)]);
    }

    #[test]
    fn when_shape_has_material_uniforms_then_set_material_uniforms_called() {
        // Arrange
        let mut world = World::new();
        let uniform_calls = insert_spy_with_uniform_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                uniforms: vec![7, 8],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = uniform_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[vec![7u8, 8]]);
    }

    #[test]
    fn when_shape_has_texture_bindings_then_bind_material_texture_called() {
        // Arrange
        let mut world = World::new();
        let texture_bind_calls = insert_spy_with_texture_bind_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                textures: vec![TextureBinding {
                    texture: TextureId(4),
                    binding: 0,
                }],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(TextureId(4), 0)]);
    }

    #[test]
    fn when_two_shapes_with_different_shaders_then_shader_dominates_blend_in_sort() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        // Shape A: ShaderHandle(1), BlendMode::Alpha
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(1),
                blend_mode: BlendMode::Alpha,
                ..Material2d::default()
            },
        ));
        // Shape B: ShaderHandle(0), BlendMode::Additive
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — ShaderHandle(0) < ShaderHandle(1), so blue drawn first
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
    }

    #[test]
    fn when_no_camera_entity_then_all_shapes_drawn_without_culling() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(5000.0, 5000.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 1);
    }
}
