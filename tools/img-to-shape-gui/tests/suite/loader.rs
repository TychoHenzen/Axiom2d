#![allow(clippy::unwrap_used)]

use img_to_shape_gui::loader::load_image_from_bytes;

/// @doc: Valid PNG bytes decode into RGBA with correct dimensions
#[test]
fn when_valid_png_then_returns_rgba_dimensions() {
    // Arrange — minimal 2×2 red PNG
    let img = image::RgbaImage::from_fn(2, 2, |_, _| image::Rgba([255, 0, 0, 255]));
    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
    .expect("PNG encode should succeed");

    // Act
    let result = load_image_from_bytes(&buf);

    // Assert
    assert!(
        result.is_ok(),
        "valid PNG should decode: {:?}",
        result.err()
    );
    let (rgba, width, height) = result.unwrap();
    assert_eq!(width, 2, "width mismatch");
    assert_eq!(height, 2, "height mismatch");
    assert_eq!(rgba.len(), 16, "RGBA should be 2×2×4=16 bytes, got {}", rgba.len());
    assert!(
        rgba.chunks(4).all(|p| p == [255, 0, 0, 255]),
        "all pixels should be solid red"
    );
}

/// @doc: Invalid image bytes return LoadError
#[test]
fn when_invalid_bytes_then_returns_error() {
    // Arrange
    let garbage = b"this is not an image";

    // Act
    let result = load_image_from_bytes(garbage);

    // Assert
    assert!(
        result.is_err(),
        "invalid image bytes should return error"
    );
}
