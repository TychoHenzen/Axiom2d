#[derive(Debug, thiserror::Error)]
pub enum AtlasError {
    #[error("atlas is full")]
    NoSpace,
    #[error("data length mismatch: expected {expected} bytes, got {actual}")]
    DataLengthMismatch { expected: usize, actual: usize },
    #[error("invalid dimensions: width and height must be non-zero")]
    InvalidDimensions,
    #[error("image decode error: {0}")]
    DecodeError(String),
}

pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn load_image_bytes(bytes: &[u8]) -> Result<ImageData, AtlasError> {
    let img = image::load_from_memory(bytes).map_err(|e| AtlasError::DecodeError(e.to_string()))?;
    let rgba = img.to_rgba8();
    Ok(ImageData {
        width: rgba.width(),
        height: rgba.height(),
        data: rgba.into_raw(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
}
