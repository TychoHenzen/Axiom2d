# Terrain Viewer Design

A standalone terrain viewer built on the real engine pipeline, for iterating on procedural terrain materials, auto-tile transitions, and WFC grid generation before integrating terrain into the card game.

## Motivation

The debate document (`docs/debate-terrain-system.md`) converged on: dual-grid topology, procedural fragment shader on quads, standalone WFC solver, `TerrainMaterial` as data, zero engine modifications required. The consensus recommended deferring the editor.

This spec flips that priority. The viewer comes first because:

- Shader-based procedural terrain materials need visual iteration to get right. You can't evaluate "does this read as grass?" from code alone.
- Transition effects (auto-tile boundaries with per-pair character) are the hardest visual problem and need experimentation before committing to the in-game implementation.
- The viewer exercises the real `Material2d` + `ShaderRegistry` pipeline, so what you see IS what the game will produce.

Reference: CardCleaner (`C:\Development\Projects\cardcleaner`) has a mature terrain system with 10 biomes, corner16/edge16/blob47 auto-tiling, WFC with soft constraints, and both regular and irregular mesh topologies. This design draws on CardCleaner's auto-tiling and WFC patterns while replacing its texture-atlas rendering with procedural shaders.

## Architecture

### Crate Structure

Two new crates in the workspace:

```
crates/
  terrain/                  # NEW library crate
    src/
      lib.rs
      material.rs           # TerrainMaterial, TerrainId, TerrainKind
      dual_grid.rs          # dual-grid coordinate mapping, bitmask computation
      wfc.rs                # WFC solver (standalone, no rendering dependency)
    src/shader/
      terrain.wgsl          # master shader + sub-shaders + noise helpers
    tests/
      main.rs               # test entry point
      suite/
        mod.rs
        material.rs
        dual_grid.rs
        wfc.rs

tools/
  terrain-viewer/           # NEW binary crate
    src/
      main.rs               # entry point, App setup
    Cargo.toml              # depends on: terrain, engine_app, engine_render,
                            #   engine_ecs, engine_input, engine_ui
```

`card_game` will depend on `terrain` when terrain is integrated into the game. The viewer binary depends only on `terrain` + engine crates, not on `card_game`.

### Dependencies

The `terrain` crate depends on:
- `engine_core` (types: `TextureId`, etc.)
- `engine_render` (types: `Material2d`, `ShaderHandle`, `ColorVertex`)
- `engine_ecs` (derives: `Component`, `Resource`)
- `rand` + `rand_chacha` (seeded RNG for WFC solver)
- `bevy_ecs` (through `engine_ecs`)

The `terrain-viewer` binary additionally depends on:
- `engine_app` (windowing, game loop)
- `engine_input` (keyboard/mouse)
- `engine_ui` (buttons, text)

## Incremental Phases

### Phase 1: Material Preview

**Goal:** Render individual terrain materials via the real shader pipeline. Iterate on per-type sub-shader techniques until each material reads clearly at game scale.

**Deliverables:**
- `TerrainMaterial` struct, `TerrainId`, `TerrainKind` enum in `terrain` crate
- `terrain.wgsl` master shader with shared noise primitives and 6 sub-shader functions
- Viewer binary rendering a single full-screen quad with the terrain shader
- Buttons to cycle through terrain types
- Runtime-adjustable uniform parameters (colors, frequencies, amplitudes, warp strengths) via buttons
- Parameter values displayed as text overlay
- Camera zoom to inspect noise at different scales

**Terrain types (initial set, maximally distinct):**
- Grass, Stone, Water, Sand, Lava, Snow

### Phase 2: Transition Preview

**Goal:** Validate auto-tile boundary rendering and per-pair transition effects on a small dual-grid.

**Deliverables:**
- Dual-grid coordinate mapping in `terrain` crate (corner bitmask computation, visual tile offset)
- Auto-tile boundary SDF computation in the shader (16 bitmask patterns)
- Noise-displaced boundary contours for organic edge shapes
- Per-pair transition effect functions for interesting combinations
- Viewer renders a small dual-grid (e.g., 4x4)
- Buttons to assign terrain types to grid regions or randomize

### Phase 3: WFC Grid

**Goal:** Generate and display full terrain maps using Wave Function Collapse.

**Deliverables:**
- WFC solver as a standalone pure function in `terrain` crate
- Constraint table (`BTreeMap` of adjacency rules)
- Cell pinning support (pre-assign specific cells before collapse)
- Viewer renders 20x15+ visible dual-grid tiles
- Camera pan to scroll the map
- Re-roll button (new seed), seed display
- Toggle terrain types on/off for WFC constraints

