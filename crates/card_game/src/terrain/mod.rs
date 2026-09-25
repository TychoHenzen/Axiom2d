mod generated {
    include!(concat!(env!("OUT_DIR"), "/terrain.rs"));
}

pub use ::terrain::prelude::{DualGrid, TerrainId, TerrainMaterial, VisualTile, default_materials};
pub use generated::tileset;
