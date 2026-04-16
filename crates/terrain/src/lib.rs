pub mod material;

use bevy_ecs::prelude::Resource;
use material::TerrainMaterial;

/// ECS resource holding all terrain material definitions.
#[derive(Resource, Debug, Clone)]
pub struct TerrainMaterials(pub Vec<TerrainMaterial>);