## Shader Architecture

### Master Shader: `terrain.wgsl`

A single WGSL file containing:

1. **Shared noise primitives** (~40-60 lines): gradient noise, Voronoi distance, fbm, domain warp, hash functions. Written once, used by all sub-shaders.

2. **Per-type sub-shader functions** (~10-25 lines each): Each terrain type uses whatever rendering technique best suits it. These are the primary iteration target.

3. **Auto-tile boundary computation** (~30-40 lines): Takes a 4-bit corner bitmask, produces a signed distance field for the boundary contour. 16 patterns total (uniform, single-corner, edge, diagonal, three-corner). Noise displacement applied on top.

4. **Per-pair transition effects** (~10-15 lines each, only for interesting pairs): Applied near the boundary based on `(my_type, neighbor_type, distance_to_edge)`. Generic fallback for undefined pairs.

5. **Fragment entry point**: Determines which terrain type the pixel belongs to (sharp boundary, not blend), renders the base material, then applies transition effects near edges.

### Sub-Shader Techniques (Starting Points)

These are initial approaches to iterate on in the viewer. The whole point is experimentation.

| Type | Primary Technique | Character |
|------|------------------|-----------|
| Grass | Layered directional noise, anisotropic stretch along wind direction | Blade-like directionality, shadow variation |
| Stone | Voronoi crack network + fbm surface variation | Large fractures, slab coloring, optional strata |
| Water | Domain-warped Voronoi caustics, animated via time uniform | Flow, refraction-like patterns, specular hints |
| Sand | Directional sine ripples + high-frequency grain sparkle | Dune ripples, warm sparkle overlay |
| Lava | Voronoi cells (cooled crust) with bright emission in gaps | Cracked plates, glowing fissures, slow drift |
| Snow | Very low-contrast gradient noise + sparse sparkle threshold | Subtle undulation, crystalline sparkle points |

### Per-Pair Transition Effects (Starting Points)

Only interesting pairs get specific effects. All others use a generic narrow darkening strip.

| From | To | Effect |
|------|----|--------|
| Stone | Sand | Scattered pebbles in sand, density falls off from edge |
| Lava | Grass | Singed/darkened blades, charred earth strip at boundary |
| Water | Sand | Wet sand darkening, foam/froth at waterline |
| Grass | Stone | Sparse grass tufts in stone cracks, moss creep |

### Uniform Layout

Per-entity uniform (`Material2d.uniforms`):

```
TileUniform {
    corner_types: vec4<u32>,    // 4 terrain IDs (dual-grid corners), u32 for WGSL alignment
    world_pos: vec2<f32>,       // world-space position for noise continuity
    seed: u32,                  // per-tile variation seed
    _pad: u32,                  // alignment padding
    materials: array<MaterialParams, 4>,  // params for the up-to-4 active types
}

MaterialParams {                // 48 bytes, 16-byte aligned
    color_a: vec4<f32>,         // primary color (rgb) + unused alpha
    color_b: vec4<f32>,         // secondary color (rgb) + unused alpha
    params: vec4<f32>,          // frequency, amplitude, warp, scale
    extra: vec4<f32>,           // type-specific (wind dir, strata angle, etc.)
}
```

Note: `vec3<f32>` in WGSL is 16-byte aligned (same size as `vec4<f32>`), so colors use `vec4<f32>` with the alpha channel unused. This avoids silent padding bugs. Total `MaterialParams` size: 64 bytes. Total `TileUniform` size: 16 + 8 + 4 + 4 + (64 * 4) = 288 bytes per tile.

In Phase 1 (single quad), only one `MaterialParams` is needed. The full 4-corner layout activates in Phase 2.

Runtime adjustment: viewer buttons modify the `MaterialParams` fields in the Rust-side `TerrainMaterial` struct, re-pack the uniform buffer, and update the entity's `Material2d.uniforms`. No shader recompilation needed.

## Data Types

```rust
/// Unique terrain type identifier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerrainId(pub u8);

/// Determines which sub-shader branch runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TerrainKind {
    Grass = 0,
    Stone = 1,
    Water = 2,
    Sand  = 3,
    Lava  = 4,
    Snow  = 5,
}

/// Describes a terrain type's visual parameters.
pub struct TerrainMaterial {
    pub id: TerrainId,
    pub kind: TerrainKind,
    pub name: &'static str,
    pub color_a: [f32; 3],
    pub color_b: [f32; 3],
    pub params: [f32; 4],       // frequency, amplitude, warp, scale
    pub extra: [f32; 4],        // type-specific tunables
}
```

## Dual-Grid Auto-Tiling

Follows the same principle as CardCleaner's corner16 format:

