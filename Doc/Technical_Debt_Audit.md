# Technical Debt Audit — Axiom2d

**Date:** 2025-03-14
**Commit:** `d9b9327`
**Test baseline:** 740 passed, 0 failed, 7 ignored, 1 doctest (across 12 crates)

---

## Executive Summary

The Axiom2d engine is in excellent shape for its stage of development: all 12 crates are implemented, 740+ tests pass, and the architecture closely follows the blueprint. This audit catalogues **30 findings** where the current implementation deviates from the vision in `Doc/Axiom_Blueprint.md` or carries technical debt worth tracking.

Most HIGH-severity items are performance-related (per-frame recomputation, uncached tessellation) rather than correctness bugs. MEDIUM items are missing subsystems that the blueprint describes but haven't been built yet. LOW items are infrastructure and tooling gaps.

| Severity | Count | Theme |
|----------|-------|-------|
| HIGH     | 6     | Performance & correctness debt in existing code |
| MEDIUM   | 14    | Architecture gaps — missing subsystems/features |
| LOW      | 10    | Infrastructure, tooling, documentation |

---

## Executive Summary Table

| ID | Severity | Effort | Category | Finding |
|----|----------|--------|----------|---------|
| TD-001 | HIGH | M | Performance | Transform propagation — no change detection |
| TD-002 | HIGH | M | Performance | Hierarchy maintenance — full rebuild every frame |
| TD-003 | HIGH | M | Performance | Visibility system — no change detection |
| TD-004 | HIGH | M | Performance | Shape tessellation — no caching |
| TD-005 | HIGH | L | Correctness | WgpuRenderer material methods are stubs |
| TD-006 | HIGH | S | Consistency | Physics systems not in DefaultPlugins |
| TD-007 | MEDIUM | L | Missing dep | naga-oil not integrated (shader composition) |
| TD-008 | MEDIUM | L | Missing dep | tiny-skia not integrated (CPU rasterization) |
| TD-009 | MEDIUM | L | Missing dep | noise crate not integrated (procedural textures) |
| TD-010 | MEDIUM | S | Missing dep | assets_manager not integrated (hot-reload) |
| TD-011 | MEDIUM | S | Version | fundsp version mismatch (0.21 vs blueprint 0.23) |
| TD-012 | MEDIUM | XL | Missing feature | Particle system (compute shaders + instanced points) |
| TD-013 | MEDIUM | XL | Missing feature | Tilemap system (chunk-based rendering) |
| TD-014 | MEDIUM | L | Missing feature | Animation system (state machines, spritesheets) |
| TD-015 | MEDIUM | M | Missing feature | Color grading post-process pass |
| TD-016 | MEDIUM | L | Missing feature | Text/glyph rendering |
| TD-017 | MEDIUM | M | Missing feature | Procedural texture generation |
| TD-018 | MEDIUM | M | Missing feature | Physics interpolation (FixedTimestep smoothing) |
| TD-019 | MEDIUM | S | Missing feature | Y-sorting (SortOrder from Y position) |
| TD-020 | MEDIUM | M | Architecture | RenderPass trait not implemented |
| TD-021 | LOW | M | Documentation | Minimal doc comments on public API |
| TD-022 | LOW | S | Documentation | 1 doctest across entire workspace |
| TD-023 | LOW | S | Infrastructure | No docs/llms.txt file |
| TD-024 | LOW | S | Infrastructure | No ARCHITECTURE.md |
| TD-025 | LOW | S | Infrastructure | No examples/ directory |
| TD-026 | LOW | S | Infrastructure | No tools/xtask build automation |
| TD-027 | LOW | S | Build | No .cargo/config.toml (sccache, linker, profile) |
| TD-028 | LOW | S | Feature flags | Missing dev/hot_reload/debug_draw/physics feature flags |
| TD-029 | LOW | M | Architecture | No function plugin support |
| TD-030 | LOW | L | Missing feature | Gamepad support (gilrs) |

**Effort key:** S = small (< 1 session), M = medium (1–2 sessions), L = large (3–5 sessions), XL = very large (5+ sessions)

---

## Tier 1: HIGH — Performance & Correctness Debt

### TD-001 — Transform propagation has no change detection

**Blueprint reference:** §Scene hierarchy — "Use change detection to skip unchanged subtrees."

