# Vector Terrain Tiles — Requirements Spec

> **For Claude (/goal):** Work through each incomplete step below.
> 1. Mark a step `[>]` when you begin working on it.
> 2. Verify each proof by running the stated command/process and confirming the expected outcome.
> 3. Mark each proof `[x]` only when the claim has been tested and matches the expected value.
> 4. A step may only be marked `[x]` once ALL its proofs are `[x]` or `[~]`.
> 5. If a proof cannot be met because requirements changed or the original condition is unreasonable:
>    - Mark it `[~]` with the original condition struck through.
>    - Add a bullet underneath: `  - Met instead: [what was actually achieved]`
>    - The step can still be `[x]` once all proofs are resolved (either `[x]` or `[~]`).
> 6. Continue until every step is `[x]` — then stop and report done.
>
> **Self-contained.** No external context needed. Run the commands listed in proofs directly.

**Goal:** Define and implement the data model for a vector-based terrain tile system that converts pixel tile art to annotated vector shapes via img-to-shape, rendered on a dual-grid with 5-variant auto-tiling and bilinear quad transforms.

**Date:** 2026-05-30

---

## Requirements

### Core Concept

Replace the current shader-based terrain rendering (procedural noise in `terrain.wgsl`) with a vector-shape-based approach:

1. Artist draws 5 tile variants per terrain type as pixel art (in tilesheets)
2. img-to-shape converts each tile image offline to annotated vector shapes (PathCommands)
3. Generated Rust source files compile into the binary (same pattern as `card_game/src/card/art/generated/`)
4. At runtime, the dual-grid auto-tiling system selects the correct variant + rotation/reflection for each visual quad based on corner16 bitmask
5. Vector shapes are lyon-tessellated and rendered through the existing unified render pipeline

### Data Model

#### Terrain type on vertices, visuals on quads (INVERTED from current DualGrid)

The current `crates/terrain/src/dual_grid.rs` stores `Vec<TerrainId>` per cell (data on faces, visuals at vertices). The new model **inverts this**: terrain types live on vertices, visual tiles are the quads (faces). This matches the CardCleaner architecture and enables irregular mesh support.

**Action:** Introduce a new `QuadGrid` type alongside (not replacing) the existing `DualGrid`. The existing `DualGrid`, `VisualTile`, WFC solver, and terrain viewer continue to work unchanged. The new `QuadGrid` is a separate data structure for the vector tile system. Migration/deprecation of the old system is deferred until the new system is proven.

#### TerrainTileSet

Top-level container for a complete terrain tileset:

```rust
pub struct TerrainTileSet {
    /// All terrain tile definitions in this set, keyed by terrain type name.
    pub tiles: BTreeMap<String, TerrainTileDefinition>,
    /// Global WFC adjacency constraints: which terrain types can be adjacent.
    pub adjacency_rules: Vec<AdjacencyRule>,
}

pub struct AdjacencyRule {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    /// Optional frequency weight (higher = more common adjacency).
    pub weight: f32,
}
```

#### TerrainTileDefinition

One terrain type (e.g., "grass", "stone"):

```rust
pub struct TerrainTileDefinition {
    pub name: String,
    /// The 5 unique tile variants for this terrain type.
    /// Indexed by `TilePattern`.
    pub variants: [TileVariant; 5],
    /// Render priority for two-layer compositing.
    /// Higher priority renders on top (foreground layer).
    pub priority: u8,
    /// Per-tile color tint range for seed-based variation.
    pub tint_range: TintRange,
}

pub struct TintRange {
    /// Hue shift range in degrees [-max, +max].
    pub hue_shift_max: f32,
    /// Brightness multiplier range [1.0 - max, 1.0 + max].
    pub brightness_shift_max: f32,
}
```

#### TileVariant and TilePattern

The 5 unique patterns and their variants:

```rust
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

pub struct TileVariant {
    pub pattern: TilePattern,
    /// Annotated vector shapes in normalized [0,1]² tile space.
    pub shapes: Vec<AnnotatedShape>,
    /// Edge identifiers for adjacency-aware variant selection.
    /// Order: [North, East, South, West].
    pub edge_ids: [EdgeId; 4],
}
```

#### AnnotatedShape

Each vector shape carries semantic metadata:

```rust
pub struct AnnotatedShape {
    /// Vector path commands in normalized [0,1]² tile space.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapePurpose {
    /// Main terrain fill area.
    Fill,
    /// Boundary/transition edge between terrain types.
    Boundary,
    /// Decorative detail (flowers, rocks, grass tufts).
    Decoration,
}

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
```

