# Technical Debt Audit — Axiom2d

> Archived reference. The active backlog now lives in `Doc/Work_Backlog.md`.
> Keep this file for historical context and debt analysis, not for day-to-day task tracking.

**Date:** 2026-03-21 (refreshed)
**Previous audit:** 2026-03-14 (commit `d9b9327`, 740 tests)
**Current baseline:** ~1,230 tests across 16 crates (12 engine + axiom2d + card_game + card_game_bin + demo + living_docs)

---

## Executive Summary

The engine and card game core are complete. Since the original audit, text rendering was implemented (ttf-parser + lyon), tracing instrumentation was added at hardware boundaries, the render sort system was replaced with DFS-based hierarchy_sort_system, and a unified render system was added to engine_ui. The card game grew from 0 to 330+ tests.

Most HIGH items from the original audit remain — they are performance optimizations that aren't bottlenecks at the card game's current entity count (~50 entities) but will matter for larger games. Several MEDIUM items were resolved. New findings from the codebase debate (Doc/archive/Codebase_Debate_2026-03-21.md) are included.

| Severity | Count | Theme |
|----------|-------|-------|
| HIGH     | 5     | Performance debt, GPU material stubs |
| MEDIUM   | 11    | Missing engine features, observability gaps |
| LOW      | 8     | Infrastructure, tooling, documentation |

---

## Resolved Since Last Audit

| ID | Finding | Resolution |
|----|---------|-----------|
| TD-006 | Physics not in DefaultPlugins | Card game uses CardGamePlugin which wires physics; pattern is game-registers-physics, not engine-auto-registers |
| TD-016 | Text/glyph rendering | Implemented via ttf-parser + lyon tessellation through shape pipeline. `draw_text` on Renderer trait, `card_text_render_system` wired in card_game_bin |
| TD-019 | Y-sorting | Superseded by DFS-based `hierarchy_sort_system` — `LocalSortOrder` controls sibling order, `SortOrder` is computed from DFS traversal. Y-sorting can be a user system setting `LocalSortOrder` |
| TD-020 | RenderPass trait | Deliberately not pursued — ECS system chain approach is simpler and idiomatic. `unified_render_system` in engine_ui demonstrates the pattern works well |

**Observability improvement:** `tracing` crate added as workspace dependency. Instrumentation added at hardware boundaries in engine_render, engine_physics, and engine_audio. tracing-subscriber with env-filter in card_game_bin.

---

## Summary Table

| ID | Severity | Effort | Category | Finding |
|----|----------|--------|----------|---------|
| TD-001 | HIGH | M | Performance | Transform propagation — no change detection |
| TD-002 | HIGH | M | Performance | Hierarchy maintenance — full rebuild every frame |
| TD-003 | HIGH | M | Performance | Visibility system — no change detection |
| TD-004 | HIGH | M | Performance | Shape tessellation — no caching |
| TD-005 | HIGH | L | Correctness | WgpuRenderer material methods are stubs |
| TD-007 | MEDIUM | L | Missing dep | naga-oil not integrated (shader composition) |
| TD-008 | MEDIUM | L | Missing dep | tiny-skia not integrated (CPU rasterization) |
| TD-009 | MEDIUM | L | Missing dep | noise crate not integrated (procedural textures) |
| TD-010 | MEDIUM | S | Missing dep | assets_manager not integrated (hot-reload) |
| TD-012 | MEDIUM | XL | Missing feature | Particle system |
| TD-013 | MEDIUM | XL | Missing feature | Tilemap system |
| TD-014 | MEDIUM | L | Missing feature | Animation system (state machines, spritesheets) |
| TD-015 | MEDIUM | M | Missing feature | Color grading post-process pass |
| TD-017 | MEDIUM | M | Missing feature | Procedural texture generation |
| TD-018 | MEDIUM | M | Missing feature | Physics interpolation (FixedTimestep smoothing) |
| TD-031 | MEDIUM | M | Observability | Renderer/PhysicsBackend/AudioBackend trait methods return () — silent failures |
| TD-032 | MEDIUM | L | Testing | No end-to-end schedule tests for card game system chains |
| TD-021 | LOW | M | Documentation | Minimal doc comments on public API |
| TD-022 | LOW | S | Documentation | Nearly zero doctests |
| TD-023 | LOW | S | Infrastructure | No docs/llms.txt file |
| TD-025 | LOW | S | Infrastructure | No examples/ directory |
| TD-027 | LOW | S | Build | No .cargo/config.toml (sccache, linker, profile) |
| TD-028 | LOW | S | Feature flags | Missing dev/hot_reload/debug_draw/physics feature flags |
| TD-030 | LOW | L | Missing feature | Gamepad support (gilrs) |

