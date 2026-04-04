# Master Backlog

Single source of truth for active work. Update this file first.

**Status legend:** `Open` | `In progress` | `Deferred`

**Reference docs:**
- `docs/Axiom_Blueprint.md` — architectural vision
- `docs/architecture_bible.md` — design principles
- `docs/Completed_Milestones.md` — completed work
- `docs/archive/` — historical debt audits, completed plans, old roadmaps

---

## Priority 1 — Architecture

Nothing is currently in progress. Pick one of these next.

- `TD-037` (`Open`): Add a render extraction phase and cached per-frame draw lists. Eliminates duplicated sorting, re-querying, and ad hoc render-time data rebuilding across `sprite_render_system`, `shape_render_system`, and `unified_render_system`. No plan yet — write one before starting.

---

## Priority 2 — Card Game Features

- `I11` (`Open`): Game session state machine.
- `I14` (`Open`): Signature-only serialization.
- `I15` (`Open`): Enforce card physics sleep behavior.
- `I16` (`Open`): Drop preview indicators for landing targets.
- `I17` (`Open`): Card highlight system.
- `I18` (`Open`): Batched card spawning.
- `I22` (`Open`): Auto-save.
- `I23` (`Open`): Generation progress UI.
- `I24` (`Open`): Pause system support.

---

## Priority 3 — Engine Gaps

Complete after the card game reaches playable state.

- `TD-038` (`Open`): Async task system for `Async`, `AsyncFixedUpdate`, `AsyncEndOfFrame` phases.
- `TD-039` (`Open`): Collision event dispatch — `OnCollision` phase needs events from physics backend.
- `TD-040` (`Open`): Entity lifecycle hooks — `OnEnable` / `OnDisable` / `OnDestroy`.
- `TD-041` (`Open`): Visibility change detection — `OnBecameVisible` frustum events.
- `TD-042` (`Open`): Pause/resume event flow — `OnPause` fires on app suspend.
- `TD-043` (`Open`): VBlank synchronization for `WaitForVBlank` phase.
- `TD-044` (`Open`): `PostRender` GPU readback / screenshot capture.
- `TD-018` (`Open`): Physics interpolation — smooth rendering between fixed steps.
- `TD-015` (`Open`): Color grading post-process pass.
- `TD-014` (`Open`): Animation system — state machines and spritesheet support.
- `TD-010` (`Open`): Hot-reload support for assets.
- `TD-007` (`Open`): Shader composition via `naga-oil` `#import`.
- `TD-008` (`Open`): CPU rasterization path (`tiny-skia`) for build-time image generation.
- `TD-009` (`Open`): Procedural texture generation (`noise` crate). Depends on TD-008.
- `TD-012` (`Open`): Particle system.
- `TD-013` (`Open`): Tilemap system.
- `TD-017` (`Open`): Procedural texture composition. Depends on TD-008/009.
- `TD-021` (`Open`): Improve public API documentation coverage.
- `TD-022` (`Open`): Add doctests for public behavior.
- `TD-023` (`Open`): Add `docs/llms.txt`.
- `TD-025` (`Open`): Add focused examples directory.
- `TD-027` (`Open`): Add `.cargo/config.toml` for local build tuning (sccache, mold, profile).
- `TD-028` (`Open`): Add missing feature flags — `dev`, `hot_reload`, `debug_draw`.
- `TD-030` (`Open`): Gamepad support (`gilrs`).

---

## Priority 4 — World Generation

- `I25` (`Open`): Tilemap grid system.
- `I26` (`Open`): Tile definitions and tile registry.
- `I27` (`Open`): Dual-grid auto-tiling.
- `I28` (`Open`): Biome definitions and affinity matching.
- `I28a` (`Open`): Biome strength precomputation grid.
- `I29` (`Open`): WFC tile solver.
- `I19a` (`Open`): Spatial coherence constraint for WFC.
- `I19b` (`Open`): No-solid-fill constraint for WFC.
- `I19` (`Open`): WFC soft modifiers.
- `I20` (`Open`): Biome distribution preview.
- `I21` (`Open`): Fog of war and line-of-sight.
- `I12` (`Open`): Cards-as-seeds world generation.
- `I13` (`Open`): Turn-based combat.

---

## Priority 5 — Devices & Simulation

- `I30` (`Open`): Jack and cable infrastructure.
- `I31` (`Open`): Card slot devices and signature chaining.
- `I32` (`Open`): Screen and button devices.
- `I33` (`Open`): Conveyor belt transport system.

---

## Stretch Goals

- `I34` (`Open`): Irregular quad mesh generation.
- `I35` (`Open`): Structure placement on maps.
- `I36` (`Open`): Enemy spawning and management.

---

## Deferred by Design

- Gamepad support (`gilrs`) — keyboard+mouse covers current needs.
- Hot reloading — restart-based iteration is fine at current scale.
- Examples directory — `demo` crate serves as reference.
