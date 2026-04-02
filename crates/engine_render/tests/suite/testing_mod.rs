#![allow(clippy::unwrap_used, clippy::float_cmp)]

use std::sync::{Arc, Mutex};

use engine_core::color::Color;
use engine_core::types::TextureId;
use engine_render::material::BlendMode;
use engine_render::rect::Rect;
use engine_render::renderer::{IDENTITY_MODEL, Renderer};
use engine_render::shader::ShaderHandle;
use engine_render::shape::ColorVertex;
use engine_render::testing::{
    BlendCapture, CompileShaderCapture, ShapeCallLog, SpyRenderer, TextCallLog, TextureBindCapture,
    UniformCapture,
};

fn call_every_renderer_method(spy: &mut SpyRenderer) {
    spy.clear(Color::WHITE);
    spy.draw_rect(Rect::default());
    spy.draw_sprite(Rect::default(), [0.0, 0.0, 1.0, 1.0]);
    spy.draw_shape(
        &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0, 1, 2],
        Color::WHITE,
        IDENTITY_MODEL,
    );
    spy.draw_colored_mesh(
        &[ColorVertex {
            position: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        }],
        &[0],
        IDENTITY_MODEL,
    );
    spy.set_view_projection([[0.0f32; 4]; 4]);
    spy.set_blend_mode(BlendMode::Additive);
    spy.set_shader(ShaderHandle(0));
    spy.set_material_uniforms(&[1]);
    spy.bind_material_texture(TextureId(0), 0);
    spy.draw_text("Test", 0.0, 0.0, 12.0, Color::WHITE);
    spy.compile_shader(ShaderHandle(1), "source").unwrap();
    spy.upload_atlas(&engine_render::testing::helpers::minimal_atlas())
        .unwrap();
    spy.apply_post_process();
    spy.resize(800, 600);
    spy.present();
}

const ALL_METHOD_NAMES: &[&str] = &[
    "clear",
    "draw_rect",
    "draw_sprite",
    "draw_shape",
    "draw_colored_mesh",
    "set_view_projection",
    "set_blend_mode",
    "set_shader",
    "set_material_uniforms",
    "bind_material_texture",
    "draw_text",
    "compile_shader",
    "upload_atlas",
    "apply_post_process",
    "resize",
    "present",
];

#[test]
fn when_each_method_called_then_log_records_matching_string() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log.clone());

    // Act
    call_every_renderer_method(&mut spy);

    // Assert
    let entries = log.lock().unwrap();
    assert_eq!(entries.as_slice(), ALL_METHOD_NAMES);
}

#[test]
fn when_draw_shape_called_with_capture_then_color_matches() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_shape_capture(shape_calls.clone());
    let color = Color::new(1.0, 0.0, 0.0, 1.0);

    // Act
    spy.draw_shape(
        &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0, 1, 2],
        color,
        IDENTITY_MODEL,
    );

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].2, color);
}

#[test]
fn when_set_blend_mode_called_twice_with_capture_then_both_calls_recorded_in_order() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let blend_calls: BlendCapture = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_blend_capture(blend_calls.clone());

    // Act
    spy.set_blend_mode(BlendMode::Alpha);
    spy.set_blend_mode(BlendMode::Additive);

    // Assert
    let calls = blend_calls.lock().unwrap();
    assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
}

#[test]
fn when_set_shader_called_with_capture_then_handle_matches() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let shader_calls: engine_render::testing::ShaderCapture = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_shader_capture(shader_calls.clone());

    // Act
    spy.set_shader(ShaderHandle(7));

    // Assert
    let calls = shader_calls.lock().unwrap();
    assert_eq!(calls.as_slice(), &[ShaderHandle(7)]);
}

#[test]
fn when_set_material_uniforms_called_with_capture_then_bytes_match() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let uniform_calls: UniformCapture = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_uniform_capture(uniform_calls.clone());

    // Act
    spy.set_material_uniforms(&[10, 20]);

    // Assert
    let calls = uniform_calls.lock().unwrap();
    assert_eq!(calls.as_slice(), &[vec![10u8, 20]]);
}

#[test]
fn when_bind_material_texture_called_with_capture_then_entry_matches() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let texture_bind_calls: TextureBindCapture = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_texture_bind_capture(texture_bind_calls.clone());

    // Act
    spy.bind_material_texture(TextureId(3), 1);

    // Assert
    let calls = texture_bind_calls.lock().unwrap();
    assert_eq!(calls.as_slice(), &[(TextureId(3), 1)]);
}

#[test]
fn when_compile_shader_called_with_capture_then_handle_and_source_recorded() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let capture: CompileShaderCapture = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_compile_shader_capture(capture.clone());

    // Act
    spy.compile_shader(ShaderHandle(7), "fn vs_shape() {}")
        .unwrap();

    // Assert
    let calls = capture.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, ShaderHandle(7));
    assert_eq!(calls[0].1, "fn vs_shape() {}");
}

#[test]
fn when_draw_text_called_with_capture_then_arguments_match() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let text_calls: TextCallLog = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_text_capture(text_calls.clone());
    let color = Color::WHITE;

    // Act
    spy.draw_text("Hello", 10.0, 20.0, 12.0, color);

    // Assert
    let calls = text_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "Hello");
    assert_eq!(calls[0].1, 10.0);
    assert_eq!(calls[0].2, 20.0);
    assert_eq!(calls[0].3, 12.0);
    assert_eq!(calls[0].4, color);
}

#[test]
fn when_draw_text_called_twice_then_both_captured_in_order() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let text_calls: TextCallLog = Arc::new(Mutex::new(Vec::new()));
    let mut spy = SpyRenderer::new(log).with_text_capture(text_calls.clone());

    // Act
    spy.draw_text("Name", 0.0, 0.0, 10.0, Color::WHITE);
    spy.draw_text("Description", 0.0, 30.0, 8.0, Color::BLACK);

    // Assert
    let calls = text_calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "Name");
    assert_eq!(calls[1].0, "Description");
}

#[test]
fn when_clear_called_with_color_capture_then_color_is_stored() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let color_capture = Arc::new(Mutex::new(None));
    let mut spy = SpyRenderer::new(log.clone()).with_color_capture(color_capture.clone());
    let expected = Color::new(1.0, 0.0, 0.5, 1.0);

    // Act
    spy.clear(expected);

    // Assert
    assert_eq!(*color_capture.lock().unwrap(), Some(expected));
}
