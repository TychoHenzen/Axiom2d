# Work Backlog

This is the canonical source of truth for active work going forward.
Update this file first. The older docs remain as archives and design references:

- `Doc/Technical_Debt_Audit.md` for the historical debt audit
- `Doc/CardCleaner_Ideas_Roadmap.md` for the detailed feature roadmap
- `Doc/Completed_Milestones.md` for completed work only

Status legend:

- `Open` = not started
- `In progress` = actively underway
- `Deferred` = intentionally not next

Order matters:

- Finish the architecture unification items below before starting new feature work.
- See `Doc/Asymmetry_Duplication_Resolution_Plan.md` for the rationale and migration rules.

## Architecture Unification First

- `TD-035` (`Open`): Replace preload/post-splash `FnMut(&mut World)` hook queues with typed startup schedules/phases registered like normal ECS systems.
- `TD-036` (`Open`): Normalize all raw platform input through a single event ingestion path, including cursor movement and wheel input, before deriving frame state resources.
- `TD-034` (`Open`): Centralize physics ownership behind a command/reconcile layer so gameplay systems stop mutating `PhysicsRes` ad hoc.
- `TD-033` (`Open`): Replace the card game's long chained interaction pipeline with explicit interaction intents/events and a smaller number of authoritative applier systems.
- `TD-037` (`Open`): Add a render extraction phase and cached per-frame draw lists to reduce duplicated sorting, re-querying, and ad hoc render-time data rebuilding.

## Priority Blockers

- `TD-032` (`Completed`): Add real end-to-end schedule tests through the full `CardGamePlugin` schedule, including multi-frame input sequences and zone transitions.
- `TD-004` (`Completed`): Add cached mesh storage so shapes are tessellated once on change and reused by render systems.
- `TD-031` (`Completed`): Make silent hardware failures visible with tracing or `Result`-based APIs.
- `TD-001/002/003` (`Completed`): Add change detection to transform propagation, hierarchy maintenance, and visibility propagation.
- `TD-005` (`Completed`): Implement the GPU-side material pipeline in `WgpuRenderer`, including textures and uniforms.

## Engine Gaps After Unification

- `TD-018` (`Open`): Add physics interpolation so rendering can smooth between fixed physics steps.
- `TD-015` (`Open`): Add a color grading post-process pass.
- `TD-010` (`Open`): Add hot reload support for assets.
- `TD-007` (`Open`): Integrate shader composition support beyond the current `#ifdef` preprocessor.
- `TD-008` (`Open`): Add a CPU rasterization path for vector work and build-time image generation.
- `TD-009` (`Open`): Add procedural texture generation support.
- `TD-012` (`Open`): Add a particle system.
- `TD-013` (`Open`): Add a tilemap system.
- `TD-014` (`Open`): Add a proper animation system with state machines and spritesheet support.
- `TD-017` (`Open`): Add procedural texture composition.
- `TD-021` (`Open`): Improve public API documentation coverage.
- `TD-022` (`Open`): Add doctests for public behavior.
- `TD-023` (`Open`): Add `docs/llms.txt`.
- `TD-025` (`Open`): Add focused examples.
- `TD-027` (`Open`): Add `.cargo/config.toml` for local build tuning.
- `TD-028` (`Open`): Add missing feature flags such as `dev`, `hot_reload`, and `debug_draw`.
- `TD-030` (`Open`): Add gamepad support.

## Card Identity and Visuals

- `I7d` (`In progress`): Finish vector card art selection and integration from `img-to-shape`.
- `I9` (`Open`): Add card inspection mode for richer UI and detail views.
- `I10` (`Open`): Add deck slots as physical consumption zones.
- `I11` (`Open`): Add a game session state machine.
- `I12` (`Open`): Add cards-as-seeds world generation.
- `I13` (`Open`): Add turn-based combat.
- `I14` (`Open`): Add signature-only serialization.
- `I15` (`Open`): Enforce card physics sleep behavior.
- `I16` (`Open`): Add drop preview indicators for landing targets.
- `I17` (`Open`): Add a card highlight system.
- `I18` (`Open`): Add batched card spawning.

## Game Loop and Persistence

- `I22` (`Open`): Add auto-save.
- `I23` (`Open`): Add generation progress UI.
- `I24` (`Open`): Add pause system support.

## World Generation

- `I19` (`Open`): Add WFC soft modifiers.
- `I20` (`Open`): Add biome distribution preview.
- `I21` (`Open`): Add fog of war and line-of-sight.
- `I25` (`Open`): Add the tilemap grid system.
- `I26` (`Open`): Add tile definitions and a tile registry.
- `I27` (`Open`): Add dual-grid auto-tiling.
- `I28` (`Open`): Add biome definitions and affinity matching.
- `I29` (`Open`): Add the WFC tile solver.
- `I19a` (`Open`): Add the spatial coherence constraint for WFC.
- `I19b` (`Open`): Add the no-solid-fill constraint for WFC.
- `I28a` (`Open`): Add the biome strength precomputation grid.

## Devices and Simulation

- `I30` (`Open`): Add jack and cable infrastructure.
- `I31` (`Open`): Add card slot devices and signature chaining.
- `I32` (`Open`): Add screen and button devices.
- `I33` (`Open`): Add a conveyor belt transport system.

## Stretch Goals

- `I34` (`Open`): Add irregular quad mesh generation.
- `I35` (`Open`): Add structure placement on maps.
- `I36` (`Open`): Add enemy spawning and management.

## Deferred By Design

- Gamepad support (`gilrs`) is deferred until keyboard and mouse stop covering the current control surface.
- Hot reloading is deferred because restart-based iteration is still acceptable at current scale.
- The examples directory is deferred because the demo crate currently serves as the minimal reference.
