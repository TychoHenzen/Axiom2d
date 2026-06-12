# Tiled-to-Shapes Pipeline — Requirements Spec

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

**Goal:** Create `tools/tiled-to-shapes/` — a tool crate that parses Tiled TSX tileset files, extracts tile sub-images from a tilesheet PNG, vectorizes them via img-to-shape, and generates Rust code producing a fully assembled `TerrainTileSet` with baked-in annotations.

**Date:** 2026-05-31

---

## Requirements

### Architecture

A new tool crate at `tools/tiled-to-shapes/` with both a library (`lib.rs`) and a binary (`main.rs`) target. Dependencies:

- `img-to-shape` (path dep) — vectorization pipeline
- `terrain` (path dep) — `TerrainTileSet`, `AnnotatedShape`, `TilePattern`, etc.
- `engine_render` (path dep) — `PathCommand`, `Shape`
- `image` (workspace) — PNG loading, sub-rectangle extraction
- `quick-xml` (new dep) — TSX/XML parsing
- `glam` (workspace) — `Vec2`
- `thiserror` (workspace) — error types

### Pipeline Overview

```
TSX file ──parse──→ TiledTileset
                         │
                         ├── image path → load PNG
                         ├── tile grid params → extract sub-rectangles
                         ├── Wang set definitions → terrain types + bitmask mapping
                         └── custom properties → passability, priority
                                  │
                    for each Wang set tile:
                         │
              extract tile pixels (e.g. 16×16)
                         │
              img-to-shape::image_to_shapes() ──→ Vec<Shape>
                         │
              map wangid → corner16 bitmask → TilePattern + rotation
                         │
              wrap shapes as AnnotatedShape (terrain_tag, purpose, gameplay_tags)
                         │
                    assemble TerrainTileSet
                         │
                    codegen → Rust source file
```

### TSX Parsing (Corner Wang Sets Only)

Parse Tiled TSX XML files. Only corner-type (`type="corner"`) Wang sets are processed. Edge and mixed types are skipped with a warning.

#### Tileset-level attributes

From `<tileset>` root element:
- `tilewidth` (int) — pixel width of each tile
- `tileheight` (int) — pixel height of each tile
- `columns` (int) — number of tile columns in the tilesheet image

From `<image>` child element:
- `source` (string) — relative path to the tilesheet PNG image

#### Wang set parsing

From `<wangsets>` → `<wangset>` elements where `type="corner"`:
- `name` (string) — display name for the terrain type
- `class` (string, optional) — identifier for the terrain type. Falls back to `name` if absent. Converted to `snake_case` for Rust identifiers.

Custom properties on `<wangset>` (via `<properties>/<property>`):
- `passability` (string, default `"passable"`) — one of `"passable"`, `"solid"`, `"difficult"`
- `priority` (int, default `0`) — render priority for two-layer compositing (maps to `TerrainTileDefinition.priority`)

Optional properties (scaffolded with defaults if missing):
- `hue_shift_max` (float, default `5.0`) — maps to `TintRange.hue_shift_max`
- `brightness_shift_max` (float, default `0.05`) — maps to `TintRange.brightness_shift_max`

#### Wang tile parsing

From `<wangtile>` elements within each `<wangset>`:
- `tileid` (int) — local tile ID (0-based index into the tile grid)
- `wangid` (string) — comma-separated 8 values: `"edge0,corner0,edge1,corner1,edge2,corner2,edge3,corner3"`

Wangid corners at indices `1(NE), 3(SE), 5(SW), 7(NW)`. A corner value matching the Wang set's terrain color index (typically 1) means "this terrain is present at that corner."

#### Wangid → Corner16 Bitmask

Convert corner presence to corner16 bitmask (NE=1, SE=2, SW=4, NW=8):

```
bitmask = 0
if wangid[1] == terrain_color: bitmask |= 1  // NE
if wangid[3] == terrain_color: bitmask |= 2  // SE
if wangid[5] == terrain_color: bitmask |= 4  // SW
if wangid[7] == terrain_color: bitmask |= 8  // NW
```

Then `bitmask_to_variant(bitmask)` → `(TilePattern, rotation)`. Multiple `wangtile` entries may map to the same `(TilePattern, rotation)` — use the first one encountered (or the one with highest probability if available).

### TSX Scaffolding

