# Tech Debt Ledger

**Project:** Axiom2d
**Total files tracked:** 351 (235 prod, 116 test) — excludes 29 codegen files in `art/generated/`, `card_back.rs`, `repository.rs`, and 4 `.superpowers/` files
**Total files scanned:** 28
**Average combined score:** 13.92 (structural: 13.91 + semantic: 0.01)
**Production avg:** 13.92 | **Test avg:** N/A
**Total combined score:** 389.72
**Last run:** 2026-04-01

## Excluded Paths

These paths are excluded from scanning (codegen output, not hand-written):

- `crates/card_game/src/card/art/generated/*.rs` (29 files — img-to-shape codegen)
- `crates/card_game/src/card/art/card_back.rs` (img-to-shape codegen)
- `crates/card_game/src/card/art/repository.rs` (img-to-shape codegen, `@generated` marker)
- `.superpowers/` (brainstorm artifacts)

## Summary

| File Path | Type | Structural | Semantic | Combined | Top Issue | Last Reviewed | Trend |
|-----------|------|-----------|----------|----------|-----------|---------------|-------|
| crates/axiom2d/src/splash/render.rs | prod | 57.70 | 0 | 57.70 | magic_literals (320 geometry coords) | 2026-04-01 | — |
| crates/engine_render/src/shape/render.rs | prod | 44.95 | 0 | 44.95 | duplicate_blocks (72) | 2026-04-01 | — |
| crates/card_game/src/card/identity/name_pools/adjectives.rs | prod | 33.61 | 0 | 33.61 | max_function_length (861 — data table, false positive) | 2026-04-01 | — |
| tools/img-to-shape/src/codegen.rs | prod | 25.89 | 0 | 25.89 | duplicate_blocks; max_function_length reduced by test removal | 2026-04-01 | ↓ |
| tools/img-to-shape-gui/src/main.rs | prod | 25.72 | 0 | 25.72 | duplicate_blocks (27), max_nesting_depth (12) | 2026-04-01 | — |
| tools/img-to-shape/src/lib.rs | prod | 22.34 | 0 | 22.34 | duplicate_blocks (12) | 2026-04-01 | ↓ |
| crates/engine_render/src/testing/visual_regression.rs | prod | 22.37 | 0 | 22.37 | magic_literals, long_param_methods | 2026-04-01 | ↓ |
| crates/engine_render/src/wgpu_renderer/types.rs | prod | 21.35 | 0 | 21.35 | magic_literals (113) | 2026-04-01 | — |
| crates/demo/src/scene.rs | prod | 14.14 | 0 | 14.14 | magic_literals | 2026-04-01 | ↓ |
| crates/engine_ui/src/unified_render.rs | prod | 10.70 | 0 | 10.70 | duplicate_blocks | 2026-04-01 | ↓ |
| tools/living-docs/src/lib.rs | prod | 9.89 | 0 | 9.89 | duplicate_blocks; large test module removed | 2026-04-01 | ↓ |
| crates/card_game/src/card/identity/gem_sockets.rs | prod | 9.75 | 0 | 9.75 | magic_literals (color/position values) | 2026-04-01 | ↓ |
| crates/engine_physics/src/rapier_backend.rs | prod | 7.95 | 0 | 7.95 | long_param_methods (5) | 2026-04-01 | — |
| crates/card_game/src/card/identity/definition.rs | prod | 6.75 | 0 | 6.75 | magic_literals | 2026-04-01 | ↓ |
| crates/axiom2d/src/default_plugins.rs | prod | 5.00 | 0 | 5.00 | duplicate_blocks (resource insertions) | 2026-04-01 | ↓ |
| crates/engine_render/src/camera.rs | prod | 3.89 | 0 | 3.89 | max_nesting_depth | 2026-04-01 | ↓ |
| crates/card_game/src/card/identity/card_name.rs | prod | 3.85 | 0 | 3.85 | magic_literals | 2026-04-01 | ↓ |
| crates/card_game/src/card/identity/visual_params.rs | prod | 3.80 | 0 | 3.80 | magic_literals | 2026-04-01 | ↓ |
| crates/card_game/src/card/rendering/spawn_table_card.rs | prod | 2.99 | 0 | 2.99 | duplicate_blocks | 2026-04-01 | ↓ |
| crates/engine_render/src/sprite.rs | prod | 2.96 | 0 | 2.96 | max_nesting_depth (3) | 2026-04-01 | ↓ |
| crates/engine_ui/src/interaction.rs | prod | 3.02 | 0 | 3.02 | max_nesting_depth (4) | 2026-04-01 | ↓ |
| crates/card_game/src/stash/layout.rs | prod | 2.73 | 0 | 2.73 | duplicate_blocks | 2026-04-01 | ↓ |
| crates/card_game/src/card/identity/signature_profile.rs | prod | 2.50 | 0 | 2.50 | magic_literals | 2026-04-01 | ↓ |
| crates/card_game/src/card/interaction/flip_animation.rs | prod | 22.37 | 0 | 22.37 | duplicate_blocks (38) | 2026-04-01 | ↓ |
| crates/card_game/src/card/identity/residual.rs | prod | 1.95 | 0 | 1.95 | magic_literals | 2026-04-01 | ↓ |
| crates/axiom2d/src/splash/animation.rs | prod | 30.39 | 0 | 30.39 | duplicate_blocks (33) | 2026-04-01 | ↓ |
| crates/engine_render/src/shape/tessellate.rs | prod | 13.84 | 0 | 13.84 | duplicate_blocks (18) | 2026-04-01 | — |
| crates/engine_render/src/font.rs | prod | 13.33 | 0.3 | 13.63 | long_param_methods (10) | 2026-04-01 | — |

