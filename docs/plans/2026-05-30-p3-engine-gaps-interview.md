# Engine Gaps (Priority 3) — Requirements Spec (Stub)

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

Fill remaining engine capability gaps. Complete after the card game reaches playable state.

---

## Steps

### Step 1: TD-038 — Async task system `[ ]`

Implement an async task spawner and completion poller for the `Async`, `AsyncFixedUpdate`, and `AsyncEndOfFrame` schedule phases. Tasks spawned in one phase complete by the corresponding async phase. Crate: `engine_ecs`.

**Proofs:**

- [ ] `rtk cargo build -p engine_ecs` exits 0
- [ ] `rtk grep "pub struct AsyncTaskPool\|pub fn spawn_task" crates/engine_ecs/src/` matches at least 1 result
- [ ] `rtk cargo test -p engine_ecs -- async_task` exits 0

---

### Step 2: TD-039 — Collision event dispatch `[ ]`

Wire the `OnCollision` schedule phase to receive `CollisionEvent`s from the physics backend. The physics step system should populate an `EventBus<CollisionEvent>` that systems in the `OnCollision` phase can read. Crate: `engine_physics`.

**Proofs:**

- [ ] `rtk cargo build -p engine_physics` exits 0
- [ ] `rtk grep "EventBus.*CollisionEvent\|collision_event_dispatch" crates/engine_physics/src/` matches at least 1 result
- [ ] `rtk cargo test -p engine_physics -- collision_event` exits 0

---

### Step 3: TD-040 — Entity lifecycle hooks `[ ]`

Implement `OnEnable`, `OnDisable`, and `OnDestroy` lifecycle hook systems. Entities gain an `Enabled` component; toggling it fires events. Despawning fires `OnDestroy`. Crate: `engine_ecs`.

**Proofs:**

- [ ] `rtk cargo build -p engine_ecs` exits 0
- [ ] `rtk grep "OnEnable\|OnDisable\|OnDestroy" crates/engine_ecs/src/` matches at least 3 results
- [ ] `rtk cargo test -p engine_ecs -- lifecycle` exits 0

---

### Step 4: TD-041 — Visibility change detection `[ ]`

Implement frustum-based visibility tracking. When an entity enters or exits the camera frustum, fire `BecameVisible` / `BecameInvisible` events via `EventBus` for the `OnBecameVisible` phase. Crate: `engine_render`.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk grep "BecameVisible\|BecameInvisible" crates/engine_render/src/` matches at least 2 results
- [ ] `rtk grep "EventBus.*Visible" crates/engine_render/src/` matches at least 1 result

---

### Step 5: TD-042 — Pause/resume event flow `[ ]`

Fire `OnPause` / `OnResume` events when the application loses or regains focus (window minimized, suspended). Crate: `engine_app`.

**Proofs:**

- [ ] `rtk cargo build -p engine_app` exits 0
- [ ] `rtk grep "OnPause\|OnResume\|PauseEvent\|ResumeEvent" crates/engine_app/src/` matches at least 2 results
- [ ] `rtk cargo test -p engine_app -- pause` exits 0

---

### Step 6: TD-043 — VBlank synchronization `[ ]`

Implement `WaitForVBlank` phase logic — synchronize frame presentation with the display's vertical blank interval. Crate: `engine_app`.

**Proofs:**

- [ ] `rtk cargo build -p engine_app` exits 0
- [ ] `rtk grep "vblank\|VBlank\|vsync\|VSync" crates/engine_app/src/` matches at least 1 result
- [ ] `rtk grep "WaitForVBlank" crates/engine_app/src/` matches at least 1 result

---

### Step 7: TD-044 — PostRender GPU readback `[ ]`

Add GPU readback support in the `PostRender` phase — copy the framebuffer to a CPU-readable buffer for screenshot capture and visual regression testing. Crate: `engine_render`.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk grep "pub fn capture_screenshot\|pub struct ScreenshotRequest\|readback" crates/engine_render/src/` matches at least 1 result
- [ ] `rtk cargo test -p engine_render -- screenshot\|readback` exits 0

---

### Step 8: TD-018 — Physics interpolation `[ ]`

Smooth rendering between fixed physics steps by interpolating `Position` between the previous and current physics state using the `FixedTimestep` accumulator remainder. Crate: `engine_physics`.

**Proofs:**

- [ ] `rtk cargo build -p engine_physics` exits 0
- [ ] `rtk grep "interpolat" crates/engine_physics/src/` matches at least 1 result
- [ ] `rtk cargo test -p engine_physics -- interpolat` exits 0

---

### Step 9: TD-015 — Color grading post-process pass `[ ]`

