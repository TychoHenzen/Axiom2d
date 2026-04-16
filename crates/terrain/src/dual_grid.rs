use crate::material::TerrainId;

/// A data grid where each cell holds a `TerrainId`.
#[derive(Clone, Debug)]
pub struct DualGrid {
    width: usize,
    height: usize,
    cells: Vec<TerrainId>,
}

/// A visual tile in the dual-grid, straddling 4 data cells.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualTile {
    /// Grid-space position of the visual tile center.
    pub x: f32,
    pub y: f32,
    /// The 4 data-cell terrain IDs: [NE, SE, SW, NW].
    pub corners: [TerrainId; 4],
    /// Per-tile seed derived from position.
    pub seed: u32,
}

impl DualGrid {
    /// Create a grid filled with a uniform terrain type.
    #[must_use]
    pub fn new(width: usize, height: usize, fill: TerrainId) -> Self {
        Self {
            width,
            height,
            cells: vec![fill; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the terrain type at `(x, y)`. Returns `None` for out-of-bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainId> {
        if x < self.width && y < self.height {
            Some(self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Set the terrain type at `(x, y)`. Panics if out of bounds.
    pub fn set(&mut self, x: usize, y: usize, id: TerrainId) {
        self.cells[y * self.width + x] = id;
    }

    /// Get terrain type, clamping out-of-bounds coordinates to the nearest edge cell.
    #[must_use]
    fn get_clamped(&self, x: i32, y: i32) -> TerrainId {
        let cx = x.clamp(0, self.width as i32 - 1) as usize;
        let cy = y.clamp(0, self.height as i32 - 1) as usize;
        self.cells[cy * self.width + cx]
    }

    /// Generate all visual tiles for the dual-grid.
    /// The visual grid is offset by (-0.5, -0.5) and has dimensions (width+1, height+1).
    #[must_use]
    pub fn visual_tiles(&self) -> Vec<VisualTile> {
        let vw = self.width + 1;
        let vh = self.height + 1;
        let mut tiles = Vec::with_capacity(vw * vh);

        for vy in 0..vh {
            for vx in 0..vw {
                // Data cells that this visual tile straddles:
                // NE = (vx, vy-1), SE = (vx, vy), SW = (vx-1, vy), NW = (vx-1, vy-1)
                let dx = vx as i32;
                let dy = vy as i32;
                let ne = self.get_clamped(dx, dy - 1);
                let se = self.get_clamped(dx, dy);
                let sw = self.get_clamped(dx - 1, dy);
                let nw = self.get_clamped(dx - 1, dy - 1);

                let seed = simple_hash(vx as u32, vy as u32);

                tiles.push(VisualTile {
                    x: vx as f32 - 0.5,
                    y: vy as f32 - 0.5,
                    corners: [ne, se, sw, nw],
                    seed,
                });
            }
        }

        tiles
    }
}

/// Compute the corner16 bitmask: which corners differ from `primary`.
/// NE=bit0, SE=bit1, SW=bit2, NW=bit3.
#[must_use]
pub fn corner_bitmask(corners: [TerrainId; 4], primary: TerrainId) -> u8 {
    let mut mask = 0u8;
    if corners[0] != primary {
        mask |= 1;
    }
    if corners[1] != primary {
        mask |= 2;
    }
    if corners[2] != primary {
        mask |= 4;
    }
    if corners[3] != primary {
        mask |= 8;
    }
    mask
}

fn simple_hash(x: u32, y: u32) -> u32 {
    let mut h = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}