**Current state:** `transform_propagation_system` in `crates/engine_scene/src/transform_propagation.rs` queries all root entities via `Query<(Entity, &Transform2D), Without<ChildOf>>` and walks the entire hierarchy unconditionally every frame. No `Changed<Transform2D>` or `Added<Transform2D>` filters are used.

**Gap:** O(n) full-tree walk every frame regardless of whether any transforms changed. In scenes with hundreds of static entities (UI panels, background tiles, scenery), this is wasted work.

**Impact:** Linear performance degradation with entity count. Acceptable during scaffolding but will bottleneck at scale.

**Resolution:** Add `Changed<Transform2D>` filter on roots. Track dirty flags through hierarchy — if a parent's GlobalTransform2D changes, its children must re-propagate. bevy_ecs provides `Changed<T>` and `Added<T>` filters natively.

**Effort:** M (1–2 sessions — requires careful handling of edge cases: newly added children, removed parents)

**Dependencies:** None — self-contained in engine_scene.

---

### TD-002 — Hierarchy maintenance rebuilds fully every frame

**Blueprint reference:** §Scene hierarchy — implied by change detection recommendation.

**Current state:** `hierarchy_maintenance_system` in `crates/engine_scene/src/hierarchy.rs` queries all `(Entity, &ChildOf)` pairs and rebuilds a `HashMap<Entity, Vec<Entity>>` from scratch each frame, then inserts/removes `Children` components accordingly.

**Gap:** Full rebuild is correct (handles reparenting) but expensive. Most frames have zero hierarchy changes.

**Impact:** O(n) work per frame where n is the number of entities with `ChildOf`. HashMap allocation every frame.

**Resolution:** Use `Changed<ChildOf>` and `Added<ChildOf>` to detect when hierarchy actually changes. Only rebuild affected subtrees. Consider a `Local<bool>` dirty flag or generation counter.

**Effort:** M

**Dependencies:** None.

---

### TD-003 — Visibility system has no change detection

**Blueprint reference:** §Scene hierarchy — visibility uses change detection by implication (same system pattern as transforms).

**Current state:** `visibility_system` in `crates/engine_scene/src/visibility.rs` walks all root entities and propagates `EffectiveVisibility` through the entire hierarchy unconditionally.

**Gap:** Same as TD-001 — O(n) walk every frame. Visibility rarely changes (typically toggled by game events, not per-frame).

**Impact:** Performance cost proportional to entity count, regardless of visibility changes.

**Resolution:** `Changed<Visible>` filter. Propagate only when a node's visibility actually changes.

**Effort:** M

**Dependencies:** Should be done alongside TD-001 for consistency.

---

### TD-004 — Shape tessellation is not cached

**Blueprint reference:** §Rendering pipeline — tessellation is a build-time or load-time operation, not per-frame.

**Current state:** `shape_render_system` in `crates/engine_render/src/shape.rs` calls `tessellate(&shape.variant)` for every visible shape on every frame. Lyon's `FillTessellator` creates new vertex/index buffers each time.

**Gap:** Identical shapes (e.g., all Circle { radius: 10.0 }) are re-tessellated from scratch every frame. Tessellation is CPU-intensive relative to buffer reuse.

**Impact:** Quadratic growth in CPU cost with shape count. Circles and polygons with identical parameters produce identical meshes.

**Resolution:** Cache `TessellatedMesh` in a component (e.g., `CachedMesh`) or a resource keyed by `ShapeVariant`. Invalidate on shape change. Alternatively, store `TessellatedMesh` alongside `Shape` and only tessellate on `Added<Shape>` or `Changed<Shape>`.

**Effort:** M

**Dependencies:** None.

---

### TD-005 — WgpuRenderer material methods are stubs

**Blueprint reference:** §Material system — "shader + parameters + textures into a data-driven definition" with "specialized pipelines cached by variant key hash."

**Current state:** In `crates/engine_render/src/wgpu_renderer.rs`, three Renderer trait methods are no-op stubs:
- `set_shader(&mut self, _shader: ShaderHandle) {}`
- `set_material_uniforms(&mut self, _data: &[u8]) {}`
- `bind_material_texture(&mut self, _texture: TextureId, _binding: u32) {}`

The ECS-side Material2d integration is complete (sorting, batching, dedup logic, SpyRenderer capture), but GPU-side pipeline cache and shader variant compilation are not implemented.

**Gap:** Material2d components are fully wired through the render systems but have zero GPU effect. Users setting custom shaders, uniforms, or texture bindings get silent no-ops.