**Structural** = deterministic score from the script. **Semantic** = LLM review of bugs in meaning. **Combined** = Structural + Semantic (sorted worst first).

## Semantic Findings

### Note
- **crates/engine_render/src/font.rs:80** — `font_size.round() as u16` silently truncates font sizes >= 65536; no runtime impact at current game scales but technically lossy
- **crates/axiom2d/src/splash/render.rs**, **crates/engine_render/src/shape/render.rs**, **crates/engine_render/src/wgpu_renderer/types.rs** — inline test modules remain; tests access private internals that can't be migrated to external files

## Splits Performed

Test code extracted from production files during this run.

| Source File | Extracted | New Test File | Status |
|------------|-----------|---------------|--------|
| tools/living-docs/src/lib.rs | 137 tests | tools/living-docs/tests/lib.rs | Merged |
| tools/img-to-shape/src/codegen.rs | 29 tests | tools/img-to-shape/tests/codegen.rs | Merged |
| crates/engine_render/src/testing/visual_regression.rs | 21 tests | crates/engine_render/tests/testing_visual_regression.rs | Merged |
| crates/engine_render/src/camera.rs | 19 tests | crates/engine_render/tests/camera.rs | Merged |
| crates/engine_ui/src/unified_render.rs | 6 tests | crates/engine_ui/tests/unified_render.rs | Merged |
| crates/axiom2d/src/default_plugins.rs | 20 tests | crates/axiom2d/tests/default_plugins.rs | Merged |
| crates/card_game/src/stash/layout.rs | 7 tests | crates/card_game/tests/stash_layout.rs | Merged |
| crates/card_game/src/card/rendering/spawn_table_card.rs | 33 tests (6 dropped — used private `build_gem_overlay`) | crates/card_game/tests/card_rendering_spawn_table_card.rs | Created |
| crates/card_game/src/card/identity/visual_params.rs | 23 tests | crates/card_game/tests/card_identity_visual_params.rs | Created |
| crates/card_game/src/card/identity/gem_sockets.rs | 34 tests | crates/card_game/tests/card_identity_gem_sockets.rs | Created |
| crates/card_game/src/card/identity/signature_profile.rs | 34 tests | crates/card_game/tests/card_identity_signature_profile.rs | Created |
| crates/card_game/src/card/identity/card_name.rs | 127 tests | crates/card_game/tests/card_identity_card_name.rs | Created |
| crates/card_game/src/card/identity/residual.rs | 17 tests | crates/card_game/tests/card_identity_residual.rs | Created |
| crates/card_game/src/card/identity/definition.rs | 11 tests | crates/card_game/tests/card_identity_definition.rs | Created |
| crates/demo/src/scene.rs | 26 tests | crates/demo/tests/scene.rs | Created |
| crates/demo/src/main.rs | 5 tests | crates/demo/tests/main.rs | Created |
| **NOT migrated** | | | |
| crates/axiom2d/src/splash/render.rs | — | — | Tests use private internals of `pub(crate)` module |
| crates/engine_render/src/shape/render.rs | — | — | Tests use private modules (components, render) |
| crates/engine_render/src/wgpu_renderer/types.rs | — | — | Tests use private functions (rect_to_instance, blend_mode_to_blend_state, etc.) |