**Effort key:** S = small (< 1 session), M = medium (1–2 sessions), L = large (3–5 sessions), XL = very large (5+ sessions)

---

## Tier 1: HIGH — Performance & Correctness Debt

### TD-001 — Transform propagation has no change detection

`transform_propagation_system` walks the entire hierarchy unconditionally every frame. No `Changed<Transform2D>` filters. O(n) per frame regardless of whether transforms changed.

**Impact:** Acceptable at card game scale (~50 entities). Will bottleneck at hundreds of static entities.
**Resolution:** Add `Changed<Transform2D>` filter on roots, track dirty flags through hierarchy.
**Effort:** M

---

### TD-002 — Hierarchy maintenance rebuilds fully every frame

`hierarchy_maintenance_system` rebuilds a `HashMap<Entity, Vec<Entity>>` from all `(Entity, &ChildOf)` pairs every frame.

**Impact:** Same as TD-001 — O(n) work per frame for zero-change frames.
**Resolution:** Use `Changed<ChildOf>` and `Added<ChildOf>` to detect actual changes.
**Effort:** M

---

### TD-003 — Visibility system has no change detection

`visibility_system` walks all root entities and propagates `EffectiveVisibility` unconditionally.

**Impact:** Same pattern as TD-001/TD-002.
**Resolution:** `Changed<Visible>` filter. Should be done alongside TD-001 for consistency.
**Effort:** M

---

### TD-004 — Shape tessellation is not cached

`shape_render_system` and `unified_render_system` call `tessellate(&shape.variant)` for every visible shape every frame. Lyon's `FillTessellator` creates new vertex/index buffers each time.

**Impact:** CPU cost grows with shape count. Card game has ~200 shapes (4-6 per card × ~30 cards).
**Resolution:** Cache `TessellatedMesh` keyed by `ShapeVariant`. Invalidate on `Changed<Shape>`.
**Effort:** M

---

### TD-005 — WgpuRenderer material methods are stubs

Three Renderer trait methods are no-ops on WgpuRenderer: `set_shader()`, `set_material_uniforms()`, `bind_material_texture()`. ECS-side Material2d integration is complete but GPU-side does nothing.

**Impact:** Custom shaders, uniforms, and texture bindings are silently ignored at the GPU level.
**Resolution:** Implement GPU pipeline cache keyed by `(ShaderHandle, BlendMode)`. Compile WGSL variants on demand.
**Effort:** L

---

## Tier 2: MEDIUM — Architecture Gaps

### TD-007 — naga-oil not integrated

Custom `preprocess()` handles `#ifdef/#endif` only. No `#import` or module composition for shader reuse.

**Impact:** Shader modularity limited. Matters when shader library grows.
**Effort:** L

---

### TD-008 — tiny-skia not integrated

No CPU-side rasterization path. Vector graphics go directly to GPU via lyon tessellation.

**Impact:** Cannot generate sprite textures from vector definitions at build time.
**Effort:** L

---

### TD-009 — noise crate not integrated

No procedural texture generation capability.

**Impact:** Missing feature for procedural content generation.
**Effort:** L. **Depends on:** TD-008.

---

### TD-010 — assets_manager not integrated (hot-reload)

Custom `AssetServer<T>` has no file watching. Asset changes require restart.

**Impact:** Slower iteration. Not critical at current scale.
**Effort:** S–M

---

### TD-012 — Particle system not implemented

No particle emitters, compute shaders, or instanced point sprite rendering.

**Impact:** Games cannot use particle effects without implementing from scratch.
**Effort:** XL

---

### TD-013 — Tilemap system not implemented

No tilemap types, chunk-based storage, or grid-based frustum culling.

**Impact:** Missing subsystem for platformers, RPGs, strategy games.
**Effort:** XL

---

### TD-014 — Animation system not implemented

No animation state machines, spritesheet animation, or transition tables.

**Impact:** All sprite animation must be manually computed in user systems.
**Effort:** L

---

### TD-015 — Color grading post-process not implemented

Only bloom exists. No exposure, contrast, saturation, or LUT-based color correction.

**Effort:** M

---

### TD-017 — Procedural texture generation not implemented

Cannot compose noise functions into textures programmatically.

**Effort:** M. **Depends on:** TD-008, TD-009.

---

### TD-018 — Physics interpolation not implemented

`physics_sync_system` copies exact physics positions — no lerp between previous and current state for smooth rendering between physics steps.

**Impact:** Visual stutter when frame rate and physics step rate are misaligned.
**Effort:** M

---

### TD-031 — Hardware trait methods silently swallow failures (NEW)