#### Edge Identity

For preventing mirror-image adjacency artifacts:

```rust
/// Identifier for a tile edge. Two adjacent quads should prefer
/// non-mirrored edge pairs when multiple rotations satisfy a bitmask.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EdgeId(pub u16);

impl EdgeId {
    pub const NONE: Self = Self(0);
}
```

**Selection rule:** When a bitmask maps to a variant that has multiple valid rotations (only diagonal has this — masks 5 and 10 are equivalent under 90° rotation), prefer the rotation whose edge IDs produce non-mirrored adjacency with already-placed neighbors. If no neighbors are placed yet, use the tile seed to pick deterministically.

### Bitmask → Variant + Transform Lookup Table

Complete mapping of all 16 corner16 bitmask values to variant + rotation. Bitmask uses NE=1, SE=2, SW=4, NW=8.

| Bitmask | Filled corners | Pattern | Rotation (CW°) | Mirror |
|---------|---------------|---------|-----------------|--------|
| 0 | none | Solid | 0 | false |
| 1 | NE | OuterCorner | 0 | false |
| 2 | SE | OuterCorner | 90 | false |
| 3 | NE+SE | Edge | 0 | false |
| 4 | SW | OuterCorner | 180 | false |
| 5 | NE+SW | Diagonal | 0 | false |
| 6 | SE+SW | Edge | 90 | false |
| 7 | NE+SE+SW | InnerCorner | 0 | false |
| 8 | NW | OuterCorner | 270 | false |
| 9 | NE+NW | Edge | 270 | false |
| 10 | SE+NW | Diagonal | 90 | false |
| 11 | NE+SE+NW | InnerCorner | 270 | false |
| 12 | SW+NW | Edge | 180 | false |
| 13 | NE+SW+NW | InnerCorner | 180 | false |
| 14 | SE+SW+NW | InnerCorner | 90 | false |
| 15 | all | Solid | 0 | false |

**Note:** Bitmask 0 = "all corners are OTHER terrain" (shows as base layer). Bitmask 15 = "all corners are THIS terrain" (shows as solid fill of this type). Both use the Solid variant but with different terrain sources.

The lookup must be a const array or static function returning `(TilePattern, rotation_degrees: u16)`.

### Two-Layer Compositing

Matches the CardCleaner architecture:
- **Base layer**: A uniform terrain fill per quad face (the "background tile"). Rendered first at lower z-order.
- **Foreground layer**: Auto-tiled terrain that composites over the base using the 5-variant system. Shapes have alpha transparency where the base should show through.

Each vertex stores both a base terrain type and an optional foreground terrain type. The foreground type drives the corner16 bitmask computation.

### Bilinear Quad Transform

Shapes are authored in normalized [0,1]² tile space. For rendering, each vertex coordinate `(u, v)` is transformed to world space using the quad's 4 corner positions:

```
P(u,v) = (1-u)(1-v)*SW + u*(1-v)*SE + (1-u)*v*NW + u*v*NE
```

Where SW, SE, NE, NW are the quad's world-space corner positions.

**Caveat for bezier curves:** Bilinear transform of a cubic bezier is NOT a cubic bezier. The transform is applied to all PathCommand coordinates (MoveTo, LineTo, CubicTo endpoints AND control points) as a vertex-level operation. This is mathematically approximate — for deformed quads, curves will distort. This is acceptable for the initial regular-grid implementation (where the transform is a simple scale+translate). For future irregular meshes, if distortion is visually unacceptable, subdivide curves into line segments before transforming.

### Seed-Based Tinting

Each tile instance gets a deterministic seed (from position hash). The seed produces:
- Hue shift within `TintRange.hue_shift_max` (e.g., ±5°)
- Brightness multiplier within `TintRange.brightness_shift_max` (e.g., ±0.05)

Applied as a vertex color modulation during tessellation (multiply shape color by tint). No shader needed.

### img-to-shape Pipeline Extension (Spec Only)

This spec defines the data model. The actual img-to-shape tilesheet extension is a separate implementation step. Key requirements for that extension:

- **Tilesheet sub-rectangle extraction**: Given a tilesheet image and a grid definition (tile width, tile height, columns, rows, padding), extract individual tiles.
- **Per-tile conversion parameters**: Each tile can have its own `ConvertConfig` (epsilon, bezier_error, etc.) since tilesheets have mixed resolutions.
- **Scale2x upsampling**: Apply for small tiles (16x16) before conversion.
- **Shape annotation**: Post-conversion, shapes can be annotated with purpose and gameplay tags. Initially done in generated code; eventually via the editor GUI.
- **Codegen output**: Generate Rust source files following card art conventions (`generated/*.rs` with `pub fn` returning shape data).

