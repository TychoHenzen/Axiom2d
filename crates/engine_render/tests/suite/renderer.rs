#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use engine_core::color::Color;
use engine_core::types::Pixels;
use engine_render::material::BlendMode;
use engine_render::rect::Rect;
use engine_render::renderer::{IDENTITY_MODEL, NullRenderer, Renderer, RendererRes};
use engine_render::shader::ShaderHandle;
use engine_render::shape::ColorVertex;
use engine_render::testing::SpyRenderer;
use engine_render::testing::helpers::minimal_atlas;

fn sample_rect() -> Rect {
    Rect {
        x: Pixels(10.0),
        y: Pixels(20.0),
        width: Pixels(100.0),
        height: Pixels(50.0),
        color: Color::WHITE,
    }
}

#[test]
fn when_null_renderer_all_methods_called_then_none_panic() {
    // Arrange
    let mut renderer = NullRenderer;

    // Act
    renderer.clear(Color::BLACK);
    renderer.draw_rect(sample_rect());
    renderer.draw_sprite(sample_rect(), [0.0, 0.0, 1.0, 1.0]);
    renderer.draw_shape(
        &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0, 1, 2],
        Color::WHITE,
        IDENTITY_MODEL,
    );
    renderer.draw_colored_mesh(
        &[ColorVertex {
            position: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        }],
        &[0],
        IDENTITY_MODEL,
    );
    renderer.set_view_projection([[0.0f32; 4]; 4]);
    renderer.set_blend_mode(BlendMode::Alpha);
    renderer.set_shader(ShaderHandle(0));
    renderer.set_material_uniforms(&[1, 2, 3]);
    renderer.bind_material_texture(engine_core::types::TextureId(0), 2);
    renderer.upload_atlas(&minimal_atlas()).unwrap();
    renderer
        .compile_shader(ShaderHandle(1), "@vertex fn vs_shape() {}")
        .unwrap();
    renderer.draw_text("Test", 10.0, 20.0, 12.0, Color::WHITE);
    renderer.apply_post_process();
    renderer.resize(800, 600);
    renderer.present();
    assert_eq!(renderer.viewport_size(), (0, 0));
}

#[test]
fn when_renderer_res_in_world_then_system_can_call_clear_via_resmut() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone());
    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(|mut renderer: bevy_ecs::prelude::ResMut<RendererRes>| {
        renderer.clear(Color::BLACK);
    });

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(log.lock().unwrap().as_slice(), &["clear"]);
}
