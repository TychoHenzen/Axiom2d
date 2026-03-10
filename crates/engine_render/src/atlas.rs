use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use engine_core::types::TextureId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureHandle {
    pub texture_id: TextureId,
    pub uv_rect: [f32; 4],
}

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

pub(crate) fn normalize_uv_rect(
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    atlas_w: u32,
    atlas_h: u32,
) -> [f32; 4] {
    let aw = atlas_w as f32;
    let ah = atlas_h as f32;
    [
        x as f32 / aw,
        y as f32 / ah,
        (x + w) as f32 / aw,
        (y + h) as f32 / ah,
    ]
}

#[derive(Resource)]
pub struct TextureAtlas {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub lookups: HashMap<TextureId, [f32; 4]>,
}

impl TextureAtlas {
    pub fn lookup(&self, id: TextureId) -> Option<[f32; 4]> {
        self.lookups.get(&id).copied()
    }
}

struct PendingImage {
    data: Vec<u8>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    texture_id: TextureId,
}

pub struct AtlasBuilder {
    allocator: guillotiere::AtlasAllocator,
    next_id: u32,
    pending: Vec<PendingImage>,
}

impl AtlasBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        let size = guillotiere::size2(width as i32, height as i32);
        Self {
            allocator: guillotiere::AtlasAllocator::new(size),
            next_id: 0,
            pending: Vec::new(),
        }
    }

    pub fn width(&self) -> u32 {
        self.allocator.size().width as u32
    }

    pub fn height(&self) -> u32 {
        self.allocator.size().height as u32
    }

    pub fn add_image(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<TextureHandle, AtlasError> {
        if width == 0 || height == 0 {
            return Err(AtlasError::InvalidDimensions);
        }
        let expected = (width * height * 4) as usize;
        if data.len() != expected {
            return Err(AtlasError::DataLengthMismatch {
                expected,
                actual: data.len(),
            });
        }
        let alloc = self
            .allocator
            .allocate(guillotiere::size2(width as i32, height as i32))
            .ok_or(AtlasError::NoSpace)?;
        let atlas_w = self.width();
        let atlas_h = self.height();
        let rect = alloc.rectangle;
        let uv_rect = normalize_uv_rect(
            rect.min.x as u32,
            rect.min.y as u32,
            (rect.max.x - rect.min.x) as u32,
            (rect.max.y - rect.min.y) as u32,
            atlas_w,
            atlas_h,
        );
        let id = self.next_id;
        self.next_id += 1;
        self.pending.push(PendingImage {
            data: data.to_vec(),
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: (rect.max.x - rect.min.x) as u32,
            height: (rect.max.y - rect.min.y) as u32,
            texture_id: TextureId(id),
        });
        Ok(TextureHandle {
            texture_id: TextureId(id),
            uv_rect,
        })
    }

    pub fn build(self) -> TextureAtlas {
        let width = self.width();
        let height = self.height();
        let mut data = vec![0u8; (width * height * 4) as usize];
        let mut lookups = HashMap::new();
        let stride = (width * 4) as usize;
        for img in &self.pending {
            let uv_rect = normalize_uv_rect(img.x, img.y, img.width, img.height, width, height);
            lookups.insert(img.texture_id, uv_rect);
            let img_stride = (img.width * 4) as usize;
            for row in 0..img.height {
                let dst_offset = ((img.y + row) as usize) * stride + (img.x as usize) * 4;
                let src_offset = (row as usize) * img_stride;
                data[dst_offset..dst_offset + img_stride]
                    .copy_from_slice(&img.data[src_offset..src_offset + img_stride]);
            }
        }
        TextureAtlas {
            data,
            width,
            height,
            lookups,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AtlasBuilder, AtlasError, load_image_bytes, normalize_uv_rect};
    use engine_core::types::TextureId;

    #[test]
    fn when_building_empty_atlas_then_pixel_buffer_is_all_zeros() {
        // Arrange
        let builder = AtlasBuilder::new(4, 4);

        // Act
        let atlas = builder.build();

        // Assert
        assert_eq!(atlas.data, vec![0u8; 4 * 4 * 4]);
    }

    #[test]
    fn when_building_atlas_with_image_then_buffer_size_matches_atlas() {
        // Arrange
        let mut builder = AtlasBuilder::new(64, 128);
        let pixel_data = vec![255u8; 2 * 2 * 4];
        builder.add_image(2, 2, &pixel_data).unwrap();

        // Act
        let atlas = builder.build();

        // Assert
        assert_eq!(atlas.data.len(), 64 * 128 * 4);
    }

    #[test]
    fn when_builder_created_then_reports_matching_dimensions() {
        // Arrange
        let builder = AtlasBuilder::new(512, 256);

        // Act
        let w = builder.width();
        let h = builder.height();

        // Assert
        assert_eq!(w, 512);
        assert_eq!(h, 256);
    }

    #[test]
    fn when_adding_single_image_then_returns_handle_with_valid_texture_id() {
        // Arrange
        let mut builder = AtlasBuilder::new(512, 512);

        // Act
        let result = builder.add_image(1, 1, &[255, 0, 0, 255]);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn when_adding_image_then_uv_rect_is_normalized_to_zero_one() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let handle = builder.add_image(2, 2, &[255; 16]).unwrap();

        // Assert
        let [u0, v0, u1, v1] = handle.uv_rect;
        assert!((0.0..=1.0).contains(&u0));
        assert!((0.0..=1.0).contains(&v0));
        assert!((0.0..=1.0).contains(&u1));
        assert!((0.0..=1.0).contains(&v1));
        assert!(u1 > u0, "uv_rect must have positive width");
        assert!(v1 > v0, "uv_rect must have positive height");
    }

    #[test]
    fn when_adding_image_that_fills_atlas_then_uv_rect_is_full_range() {
        // Arrange
        let mut builder = AtlasBuilder::new(4, 4);

        // Act
        let handle = builder.add_image(4, 4, &[255; 64]).unwrap();

        // Assert
        assert_eq!(handle.uv_rect, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn when_adding_two_images_then_each_has_distinct_texture_id() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let h1 = builder.add_image(2, 2, &[255; 16]).unwrap();
        let h2 = builder.add_image(2, 2, &[0; 16]).unwrap();

        // Assert
        assert_ne!(h1.texture_id, h2.texture_id);
    }

    #[test]
    fn when_adding_two_images_then_uv_rects_do_not_overlap() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let h1 = builder.add_image(4, 4, &[255; 64]).unwrap();
        let h2 = builder.add_image(4, 4, &[0; 64]).unwrap();

        // Assert — convert to pixel rects and check no overlap
        let [u0a, v0a, u1a, v1a] = h1.uv_rect;
        let [u0b, v0b, u1b, v1b] = h2.uv_rect;
        let no_overlap = u1a <= u0b || u1b <= u0a || v1a <= v0b || v1b <= v0a;
        assert!(
            no_overlap,
            "uv_rects overlap: {:?} vs {:?}",
            h1.uv_rect, h2.uv_rect
        );
    }

    #[test]
    fn when_adding_many_images_then_all_uv_rects_are_non_overlapping() {
        // Arrange
        let mut builder = AtlasBuilder::new(512, 512);
        let pixel_data = [128u8; 32 * 32 * 4];

        // Act
        let handles: Vec<_> = (0..16)
            .map(|_| builder.add_image(32, 32, &pixel_data).unwrap())
            .collect();

        // Assert — pairwise non-overlap
        for i in 0..handles.len() {
            for j in (i + 1)..handles.len() {
                let [u0a, v0a, u1a, v1a] = handles[i].uv_rect;
                let [u0b, v0b, u1b, v1b] = handles[j].uv_rect;
                let no_overlap = u1a <= u0b || u1b <= u0a || v1a <= v0b || v1b <= v0a;
                assert!(no_overlap, "handles {} and {} overlap", i, j);
            }
        }
    }

    #[test]
    fn when_adding_image_larger_than_atlas_then_returns_no_space_error() {
        // Arrange
        let mut builder = AtlasBuilder::new(8, 8);

        // Act
        let result = builder.add_image(16, 16, &[0; 16 * 16 * 4]);

        // Assert
        assert!(matches!(result, Err(AtlasError::NoSpace)));
    }

    #[test]
    fn when_atlas_full_then_returns_no_space_error() {
        // Arrange
        let mut builder = AtlasBuilder::new(4, 4);
        builder.add_image(4, 4, &[255; 64]).unwrap();

        // Act
        let result = builder.add_image(1, 1, &[0; 4]);

        // Assert
        assert!(matches!(result, Err(AtlasError::NoSpace)));
    }

    #[test]
    fn when_data_length_mismatches_then_returns_error() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let result = builder.add_image(1, 1, &[255, 0, 0]);

        // Assert
        assert!(matches!(
            result,
            Err(AtlasError::DataLengthMismatch {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn when_adding_zero_width_image_then_returns_invalid_dimensions() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let result = builder.add_image(0, 4, &[]);

        // Assert
        assert!(matches!(result, Err(AtlasError::InvalidDimensions)));
    }

    #[test]
    fn when_adding_zero_height_image_then_returns_invalid_dimensions() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let result = builder.add_image(4, 0, &[]);

        // Assert
        assert!(matches!(result, Err(AtlasError::InvalidDimensions)));
    }

    #[test]
    fn when_looking_up_known_texture_id_then_returns_matching_uv_rect() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);
        let handle = builder.add_image(2, 2, &[255; 16]).unwrap();
        let atlas = builder.build();

        // Act
        let result = atlas.lookup(handle.texture_id);

        // Assert
        assert_eq!(result, Some(handle.uv_rect));
    }

    #[test]
    fn when_looking_up_unknown_texture_id_then_returns_none() {
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);
        builder.add_image(2, 2, &[255; 16]).unwrap();
        let atlas = builder.build();

        // Act
        let result = atlas.lookup(TextureId(99));

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn when_looking_up_multiple_textures_then_each_returns_its_own_uv_rect() {
        // Arrange
        let mut builder = AtlasBuilder::new(64, 64);
        let h1 = builder.add_image(4, 4, &[255; 64]).unwrap();
        let h2 = builder.add_image(4, 4, &[128; 64]).unwrap();
        let h3 = builder.add_image(4, 4, &[0; 64]).unwrap();
        let atlas = builder.build();

        // Act + Assert
        assert_eq!(atlas.lookup(h1.texture_id), Some(h1.uv_rect));
        assert_eq!(atlas.lookup(h2.texture_id), Some(h2.uv_rect));
        assert_eq!(atlas.lookup(h3.texture_id), Some(h3.uv_rect));
        assert_ne!(h1.uv_rect, h2.uv_rect);
        assert_ne!(h2.uv_rect, h3.uv_rect);
    }

    #[test]
    fn when_building_atlas_then_pixel_data_appears_at_correct_offset() {
        // Arrange
        let mut builder = AtlasBuilder::new(8, 8);
        let red = [255, 0, 0, 255].repeat(2 * 2);
        let handle = builder.add_image(2, 2, &red).unwrap();

        // Act
        let atlas = builder.build();

        // Assert — sample the top-left pixel of the allocation
        let [u0, v0, _, _] = handle.uv_rect;
        let px = (u0 * atlas.width as f32) as usize;
        let py = (v0 * atlas.height as f32) as usize;
        let offset = (py * atlas.width as usize + px) * 4;
        assert_eq!(&atlas.data[offset..offset + 4], &[255, 0, 0, 255]);
    }

    #[test]
    fn when_building_atlas_with_two_images_then_neither_overwrites_the_other() {
        // Arrange
        let mut builder = AtlasBuilder::new(16, 16);
        let red = [255, 0, 0, 255].repeat(2 * 2);
        let blue = [0, 0, 255, 255].repeat(2 * 2);
        let h_red = builder.add_image(2, 2, &red).unwrap();
        let h_blue = builder.add_image(2, 2, &blue).unwrap();

        // Act
        let atlas = builder.build();

        // Assert — sample one pixel from each allocation
        let sample = |uv: [f32; 4]| -> &[u8] {
            let px = (uv[0] * atlas.width as f32) as usize;
            let py = (uv[1] * atlas.height as f32) as usize;
            let off = (py * atlas.width as usize + px) * 4;
            &atlas.data[off..off + 4]
        };
        assert_eq!(sample(h_red.uv_rect), &[255, 0, 0, 255]);
        assert_eq!(sample(h_blue.uv_rect), &[0, 0, 255, 255]);
    }

    fn make_1x1_png(r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let encoder = image::codecs::png::PngEncoder::new(&mut buf);
            use image::ImageEncoder;
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

    #[test]
    fn when_loading_image_and_adding_to_atlas_then_dimensions_preserved() {
        // Arrange
        let png_bytes = make_1x1_png(0, 255, 0, 255);
        let img = load_image_bytes(&png_bytes).unwrap();
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let handle = builder.add_image(img.width, img.height, &img.data).unwrap();

        // Assert — UV rect encodes a 1x1 pixel region
        let [u0, v0, u1, v1] = handle.uv_rect;
        let pixel_w = ((u1 - u0) * 256.0).round() as u32;
        let pixel_h = ((v1 - v0) * 256.0).round() as u32;
        assert_eq!(pixel_w, 1);
        assert_eq!(pixel_h, 1);
    }

    #[test]
    fn when_normalizing_uv_rect_then_output_is_in_zero_one_range() {
        // Act
        let uv = normalize_uv_rect(10, 20, 40, 60, 100, 100);

        // Assert
        assert_eq!(uv, [0.10, 0.20, 0.50, 0.80]);
    }

    #[test]
    fn when_normalizing_uv_rect_at_origin_then_starts_at_zero() {
        // Act
        let uv = normalize_uv_rect(0, 0, 32, 32, 256, 256);

        // Assert
        assert_eq!(uv, [0.0, 0.0, 0.125, 0.125]);
    }

    #[test]
    fn when_building_atlas_then_all_rows_of_image_are_correctly_placed() {
        // Arrange
        let mut builder = AtlasBuilder::new(4, 4);
        // Row 0: red, green; Row 1: blue, white
        #[rustfmt::skip]
        let data = [
            255, 0, 0, 255,    0, 255, 0, 255,
            0, 0, 255, 255,    255, 255, 255, 255,
        ];
        let handle = builder.add_image(2, 2, &data).unwrap();

        // Act
        let atlas = builder.build();

        // Assert
        let [u0, v0, _, _] = handle.uv_rect;
        let px = (u0 * atlas.width as f32) as usize;
        let py = (v0 * atlas.height as f32) as usize;
        let stride = atlas.width as usize * 4;
        assert_eq!(&atlas.data[py * stride + px * 4..][..4], [255, 0, 0, 255]);
        assert_eq!(
            &atlas.data[py * stride + (px + 1) * 4..][..4],
            [0, 255, 0, 255]
        );
        assert_eq!(
            &atlas.data[(py + 1) * stride + px * 4..][..4],
            [0, 0, 255, 255]
        );
        assert_eq!(
            &atlas.data[(py + 1) * stride + (px + 1) * 4..][..4],
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn when_second_image_offset_then_handle_uv_matches_build_lookup() {
        // Arrange — narrow atlas forces second image to y > 0
        let mut builder = AtlasBuilder::new(4, 8);
        builder.add_image(4, 4, &[0u8; 4 * 4 * 4]).unwrap();
        let data = [255u8; 2 * 2 * 4];
        let handle = builder.add_image(2, 2, &data).unwrap();

        // Act
        let atlas = builder.build();

        // Assert — handle UV (from add_image) must match lookup (from build)
        let looked_up = atlas.lookup(handle.texture_id).unwrap();
        assert_eq!(handle.uv_rect, looked_up);

        // Also verify pixel data at the UV location
        let [u0, v0, _, _] = handle.uv_rect;
        let px = (u0 * atlas.width as f32) as usize;
        let py = (v0 * atlas.height as f32) as usize;
        let stride = atlas.width as usize * 4;
        let off = py * stride + px * 4;
        assert_eq!(&atlas.data[off..off + 4], [255, 255, 255, 255]);
    }
}