### WFC Integration

WFC operates on terrain types at the vertex level. The existing `crates/terrain/src/wfc.rs` solver works on a regular grid and is unchanged. For the new `QuadGrid`, WFC assigns terrain types to vertices, and the auto-tile system independently selects visual tiles for each quad based on its corner vertices.

## Research Notes

### Existing Codebase (Axiom2d)

- **Terrain crate** (`crates/terrain/`): Has `DualGrid`, `TerrainMaterial`, `TerrainKind`, WFC solver, `terrain.wgsl` shader. Current approach is shader-based (procedural noise per pixel on quads). The new vector tile system is a separate approach alongside this.
- **img-to-shape tool** (`tools/img-to-shape/`): Complete pipeline — segmentation, boundary graph, RDP, bezier fitting, codegen. Key algorithms to reuse: `boundary_graph::extract_region_faces`, `bezier_fit::fit_bezier_segment`, `simplify::rdp_open`/`rdp_simplify_closed`, `segment::segment_image`.
- **Terrain viewer** (`crates/terrain_viewer/`): Standalone binary demonstrating current shader terrain. Incompatible with vector tile approach — needs separate viewer or rewrite.
- **Render pipeline**: `ColorMesh`, `Material2d`, `ShaderHandle`, `RenderLayer`, `SortOrder`, unified z-sorted rendering. Vector shapes tessellated via lyon into `TessellatedColorMesh`.
- **Card art codegen**: `crates/card_game/src/card/art/generated/` — established pattern for offline-generated vector shape Rust code.

### Prior Art (CardCleaner/Godot)

- **IrregularMesh**: Hex grid → triangle merge → quad subdivision → Lloyd relaxation. All-quad mesh with variable vertex valence (3-5).
- **MeshVertex**: Stores `TerrainType`, `TileId`, `ForegroundTileId`, `VariantIndex`.
- **MeshQuad**: 4 corner vertices, `ComputeCorner16Bitmask()`, `GetSortedCorners()` (CCW with NW at index 3).
- **IrregularTerrainRenderer**: UV-mapped quads sampling texture atlas. The vector tile approach replaces atlas UV mapping with lyon-tessellated shapes.
- **DualGridAutoTile**: `ComputeBitmask(visualX, visualY, dataGrid)` — samples 4 data cells at visual tile corners. Corner16 format: NE=1, SE=2, SW=4, NW=8.
- **CompiledTransitionResolver**: Maps (foregroundTile, backgroundTile, bitmask) → atlas coordinates. Equivalent mapping needed for vector variants.
- **Two-layer system**: Background tile per quad face + foreground auto-tile per vertex.

### Dual-Grid 5-Tile Technique

References:
- https://excaliburjs.com/blog/Dual%20Tilemap%20Autotiling%20Technique/
- https://github.com/VinceSheeler/Godot-Dual-Grid-Tilemap-Proof-of-Concept

Key insight: visual grid offset by half a tile from data grid. Each visual tile straddles 4 data cells. With 2 terrain types, only 5 unique tile patterns exist (by rotational symmetry of 4-bit bitmask): solid, outer corner, edge, diagonal, inner corner. All 16 bitmask values map to one of these 5 + a rotation.

## Open Questions

- **Irregular mesh generation** (I34): Deferred. Will port CardCleaner's hex→merge→subdivide→relax pipeline separately. Data model designed to support it (quad corners can be at arbitrary positions).
- **Editor GUI**: Deferred until pipeline works. Current terrain_viewer is incompatible with vector tiles.
- **Animated terrain**: Deferred. Water flow, lava glow, grass wind — not in v1.
- **RON serialization**: Deferred. Code-constructed `TerrainTileSet` in v1. Add serde when editor exists.
- **Camera scrolling / quad lifecycle**: Not discussed. For regular grid, all tiles can be pre-spawned. For large/scrollable maps, need pool-based recycling — design when needed.
- **Bezier distortion on irregular quads**: Accepted as approximate for now. Strategy for high-fidelity irregular meshes TBD (subdivide curves → line segments before transform, or accept visual approximation).
- **Per-tile conversion parameters for mixed-resolution tilesheets**: Each tile extraction carries its own `ConvertConfig`. Resolution metadata preserved through codegen.
- **Multi-terrain compositing beyond 2 layers**: Current spec supports base + one foreground. Extending to N foreground layers (priority-stacked) is architecturally possible but not specified in v1.

---

## Definition of Done

