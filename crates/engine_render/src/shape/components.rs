use std::ops::{Deref, DerefMut};

use bevy_ecs::prelude::Component;
use engine_core::color::Color;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::path::PathCommand;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeVariant {
    Circle { radius: f32 },
    Polygon { points: Vec<Vec2> },
    Path { commands: Vec<PathCommand> },
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub variant: ShapeVariant,
    pub color: Color,
}

pub struct TessellatedMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Vertex with baked position, RGBA color, and per-shape UV coordinates.
/// Layout matches `ShapeVertex` in the wgpu renderer (32 bytes).
/// UV encodes normalized position within the shape's bounding box \[0,1\],
/// giving shaders geometric hints about shape structure (edges, gradients).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

/// Pre-tessellated mesh with per-vertex color.
/// Used by `BakedCardMesh` to store card geometry that never changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TessellatedColorMesh {
    pub vertices: Vec<ColorVertex>,
    pub indices: Vec<u32>,
}

impl TessellatedColorMesh {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append position-only vertices with a uniform color, offsetting indices.
    pub fn push_vertices(&mut self, positions: &[[f32; 2]], indices: &[u32], color: [f32; 4]) {
        let base = self.vertices.len() as u32;
        self.vertices
            .extend(positions.iter().map(|&position| ColorVertex {
                position,
                color,
                uv: [0.0, 0.0],
            }));
        self.indices.extend(indices.iter().map(|&i| i + base));
    }

    /// Append position-only vertices with a uniform color and per-vertex UV, offsetting indices.
    pub fn push_vertices_with_uv(
        &mut self,
        positions: &[[f32; 2]],
        uvs: &[[f32; 2]],
        indices: &[u32],
        color: [f32; 4],
    ) {
        let base = self.vertices.len() as u32;
        self.vertices
            .extend(
                positions
                    .iter()
                    .zip(uvs.iter())
                    .map(|(&position, &uv)| ColorVertex {
                        position,
                        color,
                        uv,
                    }),
            );
        self.indices.extend(indices.iter().map(|&i| i + base));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// A shape overlay drawn on top of the entity's primary mesh.
/// Used for shader-driven effects (art areas, foil, etc.) that can't be baked.
#[derive(Clone, Debug)]
pub struct OverlayEntry {
    pub mesh: TessellatedColorMesh,
    pub material: crate::material::Material2d,
    pub visible: bool,
    /// When `true`, this overlay is only shown when the card is face-up.
    /// When `false`, the overlay is shown on both faces (e.g. tier condition shaders).
    pub front_only: bool,
}

/// Overlay quads drawn immediately after the entity's `ColorMesh`.
/// Each entry gets its own shader/material application and draw call,
/// but shares the entity's model transform and sort order.
#[derive(Component, Clone, Debug, Default)]
pub struct MeshOverlays(pub Vec<OverlayEntry>);

/// ECS component wrapping a pre-tessellated colored mesh for direct rendering.
/// The unified render system draws this via `draw_colored_mesh`, bypassing
/// per-frame tessellation. Game code sets this component to control what is drawn.
#[derive(Component, Clone, Debug, Default)]
pub struct ColorMesh(pub TessellatedColorMesh);

/// ECS component wrapping a persistent GPU mesh handle for direct rendering.
/// The unified render system draws this via `draw_persistent_colored_mesh`,
/// using a pre-uploaded GPU buffer instead of re-uploading vertices each frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistentColorMesh(pub crate::renderer::GpuMeshHandle);

impl Deref for ColorMesh {
    type Target = TessellatedColorMesh;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ColorMesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
