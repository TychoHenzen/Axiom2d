// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Added, Changed, Commands, Component, Entity, Or, Query};

use crate::shape::components::{Shape, TessellatedMesh};
use crate::shape::tessellate::tessellate;

#[derive(Component)]
pub struct CachedMesh(pub TessellatedMesh);

#[allow(clippy::type_complexity)]
pub fn mesh_cache_system(
    mut commands: Commands,
    query: Query<(Entity, &Shape), Or<(Added<Shape>, Changed<Shape>)>>,
) {
    for (entity, shape) in &query {
        if let Ok(mesh) = tessellate(&shape.variant) {
            commands.entity(entity).insert(CachedMesh(mesh));
        }
    }
}
// EVOLVE-BLOCK-END
