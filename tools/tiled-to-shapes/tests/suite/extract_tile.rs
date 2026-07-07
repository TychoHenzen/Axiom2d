#![allow(clippy::unwrap_used)]

use image::{Rgba, RgbaImage};
use tiled_to_shapes::extract::extract_tile;

/// @doc: extract_tile returns correct RGBA bytes for a known tile position
#[test]
fn when_extract_known_tile_then_returns_correct_pixels() {
    // Arrange — 4×4 image, 2×2 tiles, 2 columns
    let mut img = RgbaImage::new(4, 4);
    let red = Rgba([255u8, 0, 0, 255]);
    let green = Rgba([0u8, 255, 0, 255]);
    // Tile 0 (TL): red, Tile 1 (TR): green
    for y in 0..2u32 {
        for x in 0..2u32 {
            img.put_pixel(x, y, red);
        }
        for x in 2..4u32 {
            img.put_pixel(x, y, green);
        }
    }

    // Act
    let pixels = extract_tile(&img, 0, 2, 2, 2).expect("extract tile 0 should succeed");

    // Assert — 2×2 tile = 4 pixels × 4 channels = 16 bytes, all red
    assert_eq!(
        pixels.len(),
        16,
        "tile should have 16 bytes (4 pixels × 4 channels)"
    );
    assert!(
        pixels.chunks(4).all(|p| p == [255, 0, 0, 255]),
        "all pixels in tile 0 should be red [255,0,0,255]"
    );
}

/// @doc: Out-of-bounds tile ID returns TileIdOutOfBounds error
#[test]
fn when_tile_id_out_of_bounds_then_returns_error() {
    // Arrange — 4×4 image, 2×2 tiles, 2 columns → tile IDs 0-3 valid
    let img = RgbaImage::new(4, 4);

    // Act
    let result = extract_tile(&img, 4, 2, 2, 2);

    // Assert
    assert!(
        result.is_err(),
        "tile ID 4 should be out of bounds for 4-tile sheet"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("out of bounds"),
        "error should mention out of bounds, got: {err}"
    );
}