**Impact:** Feature is architecturally complete but functionally inert on GPU. Risk of user confusion — API appears to work (no errors) but does nothing visually.

**Resolution:** Implement GPU pipeline cache keyed by `(ShaderHandle, BlendMode)`. Compile WGSL shader variants on demand. Bind uniform buffers and texture bind groups per-material. This is the largest single GPU-side feature remaining.

**Effort:** L (3–5 sessions — requires wgpu pipeline management, bind group layout design, shader compilation caching)

**Dependencies:** TD-007 (naga-oil) would simplify shader variant management.

---

### TD-006 — Physics systems not registered in DefaultPlugins

**Blueprint reference:** §Plugin architecture — "DefaultPlugins group provides the standard setup. Each plugin is self-contained — all systems, resources, and events a feature needs are registered in one `build()` method."

**Current state:** `physics_step_system` and `physics_sync_system` exist in engine_physics but are **not** registered by `DefaultPlugins` in `crates/axiom2d/src/default_plugins.rs`. Users must manually add them.

**Gap:** Inconsistency with other subsystems. Audio systems (play_sound_system, spatial_audio_system) are auto-registered. Physics requires manual setup despite having the same trait-abstraction pattern (PhysicsBackend/NullPhysicsBackend/RapierBackend).

**Impact:** Users forget to register physics systems. Silent failure — entities with RigidBody/Collider do nothing without explicit system registration.

**Resolution:** Either: (a) add physics systems to DefaultPlugins behind a `physics` feature flag (matching the `render` and `audio` pattern), or (b) create a `PhysicsPlugin` struct that users explicitly add. Option (a) is more consistent with the blueprint.

**Effort:** S

**Dependencies:** TD-028 (feature flags) — should add a `physics` feature flag.

---

## Tier 2: MEDIUM — Architecture Gaps

### TD-007 — naga-oil not integrated

**Blueprint reference:** §Shaders — "Use `naga-oil` for modular WGSL shader composition with `#define_import_path` and preprocessor directives."

**Current state:** The project uses raw `naga` (wgsl-in feature) for shader parse validation only. A custom `preprocess()` function in `crates/engine_render/src/material.rs` handles `#ifdef/#endif` directives. naga-oil is not a workspace dependency.

**Gap:** Custom preprocessor is limited to `#ifdef/#endif`. No `#define_import_path`, `#import`, or module composition — the features naga-oil provides for building complex shaders from reusable modules.

**Impact:** Shader modularity is limited. As the shader library grows (material variants, post-process effects), the custom preprocessor won't scale.

**Resolution:** Add `naga-oil` to workspace dependencies. Replace custom `preprocess()` with naga-oil's preprocessor. Enable `#import` for shader module reuse.

**Effort:** L

**Dependencies:** TD-005 (material GPU implementation) would benefit from this.

---

### TD-008 — tiny-skia not integrated

**Blueprint reference:** §Code-defined assets — "For offline sprite rasterization, `tiny-skia` renders vector shapes to pixel buffers that become GPU textures."

**Current state:** Not a workspace dependency. Vector graphics go directly through Lyon tessellation to GPU vertex buffers. No CPU-side rasterization path.

**Gap:** Cannot generate sprite textures from vector definitions at build time. The blueprint envisions a pipeline: define shapes → rasterize with tiny-skia → pack into atlas → upload to GPU.

**Impact:** All sprites must be authored as pre-made images or rendered as GPU shapes. No procedural sprite generation.

**Resolution:** Add `tiny-skia` to workspace. Create a rasterization pipeline in engine_render or engine_assets that converts vector definitions to `ImageData` for atlas packing.

**Effort:** L

**Dependencies:** None, but complements TD-009 (noise) and TD-017 (procedural textures).

---

### TD-009 — noise crate not integrated

**Blueprint reference:** §Code-defined assets — "Procedural textures use the `noise` crate (Perlin, Simplex, Worley, Fbm) composed via combinators."

**Current state:** Not a workspace dependency. No procedural texture generation capability.

**Gap:** Cannot generate terrain, clouds, marble, wood, or other noise-based textures programmatically.

**Impact:** Feature gap — procedural textures must be pre-authored or skipped entirely.

**Resolution:** Add `noise = "0.9"` to workspace. Create procedural texture module in engine_assets or engine_render.

**Effort:** L

