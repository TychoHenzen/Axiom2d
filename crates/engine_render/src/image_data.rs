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
