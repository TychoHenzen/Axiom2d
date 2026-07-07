#![allow(clippy::unwrap_used)]

use img_to_shape::scale2x::scale2x_rgba;

/// @doc: scale2x on a 1×1 image produces a 2×2 image with the same pixel replicated
#[test]
fn when_1x1_image_then_2x2_replicated() {
    // Arrange — 1×1 red pixel
    let input = vec![255u8, 0, 0, 255];
    let width: u32 = 1;
    let height: u32 = 1;

    // Act
    let result = scale2x_rgba(&input, width, height);

    // Assert — 2×2 = 4 pixels = 16 bytes, all red
    assert_eq!(
        result.len(),
        16,
        "2×2 RGBA should be 16 bytes, got {}",
        result.len()
    );
    assert!(
        result.chunks(4).all(|p| p == [255, 0, 0, 255]),
        "all pixels in 2×2 output should be red"
    );
}

/// @doc: scale2x on empty input returns empty output
#[test]
fn when_empty_input_then_returns_empty() {
    // Arrange
    let input: Vec<u8> = vec![];
    let width: u32 = 0;
    let height: u32 = 0;

    // Act
    let result = scale2x_rgba(&input, width, height);

    // Assert
    assert!(result.is_empty(), "empty input should produce empty output");
}

/// @doc: scale2x preserves RGBA channel order
#[test]
fn when_multicolor_image_then_channels_preserved() {
    // Arrange — 1×2 image with red and green
    let input = vec![
        255, 0, 0, 255,   // red
        0, 255, 0, 255,   // green
    ];
    let width: u32 = 1;
    let height: u32 = 2;

    // Act
    let result = scale2x_rgba(&input, width, height);

    // Assert — 2×4 = doubled in both dimensions
    assert_eq!(result.len(), 2 * 4 * 4, "should be 2×4 output in RGBA");
    // Top rows should be red, bottom rows green
    assert_eq!(&result[0..4], &[255, 0, 0, 255], "top-left should be red");
    assert_eq!(&result[4..8], &[255, 0, 0, 255], "top-right should be red");
}