**Dependencies:** TD-008 (tiny-skia) for rasterizing noise to pixel buffers.

---

### TD-010 — assets_manager not integrated

**Blueprint reference:** §RON serialization — "The `assets_manager` crate provides built-in hot-reloading with file watching, concurrent caching, and format-agnostic loading."

**Current state:** Custom `AssetServer<T>` in engine_assets provides basic add/get/remove with reference counting and RON file loading. No file watching or hot-reload capability.

**Gap:** No hot-reload support. Asset changes require restart. The blueprint explicitly recommends `assets_manager` for this.

**Impact:** Slower iteration during development. Not critical for production but significant for developer experience.

**Resolution:** Either integrate `assets_manager` alongside or replace `AssetServer<T>`, or add file-watching to the existing implementation. The custom `AssetServer` is well-tested (26 tests) — may be worth keeping and adding hot-reload as a feature.

**Effort:** S–M (depending on approach: integrate vs. extend)

**Dependencies:** None.

---

### TD-011 — fundsp version mismatch

**Blueprint reference:** §Dependency map — `fundsp = "0.23"`.

**Current state:** Workspace uses `fundsp = "0.21"`.

**Gap:** Two minor versions behind blueprint recommendation.

**Impact:** Low — likely API-compatible, but newer versions may have bug fixes or new DSP nodes.

**Resolution:** Bump to `fundsp = "0.23"` in workspace `Cargo.toml`. Run tests to verify compatibility.

**Effort:** S

**Dependencies:** None.

---

### TD-012 — Particle system not implemented

**Blueprint reference:** §Rendering pipeline — "Particle Pass" in the render pipeline, "Particle systems use compute shaders updating position/velocity/life buffers on the GPU, rendered as instanced point sprites."

**Current state:** No particle system exists. The render pipeline goes Clear → Atlas → Camera → Sprite → Shape → Post-Process → Present, skipping the Particle Pass entirely.

**Gap:** Missing entire subsystem. Particles are a core 2D game engine feature (explosions, fire, sparkles, weather).

**Impact:** Games cannot use particle effects without implementing them from scratch.

**Resolution:** Add particle module to engine_render: `ParticleEmitter` component, compute shader for GPU-side simulation, instanced point sprite rendering, integration into the render pipeline between Shape and Post-Process passes.

**Effort:** XL

**Dependencies:** TD-005 (GPU material system) for shader variant support.

---

### TD-013 — Tilemap system not implemented

**Blueprint reference:** §Scene hierarchy — "Tilemaps should NOT be individual entities per tile. Instead, one entity per tilemap chunk handles its own internal rendering."

**Current state:** No tilemap types, systems, or rendering exist anywhere in the codebase.

**Gap:** Missing entire subsystem. Tilemaps are fundamental to many 2D game genres (platformers, RPGs, strategy).

**Impact:** Users must implement their own tilemap from scratch.

**Resolution:** Create tilemap module (possibly a new `engine_tilemap` crate or module in engine_render): `TileMap` component with chunk-based storage, `TileSet` asset type, grid-based frustum culling, instanced tile rendering.

**Effort:** XL

**Dependencies:** TD-005 (material system) for tile shader variants.

---

### TD-014 — Animation system not implemented

**Blueprint reference:** §Code-defined assets — "Animation state machines use enums with transition tables (`HashMap<(AnimState, Trigger), AnimState>`). Spritesheets are defined as uniform grids or explicit rect regions."

**Current state:** No animation types or systems. `Text` component in engine_ui has `content: String` but no text animation. No sprite animation, skeletal animation, or tween system.

**Gap:** Missing animation state machines, spritesheet animation, transition tables, and tween/interpolation utilities.

**Impact:** All movement must be manually computed in user systems. No built-in sprite animation support.

**Resolution:** Add animation module to engine_render or a new engine_animation crate: `AnimationState` enum, `AnimationClip` (frame indices + durations), `SpriteAnimator` component, `animation_system` that updates `Sprite.uv_rect` based on current frame.

**Effort:** L

**Dependencies:** Sprite system (done), TextureAtlas (done).

---

### TD-015 — Color grading post-process not implemented

**Blueprint reference:** §Rendering pipeline — `[Post-Process: Color Grade]` pass after bloom.

**Current state:** Only bloom post-processing exists (`BloomSettings` + `post_process_system`). No color grading, LUT application, or color correction.

