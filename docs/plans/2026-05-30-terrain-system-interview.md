# Procedural Terrain System — Requirements Spec

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

---

## Goal

Implement an asset-free procedural terrain system using WFC placement, dual-grid topology, and procedural fragment shaders for the card game's world backgrounds.

Covers backlog items: I25 (tilemap grid), I26 (tile definitions), I27 (dual-grid auto-tiling), I29 (WFC solver).

---

## Steps

### Step 1: WFC Solver (~200 lines)

- [ ] Create a standalone WFC (Wave Function Collapse) solver module in `crates/card_game/src/terrain/wfc.rs`.
  - Pure function: takes a `Grid<Option<TerrainId>>`, a constraint table (`BTreeMap<(TerrainId, TerrainId), bool>` with optional frequency weights), and a seed (`u64`).
  - Returns `Result<Grid<TerrainId>, WfcError>`.
  - Support cell pinning for gameplay-driven layout constraints (pre-filled `Some(id)` cells are never overwritten).
  - Use `ChaCha8Rng` seeded from the provided `u64`.
  - No rendering imports — this is a pure logic module.

**Proofs:**

- [ ] `rtk cargo build -p card_game` → exit 0
- [ ] `rtk cargo test -p card_game -- wfc` → exit 0, at least 5 tests pass
- [ ] `rtk grep "pub fn collapse" crates/card_game/src/terrain/` → function exists
- [ ] `rtk grep "use.*render\|use.*wgpu\|use.*lyon" crates/card_game/src/terrain/wfc.rs` → no matches (no rendering imports)
- [ ] `rtk grep "ChaCha8Rng" crates/card_game/src/terrain/wfc.rs` → at least 1 match

---

### Step 2: TerrainMaterial Data Type (~50 lines)

- [ ] Define `TerrainMaterial` struct in `crates/card_game/src/terrain/material.rs` with fields:
  - `color_a: [f32; 3]` — primary color
  - `color_b: [f32; 3]` — secondary color
  - `noise_type: NoiseType` — enum with `Gradient` and `Voronoi` variants
  - `noise_frequency: f32`
  - `noise_amplitude: f32`
  - `voronoi_cell_scale: f32`
  - `animation_speed: f32`
- [ ] Define `TerrainId` as a `u8` newtype wrapper.
- [ ] Construct 6–8 material instances in code (e.g., stone, grass, sand, water, dirt, snow, lava, ice). No RON serialization in v1.

**Proofs:**

- [ ] `rtk cargo build -p card_game` → exit 0
- [ ] `rtk grep "pub struct TerrainMaterial" crates/card_game/src/terrain/` → exists
- [ ] `rtk grep "pub struct TerrainId" crates/card_game/src/terrain/` → exists
- [ ] `rtk grep "Gradient\|Voronoi" crates/card_game/src/terrain/` → noise type enum variants exist

---

### Step 3: Dual-Grid Coordinate Mapping (~80 lines)

- [ ] Implement dual-grid offset logic in `crates/card_game/src/terrain/dual_grid.rs`.
  - Visual tiles straddle 4 logical grid cells, eliminating transition seams at triple-points.
  - For each visible tile, compute world position and the 4 corner `TerrainId` values.
  - Output: iterator of `(Vec2, [TerrainId; 4], u32)` tuples (position, corner IDs, seed).
  - Tile size configurable via constant or parameter.

**Proofs:**

- [ ] `rtk cargo build -p card_game` → exit 0
- [ ] `rtk cargo test -p card_game -- dual_grid` → exit 0, at least 3 tests pass
- [ ] `rtk grep "pub fn visible_tiles\|pub fn dual_grid" crates/card_game/src/terrain/` → function exists

---

### Step 4: terrain.wgsl Uber-Shader (~120 lines WGSL)

