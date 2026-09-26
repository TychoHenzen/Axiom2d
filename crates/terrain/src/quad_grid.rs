use glam::Vec2;

use crate::material::TerrainId;

/// Grid where terrain types live on vertices and quads are visual elements.
///
/// For v1 this wraps a regular grid but the API is vertex/quad-based.
#[derive(Clone, Debug, PartialEq)]
pub struct QuadGrid {
    width: usize,
    height: usize,
    vertex_terrain: Vec<TerrainId>,
}

impl QuadGrid {
    /// Create a new grid with `width × height` vertices, all filled with `fill`.
    #[must_use]
    pub fn new(width: usize, height: usize, fill: TerrainId) -> Self {
        Self {
            width,
            height,
            vertex_terrain: vec![fill; width * height],
        }
    }

    /// Get terrain type at vertex `(x, y)`.
    #[must_use]
    pub fn get_vertex(&self, x: usize, y: usize) -> Option<TerrainId> {
        if x < self.width && y < self.height {
            Some(self.vertex_terrain[y * self.width + x])
        } else {
            None
        }
    }

    /// Set terrain type at vertex `(x, y)`.
    pub fn set_vertex(&mut self, x: usize, y: usize, id: TerrainId) {
        if x < self.width && y < self.height {
            self.vertex_terrain[y * self.width + x] = id;
        }
    }

    /// Returns `(quads_x, quads_y)` = `(width-1, height-1)`.
    #[must_use]
    pub fn quad_count(&self) -> (usize, usize) {
        (self.width.saturating_sub(1), self.height.saturating_sub(1))
    }

    /// Returns `[SW, SE, NE, NW]` terrain IDs for quad at `(qx, qy)`.
    #[must_use]
    pub fn quad_corners(&self, qx: usize, qy: usize) -> Option<[TerrainId; 4]> {
        if qx + 1 < self.width && qy + 1 < self.height {
            let sw = self.vertex_terrain[qy * self.width + qx];
            let se = self.vertex_terrain[qy * self.width + (qx + 1)];
            let ne = self.vertex_terrain[(qy + 1) * self.width + (qx + 1)];
            let nw = self.vertex_terrain[(qy + 1) * self.width + qx];
            Some([sw, se, ne, nw])
        } else {
            None
        }
    }

    /// Returns world-space positions `[SW, SE, NE, NW]` for quad at `(qx, qy)`
    /// on a regular grid with given `tile_size`.
    #[must_use]
    pub fn quad_corner_positions(&self, qx: usize, qy: usize, tile_size: f32) -> Option<[Vec2; 4]> {
        if qx + 1 < self.width && qy + 1 < self.height {
            let x0 = qx as f32 * tile_size;
            let y0 = qy as f32 * tile_size;
            let x1 = (qx + 1) as f32 * tile_size;
            let y1 = (qy + 1) as f32 * tile_size;
            Some([
                Vec2::new(x0, y0), // SW
                Vec2::new(x1, y0), // SE
                Vec2::new(x1, y1), // NE
                Vec2::new(x0, y1), // NW
            ])
        } else {
            None
        }
    }
}
