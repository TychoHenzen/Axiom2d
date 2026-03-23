use bevy_ecs::prelude::Component;
use engine_render::material::Material2d;
use engine_render::shape::TessellatedColorMesh;
use glam::Vec2;

/// Pre-tessellated card geometry. Built once at spawn, never mutated.
#[derive(Component, Clone, Debug, Default)]
pub struct BakedCardMesh {
    pub front: TessellatedColorMesh,
    pub back: TessellatedColorMesh,
}

/// A shader-driven overlay quad (art area, foil effect, or back face).
#[derive(Clone, Debug)]
pub struct CardOverlay {
    pub quad: [Vec2; 4],
    pub material: Material2d,
}

/// Shader-driven overlay layers drawn on top of the baked mesh.
#[derive(Component, Clone, Debug, Default)]
pub struct CardOverlays {
    pub art: Option<CardOverlay>,
    pub foil: Option<CardOverlay>,
    pub back: Option<CardOverlay>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_baked_card_mesh_default_then_both_faces_empty() {
        // Act
        let baked = BakedCardMesh::default();

        // Assert
        assert!(baked.front.is_empty());
        assert!(baked.back.is_empty());
    }
}
