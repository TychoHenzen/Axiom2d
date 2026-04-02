#![allow(clippy::unwrap_used)]

use engine_core::types::Pixels;
use engine_render::prelude::*;
use engine_render::rect::Rect;
use engine_render::renderer::Renderer;
use engine_render::testing::visual_regression::{
    HeadlessRenderer, load_golden, padded_row_bytes, save_golden, ssim_compare, strip_row_padding,
};

#[test]
fn when_comparing_identical_buffers_then_ssim_score_is_one() {
    // Arrange
    let a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
    let b: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);

    // Act
    let score = ssim_compare(&a, &b, 64, 64);

    // Assert
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "identical pixel buffers must yield SSIM=1.0, got {score}"
    );
}

#[test]
fn when_comparing_different_buffers_then_ssim_score_is_less_than_one() {
    // Arrange
    let a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
    let b: Vec<u8> = [0, 0, 255, 255].repeat(64 * 64);

    // Act
    let score = ssim_compare(&a, &b, 64, 64);

    // Assert
    assert!(
        score < 1.0,
        "different buffers must yield SSIM<1.0, got {score}"
    );
}

#[test]
fn when_comparing_slightly_different_buffers_then_ssim_above_threshold() {
    // Arrange
    let a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
    let mut b = a.clone();
    b[0] = 254;

    // Act
    let score = ssim_compare(&a, &b, 64, 64);

    // Assert
    assert!(
        score >= 0.99,
        "single-pixel change in 64x64 must stay above 0.99 threshold, got {score}"
    );
}

#[test]
fn when_computing_padded_row_bytes_then_returns_multiple_of_256() {
    // Act
    let result = padded_row_bytes(65, 4);

    // Assert
    assert_eq!(result, 512);
    assert_eq!(result % 256, 0);
}

#[test]
fn when_width_already_aligned_then_padded_row_bytes_unchanged() {
    // Act
    let result = padded_row_bytes(64, 4);

    // Assert
    assert_eq!(result, 256);
}

#[test]
fn when_stripping_row_padding_then_produces_packed_rgba() {
    // Arrange
    let width = 2u32;
    let height = 2u32;
    let padded = padded_row_bytes(width, 4) as usize;
    let mut data = vec![0u8; padded * height as usize];
    data[0..4].copy_from_slice(&[255, 0, 0, 255]);
    data[4..8].copy_from_slice(&[0, 255, 0, 255]);
    data[padded..padded + 4].copy_from_slice(&[0, 0, 255, 255]);
    data[padded + 4..padded + 8].copy_from_slice(&[255, 255, 255, 255]);

    // Act
    let packed = strip_row_padding(&data, width, height, padded as u32, 4);

    // Assert
    assert_eq!(packed.len(), 2 * 2 * 4);
    assert_eq!(&packed[0..4], &[255, 0, 0, 255]);
    assert_eq!(&packed[4..8], &[0, 255, 0, 255]);
    assert_eq!(&packed[8..12], &[0, 0, 255, 255]);
    assert_eq!(&packed[12..16], &[255, 255, 255, 255]);
}

#[test]
fn when_creating_headless_renderer_then_viewport_matches() {
    // Arrange / Act
    let Some(renderer) = HeadlessRenderer::try_new(128, 128) else {
        return; // no GPU available — skip
    };

    // Assert
    assert_eq!(renderer.viewport_size(), (128, 128));
}

#[test]
fn when_clearing_with_red_then_readback_pixels_are_all_red() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(64, 64) else {
        return;
    };
    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);

    // Act
    renderer.clear(red);
    let pixels = renderer.render_to_buffer();

    // Assert
    assert_eq!(pixels.len(), 64 * 64 * 4);
    for chunk in pixels.chunks_exact(4) {
        assert_eq!(chunk[0], 255, "R channel");
        assert_eq!(chunk[1], 0, "G channel");
        assert_eq!(chunk[2], 0, "B channel");
        assert_eq!(chunk[3], 255, "A channel");
    }
}

#[test]
fn when_saving_golden_image_then_file_exists_at_expected_path() {
    // Arrange
    let dir = std::env::temp_dir().join("axiom2d_golden_test_save");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.png");
    let pixels: Vec<u8> = [255, 0, 0, 255].repeat(4 * 4);

    // Act
    save_golden(&path, &pixels, 4, 4).unwrap();

    // Assert
    assert!(path.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn when_loading_saved_golden_then_pixels_match_original() {
    // Arrange
    let dir = std::env::temp_dir().join("axiom2d_golden_test_roundtrip");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("roundtrip.png");
    let original: Vec<u8> = [255, 0, 0, 255].repeat(4 * 4);
    save_golden(&path, &original, 4, 4).unwrap();

    // Act
    let (loaded, w, h) = load_golden(&path).unwrap();

    // Assert
    assert_eq!(w, 4);
    assert_eq!(h, 4);
    assert_eq!(loaded, original);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn when_loading_nonexistent_golden_then_returns_error() {
    // Arrange
    let path = std::path::Path::new("/nonexistent/golden.png");

    // Act
    let result = load_golden(path);

    // Assert
    assert!(result.is_err());
}

#[test]
fn when_comparing_largely_different_buffers_then_ssim_below_threshold() {
    // Arrange
    let mut a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
    for y in 0..32 {
        for x in 0..32 {
            let idx = (y * 64 + x) * 4;
            a[idx] = 0;
            a[idx + 2] = 255;
        }
    }
    let b: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);

    // Act
    let score = ssim_compare(&a, &b, 64, 64);

    // Assert
    assert!(
        score < 0.99,
        "25% different pixels must fail 0.99 threshold, got {score}"
    );
}

#[test]
fn when_drawing_white_rect_on_black_then_rect_region_is_white() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(64, 64) else {
        return;
    };
    let black = engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0);
    renderer.clear(black);

    let proj = engine_render::camera::CameraUniform::from_camera(
        &engine_render::camera::Camera2D {
            position: glam::Vec2::new(32.0, 32.0),
            zoom: 1.0,
        },
        64.0,
        64.0,
    );
    renderer.set_view_projection(proj.view_proj);

    let white_rect = Rect {
        x: Pixels(16.0),
        y: Pixels(16.0),
        width: Pixels(32.0),
        height: Pixels(32.0),
        color: engine_core::color::Color::WHITE,
    };
    renderer.draw_rect(white_rect);

    // Act
    let pixels = renderer.render_to_buffer();

    // Assert
    let center_idx = (32 * 64 + 32) * 4;
    assert_eq!(pixels[center_idx], 255, "center R");
    assert_eq!(pixels[center_idx + 1], 255, "center G");
    assert_eq!(pixels[center_idx + 2], 255, "center B");
    assert_eq!(pixels[0], 0, "corner R");
    assert_eq!(pixels[1], 0, "corner G");
    assert_eq!(pixels[2], 0, "corner B");
}

