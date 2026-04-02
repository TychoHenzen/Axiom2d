#![allow(clippy::unwrap_used)]

use engine_render::image_data::{AtlasError, load_image_bytes};

fn make_1x1_png(r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
    use image::ImageEncoder;
    let mut buf = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut buf);
        encoder
            .write_image(&[r, g, b, a], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
    }
    buf
}

#[test]
fn when_loading_valid_png_then_returns_correct_image_data() {
    // Arrange
    let png_bytes = make_1x1_png(255, 0, 0, 255);

    // Act
    let img = load_image_bytes(&png_bytes).unwrap();

    // Assert
    assert_eq!(img.width, 1);
    assert_eq!(img.height, 1);
    assert_eq!(img.data, vec![255, 0, 0, 255]);
}

#[test]
fn when_loading_invalid_bytes_then_returns_decode_error() {
    // Act
    let result = load_image_bytes(&[0x00, 0x01, 0x02]);

    // Assert
    assert!(matches!(result, Err(AtlasError::DecodeError(_))));
}
