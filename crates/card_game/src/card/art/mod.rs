//! Card art modules and tessellation utilities.

pub mod hydrate;
pub mod repository;

pub mod armor1;
pub mod barbarian_icons_01_t;
pub mod barbarian_icons_02_t;
pub mod barbarian_icons_03_t;
pub mod barbarian_icons_04_t;
pub mod barbarian_icons_05_t;
pub mod barbarian_icons_06_t;

use engine_render::prelude::tessellate;
use engine_render::shape::{Shape, TessellatedColorMesh};

pub use repository::{ArtEntry, ShapeRepository};

/// Tessellate a slice of `Shape`s into a single `TessellatedColorMesh`.
///
/// Each shape is tessellated via lyon and its color applied to every vertex.
/// Shapes that fail tessellation or produce empty geometry are skipped.
pub fn tessellate_art_shapes(shapes: &[Shape]) -> TessellatedColorMesh {
    let mut mesh = TessellatedColorMesh::new();
    for shape in shapes {
        if let Ok(tess) = tessellate(&shape.variant) {
            if tess.vertices.is_empty() {
                continue;
            }
            let color = [shape.color.r, shape.color.g, shape.color.b, shape.color.a];
            let flipped: Vec<[f32; 2]> = tess.vertices.iter().map(|&[x, y]| [x, -y]).collect();
            let uvs = compute_aabb_uvs(&flipped);
            mesh.push_vertices_with_uv(&flipped, &uvs, &tess.indices, color);
        }
    }
    mesh
}

/// Compute per-vertex UV by normalizing positions to the shape's AABB.
#[must_use]
fn compute_aabb_uvs(positions: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for &[x, y] in positions {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let range_x = (max_x - min_x).max(f32::EPSILON);
    let range_y = (max_y - min_y).max(f32::EPSILON);
    positions
        .iter()
        .map(|&[x, y]| [(x - min_x) / range_x, (y - min_y) / range_y])
        .collect()
}