- **Data grid**: each cell holds a `TerrainId`
- **Visual grid**: offset by (-0.5, -0.5) cell, each visual tile straddles 4 data cells
- **Corner bitmask**: 4-bit value from the 4 data cells (NE=1, SE=2, SW=4, NW=8)
- **16 boundary patterns**: computed analytically as SDFs in the shader, not looked up from an atlas
- **Noise displacement**: applied to the SDF boundary for organic, non-grid-aligned edges

The `dual_grid` module provides:
- `DualGrid` struct wrapping a `Vec<TerrainId>` with width/height
- `visible_tiles(camera_rect) -> Vec<VisualTile>` returning position, 4 corner IDs, seed
- `corner_bitmask(corners: [TerrainId; 4]) -> u8` utility

## WFC Solver

Standalone pure function with no rendering dependency.

```rust
pub fn collapse(
    grid: &mut Grid<Option<TerrainId>>,
    constraints: &ConstraintTable,
    rng: &mut ChaCha8Rng,
) -> Result<(), WfcError>
```

- **Input**: partially-filled grid (`None` = uncollapsed), constraint table, seeded RNG
- **Output**: fully collapsed grid, or error on contradiction
- **Constraint table**: `BTreeMap<(TerrainId, TerrainId), AdjacencyRule>` with direction and frequency weight
- **Cell pinning**: pre-set cells to `Some(id)` before calling `collapse()`
- **Backtracking**: on contradiction, revert last assignment and try next option. If all options exhausted for a cell, propagate failure upward. Max backtrack depth to prevent infinite loops.
- **Deterministic**: same seed + same input = same output (ChaCha8Rng)

The solver knows nothing about rendering, shaders, or visual tiles. It operates on `TerrainId` values and adjacency booleans.

## Viewer Interaction

### Phase 1 Controls
- **Top bar**: `[Grass] [Stone] [Water] [Sand] [Lava] [Snow]` — terrain type buttons
- **Side/bottom**: parameter adjustment buttons with current values displayed
  - Colors: `[R+] [R-] [G+] [G-] [B+] [B-]` for color_a and color_b
  - Params: `[Freq+] [Freq-] [Amp+] [Amp-] [Warp+] [Warp-] [Scale+] [Scale-]`
  - Extra: labeled contextually per terrain kind
- **Mouse wheel**: zoom in/out to inspect noise at different scales
- **Parameter overlay**: text showing current values

### Phase 2 Controls
- Phase 1 controls still available for material tuning
- **Grid assignment**: buttons to place terrain types in grid regions
- **`[Randomize]`**: fill grid with 2-3 random types
- **Transition visible**: dual-grid boundaries render with auto-tile contours + per-pair effects

### Phase 3 Controls
- **`[Generate]`**: run WFC solver, populate the grid
- **`[Re-roll]`**: new random seed + generate
- **`[Seed: XXXXX]`**: display current seed
- **Camera pan**: click-drag to scroll the map
- **Terrain toggles**: enable/disable specific types for WFC generation

## Accepted Trade-Offs

- **Restart for structural shader changes.** Editing sub-shader technique (adding a new noise layer, changing the algorithm) requires recompile. Parameter tuning (colors, frequencies) is live via uniforms.
- **No shader hot-reload.** Iteration loop is edit WGSL → recompile → restart (~10-20s). Acceptable for a tool with 6 terrain types.
- **Switch-based dispatch.** Adding terrain type N+1 requires: new sub-shader function, new switch branch, new `TerrainKind` variant, recompile. For a card game with 6-8 types, this is fine.
- **Per-entity uniform duplication.** Each tile carries its own `MaterialParams` copies. Simpler than adding a storage buffer bind group to the engine's shape pipeline. Acceptable at 336 tiles.
- **No RON serialization in v1.** Materials are constructed in Rust code. RON comes if/when an editor needs to save/load material definitions.

## Future Direction (Not In Scope)

**Phase 4: Irregular Grid**
- Break the rectangular grid assumption. Voronoi-like quad meshes with variable cell shapes (like CardCleaner's `IrregularMesh`).
- Shader receives arbitrary quad geometry with UV mapping, not axis-aligned rectangles.
- Same material sub-shaders and transition effects apply — the boundary computation changes but the material rendering doesn't.
- Architecture note: the shader should use world-space UVs for noise evaluation (not cell-local UVs that assume rectangles) to stay compatible with irregular cells.

**Beyond Phase 4:**
- Decals and structures layered on top of terrain
- Biome affinity (CardSignature-based terrain selection)
- RON serialization for material definitions
- Editor tool for authoring materials and WFC constraints
- Integration into `card_game_bin` as game background
