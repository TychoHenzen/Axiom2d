use std::collections::BTreeMap;

use engine_render::prelude::PathCommand;
use glam::Vec2;

use crate::material::TerrainId;

// ---------------------------------------------------------------------------
// Tile pattern enum — the 5 unique dual-grid tile patterns
// ---------------------------------------------------------------------------

/// The 5 unique dual-grid tile patterns (by filled corner count + topology).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TilePattern {
    /// All 4 corners same type (bitmask 0 or 15). Solid fill.
    Solid,
    /// 1 corner filled (bitmask 1,2,4,8). Outer corner.
    OuterCorner,
    /// 2 adjacent corners filled (bitmask 3,6,9,12). Edge/side.
    Edge,
    /// 2 diagonal corners filled (bitmask 5,10). Diagonal.
    Diagonal,
    /// 3 corners filled (bitmask 7,11,13,14). Inner corner.
    InnerCorner,
}

// ---------------------------------------------------------------------------
// Shape metadata
// ---------------------------------------------------------------------------

/// What role a shape plays in a tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapePurpose {
    /// Main terrain fill area.
    Fill,
    /// Boundary/transition edge between terrain types.
    Boundary,
    /// Decorative detail (flowers, rocks, grass tufts).
    Decoration,
}

/// Gameplay-relevant tags attached to shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameplayTag {
    /// Blocks movement (walls, deep water).
    Solid,
    /// Damages the player on contact.
    HurtsPlayer,
    /// Slows movement.
    DifficultTerrain,
    /// Blocks line of sight.
    BlocksVision,
}

/// Identifier for a tile edge. Two adjacent quads should prefer
/// non-mirrored edge pairs when multiple rotations satisfy a bitmask.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EdgeId(pub u16);

impl EdgeId {
    pub const NONE: Self = Self(0);
}

// ---------------------------------------------------------------------------
// Shape + variant
// ---------------------------------------------------------------------------

/// A vector shape with semantic metadata, in normalized \[0,1\]² tile space.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotatedShape {
    /// Vector path commands in normalized \[0,1\]² tile space.
    pub path: Vec<PathCommand>,
    /// Fill color (RGBA).
    pub color: [f32; 4],
    /// Which terrain type this shape belongs to.
    pub terrain_tag: String,
    /// What role this shape plays in the tile.
    pub purpose: ShapePurpose,
    /// Gameplay-relevant tags.
    pub gameplay_tags: Vec<GameplayTag>,
}

/// One of the 5 tile variants for a terrain type.
#[derive(Clone, Debug, PartialEq)]
pub struct TileVariant {
    pub pattern: TilePattern,
    /// Annotated vector shapes in normalized \[0,1\]² tile space.
    pub shapes: Vec<AnnotatedShape>,
    /// Edge identifiers for adjacency-aware variant selection.
    /// Order: \[North, East, South, West\].
    pub edge_ids: [EdgeId; 4],
}

// ---------------------------------------------------------------------------
// Tint range
// ---------------------------------------------------------------------------

/// Per-tile color tint range for seed-based variation.
#[derive(Clone, Debug, PartialEq)]
pub struct TintRange {
    /// Hue shift range in degrees \[-max, +max\].
    pub hue_shift_max: f32,
    /// Brightness multiplier range \[1.0 - max, 1.0 + max\].
    pub brightness_shift_max: f32,
}

// ---------------------------------------------------------------------------
// Terrain tile definition + tileset
// ---------------------------------------------------------------------------

/// One terrain type (e.g., "grass", "stone").
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainTileDefinition {
    pub name: String,
    /// The 5 unique tile variants for this terrain type.
    /// Indexed by `TilePattern` order: `Solid`, `OuterCorner`, `Edge`, `Diagonal`, `InnerCorner`.
    pub variants: [TileVariant; 5],
    /// Render priority for two-layer compositing.
    /// Higher priority renders on top (foreground layer).
    pub priority: u8,
    /// Per-tile color tint range for seed-based variation.
    pub tint_range: TintRange,
}

/// Top-level container for a complete terrain tileset.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainTileSet {
    /// All terrain tile definitions in this set, keyed by terrain type name.
    pub tiles: BTreeMap<String, TerrainTileDefinition>,
    /// Global WFC adjacency constraints: which terrain types can be adjacent.
    pub adjacency_rules: Vec<AdjacencyRule>,
}

/// WFC adjacency constraint between two terrain types.
#[derive(Clone, Debug, PartialEq)]
pub struct AdjacencyRule {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    /// Optional frequency weight (higher = more common adjacency).
    pub weight: f32,
}

// ---------------------------------------------------------------------------
// Bitmask → variant + rotation lookup (Step 2)
// ---------------------------------------------------------------------------