### Step 1: Define core data types in `crates/terrain/src/tile_def.rs`

Create a new module `tile_def` in the terrain crate containing all types from the Data Model section above: `TerrainTileSet`, `TerrainTileDefinition`, `TileVariant`, `TilePattern`, `AnnotatedShape`, `ShapePurpose`, `GameplayTag`, `EdgeId`, `TintRange`, `AdjacencyRule`.

All types must derive appropriate traits (`Clone`, `Debug`, `PartialEq` where suitable). `TilePattern` and `ShapePurpose` and `GameplayTag` are enums. `EdgeId` is a newtype wrapper.

- [x] Proof: `rtk cargo check -p terrain` → exit code 0, no errors
- [x] Proof: `rtk grep "pub struct TerrainTileSet" crates/terrain/src/tile_def.rs` → match found
- [x] Proof: `rtk grep "pub enum TilePattern" crates/terrain/src/tile_def.rs` → match found with variants `Solid`, `OuterCorner`, `Edge`, `Diagonal`, `InnerCorner`
- [x] Proof: `rtk grep "pub enum ShapePurpose" crates/terrain/src/tile_def.rs` → match found with variants `Fill`, `Boundary`, `Decoration`
- [x] Proof: `rtk grep "pub enum GameplayTag" crates/terrain/src/tile_def.rs` → match found with variants `Solid`, `HurtsPlayer`, `DifficultTerrain`, `BlocksVision`
- [x] Proof: `rtk grep "pub struct EdgeId" crates/terrain/src/tile_def.rs` → match found

### Step 2: Implement bitmask → variant + rotation lookup

Create a const lookup function or table that maps all 16 corner16 bitmask values (0-15) to `(TilePattern, rotation_degrees: u16)`. Place in `tile_def.rs` or a dedicated `autotile.rs` module.

The function signature: `pub fn bitmask_to_variant(bitmask: u8) -> (TilePattern, u16)`

Must match the table in the Requirements section exactly.

- [x] Proof: `rtk cargo check -p terrain` → exit code 0
- [x] Proof: `rtk cargo test -p terrain -- bitmask_to_variant` → all tests pass, exit code 0 (6 passed)
- [x] Proof: test exists asserting all 16 bitmask values (0-15) produce valid `(TilePattern, rotation)` pairs — `bitmask_to_variant_all_16_produce_valid_pattern_and_rotation`
- [x] Proof: test exists asserting `bitmask_to_variant(0)` returns `(Solid, 0)` and `bitmask_to_variant(15)` returns `(Solid, 0)` — `bitmask_to_variant_0_and_15_return_solid_0`
- [x] Proof: test exists asserting outer corner rotations: `bitmask 1 → (OuterCorner, 0)`, `2 → (OuterCorner, 90)`, `4 → (OuterCorner, 180)`, `8 → (OuterCorner, 270)` — `bitmask_to_variant_outer_corner_rotations`
- [x] Proof: test exists asserting edge rotations: `3 → (Edge, 0)`, `6 → (Edge, 90)`, `12 → (Edge, 180)`, `9 → (Edge, 270)` — `bitmask_to_variant_edge_rotations`
- [x] Proof: test exists asserting diagonal: `5 → (Diagonal, 0)`, `10 → (Diagonal, 90)` — `bitmask_to_variant_diagonal_rotations`
- [x] Proof: test exists asserting inner corner rotations: `7 → (InnerCorner, 0)`, `14 → (InnerCorner, 90)`, `13 → (InnerCorner, 180)`, `11 → (InnerCorner, 270)` — `bitmask_to_variant_inner_corner_rotations`

### Step 3: Implement bilinear quad transform for PathCommands

Create a function that transforms `Vec<PathCommand>` from normalized [0,1]² tile space to world space given 4 quad corner positions.

Signature: `pub fn transform_path(commands: &[PathCommand], corners: [Vec2; 4]) -> Vec<PathCommand>`

Where corners are `[SW, SE, NE, NW]` in world space. The transform applies:
```
P(u,v) = (1-u)(1-v)*SW + u*(1-v)*SE + (1-u)*v*NW + u*v*NE
```

to every coordinate in every PathCommand (MoveTo position, LineTo position, CubicTo control1+control2+to).

- [x] Proof: `rtk cargo check -p terrain` → exit code 0
- [x] Proof: `rtk cargo test -p terrain -- transform_path` → all tests pass, exit code 0 (4 passed)
- [x] Proof: identity test exists — `transform_path_identity_corners_preserves_input`
- [x] Proof: scale test exists — `transform_path_scale_corners_scales_coordinates`
- [x] Proof: translation test exists — `transform_path_translation_corners_translates`
- [x] Proof: CubicTo test exists — `transform_path_cubic_to_transforms_all_three_coordinates`