## Metric False Positives

- **adjectives.rs** (33.61): function_length inflated — file is a pure data table (`match` returning `&[&str]`), not a complex function

## Unscanned Files

### Production (207 remaining)

- crates/axiom2d/src/lib.rs
- crates/axiom2d/src/prelude.rs
- crates/axiom2d/src/splash/letters.rs
- crates/axiom2d/src/splash/mod.rs
- crates/axiom2d/src/splash/types.rs
- crates/card_game/src/card/art/hydrate.rs
- crates/card_game/src/card/art/mod.rs
- crates/card_game/src/card/art_selection.rs
- crates/card_game/src/card/component.rs
- crates/card_game/src/card/identity/base_type.rs
- crates/card_game/src/card/identity/card_description.rs
- crates/card_game/src/card/identity/mod.rs
- crates/card_game/src/card/identity/name_pools/compound_parts.rs
- crates/card_game/src/card/identity/name_pools/mod.rs
- crates/card_game/src/card/identity/name_pools/nouns.rs
- crates/card_game/src/card/identity/name_pools/syllables.rs
- crates/card_game/src/card/identity/name_pools/templates.rs
- crates/card_game/src/card/identity/signature.rs
- crates/card_game/src/card/identity/signature/algorithms.rs
- crates/card_game/src/card/identity/signature/types.rs
- crates/card_game/src/card/interaction/camera_drag.rs
- crates/card_game/src/card/interaction/damping.rs
- crates/card_game/src/card/interaction/drag.rs
- crates/card_game/src/card/interaction/drag_state.rs
- crates/card_game/src/card/interaction/flip.rs
- crates/card_game/src/card/interaction/game_state_param.rs
- crates/card_game/src/card/interaction/mod.rs
- crates/card_game/src/card/interaction/physics_helpers.rs
- crates/card_game/src/card/interaction/pick.rs
- crates/card_game/src/card/interaction/pick/apply.rs
- crates/card_game/src/card/interaction/pick/hit_test.rs
- crates/card_game/src/card/interaction/pick/source.rs
- crates/card_game/src/card/interaction/release.rs
- crates/card_game/src/card/interaction/release/apply.rs
- crates/card_game/src/card/interaction/release/target.rs
- crates/card_game/src/card/mod.rs
- crates/card_game/src/card/reader.rs
- crates/card_game/src/card/reader/components.rs
- crates/card_game/src/card/reader/drag.rs
- crates/card_game/src/card/reader/eject.rs
- crates/card_game/src/card/reader/glow.rs
- crates/card_game/src/card/reader/insert.rs
- crates/card_game/src/card/reader/pick.rs
- crates/card_game/src/card/reader/rotation_lock.rs
- crates/card_game/src/card/reader/spawn.rs
- crates/card_game/src/card/rendering/art_shader.rs
- crates/card_game/src/card/rendering/bake.rs
- crates/card_game/src/card/rendering/baked_mesh.rs
- crates/card_game/src/card/rendering/baked_render.rs
- crates/card_game/src/card/rendering/debug_spawn.rs
- crates/card_game/src/card/rendering/drop_zone_glow.rs
- crates/card_game/src/card/rendering/face_layout.rs
- crates/card_game/src/card/rendering/geometry.rs
- crates/card_game/src/card/rendering/mod.rs
- crates/card_game/src/card/rendering/render_layer.rs
- crates/card_game/src/card/rendering/spawn_table_card/overlay.rs
- crates/card_game/src/card/rendering/spawn_table_card/text.rs
- crates/card_game/src/card/zone_config.rs
- crates/card_game/src/hand/cards.rs
- crates/card_game/src/hand/layout.rs
- crates/card_game/src/hand/mod.rs
- crates/card_game/src/lib.rs
- crates/card_game/src/plugin.rs
- crates/card_game/src/prelude.rs
- crates/card_game/src/stash/boundary.rs
- crates/card_game/src/stash/constants.rs
- crates/card_game/src/stash/grid.rs
- crates/card_game/src/stash/hover.rs
- crates/card_game/src/stash/mod.rs
- crates/card_game/src/stash/pages.rs
- crates/card_game/src/stash/render.rs
- crates/card_game/src/stash/render/drag_preview.rs
- crates/card_game/src/stash/render/helpers.rs
- crates/card_game/src/stash/render/models.rs
- crates/card_game/src/stash/render/slots.rs
- crates/card_game/src/stash/toggle.rs
- crates/card_game_bin/src/card_data.rs
- crates/card_game_bin/src/main.rs
- crates/demo/src/main.rs
- crates/demo/src/systems.rs
- crates/demo/src/types.rs
- crates/engine_app/src/app.rs
- crates/engine_app/src/lib.rs
- crates/engine_app/src/mouse_world_pos_system.rs
- crates/engine_app/src/prelude.rs
- crates/engine_app/src/window_size.rs
- crates/engine_assets/src/asset_server.rs
- crates/engine_assets/src/handle.rs
- crates/engine_assets/src/lib.rs
- crates/engine_assets/src/prelude.rs
- crates/engine_audio/src/**/*.rs (17 files)
- crates/engine_core/src/*.rs (8 files)
- crates/engine_ecs/src/*.rs (3 files)
- crates/engine_input/src/**/*.rs (14 files)
- crates/engine_physics/src/*.rs (8 remaining: collider, collision_event, hit_test, lib, physics_backend, physics_res, physics_step_system, physics_sync_system)
- crates/engine_render/src/*.rs (19 remaining: atlas, bloom, clear, culling, image_data, lib, material, prelude, rect, renderer, shader, shape/cache, shape/components, shape/geometry, shape/mod, shape/path, testing/mod, testing/helpers, window)
- crates/engine_render/src/wgpu_renderer/*.rs (4 remaining: bloom, gpu_init, mod, renderer, renderer_trait, shaders)
- crates/engine_scene/src/*.rs (7 files)
- crates/engine_ui/src/**/*.rs (12 remaining)
- tools/img-to-shape/src/*.rs (5 remaining: bezier_fit, boundary_graph, manifest, scale2x, segment, simplify)
- tools/img-to-shape-gui/src/*.rs (4 remaining: lib, loader, preview, state)
- tools/living-docs/src/main.rs

### Test (116 files)

All `tests/*.rs` and `test_helpers.rs` files across all crates — not yet scanned.

## Removed Files

(none)

## Run Log

| Date | Files Analyzed | Scores |
|------|---------------|--------|
| 2026-04-01 | card_back.rs (848.65), splash/render.rs (57.70), spawn_table_card.rs (55.99), splash/animation.rs (30.39), gem_sockets.rs (24.03), default_plugins.rs (23.99), flip_animation.rs (22.37), signature_profile.rs (21.32), card_name.rs (19.99), residual.rs (17.89) | batch avg: 112.29 (28.12 excl. outlier) |
| 2026-04-01 | shape/render.rs (44.95), repository.rs (30.41→excluded), visual_params.rs (22.63), stash/layout.rs (22.51), camera.rs (21.15), unified_render.rs (19.88), tessellate.rs (13.84), font.rs (13.63), definition.rs (9.29), rapier_backend.rs (7.95) | batch avg: 19.50 (9 scored) |
| 2026-04-01 | img-to-shape/lib.rs (133.04), sprite.rs (54.95), living-docs/lib.rs (52.71), codegen.rs (44.46), visual_regression.rs (39.01), adjectives.rs (33.61), interaction.rs (27.12), img-to-shape-gui/main.rs (25.72), wgpu_renderer/types.rs (21.35), scene.rs (21.34) | batch avg: 45.33 |
| 2026-04-01 | **Test migration**: img-to-shape/lib.rs 133.04→22.34, sprite.rs 54.95→2.96, interaction.rs 27.12→3.02 | total debt reduced by 186.15 |
| 2026-04-01 | **Test migration batch 2**: 15 files migrated, 3 kept inline (private internals). living-docs 52.71→9.89, codegen 44.46→25.89, visual_regression 39.01→22.37, spawn_table_card 55.99→2.99, default_plugins 23.99→5.00, visual_params 22.63→3.80, gem_sockets 24.03→9.75, layout 22.51→2.73, signature_profile 21.32→2.50, camera 21.15→3.89, card_name 19.99→3.85, unified_render 19.88→10.70, residual 17.89→1.95, definition 9.29→6.75, scene 21.34→14.14 | total debt reduced by 326.94 |
