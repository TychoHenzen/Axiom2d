# Streamline Checklist — 2026-03-20

Baseline: 1,319 tests passing, 0 failures.

## Batch 1 — Trivial

- [x] **#1** Remove `card_game/src/viewport_camera.rs` (1-line re-export shim). Update imports to use `engine_render::prelude::resolve_viewport_camera` directly.
- [x] **#2** Inline `card_game/src/scale_spring.rs` (single fn `sync_scale_spring_lock_x`) into `flip_animation.rs`. Delete the file.
- [x] **#5** Remove dead `SpyPhysicsBackend::with_collision_group_log()` builder method in `card_game/src/test_helpers.rs`.

## Batch 2 — Low Risk

- [x] **#3** Delete `engine_assets/src/scene.rs` (`SceneNodeDef`, `SceneDef`). Removed module, prelude, snapshots, and 5 unused crate deps. (-12 dead tests)
- [x] **#8** Move `axiom2d/src/splash_letters.rs` into `axiom2d/src/splash/letters.rs`. Updated module declarations.
- [x] **#17** Move card-game root-level modules (`drag_state.rs`, `flip_animation.rs`, `physics_helpers.rs`, `spawn_table_card.rs`) into `card/` submodule. Updated all imports across 8 files.

## Batch 3 — Medium Risk

- [x] **#10** Move `engine_app/src/camera_drag.rs` to `card_game/src/card/camera_drag.rs`. Updated engine_app lib/prelude and card_game mod/prelude.
- [~] **#11** ~~Merge mixer_engine.rs into mixer.rs~~ — SKIPPED: duplication is intentional (ECS thread → audio thread sync via `play_sound_system`).
- [~] **#12** ~~Extract shared render batch logic~~ — SKIPPED: each system is ~30 lines; shared part is ~10 lines. Extracting a generic would over-abstract for 2 consumers.
- [~] **#13** ~~Consolidate culling wrappers~~ — SKIPPED: culling functions differ intentionally (sprite uses width/height AABB, shape uses bounding sphere).
- [~] **#14** ~~Standardize spring pattern~~ — NOTED: `spring_step()` = pure fn for inline use; `ScaleSpring` = component+system for automatic scale animation. Convention: use ScaleSpring for any transform axis that needs spring physics, use spring_step inline only for custom multi-axis springs (like hand fan layout).

## Batch 4 — High Risk

- [~] **#7** ~~Collapse engine_ecs~~ — DEFERRED: affects every crate's Cargo.toml. Thin wrapper works fine; risk > reward.
- [~] **#9** ~~Move splash out of facade~~ — DEFERRED: changes DefaultPlugins API. Current opt-out via SkipSplash works.
- [~] **#16** ~~Consolidate UI render systems~~ — DEFERRED: 4 systems have different queries/color logic. Dispatch mechanism would be more complex than 4 small systems.
- [~] **#18** ~~Trim Renderer trait~~ — DEFERRED: stubs exist for forward compatibility with material pipeline. Needs architectural decision first.
- [~] **#20** ~~Unify UI widget rendering~~ — DEFERRED: same as #16.

## Noted (Architectural — Deferred)

- [ ] **#4** `engine_render/src/font.rs` — most functions dead (`render_text_glyphs`, `GlyphCache`, `wrap_text`). Only `measure_text()` used. Needs design decision on text pipeline ownership.
- [ ] **#6** WgpuRenderer material stubs (`set_material_uniforms`, `bind_material_texture`) — empty GPU implementations. Needs decision on whether material pipeline is trait-based or GPU-internal.
- [ ] **#15** Two sort paradigms (`RenderLayer + SortOrder` flat vs `LocalSortOrder + SORT_STRIDE` hierarchical). Needs documentation or unification.
- [ ] **#19** Missing shared render-batch abstraction. Blocked on #12 (extract shared logic first).