**Gap:** Blueprint specifies color grading as a standard post-process pass. Currently missing.

**Impact:** No built-in color grading. Games wanting cinematic color correction must implement custom shaders.

**Resolution:** Add color grading to post-process pipeline: exposure, contrast, saturation, color temperature uniforms applied via fullscreen quad pass. Optionally support LUT textures.

**Effort:** M

**Dependencies:** TD-005 (GPU material system) for shader integration.

---

### TD-016 — Text/glyph rendering not implemented

**Blueprint reference:** Not explicitly detailed, but UI text rendering is implied by the engine_ui crate and the `Text` component.

**Current state:** `Text` component exists in engine_ui (`content: String, font_size: f32, color: Color`) but is explicitly "data-only, glyph rendering deferred." No font loading, glyph rasterization, or text rendering pipeline.

**Gap:** Text components store data but render nothing. UI buttons/labels have no visible text.

**Impact:** Games cannot display text without implementing their own glyph rendering.

**Resolution:** Integrate a glyph rasterization library (e.g., `fontdue`, `ab_glyph`, or `cosmic-text`). Rasterize glyphs to atlas, render as textured quads. Add `text_render_system` to the render pipeline.

**Effort:** L

**Dependencies:** TextureAtlas (done) for glyph atlas packing.

---

### TD-017 — Procedural texture generation not implemented

**Blueprint reference:** §Code-defined assets — "Procedural textures use the `noise` crate... composed via combinators."

**Current state:** No procedural texture generation module or utilities.

**Gap:** Cannot generate textures programmatically from noise functions.

**Impact:** Missing feature — all textures must be pre-authored or loaded from files.

**Resolution:** Create procedural texture module after TD-008 and TD-009 are resolved. Compose noise functions → pixel buffer → ImageData → atlas.

**Effort:** M

**Dependencies:** TD-008 (tiny-skia), TD-009 (noise crate).

---

### TD-018 — Physics interpolation not implemented

**Blueprint reference:** §Time — "Fix Your Timestep" pattern with fixed DT implies interpolation for smooth rendering between physics steps.

**Current state:** `FixedTimestep` exists with accumulator, and `physics_step_system` uses `DeltaTime`. However, no interpolation between physics states for rendering. `physics_sync_system` copies exact physics positions to `Transform2D` — no blending between previous and current state.

**Gap:** At frame rates not aligned with physics step rate, objects will appear to stutter or jump. Classic "Fix Your Timestep" requires rendering at `alpha = accumulator / step_size` between previous and current state.

**Impact:** Visual stutter under variable frame rates or when physics step rate differs from render rate.

**Resolution:** Store previous Transform2D values. In physics_sync_system (or a separate interpolation system), lerp between previous and current position using `accumulator / step_size` alpha.

**Effort:** M

**Dependencies:** None.

---

### TD-019 — Y-sorting not implemented

**Blueprint reference:** §Render ordering — "Y-sorting for top-down games computes `SortOrder` from the entity's Y position."

**Current state:** `RenderLayer` + `SortOrder` exist and are used for render ordering. SortOrder is manually set. No automatic Y-based sorting.

**Gap:** Top-down games need automatic SortOrder computation from Y position. Currently requires manual system authoring.

**Impact:** Minor — easy for users to implement as a custom system, but the blueprint calls it out as a built-in feature.

**Resolution:** Add `y_sort_system` that writes `SortOrder(-(y as i32))` (or similar) for entities with a `YSort` marker component. Register in Phase::PostUpdate before rendering.

**Effort:** S

**Dependencies:** None.

---

### TD-020 — RenderPass trait not implemented

**Blueprint reference:** §Rendering pipeline — "Each pass implements a `RenderPass` trait with a `render(&self, encoder, view)` method."

**Current state:** Render passes are implemented as separate ECS systems (clear_system, sprite_render_system, shape_render_system, post_process_system) chained via schedule ordering. No `RenderPass` trait abstraction.

**Gap:** The blueprint envisions a trait-based linear pass chain. The current ECS system approach is arguably simpler and more in line with bevy_ecs patterns, but lacks the explicit pass-chain composition the blueprint describes.

**Impact:** Low — the current approach works well and is idiomatic for ECS. The RenderPass trait would add a layer of abstraction over the existing working system.

**Resolution:** Consider whether the RenderPass trait adds value over the current ECS system chain. If desired, create a `RenderPass` trait and wrap existing systems. This may be unnecessary complexity — the current approach may be superior for the ECS architecture.

