use std::collections::HashMap;

use bevy_ecs::prelude::{Commands, Res, ResMut, Resource};
use engine_core::types::TextureId;

pub use crate::image_data::{AtlasError, ImageData, load_image_bytes};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureHandle {
    pub texture_id: TextureId,
    pub uv_rect: [f32; 4],
}

pub fn normalize_uv_rect(x: u32, y: u32, w: u32, h: u32, atlas: (u32, u32)) -> [f32; 4] {
    let aw = atlas.0 as f32;
    let ah = atlas.1 as f32;
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

#[derive(Resource)]
pub struct AtlasUploaded;

#[allow(clippy::needless_pass_by_value)]
pub fn upload_atlas_system(
    atlas: Option<Res<TextureAtlas>>,
    uploaded: Option<Res<AtlasUploaded>>,
    mut renderer: ResMut<crate::renderer::RendererRes>,
    mut commands: Commands,
) {
    if uploaded.is_some() {
        return;
    }
    let Some(atlas) = atlas else { return };
    renderer
        .upload_atlas(&atlas)
        .expect("atlas upload should succeed");
    commands.insert_resource(AtlasUploaded);
}

fn validate_image_data(width: u32, height: u32, data: &[u8]) -> Result<(), AtlasError> {
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
    Ok(())
}

struct PendingImage {
    data: Vec<u8>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    texture_id: TextureId,
    uv_rect: [f32; 4],
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
        validate_image_data(width, height, data)?;
        let alloc = self
            .allocator
            .allocate(guillotiere::size2(width as i32, height as i32))
            .ok_or(AtlasError::NoSpace)?;
        let rect = alloc.rectangle;
        let x = rect.min.x as u32;
        let y = rect.min.y as u32;
        let w = (rect.max.x - rect.min.x) as u32;
        let h = (rect.max.y - rect.min.y) as u32;
        let uv = normalize_uv_rect(x, y, w, h, (self.width(), self.height()));
        Ok(self.push_pending(data, [x, y, w, h], uv))
    }

    fn push_pending(&mut self, data: &[u8], pos: [u32; 4], uv_rect: [f32; 4]) -> TextureHandle {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.push(PendingImage {
            data: data.to_vec(),
            x: pos[0],
            y: pos[1],
            width: pos[2],
            height: pos[3],
            texture_id: TextureId(id),
            uv_rect,
        });
        TextureHandle {
            texture_id: TextureId(id),
            uv_rect,
        }
    }

    pub fn build(self) -> TextureAtlas {
        let width = self.width();
        let height = self.height();
        let mut data = vec![0u8; (width * height * 4) as usize];
        let mut lookups = HashMap::new();
        let stride = (width * 4) as usize;
        for img in &self.pending {
            lookups.insert(img.texture_id, img.uv_rect);
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
