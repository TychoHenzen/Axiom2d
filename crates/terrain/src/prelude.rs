pub use crate::dual_grid::{DualGrid, VisualTile, corner_bitmask};
pub use crate::material::{
    MaterialParams, TerrainId, TerrainKind, TerrainMaterial, default_materials,
};
pub use crate::shader::{TERRAIN_WGSL, register_terrain_shader};
pub use crate::tile_def::{
    AdjacencyRule, AnnotatedShape, EdgeId, GameplayTag, QuadGrid, ShapePurpose,
    TerrainTileDefinition, TerrainTileSet, TilePattern, TileVariant, TintRange,
    bitmask_to_variant, compute_tint, transform_path,
};
pub use crate::wfc::{ConstraintTable, Grid as WfcGrid, WfcError, collapse};
pub use crate::{TerrainMaterials, TerrainShader};