**Effort:** M

**Dependencies:** None, but reconsider necessity.

---

## Tier 3: LOW — Infrastructure & Tooling

### TD-021 — Minimal doc comments on public API

**Blueprint reference:** §API design — "Every public item needs a one-line summary, semantic description, and an `# Examples` section with full `use` statements."

**Current state:** Most public types, traits, and functions have no `///` doc comments. Spot checks on engine_core, engine_render (Renderer trait, RendererRes, NullRenderer) show zero doc comments. Some doc comments exist on ShapeVariant and a few functions in engine_render.

**Gap:** Significant documentation deficit relative to the blueprint's standard. Public API is documented primarily through tests and naming conventions.

**Impact:** Harder for users (and LLMs) to understand API intent without reading source. `cargo doc` output is sparse.

**Resolution:** Add `///` doc comments to all public types, traits, trait methods, and free functions. Prioritize engine_core and engine_render as they are the most-used crates.

**Effort:** M (spread across sessions — can be done incrementally per crate)

**Dependencies:** None.

---

### TD-022 — Nearly zero doctests

**Blueprint reference:** §API design — "Doc tests compile and run via `cargo test`, so examples stay verified."

**Current state:** 1 doctest exists across the entire workspace (in `crates/engine_assets/src/handle.rs` — a `compile_fail` example). Zero executable doc examples.

**Gap:** Blueprint expects doctests as verified documentation. Currently almost none exist.

**Impact:** No executable API examples. Documentation and code can drift apart silently.

**Resolution:** Add `# Examples` sections with runnable doctests to key public APIs. Start with most-used types: Transform2D, Color, Rect, Sprite, Shape, Camera2D, InputState.

**Effort:** S–M (incremental)

**Dependencies:** TD-021 (doc comments are the natural home for doctests).

---

### TD-023 — No docs/llms.txt

**Blueprint reference:** §API design — "Consider a top-level `docs/llms.txt` file following the emerging convention for providing LLM-friendly project descriptions."

**Current state:** No `docs/` directory or `llms.txt` file. LLM context comes from `CLAUDE.md` and `Doc/Axiom_Blueprint.md`.

**Gap:** Blueprint recommends `docs/llms.txt` for LLM-friendly project summaries. `CLAUDE.md` serves a similar purpose but is Claude Code-specific.

**Impact:** Low — CLAUDE.md provides excellent LLM context. An llms.txt would benefit other LLM tools.

**Resolution:** Create `docs/llms.txt` with architecture overview, crate responsibilities, common patterns, and dependency relationships. Can be largely derived from CLAUDE.md.

**Effort:** S

**Dependencies:** None.

---

### TD-024 — No ARCHITECTURE.md

**Blueprint reference:** §Workspace structure — `docs/ARCHITECTURE.md` listed in the directory layout.

**Current state:** Architecture documentation lives in `Doc/Axiom_Blueprint.md` (vision) and `CLAUDE.md` (implementation state). No standalone ARCHITECTURE.md.

**Gap:** Blueprint directory layout includes `docs/ARCHITECTURE.md`. Existing docs serve the purpose but don't match the expected structure.

**Impact:** Low — documentation exists, just in different files.

**Resolution:** Create `docs/ARCHITECTURE.md` or rename/restructure existing docs. May be unnecessary if CLAUDE.md and the blueprint are sufficient.

**Effort:** S

**Dependencies:** None.

---

### TD-025 — No examples/ directory

**Blueprint reference:** §Workspace structure — `examples/` directory with `hello_world/`, `platformer/`, `particles/`.

**Current state:** A `crates/demo/` binary crate exists (solar system scene, 41 tests) but no `examples/` directory at workspace root.

**Gap:** Blueprint envisions examples as LLM generation templates: "Define canonical examples for every common task... LLMs use these as generation templates."

**Impact:** No template examples for LLM-assisted development. The demo crate partially serves this purpose but is a single complex scene rather than focused examples.

**Resolution:** Create `examples/` directory with focused, minimal examples: hello_world (window + colored rect), sprite_demo (atlas + sprites), physics_demo (bodies + collisions), audio_demo (sound effects + mixer), input_demo (keyboard + mouse + actions).

**Effort:** S–M

**Dependencies:** None.

---

### TD-026 — No tools/xtask build automation