- [ ] Write terrain fragment shader at `crates/card_game/src/terrain/terrain.wgsl` with:
  - **Noise primitives** (~30 lines): gradient noise + Voronoi noise functions (not value noise — value noise has derivative discontinuities producing visible grid artifacts).
  - **Material dispatch** (5–10 lines each, 6–8 types): switch-based dispatch calling one function per terrain type (e.g., `fn stone(...)`, `fn grass(...)`, `fn sand(...)`, `fn water(...)`).
  - **Bilinear blending**: smoothstep blending of 4 corner materials at cell boundaries using UV coordinates.
  - **World-space evaluation**: noise sampled in world space for seamlessness across cells of same material.
  - **Per-entity uniform input**: reads 4 corner terrain IDs (`u8` packed into `u32`), seed (`u32`), world position (`vec2<f32>`), and material parameters for active types.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` → exit 0 (shader compiles at startup via ShaderRegistry)
- [ ] File exists: `crates/card_game/src/terrain/terrain.wgsl` (or equivalent location)
- [ ] `rtk grep "fn stone\|fn grass\|fn sand\|fn water" crates/card_game/src/terrain/terrain.wgsl` → at least 4 material functions
- [ ] `rtk grep "smoothstep" crates/card_game/src/terrain/terrain.wgsl` → at least 1 match (blending)
- [ ] `rtk grep "gradient_noise\|voronoi" crates/card_game/src/terrain/terrain.wgsl` → noise primitives exist

---

### Step 5: Quad Spawning System (~150 lines)

- [ ] ECS system in `crates/card_game/src/terrain/` that:
  - Reads the collapsed WFC grid.
  - Computes dual-grid visual tiles for visible region + 2-tile margin.
  - Spawns shape entities with `ShapeBundle` and `Material2d` components.
  - One quad (4 vertices, 2 triangles) per visible dual-grid tile. ~336 quads / ~672 triangles for a 20×15 visible grid.
  - Uses **entity recycling pool** for camera scrolling — re-assign uniforms as camera moves, don't spawn/despawn per frame.
  - Registers terrain shader via `ShaderRegistry::register()` as a `TerrainShader` resource.
  - Must be wired into `crates/card_game_bin/src/main.rs` with correct `Phase` and ordering constraints.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` → exit 0
- [ ] `rtk cargo test -p card_game -- terrain_spawn\|terrain_quad` → exit 0, at least 3 tests pass
- [ ] `rtk grep "terrain" crates/card_game_bin/src/main.rs` → system registered
- [ ] `rtk grep "Material2d" crates/card_game/src/terrain/` → Material2d used for quad entities
- [ ] `rtk grep "ShaderRegistry\|register" crates/card_game/src/terrain/` → shader registration

---

### Step 6: Visual Validation

- [ ] Run the card game binary and verify terrain renders as background behind cards.
- [ ] Verify camera scrolling shows terrain tiles entering/exiting smoothly (no pop-in, no gaps).

**Proofs:**

- [ ] `rtk cargo build --profile profiling -p card_game_bin` → exit 0
- [ ] `rtk cargo test -p card_game` → exit 0 (full suite still passes)
- [ ] `rtk cargo clippy -p card_game -p card_game_bin` → no new warnings

---

## Research Notes

- The engine's `Material2d` component supports per-entity shader switching via `ShaderHandle`.
- `apply_material` deduplicates shader and blend mode changes via `last_shader` tracking.
- `ShaderRegistry` stores WGSL sources keyed by `ShaderHandle`, compiled at startup.
- Material uniform buffer uses bind group 2 with dynamic offsets.
- Card game already has 8 custom fragment shaders running through this pipeline.
- `ColorVertex` struct carries UV coordinates (`uv: [f32; 2]`) passed through to fragment shader.
- Full debate analysis in `docs/debate-terrain-system.md`.
- Key engine files:
  - `crates/engine_render/src/material.rs`
  - `crates/engine_render/src/shader.rs`
  - `crates/engine_render/src/wgpu_renderer/renderer_trait.rs`
- Card game shader registration pattern: see `crates/card_game/src/card/rendering/art_shader.rs`.

---

## Open Questions

- **Pool sizing**: Camera scrolling tile lifecycle — how many entities in the recycling pool, pre-generation distance beyond visible margin.
- **Gameplay pinning**: WFC grid interaction with gameplay layout — which cells are pinned for play areas (hand, stash, discard).
- **Zoom-dependent noise**: Noise frequency adjustment at different zoom levels — fixed world-space frequency or LOD-aware.
- **Z-ordering**: Terrain z-ordering interaction with existing card/UI `RenderLayer` values — terrain must render behind everything.
- **Animation (deferred)**: Water flow, lava animation — deferred to future work but per-entity uniform buffer should reserve space or be extensible.
- **Contradiction handling**: WFC contradiction handling strategy — backtracking vs. restart with different seed. Backtracking is more robust but adds complexity.

---

## Deferred (Explicitly Out of Scope)

- RON serialization of terrain materials
- Editor tool for painting terrain
- Storage buffer (shared) for material data — using per-entity uniforms for v1
- Shader hot-reload
- Animated terrain (water flow, lava glow)