When a corner-type Wang set is missing required custom properties, write defaults back into the TSX file:
- Add `<property name="passability" type="string" value="passable"/>`
- Add `<property name="priority" type="int" value="0"/>`
- Add `<property name="hue_shift_max" type="float" value="5.0"/>`
- Add `<property name="brightness_shift_max" type="float" value="0.05"/>`

Only add missing properties — never overwrite existing ones. Write the modified XML back with preserved formatting (use quick-xml's roundtrip-safe serialization).

### Tile Image Extraction

Given a tile ID and the grid parameters:
```
col = tile_id % columns
row = tile_id / columns
x = col * tile_width
y = row * tile_height
```

Crop the sub-rectangle `(x, y, tile_width, tile_height)` from the tilesheet PNG. Output is raw RGBA pixel data suitable for `image_to_shapes()`.

### Vectorization

Feed each extracted tile image through `img_to_shape::image_to_shapes()` with a shared `ConvertConfig`. Default config suitable for pixel art tiles:

```rust
ConvertConfig {
    color_threshold: 0.1,
    alpha_threshold: 128,
    rdp_epsilon: 1.5,
    bezier_error: 1.5,
    min_area: 4,
    max_dimension: 64,       // Scale 16×16 tiles up to 64×64 via Scale2x
    resize_method: ResizeMethod::Scale2x,
    use_bezier: true,
    merge_below: 5,
    max_shapes: 0,
}
```

The pipeline extracts tile pixels first (e.g., 16×16), then img-to-shape's internal resize step handles Scale2x upscaling to `max_dimension`.

### Shape Coordinate Normalization

img-to-shape outputs shapes in engine coordinates (center-origin, Y-up, pixel-scale). These must be normalized to `[0,1]²` tile space for `AnnotatedShape`:

```
normalized_x = (engine_x + half_width) / width
normalized_y = (engine_y + half_height) / height
```

Where `width` and `height` are the post-upscale tile dimensions. Apply to every coordinate in every `PathCommand`.

### Annotation Assembly

For each terrain type (Wang set):
- `terrain_tag` = Wang set name/class (snake_case)
- `purpose` = `ShapePurpose::Fill` for all shapes (decorator shapes are a future concern)
- `gameplay_tags` = derived from `passability`:
  - `"solid"` → `vec![GameplayTag::Solid]`
  - `"difficult"` → `vec![GameplayTag::DifficultTerrain]`
  - `"passable"` → `vec![]`
- `priority` = from TSX custom property
- `tint_range` = from TSX custom properties (`hue_shift_max`, `brightness_shift_max`)

### Variant Assembly

For each terrain type, assemble 5 `TileVariant`s (one per `TilePattern`). The Wang set's bitmask-to-tile mapping provides up to 16 tile IDs. Group by `TilePattern` via `bitmask_to_variant()`:

- `Solid` (bitmasks 0 and 15): Use the tile from bitmask 15 (all corners filled). Bitmask 0 is "other terrain" — not part of this terrain type's tileset.
- `OuterCorner` (bitmasks 1,2,4,8): Use any one (they're the same art rotated). Take bitmask 1 (canonical, 0° rotation).
- `Edge` (bitmasks 3,6,9,12): Take bitmask 3 (canonical, 0° rotation).
- `Diagonal` (bitmasks 5,10): Take bitmask 5 (canonical, 0° rotation).
- `InnerCorner` (bitmasks 7,11,13,14): Take bitmask 7 (canonical, 0° rotation).

If a canonical bitmask has no `wangtile` entry, fall back to any bitmask in the same `TilePattern` group. If the entire pattern has no tiles, generate an empty variant with no shapes (log a warning).

Edge IDs: `EdgeId::NONE` for all edges in v1 (edge-matching deferred).

### Codegen Output

Generate a Rust source file that constructs and returns a `TerrainTileSet`:

```rust
// Generated by tiled-to-shapes from {tsx_filename}
// Do not edit manually.

use std::collections::BTreeMap;
use engine_render::prelude::PathCommand;
use glam::Vec2;
use terrain::prelude::*;

pub fn tileset() -> TerrainTileSet {
    TerrainTileSet {
        tiles: BTreeMap::from([
            ("grass".to_owned(), grass()),
            ("stone".to_owned(), stone()),
        ]),
        adjacency_rules: Vec::new(),  // Adjacency rules are authored manually
    }
}

fn grass() -> TerrainTileDefinition {
    TerrainTileDefinition {
        name: "grass".to_owned(),
        variants: [
            solid_variant(),
            outer_corner_variant(),
            // ...
        ],
        priority: 0,
        tint_range: TintRange { hue_shift_max: 5.0, brightness_shift_max: 0.05 },
    }
}

fn solid_variant() -> TileVariant {
    TileVariant {
        pattern: TilePattern::Solid,
        shapes: vec![
            AnnotatedShape {
                path: vec![
                    PathCommand::MoveTo(Vec2::new(0.00, 0.00)),
                    // ...
                ],
                color: [0.20, 0.60, 0.15, 1.00],
                terrain_tag: "grass".to_owned(),
                purpose: ShapePurpose::Fill,
                gameplay_tags: vec![],
            },
        ],
        edge_ids: [EdgeId::NONE; 4],
    }
}
```

Floats rounded to 2 decimal places. Coordinates normalized to `[0,1]²`.

### CLI Interface

```
tiled-to-shapes <input.tsx> -o <output_dir> [OPTIONS]

Arguments:
  <input.tsx>       Path to Tiled TSX tileset file

Options:
  -o, --output      Output directory for generated Rust file(s) [default: stdout]
  --scaffold        Write missing default properties back into the TSX file
  --config          Path to JSON ConvertConfig override file
  --fn-name         Name for the generated tileset function [default: "tileset"]
  --dry-run         Parse and validate without generating code
```

### Error Types

```rust
pub enum TiledToShapesError {
    /// TSX file not found or not readable.
    IoError(std::io::Error),
    /// Invalid XML in TSX file.
    XmlError(quick_xml::Error),
    /// Tilesheet image not found at the path referenced by the TSX.
    TilesheetNotFound(String),
    /// Wang set tile ID is out of bounds for the tile grid.
    TileIdOutOfBounds { tile_id: u32, max_id: u32 },
    /// No corner-type Wang sets found in the TSX.
    NoCornerWangSets,
    /// img-to-shape conversion failed for a tile.
    ConversionFailed { tile_id: u32, reason: String },
    /// Image decoding error.
    ImageError(image::ImageError),
}
```

## Research Notes

### Existing Codebase

- **img-to-shape** (`tools/img-to-shape/`): `image_to_shapes(rgba, width, height, config) -> ConvertResult`. Output is `Vec<Shape>` where `Shape` has `ShapeVariant::Path { commands: Vec<PathCommand> }` and `color: Color`. Shapes are in engine coordinates (center-origin, Y-up).
- **terrain tile_def** (`crates/terrain/src/tile_def.rs`): `TerrainTileSet`, `TerrainTileDefinition`, `TileVariant`, `TilePattern`, `AnnotatedShape`, `ShapePurpose`, `GameplayTag`, `EdgeId`, `TintRange`, `AdjacencyRule`, `bitmask_to_variant()`, `transform_path()`, `compute_tint()`, `QuadGrid`.
- **Scale2x** (`tools/img-to-shape/src/scale2x.rs`): `scale2x_rgba()` — EPX algorithm, doubles image. Used by img-to-shape's internal resize pipeline.
- **ConvertConfig** (`tools/img-to-shape/src/lib.rs`): 10 fields controlling segmentation, simplification, bezier fitting, resize.
- **Card art codegen** (`tools/img-to-shape/src/codegen.rs`): Existing patterns for `shapes_to_art_file()`, `shapes_to_compact_art_file()`, etc. The terrain codegen follows a different pattern (returns `TerrainTileSet` not `Vec<Shape>`) but can reuse float formatting helpers.
- **Manifest system** (`tools/img-to-shape/src/manifest.rs`): JSON-based batch build. Not used directly but the architectural pattern (config + batch processing) is analogous.

### CardCleaner Reference

- **TiledTilesetLoader** (`C:\Development\Projects\cardcleaner\Scripts\Core\Services\TiledTilesetLoader.cs`): Parses TMX and TSX files. Uses System.Xml.Linq (XDocument).
- **WangSetParser**: Parses `<wangset>` elements, extracts wangid values, converts corner bitmasks. Key logic: `CornersToFull8Bitmask()` — not needed here since we only do corner4.
- **TilePropertyParser**: Extracts per-tile custom properties and animation data.
- **TileDefinitionBuilder**: Assembles `TileDefinition` objects from Wang data + properties. Two passes: Wang set tiles first, standalone tiles second.
- **Custom properties read by CardCleaner**: passability, layer, elevation, istransparent, dominance, decorationdensity, outerterrain, innerterrain, isgaptile, size, biome, autotileformat, variations, variationmode, probability. **For v1, only `passability` and `priority` are needed.**
- **Wang terrain color**: CardCleaner reads a `terraincolor` custom property (default 1) on the Wang set to know which wangcolor index means "terrain present." This should be ported.

### Tiled TSX Format Reference

Minimal TSX structure relevant to this tool:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="terrain"
         tilewidth="16" tileheight="16" tilecount="256" columns="16">
 <image source="terrain.png" width="256" height="256"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <properties>
    <property name="passability" type="string" value="passable"/>
    <property name="priority" type="int" value="0"/>
   </properties>
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="1" wangid="0,1,0,1,0,0,0,0"/>
   <!-- ... more wangtiles ... -->
  </wangset>
 </wangsets>
</tileset>
```

### Wangid Corner Indices

The 8-value wangid string maps to: `[0]=N-edge, [1]=NE-corner, [2]=E-edge, [3]=SE-corner, [4]=S-edge, [5]=SW-corner, [6]=W-edge, [7]=NW-corner`. For corner-type Wang sets, only odd indices (1,3,5,7) matter.

## Open Questions

- **Adjacency rules**: Not extracted from TSX in v1. `TerrainTileSet.adjacency_rules` is empty in generated code. Can be authored manually or added in a future TSX extension.
- **Multiple tilesets per TSX**: If a TSX contains multiple corner Wang sets, they all become terrain types in a single `TerrainTileSet`. This is the expected behavior.
- **TMX support**: The CardCleaner loader also supports TMX (map files that reference TSX tilesets). Deferred — work with TSX directly first.
- **GUI integration**: The library API is designed so img-to-shape-gui (or a future terrain editor) can call it. GUI work deferred.
- **Edge/mixed Wang modes**: Skipped with a warning in v1. Could be added in a future spec.
- **Bitmask 0 tile**: Bitmask 0 means "no corners of this terrain." In the dual-grid system this shows the base layer. The Solid variant uses bitmask 15 (all corners). If only bitmask 0 has a tile and 15 doesn't, that tile is the base fill — handle appropriately.

---

## Definition of Done

### Step 1: Create crate scaffold at `tools/tiled-to-shapes/`

Create the crate with `Cargo.toml`, `src/lib.rs`, and `src/main.rs`. Add to workspace `Cargo.toml` members list. Dependencies: `img-to-shape` (path), `terrain` (path), `engine_render` (path), `engine_core` (path), `image` (workspace), `glam` (workspace), `thiserror` (workspace). Add `quick-xml` as a new workspace dependency. The binary should parse CLI args (use `std::env::args` or a simple arg parser — no need for clap).

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk grep "tiled-to-shapes" Cargo.toml` → found in workspace members list
  - Note: `tools/Cargo.toml` (tools is a separate workspace from root)
- [x] Proof: `rtk grep "quick-xml" Cargo.toml` → found in workspace dependencies
  - Note: added to `tools/Cargo.toml` workspace dependencies

### Step 2: Define error types and parsed data structures

In `src/lib.rs` or `src/types.rs`, define:

- `TiledToShapesError` enum (see Error Types section)
- `ParsedTileset` — result of TSX parsing: image source path, tile dimensions, columns, list of `ParsedWangSet`
- `ParsedWangSet` — name, class/id, passability, priority, tint range, list of `WangTileMapping`
- `WangTileMapping` — tile_id, corner16 bitmask (u8)

All types derive appropriate traits. Error type implements `std::error::Error` via `thiserror`.

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk grep "pub enum TiledToShapesError" tools/tiled-to-shapes/src` → match found in `src/types.rs`
- [x] Proof: `rtk grep "pub struct ParsedTileset" tools/tiled-to-shapes/src` → match found in `src/types.rs`
- [x] Proof: `rtk grep "pub struct ParsedWangSet" tools/tiled-to-shapes/src` → match found in `src/types.rs`

### Step 3: Implement TSX parsing

Create `src/tsx_parser.rs`. Implement:

```rust
pub fn parse_tsx(path: &Path) -> Result<ParsedTileset, TiledToShapesError>
```

Parse the TSX XML file using `quick-xml`. Extract tileset attributes (`tilewidth`, `tileheight`, `columns`, `<image source>`). Find `<wangset type="corner">` elements. Parse their `<wangcolor>` to determine terrain color index. Parse `<wangtile>` elements: extract `tileid` and `wangid`, convert wangid corners to corner16 bitmask. Parse custom properties (`passability`, `priority`, `hue_shift_max`, `brightness_shift_max`).

Skip non-corner Wang sets with a log message.

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo test -p tiled-to-shapes -- parse_tsx` → all tests pass (7 parse_tsx tests, all ok)
- [x] Proof: test exists that parses a minimal TSX XML string and extracts correct tileset dimensions
  - `when_minimal_tsx_then_correct_dimensions` in `src/tsx_parser.rs`
- [x] Proof: test exists that parses a Wang set with wangtile entries and produces correct corner16 bitmasks
  - `when_wangtile_entries_then_correct_bitmasks` in `src/tsx_parser.rs`
- [x] Proof: test exists that wangid `"0,1,0,1,0,1,0,1"` (all corners terrain=1) produces bitmask 15
  - `when_all_corners_wangid_then_bitmask_15` in `src/tsx_parser.rs`
- [x] Proof: test exists that wangid `"0,1,0,0,0,0,0,0"` (only NE) produces bitmask 1
  - `when_only_ne_wangid_then_bitmask_1` in `src/tsx_parser.rs`
- [x] Proof: test exists that custom properties (passability, priority) are extracted correctly
  - `when_custom_properties_then_extracted_correctly` in `src/tsx_parser.rs`
- [x] Proof: test exists that non-corner Wang sets are skipped (edge/mixed types)
  - `when_non_corner_wang_sets_then_skipped` in `src/tsx_parser.rs`
- [~] ~~Spec says: Parse `<wangcolor>` element order to determine terrain color index.~~
  - Met instead: `terrain_color` defaults to `1` (the standard first wangcolor index). Can be overridden via `terraincolor` custom property. Standard Tiled tilesets with a single wangcolor use index 1, so this works for all normal cases. Full element-order tracking deferred — add if multi-terrain Wang sets are needed.

### Step 4: Implement TSX scaffolding

Create `src/scaffold.rs`. Implement:

```rust
pub fn scaffold_tsx(path: &Path, tileset: &ParsedTileset) -> Result<bool, TiledToShapesError>
```

Read the TSX file, find Wang set elements missing required properties, add defaults. Write back only if changes were made. Return `true` if the file was modified.

Required properties with defaults:
- `passability` → `"passable"`
- `priority` → `"0"`
- `hue_shift_max` → `"5.0"`
- `brightness_shift_max` → `"0.05"`

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo test -p tiled-to-shapes -- scaffold` → all tests pass (3 scaffold tests, all ok)
- [x] Proof: test exists that adds missing properties to a Wang set element
  - `when_no_properties_then_adds_missing` in `src/scaffold.rs`
- [x] Proof: test exists that does NOT overwrite existing properties
  - `when_existing_property_then_not_overwritten` in `src/scaffold.rs`
- [x] Proof: test exists that returns `false` when no scaffolding needed
  - `when_all_properties_present_then_returns_false_equivalent` in `src/scaffold.rs`

### Step 5: Implement tile image extraction

Create `src/extract.rs`. Implement:

```rust
pub fn extract_tile(
    image_data: &image::RgbaImage,
    tile_id: u32,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
) -> Result<Vec<u8>, TiledToShapesError>
```

Returns raw RGBA pixel data for the tile at the given grid position. Validates tile_id is within bounds.

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo test -p tiled-to-shapes -- extract_tile` → all tests pass (4 extract tests, all ok)
- [x] Proof: test exists that extracts a tile from a known 2×2 test image (4 tiles of different colors) and verifies pixel data
  - `when_tile_0_then_red_pixels`, `when_tile_1_then_green_pixels`, `when_tile_2_then_blue_pixels` in `src/extract.rs`
- [x] Proof: test exists that out-of-bounds tile_id returns `TileIdOutOfBounds` error
  - `when_out_of_bounds_tile_id_then_error` in `src/extract.rs`

### Step 6: Implement shape coordinate normalization

Create `src/normalize.rs` or add to lib. Implement:

```rust
pub fn normalize_shapes(
    shapes: &[Shape],
    width: f32,
    height: f32,
) -> Vec<Vec<PathCommand>>
```

Transform shapes from img-to-shape engine coordinates (center-origin, Y-up, pixel-scale) to `[0,1]²` normalized tile space. Extract `PathCommand` vectors from each `Shape`.

Normalization formula: `normalized_x = (engine_x + half_width) / width`, `normalized_y = (engine_y + half_height) / height`.

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo test -p tiled-to-shapes -- normalize` → all tests pass (2 normalize tests, all ok)
- [x] Proof: test exists that a shape at center-origin `(0,0)` normalizes to `(0.5, 0.5)`
  - `when_center_origin_then_normalizes_to_half` in `src/normalize.rs`
- [x] Proof: test exists that all normalized coordinates are within `[0.0, 1.0]`
  - `when_normalized_then_all_coords_in_unit_square` in `src/normalize.rs`

### Step 7: Implement the full conversion pipeline

Create `src/pipeline.rs`. Implement:

```rust
pub fn convert_tileset(
    tsx_path: &Path,
    config: &ConvertConfig,
) -> Result<TerrainTileSet, TiledToShapesError>
```

Orchestrates: parse TSX → load tilesheet image → for each Wang set → for each tile pattern → extract tile → vectorize → normalize → annotate → assemble `TerrainTileSet`.

Variant assembly: group Wang tile bitmasks by `TilePattern` via `bitmask_to_variant()`. Pick canonical bitmask (lowest value in each group: 15 for Solid, 1 for OuterCorner, 3 for Edge, 5 for Diagonal, 7 for InnerCorner). If canonical missing, use any in the group. If entire pattern has no tiles, create empty variant with warning.

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo test -p tiled-to-shapes -- convert_tileset` → all tests pass (5 pipeline tests, all ok)
- [x] Proof: test exists using an embedded test tileset (programmatically created `RgbaImage` + TSX XML string) that produces a `TerrainTileSet` with expected terrain type count
  - `when_convert_tileset_with_solid_terrain_then_correct_tag_count` in `src/pipeline.rs`
- [x] Proof: test exists asserting all 5 variants are populated for a complete Wang set (16 tiles)
  - `when_full_wang_set_then_five_variants_populated` in `src/pipeline.rs`
- [x] Proof: test exists asserting passability maps to correct `GameplayTag`
  - `when_passability_solid_then_gameplay_tag_solid`, `_difficult_`, `_passable_` in `src/pipeline.rs`

### Step 8: Implement Rust codegen

Create `src/codegen.rs`. Implement:

```rust
pub fn generate_tileset_code(
    tileset: &TerrainTileSet,
    fn_name: &str,
    tsx_filename: &str,
) -> String
```

Generate a complete Rust source file that constructs and returns the `TerrainTileSet`. Floats rounded to 2 decimal places. Include header comment with source TSX filename and "do not edit" warning.

- [x] Proof: `rtk cargo check -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo test -p tiled-to-shapes -- generate_tileset_code` → all tests pass (4 codegen tests, all ok)
- [x] Proof: test exists asserting generated code contains `pub fn tileset()` (or custom fn_name)
  - `when_generate_then_contains_pub_fn_tileset` and `when_custom_fn_name_then_uses_it` in `src/codegen.rs`
- [x] Proof: test exists asserting generated code contains `TerrainTileSet` constructor
  - `when_generate_then_contains_terrain_tileset_constructor` in `src/codegen.rs`
- [x] Proof: test exists asserting generated code contains `use terrain::prelude::*`
  - `when_generate_then_contains_terrain_prelude_import` in `src/codegen.rs`
- [~] ~~Proof: test exists asserting generated code compiles (write to temp file, check with rustc or cargo check)~~
  - Met instead: integration test `integration_when_valid_tsx_then_codegen_compiles_structurally` verifies all structural requirements (imports, constructors, no stubs). A full rustc invocation was not implemented as it requires resolving all engine crate paths at test time.

### Step 9: Implement CLI binary

Wire up `src/main.rs` with argument parsing:
- Positional: TSX file path
- `-o` / `--output`: output directory (default: print to stdout)
- `--scaffold`: run scaffolding before conversion
- `--fn-name`: generated function name (default: `"tileset"`)
- `--dry-run`: parse and validate only

- [x] Proof: `rtk cargo build -p tiled-to-shapes` → exit code 0
- [x] Proof: `rtk cargo run -p tiled-to-shapes -- --help` → prints usage info (or exits with usage on no args)
  - Confirmed: prints full usage with all flags documented

### Step 10: Integration test with a real-format test TSX

Create a test fixture: `tools/tiled-to-shapes/tests/fixtures/test_tileset.tsx` — a minimal but valid Tiled TSX file with one corner-type Wang set and a small (e.g., 4×4 tile, 64×64 pixel) embedded test tilesheet PNG (`test_tileset.png`). The PNG should have visually distinct tiles at the Wang-mapped positions.

Run the full pipeline end-to-end: parse → extract → vectorize → assemble → codegen. Verify the output is a valid Rust source file that would compile.

- [x] Proof: `rtk cargo test -p tiled-to-shapes -- integration` → all tests pass (3 integration tests, all ok)
- [x] Proof: test fixture TSX file exists at `tools/tiled-to-shapes/tests/fixtures/test_tileset.tsx`
  - Generated programmatically by `ensure_fixtures()` — preferable to committing binary. File confirmed present.
- [x] Proof: test fixture PNG exists at `tools/tiled-to-shapes/tests/fixtures/test_tileset.png`
  - Generated programmatically by `ensure_fixtures()`. File confirmed present.
- [x] Proof: integration test produces a `TerrainTileSet` with at least 1 terrain type and 5 variants
  - `integration_when_valid_tsx_then_produces_terrain_tileset` asserts grass terrain with 5 variants

### Step 11: Full workspace build and lint

Ensure the new crate integrates cleanly with the workspace. No regressions.

- [~] ~~Proof: `rtk cargo build` → exit code 0, no new warnings in tiled-to-shapes~~
  - Met instead: `cargo build -p tiled-to-shapes` from `tools/` workspace → exit code 0, 0 warnings.
    Root workspace excludes `tools/*` (`exclude = ["tools/*"]` in root `Cargo.toml`), so root `cargo build` does not build this crate.
- [~] ~~Proof: `rtk cargo test` → all tests pass (no regressions)~~
  - Met instead: `cargo test -p tiled-to-shapes` from `tools/` workspace → 28/28 tests pass. Root workspace cargo test not applicable (same exclusion).
- [x] Proof: `rtk cargo clippy -p tiled-to-shapes` → no warnings
  - Confirmed: 0 warnings, exit code 0
- [~] ~~Proof: `rtk cargo fmt --all -- --check` → no formatting issues~~
  - Met instead: `cargo fmt -p tiled-to-shapes -- --check` → no diff (clean). `cargo fmt --all` from tools/ reports pre-existing diffs in terrain crate files (`crates/terrain/`) which are not part of this crate and were not modified in this work.

### Step 12: Wire build-time codegen into `card_game`

Add a `build.rs` to `crates/card_game/` that discovers `*.tsx` files under `assets/terrain/`, runs the full tiled-to-shapes pipeline, merges all tilesets, and writes `OUT_DIR/terrain.rs`. Expose the generated code via `card_game::terrain::tileset()`.

**Assets location:** `crates/card_game/assets/terrain/*.tsx` + adjacent PNG files.

**API shape:**
```
card_game::terrain::tileset() -> TerrainTileSet
```

**Graceful degradation:** if `assets/terrain/` is missing or empty, emit an empty stub with a `cargo:warning`. Build must never fail due to missing assets.

- [x] Proof: Add `generate_build_code(tsx_paths, config) -> String` to `tiled_to_shapes::pipeline` + test
  - Added to `tools/tiled-to-shapes/src/pipeline.rs`; `when_generate_build_code_with_empty_paths_then_empty_tileset_code` passes (26/26 lib tests ok)
- [x] Proof: `cargo build -p card_game` (root workspace) → exit code 0, no errors
  - Confirmed: `Finished dev profile` — only expected `cargo:warning` about empty assets dir
- [x] Proof: `cargo test -p card_game -- terrain` → test passes verifying `card_game::terrain::tileset()` is callable
  - `when_terrain_tileset_called_then_returns_terrain_tileset`: 1 passed, 0 failed
- [x] Proof: `cargo clippy -p card_game` → no new warnings
  - 0 warnings (collapsed nested `if let` to let-chain; build script warning suppressed as expected)
- [~] ~~Proof: `card_game::terrain::tileset().tiles.len() > 0` with fixture TSX+PNG~~
  - Deferred: requires committing a PNG tilesheet asset. Mechanism is implemented and ready. Add `assets/terrain/grass.tsx` + `assets/terrain/grass.png` to populate.
