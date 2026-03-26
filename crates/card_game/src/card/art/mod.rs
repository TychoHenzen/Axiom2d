//! Card art — tessellation utilities and shape repository.

pub mod hydrate;
pub mod repository;

pub mod armor1;
pub mod barbarian_icons_01_t;

use engine_render::prelude::tessellate;
use engine_render::shape::{Shape, TessellatedColorMesh};

pub use repository::{ArtEntry, ShapeRepository, select_art_for_signature};

/// Tessellates a slice of `Shape`s into a single `TessellatedColorMesh`, skipping failures.
pub fn tessellate_art_shapes(shapes: &[Shape]) -> TessellatedColorMesh {
    let mut mesh = TessellatedColorMesh::new();
    for shape in shapes {
        if let Ok(tess) = tessellate(&shape.variant) {
            if tess.vertices.is_empty() {
                continue;
            }
            let color = [shape.color.r, shape.color.g, shape.color.b, shape.color.a];
            let flipped: Vec<[f32; 2]> = tess.vertices.iter().map(|&[x, y]| [x, -y]).collect();
            mesh.push_vertices(&flipped, &tess.indices, color);
        }
    }
    mesh
}

/// Returns the axis-aligned bounding box `(min, max)` of the mesh, or `None` if empty.
pub fn art_bounding_box(mesh: &TessellatedColorMesh) -> Option<([f32; 2], [f32; 2])> {
    let first = mesh.vertices.first()?;
    let mut min = first.position;
    let mut max = first.position;
    for v in &mesh.vertices[1..] {
        min[0] = min[0].min(v.position[0]);
        min[1] = min[1].min(v.position[1]);
        max[0] = max[0].max(v.position[0]);
        max[1] = max[1].max(v.position[1]);
    }
    Some((min, max))
}

/// Uniformly scales and centers a mesh to fit within `region_half_w × region_half_h`
/// centered at `(0, region_center_y)`. Returns an empty mesh if the input is empty.
pub fn fit_art_mesh_to_region(
    mesh: &TessellatedColorMesh,
    region_half_w: f32,
    region_half_h: f32,
    region_center_y: f32,
) -> TessellatedColorMesh {
    use engine_render::shape::ColorVertex;

    let Some((min, max)) = art_bounding_box(mesh) else {
        return TessellatedColorMesh::new();
    };

    let art_w = max[0] - min[0];
    let art_h = max[1] - min[1];
    if art_w == 0.0 || art_h == 0.0 {
        return TessellatedColorMesh::new();
    }

    let scale = (region_half_w * 2.0 / art_w).min(region_half_h * 2.0 / art_h);
    let centroid_x = (min[0] + max[0]) / 2.0;
    let centroid_y = (min[1] + max[1]) / 2.0;

    let vertices = mesh
        .vertices
        .iter()
        .map(|v| ColorVertex {
            position: [
                (v.position[0] - centroid_x) * scale,
                (v.position[1] - centroid_y) * scale + region_center_y,
            ],
            color: v.color,
        })
        .collect();

    TessellatedColorMesh {
        vertices,
        indices: mesh.indices.clone(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use engine_render::shape::TessellatedColorMesh;

    #[test]
    fn when_fitting_art_into_region_then_all_vertices_within_bounds() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        mesh.push_vertices(
            &[
                [-100.0, -100.0],
                [100.0, -100.0],
                [100.0, 100.0],
                [-100.0, 100.0],
            ],
            &[0, 1, 2, 0, 2, 3],
            [1.0, 0.0, 0.0, 1.0],
        );
        let half_w = 24.0_f32;
        let half_h = 16.65_f32;
        let center_y = -7.2_f32;

        // Act
        let result = fit_art_mesh_to_region(&mesh, half_w, half_h, center_y);

        // Assert
        for v in &result.vertices {
            let [x, y] = v.position;
            assert!(
                x >= -half_w && x <= half_w,
                "x={x} outside [-{half_w}, {half_w}]"
            );
            assert!(
                y >= center_y - half_h && y <= center_y + half_h,
                "y={y} outside [{}, {}]",
                center_y - half_h,
                center_y + half_h
            );
        }
    }

    #[test]
    fn when_fitting_art_with_nonsquare_input_then_aspect_ratio_preserved() {
        // Arrange — 2:1 aspect ratio input (400 wide × 200 tall)
        let mut mesh = TessellatedColorMesh::new();
        mesh.push_vertices(
            &[
                [-200.0, -100.0],
                [200.0, -100.0],
                [200.0, 100.0],
                [-200.0, 100.0],
            ],
            &[0, 1, 2, 0, 2, 3],
            [1.0, 1.0, 1.0, 1.0],
        );
        // Square target region
        let half = 20.0_f32;

        // Act
        let result = fit_art_mesh_to_region(&mesh, half, half, 0.0);

        // Assert — output bounding box should also have 2:1 ratio
        let (min, max) = art_bounding_box(&result).unwrap();
        let out_w = max[0] - min[0];
        let out_h = max[1] - min[1];
        let ratio = out_w / out_h;
        assert!(
            (ratio - 2.0).abs() < 0.01,
            "expected 2:1 aspect ratio, got {ratio:.4}"
        );
    }

    #[test]
    fn when_fitting_art_then_indices_preserved_unchanged() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        mesh.push_vertices(
            &[[-50.0, -50.0], [50.0, -50.0], [50.0, 50.0]],
            &[0, 1, 2],
            [1.0, 0.0, 0.0, 1.0],
        );
        mesh.push_vertices(
            &[[10.0, 10.0], [60.0, 10.0], [60.0, 60.0]],
            &[0, 1, 2],
            [0.0, 1.0, 0.0, 1.0],
        );
        let original_indices = mesh.indices.clone();

        // Act
        let result = fit_art_mesh_to_region(&mesh, 20.0, 20.0, 0.0);

        // Assert
        assert_eq!(result.indices, original_indices);
    }

    #[test]
    fn when_computing_bbox_from_empty_mesh_then_returns_none() {
        // Arrange
        let mesh = TessellatedColorMesh::new();

        // Act
        let result = art_bounding_box(&mesh);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_computing_bbox_with_multiple_batches_then_spans_all() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        mesh.push_vertices(
            &[[-50.0, 20.0], [-10.0, 20.0], [-10.0, 50.0]],
            &[0, 1, 2],
            [1.0, 0.0, 0.0, 1.0],
        );
        mesh.push_vertices(
            &[[0.0, 30.0], [100.0, 30.0], [100.0, 80.0]],
            &[0, 1, 2],
            [0.0, 1.0, 0.0, 1.0],
        );

        // Act
        let result = art_bounding_box(&mesh);

        // Assert
        let (min, max) = result.expect("non-empty mesh");
        assert_eq!(min, [-50.0, 20.0]);
        assert_eq!(max, [100.0, 80.0]);
    }
}
