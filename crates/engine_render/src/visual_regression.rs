use std::path::Path;

use image::RgbaImage;

#[derive(Debug)]
pub enum GoldenError {
    Io(std::io::Error),
    Image(image::ImageError),
}

impl std::fmt::Display for GoldenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Image(e) => write!(f, "image error: {e}"),
        }
    }
}

impl std::error::Error for GoldenError {}

impl From<std::io::Error> for GoldenError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<image::ImageError> for GoldenError {
    fn from(e: image::ImageError) -> Self {
        Self::Image(e)
    }
}

pub fn save_golden(
    path: &Path,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<(), GoldenError> {
    let img = RgbaImage::from_raw(width, height, pixels.to_vec())
        .expect("pixel buffer must have exactly width * height * 4 bytes");
    img.save(path)?;
    Ok(())
}

pub fn load_golden(path: &Path) -> Result<(Vec<u8>, u32, u32), GoldenError> {
    let img = image::open(path)?.into_rgba8();
    let (w, h) = img.dimensions();
    Ok((img.into_raw(), w, h))
}

const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;

pub fn padded_row_bytes(width: u32, bytes_per_pixel: u32) -> u32 {
    let raw = width * bytes_per_pixel;
    let align = COPY_BYTES_PER_ROW_ALIGNMENT;
    (raw + align - 1) / align * align
}

pub fn strip_row_padding(data: &[u8], width: u32, height: u32, padded_row: u32) -> Vec<u8> {
    let row_bytes = width * 4;
    let mut out = Vec::with_capacity((row_bytes * height) as usize);
    for y in 0..height {
        let start = (y * padded_row) as usize;
        let end = start + row_bytes as usize;
        out.extend_from_slice(&data[start..end]);
    }
    out
}

pub fn ssim_compare(a: &[u8], b: &[u8], width: u32, height: u32) -> f32 {
    let img_a = RgbaImage::from_raw(width, height, a.to_vec())
        .expect("buffer a must have exactly width * height * 4 bytes");
    let img_b = RgbaImage::from_raw(width, height, b.to_vec())
        .expect("buffer b must have exactly width * height * 4 bytes");
    image_compare::rgba_hybrid_compare(&img_a, &img_b)
        .expect("images must have identical dimensions")
        .score as f32
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::ssim_compare;

    #[test]
    fn when_comparing_identical_buffers_then_ssim_score_is_one() {
        // Arrange
        let a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);
        let b: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);

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
        let a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64); // solid red
        let b: Vec<u8> = vec![0, 0, 255, 255].repeat(64 * 64); // solid blue

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(score < 1.0, "different buffers must yield SSIM<1.0, got {score}");
    }

    #[test]
    fn when_comparing_slightly_different_buffers_then_ssim_above_threshold() {
        // Arrange
        let a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);
        let mut b = a.clone();
        b[0] = 254; // one pixel's red channel differs by 1

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score >= 0.99,
            "single-pixel change in 64x64 must stay above 0.99 threshold, got {score}"
        );
    }

    use super::{padded_row_bytes, strip_row_padding};

    #[test]
    fn when_computing_padded_row_bytes_then_returns_multiple_of_256() {
        // Arrange — 65 pixels wide, 4 bytes per pixel = 260 raw bytes (not aligned)

        // Act
        let result = padded_row_bytes(65, 4);

        // Assert
        assert_eq!(result, 512); // next multiple of 256 above 260
        assert_eq!(result % 256, 0);
    }

    #[test]
    fn when_width_already_aligned_then_padded_row_bytes_unchanged() {
        // Arrange — 64 pixels * 4 bpp = 256, already aligned

        // Act
        let result = padded_row_bytes(64, 4);

        // Assert
        assert_eq!(result, 256);
    }

    #[test]
    fn when_stripping_row_padding_then_produces_packed_rgba() {
        // Arrange — 2x2 image, 4 bpp, padded to 256 bytes per row
        let width = 2u32;
        let height = 2u32;
        let padded = padded_row_bytes(width, 4) as usize; // 256
        let mut data = vec![0u8; padded * height as usize];
        // Row 0: pixel (0,0)=red, pixel (1,0)=green
        data[0..4].copy_from_slice(&[255, 0, 0, 255]);
        data[4..8].copy_from_slice(&[0, 255, 0, 255]);
        // Row 1: pixel (0,1)=blue, pixel (1,1)=white
        data[padded..padded + 4].copy_from_slice(&[0, 0, 255, 255]);
        data[padded + 4..padded + 8].copy_from_slice(&[255, 255, 255, 255]);

        // Act
        let packed = strip_row_padding(&data, width, height, padded as u32);

        // Assert
        assert_eq!(packed.len(), 2 * 2 * 4);
        assert_eq!(&packed[0..4], &[255, 0, 0, 255]); // red
        assert_eq!(&packed[4..8], &[0, 255, 0, 255]); // green
        assert_eq!(&packed[8..12], &[0, 0, 255, 255]); // blue
        assert_eq!(&packed[12..16], &[255, 255, 255, 255]); // white
    }

    use super::{load_golden, save_golden};

    #[test]
    fn when_saving_golden_image_then_file_exists_at_expected_path() {
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_golden_test_save");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        let pixels: Vec<u8> = vec![255, 0, 0, 255].repeat(4 * 4);

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
        let original: Vec<u8> = vec![255, 0, 0, 255].repeat(4 * 4);
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
        let mut a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64); // solid red
        // Change top-left quadrant (32x32 = 1024 pixels) to blue
        for y in 0..32 {
            for x in 0..32 {
                let idx = (y * 64 + x) * 4;
                a[idx] = 0;     // R
                a[idx + 2] = 255; // B
            }
        }
        let b: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64); // solid red

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score < 0.99,
            "25% different pixels must fail 0.99 threshold, got {score}"
        );
    }
}
