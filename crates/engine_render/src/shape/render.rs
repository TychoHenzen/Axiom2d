use bevy_ecs::prelude::{Query, ResMut};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use super::components::{Shape, ShapeVariant, Stroke};
use super::tessellate::{shape_aabb, tessellate, tessellate_stroke};
use crate::camera::Camera2D;
use crate::culling::{aabb_intersects_view_rect, camera_view_rect};
use crate::material::{Material2d, apply_material};
use crate::renderer::RendererRes;

pub fn affine2_to_mat4(affine: &glam::Affine2) -> [[f32; 4]; 4] {
    let m = affine.matrix2;
    let t = affine.translation;
    [
        [m.x_axis.x, m.x_axis.y, 0.0, 0.0],
        [m.y_axis.x, m.y_axis.y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t.x, t.y, 0.0, 1.0],
    ]
}

fn is_shape_culled(pos: Vec2, variant: &ShapeVariant, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let (local_min, local_max) = shape_aabb(variant);
    let r = local_min.abs().max(local_max.abs()).length();
    let entity_min = Vec2::new(pos.x - r, pos.y - r);
    let entity_max = Vec2::new(pos.x + r, pos.y + r);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}

fn compute_view_rect(
    camera_query: &Query<&Camera2D>,
    renderer: &RendererRes,
) -> Option<(Vec2, Vec2)> {
    camera_query.iter().next().map(|cam| {
        let (vw, vh) = renderer.viewport_size();
        camera_view_rect(cam, vw as f32, vh as f32)
    })
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
        Option<&Stroke>,
    )>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);
    let mut shapes: Vec<_> = query.iter().filter(|t| t.4.is_none_or(|v| v.0)).collect();
    shapes.sort_by_key(|t| {
        (
            t.2.copied().unwrap_or(RenderLayer::World),
            t.3.copied().unwrap_or_default(),
        )
    });
    let mut last_shader = None;
    let mut last_blend_mode = None;
    for (shape, transform, _, _, _, mat, stroke) in shapes {
        if is_shape_culled(transform.0.translation, &shape.variant, view_rect) {
            continue;
        }
        apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);
        let model = affine2_to_mat4(&transform.0);
        let mesh = tessellate(&shape.variant);
        renderer.draw_shape(&mesh.vertices, &mesh.indices, shape.color, model);
        if let Some(stroke) = stroke {
            let sm = tessellate_stroke(&shape.variant, stroke.width);
            renderer.draw_shape(&sm.vertices, &sm.indices, stroke.color, model);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::color::Color;
    use engine_core::types::TextureId;
    use glam::Affine2;

    use super::*;
    use crate::camera::Camera2D;
    use crate::material::{BlendMode, TextureBinding};
    use crate::shader::ShaderHandle;
    use crate::shape::{PathCommand, ShapeVariant};
    use crate::testing::{
        insert_spy, insert_spy_with_blend_capture, insert_spy_with_shader_capture,
        insert_spy_with_shape_and_viewport, insert_spy_with_shape_capture,
        insert_spy_with_texture_bind_capture, insert_spy_with_uniform_capture,
        insert_spy_with_viewport,
    };

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

    #[test]
    fn when_shape_has_no_stroke_then_draw_shape_called_once() {
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
    fn when_shape_has_stroke_then_draw_shape_called_twice() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Stroke {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                width: 2.0,
            },
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
        assert_eq!(count, 2);
    }

    #[test]
    fn when_shape_has_stroke_then_fill_color_first_stroke_color_second() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Stroke {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                width: 2.0,
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, Color::WHITE);
        assert_eq!(calls[1].2, Color::new(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn when_shape_has_stroke_then_stroke_model_matches_fill_model() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            Stroke {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                width: 2.0,
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — both fill and stroke share the same model matrix
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].3, calls[1].3, "fill and stroke must share model");
        let model = calls[0].3;
        assert!((model[3][0] - 100.0).abs() < 1e-4, "tx={}", model[3][0]);
        assert!((model[3][1] - 50.0).abs() < 1e-4, "ty={}", model[3][1]);
    }

    #[test]
    fn when_invisible_shape_has_stroke_then_neither_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Stroke {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                width: 2.0,
            },
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
    fn when_culled_shape_has_stroke_then_neither_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100_000.0, 100_000.0))),
            Stroke {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                width: 2.0,
            },
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
    fn when_two_shapes_with_different_blend_modes_then_both_blend_modes_applied() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
            Material2d {
                blend_mode: BlendMode::Multiply,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
        ));

        // Act
        run_system(&mut world);

        // Assert — SortOrder determines draw order, not blend mode
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Multiply]);
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
    fn when_shape_at_known_position_then_model_matrix_contains_translation() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert — vertices are local-space, model matrix carries the transform
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let model = calls[0].3;
        assert!((model[3][0] - 100.0).abs() < 1e-4, "tx={}", model[3][0]);
        assert!((model[3][1] - 200.0).abs() < 1e-4, "ty={}", model[3][1]);
        // Local vertices should be centered around origin (circle r=30)
        for vertex in &calls[0].0 {
            assert!(
                vertex[0].abs() <= 30.1,
                "local x={} should be within radius",
                vertex[0]
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

    #[test]
    fn when_shape_barely_inside_view_due_to_radius_then_drawn() {
        // Arrange — circle at (790, 300) r=30: AABB [760, 820] overlaps view [0, 800]
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
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
        // Arrange — circle at (400, 590) r=30: AABB y [560, 620] overlaps view [0, 600]
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
        // Arrange — circle r=30 at (5,5): AABB [-25, 35] overlaps view [0, 100]
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
        // Arrange — circle r=30 at (-20,-20): AABB [-50, 10] edge-touches view [-50, 50]
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

    fn colored_shape(color: Color) -> Shape {
        Shape { color, ..default_shape() }
    }

    #[test]
    fn when_two_shapes_with_different_shaders_then_sort_order_controls_draw_order() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            colored_shape(red),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
            Material2d { shader: ShaderHandle(1), ..Material2d::default() },
        ));
        world.spawn((
            colored_shape(blue),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
            Material2d { shader: ShaderHandle(0), blend_mode: BlendMode::Additive, ..Material2d::default() },
        ));

        // Act
        run_system(&mut world);

        // Assert — SortOrder(0) drawn first regardless of shader
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

    #[test]
    fn when_shape_render_system_has_path_shape_then_draw_shape_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            Shape {
                variant: ShapeVariant::Path {
                    commands: vec![
                        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                        PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                        PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                        PathCommand::Close,
                    ],
                },
                color: Color::new(1.0, 1.0, 1.0, 1.0),
            },
            GlobalTransform2D(Affine2::IDENTITY),
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

    #[test]
    fn when_path_shape_outside_camera_view_then_not_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            Shape {
                variant: ShapeVariant::Path {
                    commands: vec![
                        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                        PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                        PathCommand::LineTo(Vec2::new(5.0, 10.0)),
                        PathCommand::Close,
                    ],
                },
                color: Color::new(1.0, 1.0, 1.0, 1.0),
            },
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
        assert_eq!(count, 0);
    }

    #[test]
    fn when_shape_at_view_edge_then_not_culled() {
        // Arrange — circle r=10 at (395,300): AABB [385,405]×[290,310] overlaps view [0,800]×[0,600]
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(395.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert — the shape should be rendered (not culled)
        let calls = log.lock().unwrap();
        assert!(
            calls.iter().any(|c| c == "draw_shape"),
            "shape at view edge should be rendered, not culled"
        );
    }

    #[test]
    fn when_shape_at_positive_y_near_view_edge_then_not_culled() {
        // Arrange — viewport 2×2 → view y ∈ [-1, 1]; circle r=1 at (0,2): AABB y [0.586, 3.414] overlaps
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 2, 2);
        world.spawn(Camera2D {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        });
        world.spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 1.0 },
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(0.0, 2.0))),
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
        assert_eq!(
            count, 1,
            "shape whose AABB overlaps view_max.y should be rendered"
        );
    }
}
