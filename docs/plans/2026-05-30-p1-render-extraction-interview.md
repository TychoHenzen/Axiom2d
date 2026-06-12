# Render Extraction Phase — Requirements Spec (Stub)

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
> **Stub spec.** Requirements are derived from backlog one-liners. Run `/interview` on individual items before implementing to fill in behavioral details, edge cases, and error handling.

## Context

**Backlog ID:** TD-037 (Priority 1 — Architecture)

**Goal:** Add a render extraction phase and cached per-frame draw lists to eliminate duplicated sorting, re-querying, and ad hoc render-time data rebuilding across `sprite_render_system`, `shape_render_system`, and `unified_render_system`.

The three render systems currently each independently query entities, sort by `RenderLayer`+`SortOrder`, and rebuild draw data every frame. The extraction phase should:
1. Run once per frame before any render system
2. Query all renderable entities, sort them, build a cached draw list
3. Each render system reads from the cached list instead of re-querying

**Test convention:** Tests live in `crates/<crate>/tests/suite/` with `when_action_then_outcome` naming. Systems must be wired into `crates/card_game_bin/src/main.rs`.

---

## Steps

### Step 1: Define ExtractedDrawList resource

- [ ] Create a resource type that holds the sorted, per-frame draw list. Should contain entity references with their `RenderLayer`, `SortOrder`, transform, material, and shape/sprite data. Inserted/updated once per frame.

**Proofs:**
- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk grep "pub struct ExtractedDrawList\|pub struct DrawRecord" crates/engine_render/src/` shows type exists

---

### Step 2: Implement render_extract_system

- [ ] System that runs in the Render phase (or a new sub-phase before render systems), queries all renderable entities, sorts by (`RenderLayer`, `SortOrder`), and populates `ExtractedDrawList`.

**Proofs:**
- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk cargo test -p engine_render -- render_extract\|draw_list` exits 0, at least 3 tests run
- [ ] `rtk grep "render_extract" crates/engine_render/src/` shows system function exists

---

### Step 3: Refactor sprite_render_system to consume ExtractedDrawList

- [ ] Remove independent entity querying and sorting from `sprite_render_system`. Read from `ExtractedDrawList` instead.

**Proofs:**
- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk cargo test -p engine_render` exits 0 (full crate tests pass)

---

### Step 4: Refactor shape_render_system to consume ExtractedDrawList

- [ ] Same as Step 3 for shape rendering.

**Proofs:**
- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk cargo test -p engine_render` exits 0

---

### Step 5: Refactor unified_render_system to consume ExtractedDrawList

- [ ] Same as Step 3 for the unified render system in `engine_ui`.

**Proofs:**
- [ ] `rtk cargo build -p engine_ui` exits 0
- [ ] `rtk cargo test -p engine_ui` exits 0

---

### Step 6: Wire into schedule and validate

- [ ] Register `render_extract_system` in the correct phase with ordering constraints (`.before()` the render systems). Verify in `crates/card_game_bin/src/main.rs`.

**Proofs:**
- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk cargo test` exits 0 (full workspace tests pass)
- [ ] `rtk cargo clippy --workspace` produces no new warnings
- [ ] `rtk grep "render_extract" crates/card_game_bin/src/main.rs` shows system registered

---

## Open Questions

> These MUST be resolved via `/interview` before implementation begins.

- What data does each render system actually need from the draw list? Are sprite/shape/text draw records homogeneous or do they need separate subtypes?
- Should the draw list be a single sorted `Vec<DrawRecord>` or have separate sprite/shape/text sublists?
- Should the extraction happen in a new sub-phase (e.g., `PreRender` or `Extract`) or at the start of the existing `Render` phase with `.before()` ordering?
- How does this interact with the existing `RenderLayer` visibility culling? Does extraction handle culling too, or is that a separate concern?
- What is the cache invalidation strategy? Rebuild every frame, or dirty-flag on component changes?
