use image::ImageReader;
use std::io::Cursor;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to decode image: {0}")]
    DecodeFailed(#[from] image::ImageError),
}

/// Load an image from raw bytes (PNG, etc.) and return RGBA8 pixel data.
pub fn load_image_from_bytes(data: &[u8]) -> Result<(Vec<u8>, u32, u32), LoadError> {
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| LoadError::DecodeFailed(image::ImageError::IoError(e)))?
        .decode()?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Ok((rgba.into_raw(), w, h))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_png(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
        let mut buf = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buf);
        image::ImageEncoder::write_image(
            encoder,
            img.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
        buf
    }

    // TC018
    #[test]
    fn when_valid_png_bytes_loaded_then_returns_correct_dimensions() {
        // Arrange
        let png = make_png(2, 2);

        // Act
        let (rgba, w, h) = load_image_from_bytes(&png).unwrap();

        // Assert
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        assert_eq!(rgba.len(), 16);
    }

    // TC019
    #[test]
    fn when_invalid_bytes_loaded_then_returns_decode_error() {
        // Arrange
        let garbage = b"not a png";

        // Act
        let result = load_image_from_bytes(garbage);

        // Assert
        assert!(result.is_err());
    }

    // TC020
    #[test]
    fn when_empty_bytes_loaded_then_returns_error_not_panic() {
        // Arrange
        let empty: &[u8] = &[];

        // Act
        let result = load_image_from_bytes(empty);

        // Assert
        assert!(result.is_err());
    }
}