Add a color grading post-process pass (LUT-based or parametric: exposure, contrast, saturation, color balance). Runs in the post-process stage of the render pipeline. Crate: `engine_render`.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk grep "ColorGrading\|color_grading\|color_grade" crates/engine_render/src/` matches at least 1 result
- [ ] `rtk grep "\.wgsl" crates/engine_render/src/` — a new color grading shader file exists or is referenced

---

### Step 10: TD-014 — Animation system `[ ]`

Implement a sprite animation system with state machines and spritesheet support. Animate UV offsets, component fields, or sprite indices over time with configurable transitions. Crate: `engine_render` (or new `engine_animation` — design decision needed).

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0 (or the chosen crate)
- [ ] `rtk grep "AnimationState\|AnimationClip\|Spritesheet" crates/engine_render/src/` matches at least 2 results
- [ ] `rtk cargo test -p engine_render -- animation` exits 0

---

### Step 11: TD-010 — Hot-reload support `[ ]`

Add file-watching and hot-reload infrastructure for code-defined assets. When source data changes, affected resources are rebuilt without restarting the application. Crate: `engine_assets`. Gated behind a `hot_reload` feature flag.

**Proofs:**

- [ ] `rtk cargo build -p engine_assets --features hot_reload` exits 0
- [ ] `rtk grep "hot_reload\|file_watcher\|FileWatcher" crates/engine_assets/src/` matches at least 1 result
- [ ] `rtk grep 'hot_reload' crates/engine_assets/Cargo.toml` matches (feature flag exists)

---

### Step 12: TD-007 — Shader composition via naga-oil `[ ]`

Integrate `naga-oil` to enable `#import` directives in WGSL shaders, allowing shared shader utilities across passes. Crate: `engine_render`.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk grep "naga.oil\|naga_oil" crates/engine_render/Cargo.toml` matches (dependency added)
- [ ] `rtk grep "#import\|naga_oil" crates/engine_render/src/` matches at least 1 result

---

### Step 13: TD-008 — CPU rasterization path (tiny-skia) `[ ]`

Add a CPU rasterization backend using `tiny-skia` for build-time image generation. Enables procedural texture baking without a GPU context. Crate: `engine_render` (or `engine_assets` — design decision needed).

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0 (or the chosen crate)
- [ ] `rtk grep "tiny.skia\|tiny_skia" crates/engine_render/Cargo.toml` matches (dependency added)
- [ ] `rtk cargo test -p engine_render -- cpu_raster\|tiny_skia` exits 0

---

### Step 14: TD-009 — Procedural texture generation `[ ]`

Implement procedural texture generation using the `noise` crate (Perlin, Simplex, Worley, etc.). Rasterizes noise functions to pixel buffers via the CPU rasterization path (TD-008). Depends on TD-008.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0 (or the chosen crate)
- [ ] `rtk grep "noise" crates/engine_render/Cargo.toml` matches (dependency added)
- [ ] `rtk grep "ProceduralTexture\|NoiseTexture\|noise_texture" crates/engine_render/src/` matches at least 1 result

---

### Step 15: TD-012 — Particle system `[ ]`

Implement a 2D particle system — emitters, lifetime, velocity, color/size over life, burst and continuous modes. Integrates with the existing shape/sprite render pipeline. Crate: `engine_render` (or new crate — design decision needed).

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0 (or the chosen crate)
- [ ] `rtk grep "ParticleEmitter\|ParticleSystem\|particle" crates/engine_render/src/` matches at least 2 results
- [ ] `rtk cargo test -p engine_render -- particle` exits 0

---

### Step 16: TD-013 — Tilemap system `[ ]`

Implement engine-level tilemap rendering — efficient batched rendering of tile grids with configurable tile size, multiple layers, and atlas UV mapping. Crate: `engine_render`.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0
- [ ] `rtk grep "Tilemap\|TileLayer\|tilemap" crates/engine_render/src/` matches at least 2 results
- [ ] `rtk cargo test -p engine_render -- tilemap` exits 0

---

### Step 17: TD-017 — Procedural texture composition `[ ]`

Compose multiple procedural texture layers (noise, gradients, masks) into final textures. Builds on TD-008 (CPU raster) and TD-009 (noise generation). Depends on TD-008, TD-009.

**Proofs:**

- [ ] `rtk cargo build -p engine_render` exits 0 (or the chosen crate)
- [ ] `rtk grep "TextureCompose\|compose_texture\|LayerStack" crates/engine_render/src/` matches at least 1 result
- [ ] `rtk cargo test -p engine_render -- texture_compos` exits 0

---

### Step 18: TD-021 — Improve public API documentation coverage `[ ]`

