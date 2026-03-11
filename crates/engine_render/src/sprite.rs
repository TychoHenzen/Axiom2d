use bevy_ecs::prelude::{Component, Query, ResMut};
use engine_core::color::Color;
use engine_core::types::{Pixels, TextureId};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use crate::camera::{Camera2D, aabb_intersects_view_rect, camera_view_rect};
use crate::rect::Rect;
use crate::renderer::RendererRes;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
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
        .filter(|(_, _, _, _, vis)| !vis.is_some_and(|v| !v.0))
        .collect();

    sprites.sort_by_key(|(_, _, layer, sort, _)| {
        (
            layer.copied().unwrap_or(RenderLayer::World),
            sort.copied().unwrap_or_default(),
        )
    });

    for (sprite, transform, _, _, _) in sprites {
        let pos = transform.0.translation;

        if let Some((view_min, view_max)) = view_rect {
            let entity_min = Vec2::new(pos.x, pos.y);
            let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
            if !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max) {
                continue;
            }
        }

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
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use glam::Affine2;

    use super::*;
    use crate::testing::SpyRenderer;

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

    fn insert_spy(world: &mut World) -> Arc<Mutex<Vec<String>>> {
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        log
    }

    fn insert_spy_with_capture(world: &mut World) -> crate::testing::SpriteCallLog {
        let log = Arc::new(Mutex::new(Vec::new()));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::with_sprite_capture(log, calls.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));
        calls
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
    fn when_entity_has_effective_visibility_true_then_draw_sprite_called() {
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(true),
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

    #[test]
    fn when_two_sprites_on_different_layers_then_background_drawn_before_world() {
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
        let calls = insert_spy_with_capture(&mut world);
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