/// Maps a corner16 bitmask (0-15) to a `(TilePattern, rotation_degrees)`.
///
/// Bitmask encoding: NE=1, SE=2, SW=4, NW=8.
#[must_use]
pub const fn bitmask_to_variant(bitmask: u8) -> (TilePattern, u16) {
    match bitmask {
        1 => (TilePattern::OuterCorner, 0),
        2 => (TilePattern::OuterCorner, 90),
        3 => (TilePattern::Edge, 0),
        4 => (TilePattern::OuterCorner, 180),
        5 => (TilePattern::Diagonal, 0),
        6 => (TilePattern::Edge, 90),
        7 => (TilePattern::InnerCorner, 0),
        8 => (TilePattern::OuterCorner, 270),
        9 => (TilePattern::Edge, 270),
        10 => (TilePattern::Diagonal, 90),
        11 => (TilePattern::InnerCorner, 270),
        12 => (TilePattern::Edge, 180),
        13 => (TilePattern::InnerCorner, 180),
        14 => (TilePattern::InnerCorner, 90),
        // 0 (all other), 15 (all same), and out-of-range all map to solid.
        _ => (TilePattern::Solid, 0),
    }
}

// ---------------------------------------------------------------------------
// Bilinear quad transform (Step 3)
// ---------------------------------------------------------------------------

/// Bilinear interpolation: maps (u,v) in \[0,1\]² to world space
/// given quad corners \[SW, SE, NE, NW\].
fn bilinear(u: f32, v: f32, corners: [Vec2; 4]) -> Vec2 {
    let [sw, se, ne, nw] = corners;
    (1.0 - u) * (1.0 - v) * sw + u * (1.0 - v) * se + (1.0 - u) * v * nw + u * v * ne
}

/// Transforms path commands from normalized \[0,1\]² tile space to world space
/// using bilinear interpolation over quad corners \[SW, SE, NE, NW\].
#[must_use]
pub fn transform_path(commands: &[PathCommand], corners: [Vec2; 4]) -> Vec<PathCommand> {
    commands
        .iter()
        .map(|cmd| match *cmd {
            PathCommand::MoveTo(p) => PathCommand::MoveTo(bilinear(p.x, p.y, corners)),
            PathCommand::LineTo(p) => PathCommand::LineTo(bilinear(p.x, p.y, corners)),
            PathCommand::QuadraticTo { control, to } => PathCommand::QuadraticTo {
                control: bilinear(control.x, control.y, corners),
                to: bilinear(to.x, to.y, corners),
            },
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => PathCommand::CubicTo {
                control1: bilinear(control1.x, control1.y, corners),
                control2: bilinear(control2.x, control2.y, corners),
                to: bilinear(to.x, to.y, corners),
            },
            PathCommand::Close => PathCommand::Close,
            PathCommand::Reverse => PathCommand::Reverse,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Seed-based tint (Step 4)
// ---------------------------------------------------------------------------

/// Computes a color tint multiplier from a deterministic seed and tint range.
///
/// Returns an RGBA multiplier (values near 1.0) suitable for vertex color
/// modulation during tessellation.
#[must_use]
pub fn compute_tint(seed: u32, range: &TintRange) -> [f32; 4] {
    if range.hue_shift_max == 0.0 && range.brightness_shift_max == 0.0 {
        return [1.0, 1.0, 1.0, 1.0];
    }

    // Simple deterministic hash to produce two independent values in [-1, 1].
    let h1 = seed.wrapping_mul(2_654_435_761); // Knuth multiplicative hash
    let h2 = seed.wrapping_mul(2_246_822_519);
    let norm1 = (h1 as f32) / (u32::MAX as f32) * 2.0 - 1.0; // [-1, 1]
    let norm2 = (h2 as f32) / (u32::MAX as f32) * 2.0 - 1.0;

    let hue_shift_deg = norm1 * range.hue_shift_max;
    let brightness = 1.0 + norm2 * range.brightness_shift_max;

    // Approximate hue rotation as RGB channel multipliers.
    // For small hue shifts (±5-10°), a linear approximation is adequate.
    let hue_rad = hue_shift_deg * std::f32::consts::PI / 180.0;
    let cos_h = hue_rad.cos();
    let sin_h = hue_rad.sin();

    // Hue rotation matrix applied to (1,1,1) to get per-channel multipliers.
    // Using the luminance-preserving hue rotation:
    //   R' = 0.213 + 0.787*cos - 0.213*sin  (applied to white → stays near 1.0)
    let r = (0.213 + 0.787 * cos_h + 0.213 * sin_h) * brightness;
    let g = (0.715 - 0.715 * cos_h + 0.143 * sin_h) * brightness;
    let b = (0.072 - 0.072 * cos_h - 0.283 * sin_h + sin_h) * brightness;

    [r, g, b, 1.0]
}

// ---------------------------------------------------------------------------
// QuadGrid (Step 5)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Example tileset sub-module (Step 6)
// ---------------------------------------------------------------------------

pub mod example;
