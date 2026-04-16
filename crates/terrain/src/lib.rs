pub mod dual_grid;
pub mod material;
pub mod shader;

use bevy_ecs::prelude::Resource;
use engine_render::shader::ShaderHandle;
use material::TerrainMaterial;

/// ECS resource holding all terrain material definitions.
#[derive(Resource, Debug, Clone)]
pub struct TerrainMaterials(pub Vec<TerrainMaterial>);

/// ECS resource holding the terrain shader handle.
#[derive(Resource, Debug, Clone, Copy)]
pub struct TerrainShader(pub ShaderHandle);
