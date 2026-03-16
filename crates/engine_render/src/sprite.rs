use bevy_ecs::prelude::{Component, Query, ResMut};
use engine_core::color::Color;
use engine_core::types::{Pixels, TextureId};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::camera::Camera2D;
use crate::culling::{aabb_intersects_view_rect, camera_view_rect};
use crate::material::{Material2d, apply_material};
use crate::rect::Rect;
use crate::renderer::RendererRes;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: TextureId,
    pub uv_rect: [f32; 4],
    pub color: Color,
    pub width: Pixels,
    pub height: Pixels,
}

#[allow(clippy::type_complexity)]
pub fn sprite_render_system(
    query: Query<(
        &Sprite,
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

    let mut sprites: Vec<_> = query
        .iter()
        .filter(|(_, _, _, _, vis, _)| vis.is_none_or(|v| v.0))
        .collect();

    sprites.sort_by_key(|(_, _, layer, sort, _, _mat)| {
        (
            layer.copied().unwrap_or(RenderLayer::World),
            sort.copied().unwrap_or_default(),
        )
    });

    let mut last_shader = None;
    let mut last_blend_mode = None;

    for (sprite, transform, _, _, _, mat) in sprites {
        let pos = transform.0.translation;

        if let Some((view_min, view_max)) = view_rect {
            let entity_min = Vec2::new(pos.x, pos.y);
            let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
            if !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max) {
                continue;
            }
        }

        apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);

        let rect = Rect {
            x: Pixels(pos.x),
            y: Pixels(pos.y),
            width: sprite.width,
            height: sprite.height,
            color: sprite.color,
        };
        renderer.draw_sprite(rect, sprite.uv_rect);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use bevy_ecs::prelude::*;
    use glam::Affine2;

    use super::*;
    use crate::material::{BlendMode, Material2d, TextureBinding};
    use crate::shader::ShaderHandle;
    use crate::testing::{
        insert_spy, insert_spy_with_blend_and_sprite_capture, insert_spy_with_blend_capture,
        insert_spy_with_shader_capture, insert_spy_with_sprite_capture,
        insert_spy_with_texture_bind_capture, insert_spy_with_uniform_capture,
        insert_spy_with_viewport,
    };

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(sprite_render_system);
        schedule.run(world);
    }

    fn default_sprite() -> Sprite {
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(32.0),
            height: Pixels(32.0),
        }
    }

    #[test]
    fn when_sprite_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let sprite = Sprite {
            texture: TextureId(7),
            uv_rect: [0.1, 0.2, 0.9, 0.8],
            color: Color::new(1.0, 0.5, 0.0, 1.0),
            width: Pixels(64.0),
            height: Pixels(128.0),
        };

        // Act
        let ron = ron::to_string(&sprite).unwrap();
        let back: Sprite = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(sprite, back);
    }

    #[test]
    fn when_entity_has_sprite_and_global_transform_then_draw_sprite_called_once() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn when_entity_has_sprite_but_no_global_transform_then_draw_sprite_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn(default_sprite());

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 0);
    }

    /// @doc: EffectiveVisibility(false) is the earliest cull — filtered before sorting or frustum tests
    #[test]
    fn when_entity_has_effective_visibility_false_then_draw_sprite_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
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
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn when_entity_has_no_effective_visibility_then_draw_sprite_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn when_two_visible_sprites_then_draw_sprite_called_twice() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 2);
    }

    /// @doc: `RenderLayer` is the primary sort key — Background draws before World regardless of `SortOrder`
    #[test]
    fn when_two_sprites_on_different_layers_then_background_drawn_before_world() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
    }

    #[test]
    fn when_two_sprites_same_layer_different_sort_order_then_lower_drawn_first() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(10),
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
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
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
    }

    #[test]
    fn when_sprite_has_no_render_layer_then_treated_as_world_layer() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
    }

    #[test]
    fn when_sprite_has_no_sort_order_then_treated_as_zero() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(-1),
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, red);
        assert_eq!(calls[1].0.color, blue);
    }

    #[test]
    fn when_sprite_at_known_position_then_rect_xy_match_translation() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(100.0, 200.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.x, Pixels(100.0));
        assert_eq!(calls[0].0.y, Pixels(200.0));
    }

    #[test]
    fn when_sprite_with_known_dimensions_then_rect_size_matches() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        world.spawn((
            Sprite {
                width: Pixels(48.0),
                height: Pixels(96.0),
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.width, Pixels(48.0));
        assert_eq!(calls[0].0.height, Pixels(96.0));
    }

    #[test]
    fn when_sprite_with_known_color_then_rect_color_matches() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let color = Color::new(1.0, 0.0, 0.5, 1.0);
        world.spawn((
            Sprite {
                color,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.color, color);
    }

    #[test]
    fn when_sprite_with_known_uv_rect_then_draw_sprite_receives_matching_uv() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let uv = [0.25, 0.0, 0.75, 1.0];
        world.spawn((
            Sprite {
                uv_rect: uv,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].1, uv);
    }

    #[test]
    fn when_sprite_has_no_material_then_set_blend_mode_called_with_alpha() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha]);
    }

    #[test]
    fn when_sprite_has_additive_material_then_set_blend_mode_called_with_additive() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
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
    fn when_sprite_has_multiply_material_then_set_blend_mode_called_with_multiply() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Multiply,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Multiply]);
    }

    #[test]
    fn when_two_sprites_with_different_blend_modes_then_both_blend_modes_applied() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
        ));

        // Act
        run_system(&mut world);

        // Assert — SortOrder determines draw order
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
    }

    /// @doc: `apply_material` deduplicates — `set_blend_mode` only called when mode actually changes between sprites
    #[test]
    fn when_two_sprites_with_same_blend_mode_then_set_blend_mode_called_once() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
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
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], BlendMode::Additive);
    }

    #[test]
    fn when_two_sprites_with_same_blend_mode_then_both_drawn() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn when_different_layers_then_layer_overrides_blend_mode_order() {
        // Arrange
        let mut world = World::new();
        let (blend_calls, sprite_calls) = insert_spy_with_blend_and_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let draws = sprite_calls.lock().unwrap();
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].0.color, red);
        assert_eq!(draws[1].0.color, blue);
        let blends = blend_calls.lock().unwrap();
        assert_eq!(blends.as_slice(), &[BlendMode::Additive, BlendMode::Alpha]);
    }

    #[test]
    fn when_same_layer_and_blend_different_sort_order_then_lower_sort_first() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(10),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(1),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
    }

    #[test]
    fn when_invisible_entity_with_material_then_no_blend_or_draw_calls() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert!(calls.is_empty());
    }

    /// @doc: Frustum culling skips draw calls for sprites whose AABB falls entirely outside the camera view rect
    #[test]
    fn when_sprite_fully_outside_camera_view_then_draw_sprite_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(2000.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn when_sprite_fully_inside_camera_view_then_draw_sprite_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(400.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
    }

    /// @doc: Without a `Camera2D` entity, frustum culling is disabled entirely — all sprites are drawn
    #[test]
    fn when_no_camera_entity_then_all_sprites_drawn_without_culling() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(2000.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn when_sprite_just_inside_view_right_edge_due_to_width_then_drawn() {
        // Sprite at x=-5, width=32 → AABB [-5, 27] overlaps view [0, 800].
        // If + were mutated to -: max = -5-32 = -37 < 0 → culled.
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            Sprite {
                width: Pixels(32.0),
                height: Pixels(32.0),
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(-5.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1, "sprite overlapping left edge should be drawn");
    }

    #[test]
    fn when_sprite_just_inside_view_bottom_edge_due_to_height_then_drawn() {
        // Sprite at y=-5, height=32 → AABB [-5, 27] overlaps view [0, 600].
        // If + mutated to -: max_y = -5-32 = -37 < 0 → culled.
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            Sprite {
                width: Pixels(32.0),
                height: Pixels(32.0),
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(400.0, -5.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1, "sprite overlapping top edge should be drawn");
    }

    #[test]
    fn when_sprite_has_material_then_set_shader_called_with_material_shader() {
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(5),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(5)]);
    }

    #[test]
    fn when_sprite_has_no_material_then_set_shader_called_with_default() {
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(0)]);
    }

    #[test]
    fn when_two_sprites_with_different_shaders_then_set_shader_called_for_each() {
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(1),
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(2),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert!(calls.contains(&ShaderHandle(1)));
        assert!(calls.contains(&ShaderHandle(2)));
    }

    #[test]
    fn when_two_sprites_with_same_shader_then_set_shader_called_once() {
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(2),
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(2),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], ShaderHandle(2));
    }

    #[test]
    fn when_sprite_has_material_uniforms_then_set_material_uniforms_called() {
        // Arrange
        let mut world = World::new();
        let uniform_calls = insert_spy_with_uniform_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                uniforms: vec![1, 2, 3, 4],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = uniform_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[vec![1u8, 2, 3, 4]]);
    }

    #[test]
    fn when_sprite_has_no_material_then_set_material_uniforms_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(!log.iter().any(|s| s == "set_material_uniforms"));
    }

    #[test]
    fn when_sprite_has_empty_uniforms_then_set_material_uniforms_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                uniforms: vec![],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(!log.iter().any(|s| s == "set_material_uniforms"));
    }

    #[test]
    fn when_two_sprites_with_different_shaders_then_sort_order_controls_draw_order() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
            Material2d {
                shader: ShaderHandle(1),
                ..Material2d::default()
            },
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — SortOrder(0) drawn first regardless of shader
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
    }

    #[test]
    fn when_same_shader_different_blend_then_sort_order_controls_draw_order() {
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Alpha,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — SortOrder determines order
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
    }

    #[test]
    fn when_sprite_has_texture_bindings_then_bind_material_texture_called() {
        // Arrange
        let mut world = World::new();
        let texture_bind_calls = insert_spy_with_texture_bind_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                textures: vec![TextureBinding {
                    texture: TextureId(1),
                    binding: 2,
                }],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(TextureId(1), 2)]);
    }

    #[test]
    fn when_sprite_has_multiple_texture_bindings_then_all_forwarded_in_order() {
        // Arrange
        let mut world = World::new();
        let texture_bind_calls = insert_spy_with_texture_bind_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                textures: vec![
                    TextureBinding {
                        texture: TextureId(1),
                        binding: 0,
                    },
                    TextureBinding {
                        texture: TextureId(2),
                        binding: 1,
                    },
                ],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(TextureId(1), 0), (TextureId(2), 1)]);
    }

    #[test]
    fn when_sprite_has_no_material_then_bind_material_texture_not_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(!log.iter().any(|s| s == "bind_material_texture"));
    }

    /// @doc: Edge-touching sprites are drawn — conservative culling avoids popping artifacts at view boundaries
    #[test]
    fn when_sprite_straddles_camera_view_edge_then_draw_sprite_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        // Sprite at x=795, width=32 → AABB [795, 827] overlaps view [0, 800]
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(795.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
    }
}