### Step 4: Implement seed-based tint computation

Create a function that computes a color tint from a seed and `TintRange`:

Signature: `pub fn compute_tint(seed: u32, range: &TintRange) -> [f32; 4]`

Returns an RGBA multiplier (values near 1.0) that can be applied to vertex colors.

- [x] Proof: `rtk cargo check -p terrain` → exit code 0
- [x] Proof: `rtk cargo test -p terrain -- compute_tint` → all tests pass, exit code 0 (3 passed)
- [x] Proof: test exists asserting zero tint range produces `[1.0, 1.0, 1.0, 1.0]` — `compute_tint_zero_range_returns_identity`
- [x] Proof: test exists asserting different seeds produce different tints (for non-zero range) — `compute_tint_different_seeds_produce_different_tints`
- [x] Proof: test exists asserting same seed always produces same tint (deterministic) — `compute_tint_same_seed_is_deterministic`

### Step 5: Create QuadGrid data structure

New data structure where terrain types live on vertices and quads are the visual elements. For v1 this wraps a regular grid but the API is vertex/quad-based (not cell-based).

```rust
pub struct QuadGrid {
    /// Width and height in data vertices.
    width: usize,
    height: usize,
    /// Terrain type per vertex.
    vertex_terrain: Vec<TerrainId>,
}
```

API:
- `new(width, height, fill: TerrainId) -> Self`
- `get_vertex(&self, x, y) -> Option<TerrainId>`
- `set_vertex(&mut self, x, y, id: TerrainId)`
- `quad_count(&self) -> (usize, usize)` — returns `(width-1, height-1)`
- `quad_corners(&self, qx, qy) -> Option<[TerrainId; 4]>` — returns `[SW, SE, NE, NW]` for quad at `(qx, qy)` where `qx < width-1`, `qy < height-1`
- `quad_corner_positions(&self, qx, qy, tile_size: f32) -> Option<[Vec2; 4]>` — returns world-space positions for a regular grid

- [x] Proof: `rtk cargo check -p terrain` → exit code 0
- [x] Proof: `rtk cargo test -p terrain -- quad_grid` → all tests pass, exit code 0 (3 passed)
- [x] Proof: test exists for `quad_corners` returning correct 4 terrain IDs from vertex data — `quad_grid_quad_corners_returns_correct_terrain_ids`
- [x] Proof: test exists for `quad_count` returning `(width-1, height-1)` — `quad_grid_quad_count_returns_width_minus_1_height_minus_1`
- [x] Proof: test exists for `quad_corner_positions` returning correctly spaced positions — `quad_grid_corner_positions_correctly_spaced`

### Step 6: Create a hand-authored example TerrainTileSet

Create `crates/terrain/src/tile_def/example.rs` with a function `pub fn example_tileset() -> TerrainTileSet` that constructs a minimal tileset with 2 terrain types (e.g., "grass" and "stone"), each with 5 variants containing simple hand-authored shapes (rectangles and triangles in PathCommand form).

This serves as both documentation and a smoke test for the data model.

- [x] Proof: `rtk cargo check -p terrain` → exit code 0
- [x] Proof: `rtk cargo test -p terrain -- example_tileset` → all tests pass, exit code 0 (4 passed)
- [x] Proof: test exists asserting `example_tileset()` returns a set with exactly 2 terrain types — `example_tileset_has_two_terrain_types`
- [x] Proof: test exists asserting each terrain type has exactly 5 variants — `example_tileset_each_type_has_five_variants`
- [x] Proof: test exists asserting each variant has at least 1 shape with non-empty path commands — `example_tileset_each_variant_has_nonempty_shapes`
- [x] Proof: test exists asserting all shape coordinates are within [0.0, 1.0] range (normalized tile space) — `example_tileset_all_coordinates_in_unit_range`

### Step 7: Register new module in terrain crate and verify full build

Add `tile_def` module (and any sub-modules like `autotile`, `example`) to `crates/terrain/src/lib.rs`. Add to prelude. Ensure the full workspace builds cleanly.

- [x] Proof: `rtk cargo build` → exit code 0, no new warnings in terrain crate
- [x] Proof: `rtk cargo test` → all existing tests still pass (no regressions) — 1543 passed, 1 ignored
- [x] Proof: `rtk cargo clippy` → no new warnings in terrain crate
- [x] Proof: `rtk grep "pub mod tile_def" crates/terrain/src/lib.rs` → match found
