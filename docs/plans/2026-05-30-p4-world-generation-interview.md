# World Generation (Priority 4) — Requirements Spec (Stub)

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
>
> ⚠️ **Stub spec.** Requirements are derived from backlog one-liners. Run `/interview` on individual items before implementing to fill in behavioral details, edge cases, and error handling.

---

## Goal

Implement world generation systems for the card game — tilemap infrastructure, biome system, WFC solver, and gameplay systems.

**Cross-reference:** Items I25 (tilemap grid), I26 (tile definitions), I27 (dual-grid auto-tiling), and I29 (WFC solver) overlap with the Terrain System spec in `docs/plans/2026-05-30-terrain-system-interview.md`. The terrain spec covers the visual/rendering side (procedural shaders, dual-grid UV mapping, wgpu pipeline). This spec covers the gameplay/data side (grid storage, biome logic, generation constraints, gameplay integration). The WFC solver itself is owned by the terrain spec — this spec consumes its output.

---

## Steps

### Step 1: I25 — Tilemap grid system `[ ]`

Implement a generic `Grid<T>` data structure for world tiles — 2D array with coordinate conversion, bounds checking, neighbor iteration, and region queries. Pure data, no rendering. Module: `crates/card_game/src/terrain/grid.rs` (or reuse if already created by terrain spec).

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "pub struct Grid" crates/card_game/src/terrain/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- grid` exits 0, at least 3 tests pass

---

### Step 2: I26 — Tile definitions and tile registry `[ ]`

Define tile types with gameplay properties (walkable, resource yield, movement cost, etc.) and a registry for lookup by ID. Module: `crates/card_game/src/terrain/tile.rs` or `crates/card_game/src/world/tile.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "pub struct TileDef\|pub struct TileRegistry\|TileProperties" crates/card_game/src/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- tile_def\|tile_registry` exits 0

---

### Step 3: I27 — Dual-grid auto-tiling (gameplay side) `[ ]`

Implement the gameplay-side dual-grid logic — given a `Grid<TileId>`, determine visual tile variants based on neighbor context (4-corner lookup). The visual rendering side (shader selection, UV mapping) is in the terrain spec. Module: `crates/card_game/src/terrain/dual_grid.rs` (may share with terrain spec).

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "dual_grid\|DualGrid\|corner_lookup" crates/card_game/src/terrain/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- dual_grid` exits 0

---

### Step 4: I28 — Biome definitions and affinity matching `[ ]`

Define biome types with terrain-type affinity tables — each biome specifies which tile types it prefers and at what frequency. Biomes influence WFC frequency weights. Module: `crates/card_game/src/world/biome.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "pub struct Biome\|BiomeAffinity\|pub enum BiomeType" crates/card_game/src/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- biome` exits 0, at least 2 tests pass

---

### Step 5: I28a — Biome strength precomputation grid `[ ]`

Precompute a grid of biome influence values across the map. Each cell stores the strength of each nearby biome, enabling smooth biome blending and transition zones. Depends on I28. Module: `crates/card_game/src/world/biome_grid.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "BiomeStrength\|biome_strength\|precompute_biome" crates/card_game/src/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- biome_strength\|biome_grid` exits 0

---

### Step 6: I29 — WFC tile solver (gameplay integration) `[ ]`

Integrate the WFC solver (owned by the terrain spec in `crates/card_game/src/terrain/wfc.rs`) with gameplay data — feed tile definitions and biome affinities as WFC constraints and frequency weights. This step does NOT reimplement the solver; it builds the constraint tables from gameplay data and calls the solver. Module: `crates/card_game/src/world/generation.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "pub fn generate_world\|build_constraints\|WfcConfig" crates/card_game/src/world/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- generate_world\|world_gen` exits 0

---

### Step 7: I19a — Spatial coherence constraint for WFC `[ ]`

Add a constraint to the WFC solver (or its configuration layer) that encourages adjacent tiles to form spatially coherent regions rather than checkerboard noise. Implemented as a weight modifier that boosts same-type neighbors. Depends on I29. Module: `crates/card_game/src/world/wfc_constraints.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "coherence\|SpatialCoherence\|neighbor_boost" crates/card_game/src/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- coherence` exits 0

---

### Step 8: I19b — No-solid-fill constraint for WFC `[ ]`

Add a constraint that prevents WFC from producing large uniform rectangles of the same tile type. Implemented as a penalty when a cell's entire neighborhood is the same type. Depends on I29. Module: `crates/card_game/src/world/wfc_constraints.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "solid_fill\|NoSolidFill\|uniformity_penalty" crates/card_game/src/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- solid_fill\|no_solid` exits 0

