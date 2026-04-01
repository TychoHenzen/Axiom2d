use std::collections::HashMap;

use crate::atlas::TextureAtlas;

pub fn minimal_atlas() -> TextureAtlas {
    TextureAtlas {
        data: vec![255; 4],
        width: 1,
        height: 1,
        lookups: HashMap::default(),
    }
}