**Source:** Codebase debate (Bryan Cantrill's analysis)

All methods on `Renderer`, `PhysicsBackend`, and `AudioBackend` traits return `()`. If GPU rejects a shader, atlas upload fails, or audio device is missing — no indication to the caller. `CpalBackend` silently degrades to no audio when no output device is available. `NullPhysicsBackend` returns `None` for position queries, causing systems to silently skip updates.

**Impact:** Hard-to-diagnose failures. The tracing instrumentation added at hardware boundaries helps but doesn't make the API contracts explicit about failure.
**Resolution:** Consider `Result` return types for fallible operations (shader compilation, atlas upload) or at minimum add tracing::warn! calls for degraded operation.
**Effort:** M (API change ripples through all trait impls and callers)

---

### TD-032 — No end-to-end schedule tests for card game (NEW)

**Source:** Codebase debate (Kent Beck's analysis)

330+ card game tests all run single systems in isolation. The `CardGamePlugin` wires 15+ systems with `.chain()` ordering, but no test exercises an actual pick-drag-release cycle through the real schedule. Plugin tests only verify resource insertion. If system ordering in `register_systems()` is wrong, no test catches it.

**Impact:** Individual system tests pass but system interaction bugs go undetected.
**Resolution:** Add integration tests that create a full `App` with `CardGamePlugin`, spawn cards, simulate multi-frame input sequences, and assert on final game state.
**Effort:** L (requires test helpers for multi-frame input simulation)

---

## Tier 3: LOW — Infrastructure & Tooling

### TD-021 — Minimal doc comments on public API
Most public types/traits/functions lack `///` doc comments.
**Effort:** M (incremental)

### TD-022 — Nearly zero doctests
1 compile_fail doctest. Zero executable doc examples.
**Effort:** S–M

### TD-023 — No docs/llms.txt
CLAUDE.md serves the purpose but an llms.txt would benefit other LLM tools.
**Effort:** S

### TD-025 — No examples/ directory
Demo crate exists but no focused, minimal examples.
**Effort:** S–M

### TD-027 — No .cargo/config.toml
No sccache, fast linker, or dev profile tuning configured.
**Effort:** S

### TD-028 — Missing feature flags
Only `render` and `audio` flags exist. No `physics`, `dev`, `hot_reload`, or `debug_draw`.
**Effort:** S

### TD-030 — Gamepad support (gilrs)
Deferred by design — keyboard+mouse covers current needs.
**Effort:** L

---

## Removed Items

| Original ID | Finding | Reason Removed |
|-------------|---------|----------------|
| TD-006 | Physics not in DefaultPlugins | Pattern resolved: game-level plugin registers physics, not DefaultPlugins |
| TD-011 | fundsp version mismatch | Version management is routine maintenance, not tech debt |
| TD-016 | Text/glyph rendering | Implemented (ttf-parser + lyon) |
| TD-019 | Y-sorting | Superseded by DFS hierarchy_sort_system |
| TD-020 | RenderPass trait | ECS system chain approach chosen deliberately |
| TD-024 | No ARCHITECTURE.md | CLAUDE.md + Blueprint serve the purpose adequately |
| TD-026 | No tools/xtask | living-docs tool exists; xtask is nice-to-have, not debt |
| TD-029 | No function plugin support | Minor ergonomic gap, not tech debt |

---

## Severity / Effort Matrix

```
              │  S (small)  │  M (medium) │  L (large)  │  XL (v.large) │
──────────────┼─────────────┼─────────────┼─────────────┼───────────────┤
  HIGH        │             │  TD-001     │  TD-005     │               │
              │             │  TD-002     │             │               │
              │             │  TD-003     │             │               │
              │             │  TD-004     │             │               │
──────────────┼─────────────┼─────────────┼─────────────┼───────────────┤
  MEDIUM      │  TD-010     │  TD-015     │  TD-007     │  TD-012       │
              │             │  TD-017     │  TD-008     │  TD-013       │
              │             │  TD-018     │  TD-009     │               │
              │             │  TD-031     │  TD-014     │               │
              │             │             │  TD-032     │               │
──────────────┼─────────────┼─────────────┼─────────────┼───────────────┤
  LOW         │  TD-022     │  TD-021     │  TD-030     │               │
              │  TD-023     │             │             │               │
              │  TD-025     │             │             │               │
              │  TD-027     │             │             │               │
              │  TD-028     │             │             │               │
```

**Recommended priority order (maximum impact per effort):**
1. TD-004 (HIGH/M) — Shape tessellation caching (biggest perf win for card game)
2. TD-001 + TD-002 + TD-003 (HIGH/M) — Change detection trio
3. TD-032 (MEDIUM/L) — End-to-end schedule tests
4. TD-031 (MEDIUM/M) — Silent failure observability
5. TD-005 (HIGH/L) — GPU material implementation