Add `#![warn(missing_docs)]` to all engine crates and fill in missing doc comments for all `pub` items. Workspace-level.

**Proofs:**

- [ ] `rtk cargo doc --workspace --no-deps` exits 0 with no warnings about missing docs
- [ ] `rtk grep "missing_docs" crates/engine_core/src/lib.rs` matches (lint enabled)
- [ ] `rtk grep "missing_docs" crates/engine_render/src/lib.rs` matches (lint enabled)

---

### Step 19: TD-022 — Add doctests for public behavior `[ ]`

Add `/// # Examples` doc blocks with runnable examples to key public functions and structs across engine crates. Workspace-level.

**Proofs:**

- [ ] `rtk cargo test --workspace --doc` exits 0
- [ ] `rtk grep "# Examples" crates/engine_core/src/` matches at least 3 results
- [ ] `rtk grep "# Examples" crates/engine_ecs/src/` matches at least 2 results

---

### Step 20: TD-023 — Add docs/llms.txt `[ ]`

Create a `docs/llms.txt` file summarizing the project for LLM consumption — crate map, key types, API patterns, and conventions. Workspace-level.

**Proofs:**

- [ ] File `docs/llms.txt` exists and is non-empty
- [ ] `rtk grep "engine_core\|engine_render\|engine_ecs" docs/llms.txt` matches at least 3 results

---

### Step 21: TD-025 — Add focused examples directory `[ ]`

Create an `examples/` directory (or `crates/examples/`) with small, focused examples demonstrating individual engine features (windowing, shapes, sprites, input, audio, physics). Workspace-level.

**Proofs:**

- [ ] Directory `examples/` exists with at least 3 `.rs` files
- [ ] `rtk cargo build --examples` exits 0

---

### Step 22: TD-027 — Add .cargo/config.toml `[ ]`

Add `.cargo/config.toml` with local build tuning — sccache configuration, linker settings, profile overrides for dev builds. Workspace-level.

**Proofs:**

- [ ] File `.cargo/config.toml` exists and is non-empty
- [ ] `rtk cargo build` exits 0 (config doesn't break the build)

---

### Step 23: TD-028 — Add missing feature flags `[ ]`

Add `dev`, `hot_reload`, and `debug_draw` feature flags to relevant crates. Gate debug-only systems and hot-reload infrastructure behind these flags. Workspace-level + individual crates.

**Proofs:**

- [ ] `rtk grep 'debug_draw\|hot_reload' Cargo.toml` matches (workspace features defined)
- [ ] `rtk cargo build --features debug_draw` exits 0
- [ ] `rtk cargo build` exits 0 (default build unaffected)

---

### Step 24: TD-030 — Gamepad support `[ ]`

Add gamepad input via `gilrs`. Map gamepad axes and buttons to the existing `engine_input` abstraction layer. Crate: `engine_input`.

**Proofs:**

- [ ] `rtk cargo build -p engine_input` exits 0
- [ ] `rtk grep "gilrs" crates/engine_input/Cargo.toml` matches (dependency added)
- [ ] `rtk grep "Gamepad\|gamepad" crates/engine_input/src/` matches at least 2 results

---

## Dependency Graph

```
TD-008 (CPU raster) ──► TD-009 (noise textures) ──► TD-017 (texture composition)
TD-007 (naga-oil) is independent but benefits TD-015, TD-009
TD-039 (collision events) ──► TD-040 (lifecycle hooks) are complementary
TD-028 (feature flags) ──► TD-010 (hot-reload), TD-011 gated behind flags
```

## Open Questions

- **Crate placement for TD-014 (animation), TD-012 (particles), TD-013 (tilemap):** These could live in `engine_render` or become separate crates (`engine_animation`, `engine_particles`, `engine_tilemap`). Separate crates follow the existing flat workspace pattern but increase compile units. Run `/interview` to decide.
- **TD-008/009/017 crate ownership:** CPU rasterization could be `engine_render` (same rendering concern) or `engine_assets` (build-time generation is an asset pipeline concern). Run `/interview` to decide.
- **TD-014 animation scope:** State machines + spritesheet is the minimum. Does this also cover tweening, skeletal animation, or property animation tracks? Run `/interview` to scope.
- **TD-021/022 coverage targets:** What percentage of public items need docs? Which crates are highest priority? Run `/interview` per crate.
- **TD-028 feature flag granularity:** Should `debug_draw` be per-crate or workspace-wide? Should `dev` aggregate all debug features? Run `/interview` to finalize the flag taxonomy.
- **All items need detailed design before implementation.** The backlog entries are one-liners. Run `/interview` on each item to fill in behavioral details, edge cases, error handling, and API surface.