#[test]
fn when_rendering_same_scene_twice_then_buffers_are_identical() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(64, 64) else {
        return;
    };
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

    // Act
    renderer.clear(blue);
    let pixels_a = renderer.render_to_buffer();
    renderer.clear(blue);
    let pixels_b = renderer.render_to_buffer();

    // Assert
    assert_eq!(
        pixels_a, pixels_b,
        "two renders of the same scene must be identical"
    );
}

#[test]
fn when_rendered_frame_compared_to_golden_then_ssim_passes_threshold() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(64, 64) else {
        return;
    };
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);
    renderer.clear(blue);
    let pixels = renderer.render_to_buffer();

    let golden = pixels.clone();

    // Act
    let score = ssim_compare(&pixels, &golden, 64, 64);

    // Assert
    assert!(
        score >= 0.99,
        "identical render vs golden must pass 0.99 threshold, got {score}"
    );
}

#[test]
fn when_rendered_frame_differs_from_golden_then_ssim_fails_threshold() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(64, 64) else {
        return;
    };
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);
    renderer.clear(blue);
    let blue_pixels = renderer.render_to_buffer();

    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
    renderer.clear(red);
    let red_pixels = renderer.render_to_buffer();

    // Act
    let score = ssim_compare(&red_pixels, &blue_pixels, 64, 64);

    // Assert
    assert!(
        score < 0.99,
        "different render vs golden must fail 0.99 threshold, got {score}"
    );
}

fn setup_centered_camera(renderer: &mut HeadlessRenderer, size: f32) {
    let half = size / 2.0;
    let proj = engine_render::camera::CameraUniform::from_camera(
        &engine_render::camera::Camera2D {
            position: glam::Vec2::new(half, half),
            zoom: 1.0,
        },
        size,
        size,
    );
    renderer.set_view_projection(proj.view_proj);
}

fn draw_circle_at_center(renderer: &mut HeadlessRenderer, center: f32, radius: f32) {
    let mesh =
        engine_render::shape::tessellate(&engine_render::shape::ShapeVariant::Circle { radius })
            .unwrap();
    let model: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [center, center, 0.0, 1.0],
    ];
    renderer.draw_shape(
        &mesh.vertices,
        &mesh.indices,
        engine_core::color::Color::WHITE,
        model,
    );
}

#[test]
fn when_rendering_circle_shape_then_center_pixel_is_non_background() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(128, 128) else {
        return;
    };
    renderer.clear(engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0));
    setup_centered_camera(&mut renderer, 128.0);
    draw_circle_at_center(&mut renderer, 64.0, 20.0);

    // Act
    let pixels = renderer.render_to_buffer();

    // Assert
    let idx = (64 * 128 + 64) * 4;
    let is_non_black = pixels[idx] > 0 || pixels[idx + 1] > 0 || pixels[idx + 2] > 0;
    assert!(
        is_non_black,
        "center pixel should be non-black after drawing circle, got [{}, {}, {}]",
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2]
    );
}

#[test]
fn when_draw_text_on_headless_then_non_background_pixels_exist() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(128, 128) else {
        return;
    };
    renderer.clear(engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0));
    setup_centered_camera(&mut renderer, 128.0);

    // Act
    renderer.draw_text("A", 40.0, 40.0, 48.0, engine_core::color::Color::WHITE);
    let pixels = renderer.render_to_buffer();

    // Assert
    let has_non_black = pixels
        .chunks_exact(4)
        .any(|px| px[0] > 0 || px[1] > 0 || px[2] > 0);
    assert!(has_non_black, "draw_text must produce visible pixels");
}

#[test]
fn when_draw_text_twice_with_same_input_then_buffers_identical() {
    // Arrange
    let Some(mut renderer) = HeadlessRenderer::try_new(128, 128) else {
        return;
    };

    // Act — first render
    renderer.clear(engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0));
    setup_centered_camera(&mut renderer, 128.0);
    renderer.draw_text("Hi", 20.0, 20.0, 32.0, engine_core::color::Color::WHITE);
    let pixels1 = renderer.render_to_buffer();

    // Act — second render
    renderer.clear(engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0));
    setup_centered_camera(&mut renderer, 128.0);
    renderer.draw_text("Hi", 20.0, 20.0, 32.0, engine_core::color::Color::WHITE);
    let pixels2 = renderer.render_to_buffer();

    // Assert
    assert_eq!(
        pixels1, pixels2,
        "identical draw_text calls must produce identical pixels"
    );
}