**Blueprint reference:** §Workspace structure — `tools/xtask/` listed in the directory layout.

**Current state:** `tools/` directory exists but only contains `living-docs/`. No `xtask` crate.

**Gap:** No `cargo xtask` automation for common tasks (generating docs, running specific test suites, code generation).

**Impact:** Low — CI workflows handle automated tasks. xtask would improve local developer experience.

**Resolution:** Create `tools/xtask/` binary crate with common tasks. Can start minimal and grow.

**Effort:** S

**Dependencies:** None.

---

### TD-027 — No .cargo/config.toml

**Blueprint reference:** §Compile times — `.cargo/config.toml` with sccache, mold/lld linker, dev profile optimizations.

**Current state:** `.cargo/` directory contains only `audit.toml` and `mutants.toml`. No `config.toml`.

**Gap:** No build optimizations configured: no sccache, no fast linker, no dev profile tuning (line-tables-only debug, dependency optimization).

**Impact:** Potentially slower compile times than necessary. The project builds on Windows (MSVC toolchain) where mold isn't applicable, but lld and sccache are available.

**Resolution:** Create `.cargo/config.toml` with Windows-appropriate settings: sccache wrapper, MSVC-compatible linker optimizations, dev profile tuning. Note: mold is Linux-only; use `lld` or the default MSVC linker.

**Effort:** S

**Dependencies:** sccache installation on development machine.

---

### TD-028 — Missing feature flags

**Blueprint reference:** §Workspace structure — "Feature flags follow Bevy's three-tier model: subsystem features (`render`, `audio`, `physics`), development features (`dev`, `hot_reload`, `debug_draw`), and asset format features."

**Current state:** Two feature flags exist: `render` (default on) and `audio` (default on) in the axiom2d facade crate. No `physics`, `dev`, `hot_reload`, or `debug_draw` flags.

**Gap:** Incomplete feature flag coverage. Physics is always compiled but not auto-registered (see TD-006). No development-time feature flags for hot-reload or debug visualization.

**Impact:** Cannot compile without physics dependency. No debug drawing utilities. No hot-reload gate.

**Resolution:** Add feature flags: `physics` (default on, gates engine_physics), `dev` (enables debug assertions and verbose logging), `debug_draw` (enables debug visualization systems), `hot_reload` (gates file watching in asset loading).

**Effort:** S

**Dependencies:** TD-006 (physics DefaultPlugins integration).

---

### TD-029 — No function plugin support

**Blueprint reference:** §Plugin architecture — "The plugin architecture uses two forms: function plugins (`fn my_plugin(app: &mut App)`) for simple cases and struct plugins with `Default` for configurable ones."

**Current state:** Only struct-based plugins exist: `pub trait Plugin { fn build(&self, app: &mut App); }`. No `impl Plugin for fn` or `IntoPlugin` adapter for function pointers.

**Gap:** Blueprint recommends function plugins for simple cases. Currently requires a unit struct + impl for every plugin.

**Impact:** Minor boilerplate. Struct plugins work fine but are more verbose for simple cases.

**Resolution:** Add blanket impl: `impl<F: Fn(&mut App)> Plugin for F { fn build(&self, app: &mut App) { self(app) } }`. Or add an `IntoPlugin` trait.

**Effort:** S (possibly a single-line blanket impl, but may need careful handling of trait object safety)

**Dependencies:** None.

---

### TD-030 — Gamepad support not implemented

**Blueprint reference:** §Input — Step 1.4 in roadmap, "GamepadState resource via gilrs."

**Current state:** Marked `[NOT STARTED]` in `Doc/Implementation_Roadmap.md`. No gilrs dependency. No gamepad types.

**Gap:** Missing input source. Keyboard + mouse cover most 2D games, but gamepads are standard for platformers and action games.

**Impact:** Low priority — roadmap explicitly defers this ("Can skip until needed").

**Resolution:** Add `gilrs` dependency, create `GamepadState` resource, integrate axes/buttons into `ActionMap`.

**Effort:** L

**Dependencies:** engine_input (done).

---

## Positive Alignment Summary

The following blueprint recommendations are **well-implemented**:

