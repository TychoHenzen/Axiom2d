use image::RgbaImage;

use crate::types::TiledToShapesError;

/// Extract raw RGBA pixel data for a single tile from a tilesheet image.
///
/// Returns the pixel bytes for the tile at the given grid position.
/// The caller feeds this directly into `img_to_shape::image_to_shapes()`.
pub fn extract_tile(
    image_data: &RgbaImage,
    tile_id: u32,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
) -> Result<Vec<u8>, TiledToShapesError> {
    let img_w = image_data.width();
    let img_h = image_data.height();

    // Validate tile_id is within the grid
    let total_rows = img_h / tile_height;
    let total_cols = columns;
    let max_id = total_rows.saturating_mul(total_cols).saturating_sub(1);
    if tile_id > max_id {
        return Err(TiledToShapesError::TileIdOutOfBounds { tile_id, max_id });
    }

    let col = tile_id % columns;
    let row = tile_id / columns;
    let x = col * tile_width;
    let y = row * tile_height;

    // Bounds-check the sub-rectangle against the image
    if x + tile_width > img_w || y + tile_height > img_h {
        return Err(TiledToShapesError::TileIdOutOfBounds { tile_id, max_id });
    }

    let mut pixels = Vec::with_capacity((tile_width * tile_height * 4) as usize);
    for py in y..y + tile_height {
        for px in x..x + tile_width {
            let p = image_data.get_pixel(px, py);
            pixels.extend_from_slice(&p.0);
        }
    }

    Ok(pixels)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn make_2x2_tilesheet() -> RgbaImage {
        // 4×4 image containing four 2×2 tiles:
        // TL=red, TR=green, BL=blue, BR=white
        let mut img = RgbaImage::new(4, 4);
        let red = Rgba([255u8, 0, 0, 255]);
        let green = Rgba([0u8, 255, 0, 255]);
        let blue = Rgba([0u8, 0, 255, 255]);
        let white = Rgba([255u8, 255, 255, 255]);

        // Tile 0 (TL): columns 0-1, rows 0-1
        for y in 0..2u32 {
            for x in 0..2u32 {
                img.put_pixel(x, y, red);
            }
        }
        // Tile 1 (TR): columns 2-3, rows 0-1
        for y in 0..2u32 {
            for x in 2..4u32 {
                img.put_pixel(x, y, green);
            }
        }
        // Tile 2 (BL): columns 0-1, rows 2-3
        for y in 2..4u32 {
            for x in 0..2u32 {
                img.put_pixel(x, y, blue);
            }
        }
        // Tile 3 (BR): columns 2-3, rows 2-3
        for y in 2..4u32 {
            for x in 2..4u32 {
                img.put_pixel(x, y, white);
            }
        }
        img
    }

    #[test]
    fn when_tile_0_then_red_pixels() {
        // Arrange
        let img = make_2x2_tilesheet();
        // Act
        let pixels = extract_tile(&img, 0, 2, 2, 2).expect("extract should succeed");
        // Assert — all pixels are red
        assert_eq!(pixels.len(), 2 * 2 * 4);
        assert!(
            pixels.chunks(4).all(|p| p == [255, 0, 0, 255]),
            "expected all red pixels, got: {pixels:?}"
        );
    }

    #[test]
    fn when_tile_1_then_green_pixels() {
        let img = make_2x2_tilesheet();
        let pixels = extract_tile(&img, 1, 2, 2, 2).expect("extract should succeed");
        assert!(pixels.chunks(4).all(|p| p == [0, 255, 0, 255]));
    }

    #[test]
    fn when_tile_2_then_blue_pixels() {
        let img = make_2x2_tilesheet();
        let pixels = extract_tile(&img, 2, 2, 2, 2).expect("extract should succeed");
        assert!(pixels.chunks(4).all(|p| p == [0, 0, 255, 255]));
    }

    #[test]
    fn when_out_of_bounds_tile_id_then_error() {
        // Arrange — 4×4 image, 2×2 tiles, 2 columns → 4 tiles (IDs 0-3)
        let img = make_2x2_tilesheet();
        // Act
        let result = extract_tile(&img, 4, 2, 2, 2);
        // Assert
        assert!(
            matches!(
                result,
                Err(TiledToShapesError::TileIdOutOfBounds { tile_id: 4, .. })
            ),
            "expected TileIdOutOfBounds, got: {result:?}"
        );
    }
}