---

### Step 9: I19 — WFC soft modifiers `[ ]`

Implement frequency hints and preference weights for WFC — biome affinities, rarity curves, and player-influenced biases that nudge tile selection without hard constraints. Depends on I29. Module: `crates/card_game/src/world/wfc_constraints.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "FrequencyHint\|SoftModifier\|preference_weight" crates/card_game/src/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- soft_modifier\|frequency_hint` exits 0

---

### Step 10: I20 — Biome distribution preview `[ ]`

Visualization system that renders a preview of biome layout before full terrain generation. Shows biome boundaries, influence gradients, and seed points as a debug overlay or separate UI panel. Depends on I28, I28a. Module: `crates/card_game/src/world/biome_preview.rs`. System wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "biome_preview\|BiomePreview" crates/card_game/src/` matches at least 1 result
- [ ] `rtk grep "biome_preview" crates/card_game_bin/src/main.rs` matches (system registered)

---

### Step 11: I21 — Fog of war and line-of-sight `[ ]`

Implement fog of war — tiles start hidden, revealed when within line-of-sight of the player or units. Track explored vs visible vs hidden states per tile. Render fog as a darkening overlay. Module: `crates/card_game/src/world/fog.rs`. System wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "FogOfWar\|FogState\|line_of_sight\|fog_system" crates/card_game/src/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- fog\|line_of_sight` exits 0

---

### Step 12: I12 — Cards-as-seeds world generation `[ ]`

Use card signatures (from the existing `Signature` system) to seed terrain generation parameters. A card's signature deterministically produces a unique world layout — same card always generates the same map. Depends on full world gen pipeline (I25, I26, I28, I29). Module: `crates/card_game/src/world/card_seed.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "card_seed\|CardSeed\|signature_to_seed\|seed_from_signature" crates/card_game/src/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- card_seed` exits 0, including a determinism test (same signature produces same grid)

---

### Step 13: I13 — Turn-based combat `[ ]`

Implement a turn-based combat system using cards on the generated world. Players and enemies take turns performing actions (move, attack, use card ability). Combat resolves on the tile grid. Module: `crates/card_game/src/combat/`. System wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "TurnPhase\|CombatState\|combat_system" crates/card_game/src/combat/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- combat\|turn_based` exits 0
- [ ] `rtk grep "combat" crates/card_game_bin/src/main.rs` matches (system registered)

---

## Dependency Graph

```
I25 (grid) ──► I26 (tile defs) ──► I29 (WFC integration)
                                       │
I28 (biomes) ──► I28a (biome grid) ────┘
                                       │
                         ┌─────────────┤
                         ▼             ▼
                   I19a (coherence)  I19b (no-solid-fill)
                         │             │
                         └──────┬──────┘
                                ▼
                          I19 (soft modifiers)
                                │
I27 (dual-grid) ◄───────── terrain spec owns visual side

I28 + I28a ──► I20 (biome preview)

I25 + I26 + I28 + I29 ──► I12 (cards-as-seeds)

I21 (fog of war) — independent, needs I25 grid

I13 (combat) — needs I25 grid + I12 world gen
```

## Open Questions

- **WFC solver ownership:** The terrain spec (`2026-05-30-terrain-system-interview.md`) owns the WFC solver implementation in `crates/card_game/src/terrain/wfc.rs`. This spec's I29 step builds the constraint tables and calls that solver. If the solver API changes, both specs need updating. Run `/interview` on I29 to clarify the boundary.
- **Module organization:** Should world-generation code live in `crates/card_game/src/world/` (separate from terrain rendering in `src/terrain/`) or be a peer module? The dual-grid logic (I27) is shared between gameplay and rendering — who owns the module? Run `/interview` to decide.
- **I13 (combat) scope:** Turn-based combat is a major system. The one-liner doesn't specify: action economy, damage model, card ability resolution, win/loss conditions, AI for enemies. This needs a full `/interview` and likely its own dedicated spec before implementation.
- **I12 (cards-as-seeds) determinism:** How much of the world is seeded by the card vs procedural? Does every card property map to a generation parameter, or only the signature hash? Run `/interview` to scope.
- **I21 (fog of war) rendering:** The fog overlay needs to integrate with the existing render pipeline without a separate render pass (per architecture bible). Implementation approach needs `/interview`.
- **I20 (biome preview) UX:** Is this a debug-only overlay toggled with a key, or a gameplay UI element shown during world creation? Run `/interview` to determine.
- **All items need detailed design before implementation.** Run `/interview` on each item individually.
