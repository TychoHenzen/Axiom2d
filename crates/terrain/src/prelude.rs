pub use crate::dual_grid::{DualGrid, VisualTile, corner_bitmask};
pub use crate::material::{
    MaterialParams, TerrainId, TerrainKind, TerrainMaterial, default_materials,
};
pub use crate::shader::{TERRAIN_WGSL, register_terrain_shader};
pub use crate::wfc::{ConstraintTable, Grid as WfcGrid, WfcError, collapse};
pub use crate::{TerrainMaterials, TerrainShader};
