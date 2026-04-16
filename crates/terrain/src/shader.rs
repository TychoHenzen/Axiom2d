use engine_render::shader::{ShaderHandle, ShaderRegistry};

pub const TERRAIN_WGSL: &str = include_str!("shader/terrain.wgsl");

/// Register the terrain shader and return its handle.
pub fn register_terrain_shader(registry: &mut ShaderRegistry) -> ShaderHandle {
    registry.register(TERRAIN_WGSL)
}