| Blueprint Recommendation | Implementation Status |
|--------------------------|----------------------|
| Archetypal ECS with bevy_ecs | Fully implemented via engine_ecs wrapper |
| Function-parameter systems | All systems are plain `fn` with typed params |
| 5-phase scheduling (Input→PreUpdate→Update→PostUpdate→Render) | Phase enum with ScheduleLabel, PHASE_COUNT=5 |
| Code-defined assets (no binary files) | All assets are Rust code or RON data |
| Lyon for vector tessellation | Shape component with Circle/Polygon variants |
| fundsp for audio synthesis | SoundEffect with factory closures, SoundLibrary |
| cpal for audio output | CpalBackend with real sample mixing |
| WGSL shaders | Quad + shape + bloom shaders in WGSL |
| Trait-abstracted hardware | Renderer, AudioBackend, PhysicsBackend traits |
| Null implementations for testing | NullRenderer, NullAudioBackend, NullPhysicsBackend |
| SpyRenderer / recording pattern | SpyRenderer with 6+ capture types, builder API |
| Instanced quad rendering | WgpuRenderer with Instance buffer, single draw_indexed |
| Texture atlas with guillotiere | AtlasBuilder + ImageData + load_image_bytes |
| Camera2D with frustum culling | AABB culling in sprite + shape render systems |
| Bloom post-processing | Full pipeline: brightness → blur → composite |
| Material2d with blend modes | 3 BlendMode variants, 3 pipelines each for quad/shape |
| Parent-child hierarchy | ChildOf/Children with hierarchy_maintenance_system |
| Transform propagation | GlobalTransform2D from hierarchy |
| Visibility inheritance | Visible/EffectiveVisibility through hierarchy |
| RenderLayer + SortOrder | 5-layer enum + i32 sort order |
| RON serialization | serde derives on all pure-data types |
| Property-based testing | proptest across 7 crates |
| Snapshot testing | insta across 3 crates |
| Visual regression testing | HeadlessRenderer + SSIM comparison |
| Flat workspace of crates | 12 crates under crates/ with virtual manifest |
| Plugin architecture | Plugin trait + DefaultPlugins group |
| Feature flags for subsystems | `render` and `audio` flags in facade crate |
| NewType wrappers | Pixels, Seconds, TextureId, EntityId, PlaybackId |
| Enums over magic numbers | BlendMode, RenderLayer, Phase, MixerTrack, etc. |
| Deterministic game loop | FixedTimestep + FakeClock + time_system |
| Prelude modules per crate | All 12 crates export preludes |
| rapier2d physics | Full integration with collision events |
| Spatial audio | Distance attenuation + constant-power panning |
| Scene serialization | SceneDef/SceneNodeDef with RON support |
| Generic asset server | AssetServer\<T\> with Handle\<T\> + ref counting |

---

## Appendix: Severity / Effort Matrix

```
              │  S (small)  │  M (medium) │  L (large)  │  XL (v.large) │
──────────────┼─────────────┼─────────────┼─────────────┼───────────────┤
  HIGH        │  TD-006     │  TD-001     │  TD-005     │               │
              │             │  TD-002     │             │               │
              │             │  TD-003     │             │               │
              │             │  TD-004     │             │               │
──────────────┼─────────────┼─────────────┼─────────────┼───────────────┤
  MEDIUM      │  TD-011     │  TD-015     │  TD-007     │  TD-012       │
              │  TD-019     │  TD-017     │  TD-008     │  TD-013       │
              │  TD-010     │  TD-018     │  TD-009     │               │
              │             │  TD-020     │  TD-014     │               │
              │             │             │  TD-016     │               │
──────────────┼─────────────┼─────────────┼─────────────┼───────────────┤
  LOW         │  TD-022     │  TD-021     │  TD-030     │               │
              │  TD-023     │  TD-029     │             │               │
              │  TD-024     │             │             │               │
              │  TD-025     │             │             │               │
              │  TD-026     │             │             │               │
              │  TD-027     │             │             │               │
              │  TD-028     │             │             │               │
```

**Recommended priority order (maximum impact per effort):**
1. TD-006 (HIGH/S) — Physics DefaultPlugins consistency
2. TD-019 (MEDIUM/S) — Y-sorting
3. TD-011 (MEDIUM/S) — fundsp version bump
4. TD-004 (HIGH/M) — Shape tessellation caching
5. TD-001 + TD-002 + TD-003 (HIGH/M) — Change detection trio
6. TD-028 (LOW/S) — Feature flags
7. TD-027 (LOW/S) — .cargo/config.toml
8. TD-005 (HIGH/L) — GPU material implementation
