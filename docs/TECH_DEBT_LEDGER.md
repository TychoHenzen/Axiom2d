# Tech Debt Ledger

**Project:** Axiom2d
**Total files tracked:** 390 (234 prod, 156 test) — excludes 29 codegen files in `art/generated/`, `card_back.rs`, `repository.rs`, and 4 `.superpowers/` files
**Total files scanned:** 390
**Average combined score:** 6.55 (structural: 6.55 + semantic: 0.00)
**Production avg:** 5.72 | **Test avg:** 7.79
**Total combined score:** 2554.08
**Last run:** 2026-04-02

## Excluded Paths

These paths are excluded from scanning (codegen output, not hand-written):

- `crates/card_game/src/card/art/generated/*.rs` (29 files — img-to-shape codegen)
- `crates/card_game/src/card/art/card_back.rs` (img-to-shape codegen)
- `crates/card_game/src/card/art/repository.rs` (img-to-shape codegen, `@generated` marker)
- `.superpowers/` (brainstorm artifacts)

## Summary

| File Path | Type | Structural | Semantic | Combined | Top Issue | Last Reviewed | Trend |
|-----------|------|-----------|----------|----------|-----------|---------------|-------|
| tools/img-to-shape/tests/suite/core_lib.rs | test | 116.00 | 0 | 116.00 | duplicate_blocks (173) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_render.rs | test | 65.30 | 0 | 65.30 | duplicate_blocks (120) | 2026-04-02 | — |
| crates/axiom2d/src/splash/render.rs | prod | 57.70 | 0 | 57.70 | magic_literals (320 geometry coords) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_interaction_pick.rs | test | 57.56 | 0 | 57.56 | duplicate_blocks (103) | 2026-04-02 | — |
| crates/engine_render/tests/suite/sprite.rs | test | 53.69 | 0 | 53.69 | duplicate_blocks (103) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_rendering_spawn_table_card.rs | test | 52.21 | 0 | 52.21 | duplicate_blocks (85) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_reader.rs | test | 49.43 | 0 | 49.43 | duplicate_blocks (77) | 2026-04-02 | — |
| tools/living-docs/tests/lib.rs | test | 46.62 | 0 | 46.62 | duplicate_blocks (60) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_interaction_release.rs | test | 45.02 | 0 | 45.02 | duplicate_blocks (45) | 2026-04-02 | — |
| crates/engine_render/src/shape/render.rs | prod | 44.95 | 0 | 44.95 | duplicate_blocks (72) | 2026-04-02 | — |
| crates/card_game/tests/suite/hand_layout.rs | test | 43.59 | 0 | 43.59 | duplicate_blocks (53) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shape_render.rs | test | 42.19 | 0 | 42.19 | duplicate_blocks (71) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shape_tessellate.rs | test | 38.52 | 0 | 38.52 | duplicate_blocks (53) | 2026-04-02 | — |
| crates/card_game/src/card/identity/name_pools/adjectives.rs | prod | 33.61 | 0 | 33.61 | max_function_length (861 — data table, false positive) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_signature.rs | test | 31.80 | 0 | 31.80 | magic_literals (248) | 2026-04-02 | — |
| crates/axiom2d/src/splash/animation.rs | prod | 30.39 | 0 | 30.39 | duplicate_blocks (33) | 2026-04-02 | — |
| tools/img-to-shape/src/codegen.rs | prod | 25.89 | 0 | 25.89 | duplicate_blocks; max_function_length | 2026-04-02 | — |
| tools/img-to-shape-gui/src/main.rs | prod | 25.72 | 0 | 25.72 | duplicate_blocks (27), max_nesting_depth (12) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/interaction.rs | test | 25.64 | 0 | 25.64 | duplicate_blocks (33) | 2026-04-02 | — |
| crates/card_game/tests/suite/integration_tests.rs | test | 25.33 | 0 | 25.33 | duplicate_blocks (34) | 2026-04-02 | — |
| crates/card_game/src/card/identity/base_type.rs | prod | 25.15 | 0 | 25.15 | duplicate_blocks (36) | 2026-04-02 | — |
| crates/engine_render/src/shape/path.rs | prod | 23.58 | 0 | 23.58 | magic_literals (112) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_interaction_drag.rs | test | 23.49 | 0 | 23.49 | duplicate_blocks (31) | 2026-04-02 | — |
| crates/engine_audio/src/spatial.rs | prod | 23.28 | 0 | 23.28 | duplicate_blocks (23) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_boundary.rs | test | 23.09 | 0 | 23.09 | duplicate_blocks (26) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/rapier_backend.rs | test | 22.94 | 0 | 22.94 | duplicate_blocks (22) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/flip_animation.rs | prod | 22.37 | 0 | 22.37 | duplicate_blocks (38) | 2026-04-02 | — |
| crates/engine_render/src/testing/visual_regression.rs | prod | 22.37 | 0 | 22.37 | magic_literals, long_param_methods | 2026-04-02 | — |
| tools/img-to-shape/src/lib.rs | prod | 22.34 | 0 | 22.34 | duplicate_blocks (12) | 2026-04-02 | — |
| crates/engine_render/tests/suite/atlas.rs | test | 22.25 | 0 | 22.25 | magic_literals (189) | 2026-04-02 | — |
| crates/engine_scene/src/transform_propagation.rs | prod | 22.07 | 0 | 22.07 | duplicate_blocks (26) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_interaction_flip_animation.rs | test | 21.46 | 0 | 21.46 | duplicate_blocks (38) | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/types.rs | prod | 21.35 | 0 | 21.35 | magic_literals (113) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_layout.rs | test | 21.06 | 0 | 21.06 | duplicate_blocks (33) | 2026-04-02 | — |
| crates/engine_core/src/scale_spring.rs | prod | 20.67 | 0 | 20.67 | duplicate_blocks (23) | 2026-04-02 | — |
| crates/axiom2d/tests/suite/default_plugins.rs | test | 19.64 | 0 | 19.64 | duplicate_blocks (37) | 2026-04-02 | — |
| crates/engine_ui/src/widget/progress_bar.rs | prod | 19.53 | 0 | 19.53 | duplicate_blocks (19) | 2026-04-02 | — |
| crates/axiom2d/tests/suite/splash_animation.rs | test | 19.49 | 0 | 19.49 | duplicate_blocks (32) | 2026-04-02 | — |
| crates/axiom2d/src/splash/letters.rs | prod | 18.72 | 0 | 18.72 | magic_literals (188) | 2026-04-02 | — |
| crates/engine_render/tests/suite/camera.rs | test | 18.44 | 0 | 18.44 | magic_literals (137) | 2026-04-02 | — |
| crates/engine_render/tests/suite/testing_visual_regression.rs | test | 18.41 | 0 | 18.41 | magic_literals (177) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_card_name.rs | test | 18.02 | 0 | 18.02 | magic_literals (88) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_signature_profile.rs | test | 17.96 | 0 | 17.96 | magic_literals (105) | 2026-04-02 | — |
| crates/engine_audio/src/playback/system.rs | prod | 17.14 | 0 | 17.14 | duplicate_blocks (24) | 2026-04-02 | — |
| crates/engine_render/src/shape/geometry.rs | prod | 16.50 | 0 | 16.50 | magic_literals (85) | 2026-04-02 | — |
| crates/engine_physics/src/physics_backend.rs | prod | 16.41 | 0 | 16.41 | max_function_length (288 — trait definition) | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/renderer_trait.rs | prod | 15.77 | 0 | 15.77 | duplicate_blocks (12) | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/gpu_init.rs | prod | 15.74 | 0 | 15.74 | duplicate_blocks (18) | 2026-04-02 | — |
| tools/img-to-shape/src/bezier_fit.rs | prod | 15.65 | 0 | 15.65 | magic_literals (62) | 2026-04-02 | — |
| tools/img-to-shape-gui/src/preview.rs | prod | 15.59 | 0 | 15.59 | duplicate_blocks (19) | 2026-04-02 | — |
| crates/engine_ui/src/widget/panel.rs | prod | 15.34 | 0 | 15.34 | duplicate_blocks (19) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_residual.rs | test | 14.53 | 0 | 14.53 | magic_literals (88) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_interaction_camera_drag.rs | test | 14.18 | 0 | 14.18 | duplicate_blocks (20) | 2026-04-02 | — |
| crates/demo/src/scene.rs | prod | 14.14 | 0 | 14.14 | magic_literals | 2026-04-02 | — |
| crates/engine_app/tests/suite/app.rs | test | 13.87 | 0 | 13.87 | duplicate_blocks (21) | 2026-04-02 | — |
| crates/engine_render/src/shape/tessellate.rs | prod | 13.84 | 0 | 13.84 | duplicate_blocks (18) | 2026-04-02 | — |
| crates/engine_render/src/font.rs | prod | 13.33 | 0.3 | 13.63 | long_param_methods (10) | 2026-04-02 | — |
| crates/demo/src/systems.rs | prod | 13.60 | 0 | 13.60 | duplicate_blocks (17) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_pages.rs | test | 13.02 | 0 | 13.02 | magic_literals (85) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_gem_sockets.rs | test | 12.83 | 0 | 12.83 | magic_literals (71) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_visual_params.rs | test | 12.36 | 0 | 12.36 | magic_literals (68) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_hover.rs | test | 12.10 | 0 | 12.10 | duplicate_blocks (16) | 2026-04-02 | — |
| crates/engine_ui/src/widget/button.rs | prod | 12.07 | 0 | 12.07 | duplicate_blocks (18) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/unified_render.rs | test | 11.78 | 0 | 11.78 | duplicate_blocks (18) | 2026-04-02 | — |
| tools/img-to-shape/src/boundary_graph.rs | prod | 11.62 | 0 | 11.62 | magic_literals (36) | 2026-04-02 | — |
| crates/engine_ui/src/layout/flex.rs | prod | 11.32 | 0 | 11.32 | magic_literals (48) | 2026-04-02 | — |
| crates/engine_ui/src/layout/system.rs | prod | 11.21 | 0 | 11.21 | duplicate_blocks (11) | 2026-04-02 | — |
| crates/engine_render/src/testing/mod.rs | prod | 10.97 | 0 | 10.97 | long_param_methods (7) | 2026-04-02 | — |
| crates/engine_render/tests/suite/bloom.rs | test | 10.84 | 0 | 10.84 | duplicate_blocks (12) | 2026-04-02 | — |
| tools/img-to-shape/src/segment.rs | prod | 10.74 | 0 | 10.74 | magic_literals (74) | 2026-04-02 | — |
| tools/img-to-shape/src/simplify.rs | prod | 10.73 | 0 | 10.73 | magic_literals (80) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/damping.rs | prod | 10.72 | 0 | 10.72 | duplicate_blocks (15) | 2026-04-02 | — |
| crates/engine_ui/src/unified_render.rs | prod | 10.70 | 0 | 10.70 | duplicate_blocks | 2026-04-02 | — |
| crates/card_game/src/card/identity/name_pools/compound_parts.rs | prod | 10.41 | 0 | 10.41 | max_function_length (183 — data table, false positive) | 2026-04-02 | — |
| crates/engine_ui/src/render.rs | prod | 10.23 | 0 | 10.23 | duplicate_blocks (13) | 2026-04-02 | — |
| crates/engine_render/tests/suite/font.rs | test | 10.16 | 0 | 10.16 | magic_literals (50) | 2026-04-02 | — |
| crates/axiom2d/tests/suite/splash_render.rs | test | 9.95 | 0 | 9.95 | duplicate_blocks (9) | 2026-04-02 | — |
| tools/img-to-shape/src/scale2x.rs | prod | 9.94 | 0 | 9.94 | magic_literals (59) | 2026-04-02 | — |
| tools/living-docs/src/lib.rs | prod | 9.89 | 0 | 9.89 | duplicate_blocks | 2026-04-02 | — |
| crates/engine_render/src/shape/components.rs | prod | 9.80 | 0 | 9.80 | magic_literals (42) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/codegen.rs | test | 9.77 | 0 | 9.77 | duplicate_blocks (11) | 2026-04-02 | — |
| crates/card_game/src/card/identity/gem_sockets.rs | prod | 9.75 | 0 | 9.75 | magic_literals | 2026-04-02 | — |
| crates/engine_scene/src/visibility.rs | prod | 9.67 | 0 | 9.67 | duplicate_blocks (14) | 2026-04-02 | — |
| crates/engine_audio/src/mixer_engine.rs | prod | 9.22 | 0 | 9.22 | magic_literals (42) | 2026-04-02 | — |
| crates/engine_input/src/mouse/state.rs | prod | 9.17 | 0 | 9.17 | max_function_length (258) | 2026-04-02 | — |
| crates/card_game/src/card/art_selection.rs | prod | 9.12 | 0 | 9.12 | magic_literals (66) | 2026-04-02 | — |
| crates/engine_render/src/culling.rs | prod | 9.10 | 0 | 9.10 | magic_literals (67) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_rendering_bake.rs | test | 8.92 | 0 | 8.92 | duplicate_blocks (9) | 2026-04-02 | — |
| tools/img-to-shape-gui/src/state.rs | prod | 8.83 | 0 | 8.83 | magic_literals (31) | 2026-04-02 | — |
| crates/engine_physics/src/physics_sync_system.rs | prod | 8.61 | 0 | 8.61 | magic_literals (38) | 2026-04-02 | — |
| crates/demo/tests/suite/scene.rs | test | 8.56 | 0 | 8.56 | duplicate_blocks (13) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/mixer_engine.rs | test | 8.56 | 0 | 8.56 | magic_literals (42) | 2026-04-02 | — |
| tools/img-to-shape/src/manifest.rs | prod | 8.37 | 0 | 8.37 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_grid.rs | test | 8.31 | 0 | 8.31 | magic_literals (69) | 2026-04-02 | — |
| crates/engine_core/src/time.rs | prod | 8.27 | 0 | 8.27 | magic_literals (36) | 2026-04-02 | — |
| crates/engine_ui/src/text_render.rs | prod | 7.98 | 0 | 7.98 | duplicate_blocks (9) | 2026-04-02 | — |
| crates/engine_assets/src/asset_server.rs | prod | 7.97 | 0 | 7.97 | max_function_length (229) | 2026-04-02 | — |
| crates/engine_physics/src/rapier_backend.rs | prod | 7.95 | 0 | 7.95 | long_param_methods (5) | 2026-04-02 | — |
| crates/engine_render/src/renderer.rs | prod | 7.91 | 0 | 7.91 | long_param_methods (6) | 2026-04-02 | — |
| crates/engine_input/src/keyboard/system.rs | prod | 7.64 | 0 | 7.64 | duplicate_blocks (12) | 2026-04-02 | — |
| crates/engine_core/src/event_bus.rs | prod | 7.62 | 0 | 7.62 | max_function_length (173) | 2026-04-02 | — |
| crates/engine_audio/src/backend/cpal.rs | prod | 7.55 | 0 | 7.55 | max_function_length (176) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_reader_glow.rs | test | 7.42 | 0 | 7.42 | duplicate_blocks (11) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_interaction_flip.rs | test | 7.32 | 0 | 7.32 | duplicate_blocks (10) | 2026-04-02 | — |
| crates/engine_app/src/app.rs | prod | 7.27 | 0 | 7.27 | long_param_methods (4) | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/bloom.rs | prod | 7.19 | 0 | 7.19 | long_param_methods (8) | 2026-04-02 | — |
| crates/engine_core/src/transform.rs | prod | 7.07 | 0 | 7.07 | magic_literals (34) | 2026-04-02 | — |
| crates/card_game/src/card/reader/spawn.rs | prod | 6.75 | 0.3 | 7.05 | magic_literals (34) | 2026-04-02 | — |
| crates/engine_render/src/shader.rs | prod | 6.85 | 0 | 6.85 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/engine_scene/src/hierarchy.rs | prod | 6.84 | 0 | 6.84 | duplicate_blocks (7) | 2026-04-02 | — |
| crates/card_game/src/card/identity/definition.rs | prod | 6.75 | 0 | 6.75 | magic_literals | 2026-04-02 | — |
| crates/card_game/tests/suite/card_rendering_baked_render.rs | test | 6.69 | 0 | 6.69 | duplicate_blocks (8) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/bake.rs | prod | 6.37 | 0.3 | 6.67 | max_nesting_depth (4); 8 dead code lines | 2026-04-02 | — |
| crates/card_game/src/card/interaction/release/target.rs | prod | 6.63 | 0 | 6.63 | max_function_length (90) | 2026-04-02 | — |
| crates/engine_app/src/mouse_world_pos_system.rs | prod | 6.44 | 0 | 6.44 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_rendering_art_shader.rs | test | 6.14 | 0 | 6.14 | max_function_length (160) | 2026-04-02 | — |
| crates/card_game/src/card/identity/name_pools/templates.rs | prod | 6.00 | 0 | 6.00 | magic_literals (22) | 2026-04-02 | — |
| crates/engine_render/src/atlas.rs | prod | 6.00 | 0 | 6.00 | long_param_methods (4) | 2026-04-02 | — |
| crates/demo/tests/suite/core_main.rs | test | 5.63 | 0 | 5.63 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/engine_core/tests/suite/spring.rs | test | 5.63 | 0 | 5.63 | magic_literals (39) | 2026-04-02 | — |
| crates/engine_ui/src/layout/anchor.rs | prod | 5.49 | 0 | 5.49 | magic_literals (37) | 2026-04-02 | — |
| crates/axiom2d/src/splash/types.rs | prod | 5.47 | 0 | 5.47 | duplicate_blocks (8) | 2026-04-02 | — |
| crates/card_game/src/test_helpers.rs | test | 5.31 | 0 | 5.31 | long_param_methods (6) | 2026-04-02 | — |
| crates/engine_render/src/shape/cache.rs | prod | 5.26 | 0 | 5.26 | duplicate_blocks (3) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/spawn_table_card/overlay.rs | prod | 5.14 | 0 | 5.14 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/card_game/src/card/identity/card_description.rs | prod | 5.09 | 0 | 5.09 | magic_literals (22) | 2026-04-02 | — |
| crates/card_game/src/stash/hover.rs | prod | 4.70 | 0.3 | 5.00 | max_function_length | 2026-04-02 | — |
| crates/axiom2d/src/default_plugins.rs | prod | 5.00 | 0 | 5.00 | duplicate_blocks | 2026-04-02 | — |
| crates/engine_ui/src/theme.rs | prod | 4.99 | 0 | 4.99 | magic_literals (37) | 2026-04-02 | — |
| crates/engine_render/src/material.rs | prod | 4.84 | 0 | 4.84 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/card_game/src/card/identity/signature/types.rs | prod | 4.71 | 0 | 4.71 | magic_literals (15) | 2026-04-02 | — |
| crates/card_game/src/card/art/hydrate.rs | prod | 4.71 | 0 | 4.71 | max_nesting_depth (7) | 2026-04-02 | — |
| crates/card_game/src/hand/layout.rs | prod | 4.60 | 0 | 4.60 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/engine_input/src/mouse/system.rs | prod | 4.53 | 0 | 4.53 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/engine_core/src/types.rs | prod | 4.49 | 0 | 4.49 | magic_literals (19) | 2026-04-02 | — |
| crates/engine_physics/src/physics_step_system.rs | prod | 4.47 | 0 | 4.47 | duplicate_blocks (4) | 2026-04-02 | — |
| crates/card_game_bin/src/card_data.rs | prod | 4.43 | 0 | 4.43 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/engine_physics/src/lib.rs | prod | 4.39 | 0 | 4.39 | long_param_methods (5) | 2026-04-02 | — |
| crates/engine_physics/src/hit_test.rs | prod | 4.22 | 0 | 4.22 | magic_literals (23) | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/renderer.rs | prod | 4.20 | 0 | 4.20 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/card_game/src/stash/pages.rs | prod | 4.17 | 0 | 4.17 | magic_literals | 2026-04-02 | — |
| crates/engine_core/tests/suite/color.rs | test | 4.14 | 0 | 4.14 | magic_literals (13) | 2026-04-02 | — |
| crates/card_game/src/stash/grid.rs | prod | 4.08 | 0 | 4.08 | max_nesting_depth (5); long_param_methods (3) | 2026-04-02 | — |
| crates/engine_scene/src/sort_propagation.rs | prod | 4.02 | 0 | 4.02 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/engine_render/src/rect.rs | prod | 3.98 | 0 | 3.98 | magic_literals (15) | 2026-04-02 | — |
| crates/card_game/src/stash/boundary.rs | prod | 3.95 | 0 | 3.95 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/tests/suite/hand_cards.rs | test | 3.90 | 0 | 3.90 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/engine_render/src/camera.rs | prod | 3.89 | 0 | 3.89 | max_nesting_depth | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_name_pools_compound_parts.rs | test | 3.88 | 0 | 3.88 | duplicate_blocks (3) | 2026-04-02 | — |
| crates/card_game/src/card/identity/card_name.rs | prod | 3.85 | 0 | 3.85 | magic_literals | 2026-04-02 | — |
| crates/card_game/src/card/interaction/pick/apply.rs | prod | 3.82 | 0 | 3.82 | duplicate_blocks (5) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_definition.rs | test | 3.82 | 0 | 3.82 | magic_literals (24) | 2026-04-02 | — |
| crates/card_game/src/card/identity/visual_params.rs | prod | 3.80 | 0 | 3.80 | magic_literals | 2026-04-02 | — |
| crates/card_game/src/card/reader/eject.rs | prod | 3.73 | 0 | 3.73 | max_function_length (41) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/render_layer.rs | prod | 3.70 | 0 | 3.70 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/face_layout.rs | prod | 3.62 | 0 | 3.62 | magic_literals (19) | 2026-04-02 | — |
| crates/card_game_bin/src/main.rs | prod | 3.60 | 0 | 3.60 | max_nesting_depth (5); magic_literals (14) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/flip.rs | prod | 3.60 | 0 | 3.60 | max_nesting_depth (4) | 2026-04-02 | — |
| tools/living-docs/src/main.rs | prod | 3.59 | 0 | 3.59 | long_param_methods (2) | 2026-04-02 | — |
| crates/engine_input/src/keyboard/state.rs | prod | 3.50 | 0 | 3.50 | duplicate_blocks (4) | 2026-04-02 | — |
| crates/card_game/src/card/zone_config.rs | prod | 3.48 | 0 | 3.48 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/engine_audio/src/sound/data.rs | prod | 3.37 | 0 | 3.37 | magic_literals (13) | 2026-04-02 | — |
| crates/engine_physics/src/collider.rs | prod | 3.37 | 0 | 3.37 | max_nesting_depth (4) | 2026-04-02 | — |
| tools/img-to-shape-gui/src/loader.rs | prod | 3.25 | 0 | 3.25 | max_function_length (62) | 2026-04-02 | — |
| crates/card_game/src/card/reader/pick.rs | prod | 3.24 | 0 | 3.24 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/engine_ui/src/widget/node.rs | prod | 3.21 | 0 | 3.21 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/baked_render.rs | prod | 3.20 | 0 | 3.20 | duplicate_blocks | 2026-04-02 | — |
| crates/card_game/tests/suite/card_identity_name_pools_syllables.rs | test | 3.16 | 0 | 3.16 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/release/apply.rs | prod | 3.08 | 0 | 3.08 | duplicate_blocks (3) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/art_shader.rs | prod | 2.73 | 0.3 | 3.03 | max_nesting_depth | 2026-04-02 | — |
| crates/engine_ui/src/interaction.rs | prod | 3.02 | 0 | 3.02 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/reader/glow.rs | prod | 3.01 | 0 | 3.01 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/pick.rs | prod | 3.01 | 0 | 3.01 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/physics_helpers.rs | prod | 3.00 | 0 | 3.00 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/spawn_table_card.rs | prod | 2.99 | 0 | 2.99 | duplicate_blocks | 2026-04-02 | — |
| crates/engine_render/src/sprite.rs | prod | 2.96 | 0 | 2.96 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/stash/render/slots.rs | prod | 2.92 | 0 | 2.92 | max_nesting_depth (6) | 2026-04-02 | — |
| crates/card_game/src/plugin.rs | prod | 2.87 | 0 | 2.87 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/debug_spawn.rs | prod | 2.82 | 0 | 2.82 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_core/src/color.rs | prod | 2.81 | 0 | 2.81 | long_param_methods (2) | 2026-04-02 | — |
| crates/card_game/src/card/reader/drag.rs | prod | 2.80 | 0 | 2.80 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/card_game/src/stash/layout.rs | prod | 2.73 | 0 | 2.73 | duplicate_blocks | 2026-04-02 | — |
| crates/card_game/src/card/interaction/drag.rs | prod | 2.73 | 0 | 2.73 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_render/tests/suite/wgpu_renderer_renderer.rs | test | 2.72 | 0 | 2.72 | magic_literals (25) | 2026-04-02 | — |
| crates/engine_render/src/clear.rs | prod | 2.61 | 0 | 2.61 | max_function_length (34) | 2026-04-02 | — |
| crates/engine_render/src/image_data.rs | prod | 2.61 | 0 | 2.61 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/camera_drag.rs | prod | 2.53 | 0 | 2.53 | max_nesting_depth | 2026-04-02 | — |
| crates/card_game/src/card/identity/signature_profile.rs | prod | 2.50 | 0 | 2.50 | magic_literals | 2026-04-02 | — |
| crates/engine_audio/src/sound/effect.rs | prod | 2.41 | 0 | 2.41 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/stash/render.rs | prod | 2.38 | 0 | 2.38 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/spawn_table_card/text.rs | prod | 2.36 | 0 | 2.36 | cyclomatic_density (18.6/100) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_rendering_drop_zone_glow.rs | test | 2.34 | 0 | 2.34 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_reader_spawn.rs | test | 2.33 | 0 | 2.33 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/card_game/src/stash/render/drag_preview.rs | prod | 2.31 | 0 | 2.31 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/engine_input/src/key_code.rs | prod | 2.29 | 0 | 2.29 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/card_game/src/card/reader/insert.rs | prod | 2.28 | 0 | 2.28 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/identity/signature/algorithms.rs | prod | 2.27 | 0 | 2.27 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/pick/source.rs | prod | 2.13 | 0 | 2.13 | max_nesting_depth (5) | 2026-04-02 | — |
| crates/engine_audio/src/mixer.rs | prod | 2.13 | 0 | 2.13 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/card/art/mod.rs | prod | 2.10 | 0 | 2.10 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/engine_assets/src/handle.rs | prod | 2.10 | 0 | 2.10 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_audio/src/sound/library.rs | prod | 2.09 | 0 | 2.09 | max_function_length (48) | 2026-04-02 | — |
| crates/card_game/tests/suite/plugin.rs | test | 2.09 | 0 | 2.09 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_rendering_debug_spawn.rs | test | 2.06 | 0 | 2.06 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/identity/residual.rs | prod | 1.95 | 0 | 1.95 | magic_literals | 2026-04-02 | — |
| crates/engine_render/src/bloom.rs | prod | 1.93 | 0 | 1.93 | magic_literals | 2026-04-02 | — |
| crates/card_game/src/card/rendering/drop_zone_glow.rs | prod | 1.91 | 0 | 1.91 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/card_game/src/card/identity/name_pools/nouns.rs | prod | 1.89 | 0 | 1.89 | file_length (318 — data table) | 2026-04-02 | — |
| crates/demo/src/lib.rs | prod | 1.85 | 0 | 1.85 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/pick/hit_test.rs | prod | 1.83 | 0 | 1.83 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/geometry.rs | prod | 1.81 | 0 | 1.81 | magic_literals (9) | 2026-04-02 | — |
| crates/engine_scene/src/render_order.rs | prod | 1.79 | 0 | 1.79 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/engine_scene/src/spawn_child.rs | prod | 1.78 | 0 | 1.78 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_audio/src/playback/buffer.rs | prod | 1.73 | 0 | 1.73 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_input/src/button_state.rs | prod | 1.71 | 0 | 1.71 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/card/reader/rotation_lock.rs | prod | 1.65 | 0 | 1.65 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_input/src/action_map.rs | prod | 1.63 | 0 | 1.63 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/stash/toggle.rs | prod | 1.61 | 0 | 1.61 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_audio/src/backend/traits.rs | prod | 1.61 | 0 | 1.61 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_ecs/src/schedule.rs | prod | 1.57 | 0 | 1.57 | max_nesting_depth (4) | 2026-04-02 | — |
| crates/card_game/src/hand/cards.rs | prod | 1.55 | 0 | 1.55 | file_length | 2026-04-02 | — |
| crates/card_game/src/card/identity/name_pools/syllables.rs | prod | 1.53 | 0 | 1.53 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/card_game/src/stash/constants.rs | prod | 1.44 | 0 | 1.44 | magic_literals (13) | 2026-04-02 | — |
| crates/card_game/src/card/component.rs | prod | 1.42 | 0 | 1.42 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_input/src/mouse_button.rs | prod | 1.42 | 0 | 1.42 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_audio/src/audio_res.rs | prod | 1.39 | 0 | 1.39 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_physics/src/physics_res.rs | prod | 1.39 | 0 | 1.39 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_render/src/window.rs | prod | 1.38 | 0 | 1.38 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_ui/src/widget/text.rs | prod | 1.34 | 0 | 1.34 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_core/src/spring.rs | prod | 1.33 | 0 | 1.33 | long_param_methods (1) | 2026-04-02 | — |
| crates/engine_app/src/window_size.rs | prod | 1.32 | 0 | 1.32 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_physics/src/rigid_body.rs | prod | 1.26 | 0 | 1.26 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/card_game/src/card/identity/name_pools/mod.rs | prod | 1.18 | 0 | 1.18 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/demo/src/types.rs | prod | 1.08 | 0 | 1.08 | duplicate_blocks (1) | 2026-04-02 | — |
| crates/engine_scene/src/lib.rs | prod | 1.07 | 0 | 1.07 | max_nesting_depth (3) | 2026-04-02 | — |
| crates/engine_render/src/testing/helpers.rs | prod | 1.04 | 0 | 1.04 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_input/src/keyboard/buffer.rs | prod | 0.94 | 0 | 0.94 | cyclomatic_density | 2026-04-02 | — |
| crates/engine_input/src/mouse/buffer.rs | prod | 0.94 | 0 | 0.94 | cyclomatic_density | 2026-04-02 | — |
| crates/card_game/src/stash/render/models.rs | prod | 0.84 | 0 | 0.84 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_ui/src/ui_event.rs | prod | 0.83 | 0 | 0.83 | cyclomatic_density | 2026-04-02 | — |
| crates/engine_ui/src/lib.rs | prod | 0.78 | 0 | 0.78 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_physics/src/collision_event.rs | prod | 0.74 | 0 | 0.74 | cyclomatic_density | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/shaders.rs | prod | 0.66 | 0 | 0.66 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/engine_audio/src/test_helpers.rs | test | 0.64 | 0 | 0.64 | max_nesting_depth (2) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/baked_mesh.rs | prod | 0.62 | 0 | 0.62 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/demo/src/main.rs | prod | 0.54 | 0 | 0.54 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/src/card/reader/components.rs | prod | 0.53 | 0 | 0.53 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/src/prelude.rs | prod | 0.49 | 0 | 0.49 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/game_state_param.rs | prod | 0.48 | 0 | 0.48 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_render/src/lib.rs | prod | 0.46 | 0 | 0.46 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/drag_state.rs | prod | 0.40 | 0 | 0.40 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_render/src/prelude.rs | prod | 0.38 | 0 | 0.38 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/axiom2d/src/prelude.rs | prod | 0.36 | 0 | 0.36 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/src/card/reader.rs | prod | 0.36 | 0 | 0.36 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_render/src/shape/mod.rs | prod | 0.36 | 0 | 0.36 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_core/src/error.rs | prod | 0.34 | 0 | 0.34 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_render/src/wgpu_renderer/mod.rs | prod | 0.34 | 0 | 0.34 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_ui/src/layout/margin.rs | prod | 0.34 | 0 | 0.34 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_ui/src/prelude.rs | prod | 0.34 | 0 | 0.34 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/src/stash/render/helpers.rs | prod | 0.33 | 0 | 0.33 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/axiom2d/src/splash/mod.rs | prod | 0.33 | 0 | 0.33 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_audio/src/prelude.rs | prod | 0.33 | 0 | 0.33 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_ecs/src/prelude.rs | prod | 0.32 | 0 | 0.32 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/engine_input/src/prelude.rs | prod | 0.32 | 0 | 0.32 | max_nesting_depth (1) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_art_selection.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/card_game/tests/suite/card_zone_config.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/card_game/tests/suite/stash_toggle.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_app/tests/suite/mouse_world_pos_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_assets/tests/asset_server.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/backend_cpal.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/backend_traits.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/mixer.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/playback_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/sound_data.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/sound_effect.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/sound_library.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/spatial.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_core/tests/suite/event_bus.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_core/tests/suite/scale_spring.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_core/tests/suite/time.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_core/tests/suite/transform.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_core/tests/suite/types.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ecs/tests/schedule.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_input/tests/suite/action_map.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_input/tests/suite/keyboard_state.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_input/tests/suite/keyboard_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_input/tests/suite/mouse_state.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_input/tests/suite/mouse_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/collider.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/hit_test.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/physics_backend.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/physics_step_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/physics_sync_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/rigid_body.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/clear.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/culling.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/image_data.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/material.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/rect.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/renderer.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shader.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shape_cache.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shape_components.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shape_geometry.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/shape_path.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/testing_mod.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/wgpu_renderer_gpu_init.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/wgpu_renderer_shaders.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_render/tests/suite/wgpu_renderer_types.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/hierarchy.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/render_order.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/sort_propagation.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/spawn_child.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/transform_propagation.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/visibility.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/layout_anchor.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/layout_flex.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/layout_system.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/render.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/text_render.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/theme.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/widget_button.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/widget_node.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/widget_panel.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/widget_progress_bar.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape-gui/tests/suite/loader.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape-gui/tests/suite/preview.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape-gui/tests/suite/state.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/bezier_fit.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/boundary_graph.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/manifest.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/scale2x.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/segment.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/simplify.rs | test | 0.30 | 0 | 0.30 | smell_markers (1) | 2026-04-02 | — |
| crates/card_game/tests/suite/mod.rs | test | 0.11 | 0 | 0.11 | file_length (37) | 2026-04-02 | — |
| crates/engine_render/tests/suite/mod.rs | test | 0.07 | 0 | 0.07 | file_length (24) | 2026-04-02 | — |
| crates/engine_ui/tests/suite/mod.rs | test | 0.04 | 0 | 0.04 | file_length (12) | 2026-04-02 | — |
| crates/card_game/src/card/identity/mod.rs | prod | 0.03 | 0 | 0.03 | file_length (10) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/mod.rs | prod | 0.03 | 0 | 0.03 | file_length (10) | 2026-04-02 | — |
| crates/card_game/src/card/mod.rs | prod | 0.03 | 0 | 0.03 | file_length (9) | 2026-04-02 | — |
| crates/card_game/src/card/rendering/mod.rs | prod | 0.03 | 0 | 0.03 | file_length (10) | 2026-04-02 | — |
| crates/engine_audio/src/lib.rs | prod | 0.03 | 0 | 0.03 | file_length (10) | 2026-04-02 | — |
| crates/engine_core/src/lib.rs | prod | 0.03 | 0 | 0.03 | file_length (9) | 2026-04-02 | — |
| crates/engine_core/src/prelude.rs | prod | 0.03 | 0 | 0.03 | file_length (9) | 2026-04-02 | — |
| crates/engine_ui/src/widget/mod.rs | prod | 0.03 | 0 | 0.03 | file_length (11) | 2026-04-02 | — |
| crates/engine_audio/tests/suite/mod.rs | test | 0.03 | 0 | 0.03 | file_length (9) | 2026-04-02 | — |
| crates/card_game/src/lib.rs | prod | 0.02 | 0 | 0.02 | file_length (6) | 2026-04-02 | — |
| crates/card_game/src/stash/mod.rs | prod | 0.02 | 0 | 0.02 | file_length (8) | 2026-04-02 | — |
| crates/engine_audio/src/playback/mod.rs | prod | 0.02 | 0 | 0.02 | file_length (6) | 2026-04-02 | — |
| crates/engine_audio/src/sound/mod.rs | prod | 0.02 | 0 | 0.02 | file_length (6) | 2026-04-02 | — |
| crates/engine_input/src/keyboard/mod.rs | prod | 0.02 | 0 | 0.02 | file_length (6) | 2026-04-02 | — |
| crates/engine_input/src/lib.rs | prod | 0.02 | 0 | 0.02 | file_length (7) | 2026-04-02 | — |
| crates/engine_input/src/mouse/mod.rs | prod | 0.02 | 0 | 0.02 | file_length (7) | 2026-04-02 | — |
| crates/engine_physics/src/prelude.rs | prod | 0.02 | 0 | 0.02 | file_length (8) | 2026-04-02 | — |
| crates/engine_scene/src/prelude.rs | prod | 0.02 | 0 | 0.02 | file_length (6) | 2026-04-02 | — |
| crates/engine_ui/src/layout/mod.rs | prod | 0.02 | 0 | 0.02 | file_length (8) | 2026-04-02 | — |
| crates/engine_core/tests/suite/mod.rs | test | 0.02 | 0 | 0.02 | file_length (7) | 2026-04-02 | — |
| crates/engine_physics/tests/suite/mod.rs | test | 0.02 | 0 | 0.02 | file_length (7) | 2026-04-02 | — |
| crates/engine_scene/tests/suite/mod.rs | test | 0.02 | 0 | 0.02 | file_length (6) | 2026-04-02 | — |
| tools/img-to-shape/tests/suite/mod.rs | test | 0.02 | 0 | 0.02 | file_length (8) | 2026-04-02 | — |
| crates/axiom2d/src/lib.rs | prod | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| crates/card_game/src/card/identity/signature.rs | prod | 0.01 | 0 | 0.01 | file_length (4) | 2026-04-02 | — |
| crates/card_game/src/card/interaction/release.rs | prod | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| crates/card_game/src/hand/mod.rs | prod | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| crates/engine_app/src/lib.rs | prod | 0.01 | 0 | 0.01 | file_length (4) | 2026-04-02 | — |
| crates/engine_app/src/prelude.rs | prod | 0.01 | 0 | 0.01 | file_length (4) | 2026-04-02 | — |
| crates/engine_assets/src/lib.rs | prod | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| crates/engine_assets/src/prelude.rs | prod | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_audio/src/backend/mod.rs | prod | 0.01 | 0 | 0.01 | file_length (4) | 2026-04-02 | — |
| crates/engine_audio/src/playback/id.rs | prod | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_ecs/src/lib.rs | prod | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| tools/img-to-shape-gui/src/lib.rs | prod | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| crates/axiom2d/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/axiom2d/tests/suite/mod.rs | test | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| crates/card_game/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/demo/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/demo/tests/suite/mod.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_app/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_app/tests/suite/mod.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_audio/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_core/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_input/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_input/tests/suite/mod.rs | test | 0.01 | 0 | 0.01 | file_length (5) | 2026-04-02 | — |
| crates/engine_physics/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_render/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_scene/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| crates/engine_ui/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| tools/img-to-shape-gui/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |
| tools/img-to-shape-gui/tests/suite/mod.rs | test | 0.01 | 0 | 0.01 | file_length (3) | 2026-04-02 | — |
| tools/img-to-shape/tests/main.rs | test | 0.01 | 0 | 0.01 | file_length (2) | 2026-04-02 | — |

**Structural** = deterministic score from the script. **Semantic** = LLM review of bugs in meaning. **Combined** = Structural + Semantic (sorted worst first).

## Semantic Findings

### Note
- **crates/engine_render/src/font.rs:80** — `font_size.round() as u16` silently truncates font sizes >= 65536; no runtime impact at current game scales but technically lossy
- **crates/card_game/src/card/rendering/art_shader.rs:82**, **crates/card_game/src/stash/hover.rs:113** — uniform byte offsets `[8..12]`, `[12..16]` hardcoded to match `ArtRegionParams` struct layout; will silently break if struct fields are reordered
- **crates/card_game/src/card/rendering/bake.rs:151** — `card_back::card_front2()` function name is misleading when used for back-face art (named `card_front2` but called in `bake_back_face`)

## Splits Performed

Test code extracted from production files during prior runs.

| Source File | Extracted | New Test File | Status |
|------------|-----------|---------------|--------|
| tools/living-docs/src/lib.rs | 137 tests | tools/living-docs/tests/lib.rs | Merged |
| tools/img-to-shape/src/codegen.rs | 29 tests | tools/img-to-shape/tests/codegen.rs | Merged |
| crates/engine_render/src/testing/visual_regression.rs | 21 tests | crates/engine_render/tests/testing_visual_regression.rs | Merged |
| crates/engine_render/src/camera.rs | 19 tests | crates/engine_render/tests/camera.rs | Merged |
| crates/engine_ui/src/unified_render.rs | 6 tests | crates/engine_ui/tests/unified_render.rs | Merged |
| crates/axiom2d/src/default_plugins.rs | 20 tests | crates/axiom2d/tests/default_plugins.rs | Merged |
| crates/card_game/src/stash/layout.rs | 7 tests | crates/card_game/tests/stash_layout.rs | Merged |
| crates/card_game/src/card/rendering/spawn_table_card.rs | 33 tests | crates/card_game/tests/card_rendering_spawn_table_card.rs | Created |
| crates/card_game/src/card/identity/visual_params.rs | 23 tests | crates/card_game/tests/card_identity_visual_params.rs | Created |
| crates/card_game/src/card/identity/gem_sockets.rs | 34 tests | crates/card_game/tests/card_identity_gem_sockets.rs | Created |
| crates/card_game/src/card/identity/signature_profile.rs | 34 tests | crates/card_game/tests/card_identity_signature_profile.rs | Created |
| crates/card_game/src/card/identity/card_name.rs | 127 tests | crates/card_game/tests/card_identity_card_name.rs | Created |
| crates/card_game/src/card/identity/residual.rs | 17 tests | crates/card_game/tests/card_identity_residual.rs | Created |
| crates/card_game/src/card/identity/definition.rs | 11 tests | crates/card_game/tests/card_identity_definition.rs | Created |
| crates/demo/src/scene.rs | 26 tests | crates/demo/tests/scene.rs | Created |
| crates/demo/src/main.rs | 5 tests | crates/demo/tests/main.rs | Created |
| crates/engine_render/src/bloom.rs | 8 tests | crates/engine_render/tests/suite/bloom.rs | Merged |
| crates/card_game/src/hand/cards.rs | 10 tests | crates/card_game/tests/suite/hand_cards.rs | Merged |
| crates/card_game/src/card/rendering/baked_render.rs | 7 tests | crates/card_game/tests/suite/card_rendering_baked_render.rs | Created |
| crates/card_game/src/card/interaction/camera_drag.rs | 16 tests | crates/card_game/tests/suite/card_interaction_camera_drag.rs | Created |
| crates/card_game/src/card/rendering/art_shader.rs | 28 tests | crates/card_game/tests/suite/card_rendering_art_shader.rs | Created |
| crates/card_game/src/stash/hover.rs | 13 tests | crates/card_game/tests/suite/stash_hover.rs | Merged |
| crates/card_game/src/stash/pages.rs | 21 tests | crates/card_game/tests/suite/stash_pages.rs | Merged |
| crates/engine_audio/src/mixer_engine.rs | 8 tests | crates/engine_audio/tests/suite/mixer_engine.rs | Merged |
| crates/card_game/src/stash/boundary.rs | 9 tests | crates/card_game/tests/suite/stash_boundary.rs | Created |
| crates/card_game/src/card/interaction/flip.rs | 9 tests | crates/card_game/tests/suite/card_interaction_flip.rs | Created |
| crates/card_game/src/card/rendering/drop_zone_glow.rs | 3 tests | crates/card_game/tests/suite/card_rendering_drop_zone_glow.rs | Created |
| crates/engine_core/src/spring.rs | 5 tests | crates/engine_core/tests/suite/spring.rs | Merged |
| crates/engine_core/src/color.rs | 4 tests | crates/engine_core/tests/suite/color.rs | Merged |
| crates/card_game/src/card/reader/glow.rs | 5 tests | crates/card_game/tests/suite/card_reader_glow.rs | Created |
| crates/card_game/src/card/identity/name_pools/compound_parts.rs | 5 tests | crates/card_game/tests/suite/card_identity_name_pools_compound_parts.rs | Created |
| crates/card_game/src/card/rendering/debug_spawn.rs | 5 tests | crates/card_game/tests/suite/card_rendering_debug_spawn.rs | Created |
| crates/card_game/src/card/identity/name_pools/syllables.rs | 5 tests | crates/card_game/tests/suite/card_identity_name_pools_syllables.rs | Created |

### Pending Splits

(none — all previously blocked files resolved)

## Metric False Positives

- **adjectives.rs** (33.61): function_length inflated — file is a pure data table (`match` returning `&[&str]`), not a complex function
- **compound_parts.rs** (10.41): max_function_length 183 — `match` returning static string arrays, pure data table like adjectives.rs
- **nouns.rs** (1.89): file_length 318 — pure data table of string arrays, no logic
- **hand/cards.rs** (1.55): max_function_length 164 — script counts `impl` block as a function, not a real complexity issue
- **reader/spawn.rs** (6.75): magic_literals 34 — most are named constants, not unnamed magic numbers
- **face_layout.rs** (3.62): magic_literals 19 — card face layout constants, not unnamed magic numbers
- **geometry.rs** (1.81): magic_literals 9 — named geometry constants, not unnamed magic numbers
- **physics_backend.rs** (16.41): max_function_length 288 — trait definition with many methods, not a single long function
- **base_type.rs** (25.15): duplicate_blocks 36 — `BaseCardType` builder methods follow identical patterns, not copy-paste
- **splash/letters.rs** (18.72): magic_literals 188 — polygon vertex coordinates for vector letter rendering
- **scale_spring.rs** (20.67): duplicate_blocks 23 — spring physics equations reuse similar stepping patterns
- **spatial.rs** (23.28): duplicate_blocks 23 — ECS system queries follow identical patterns, not extractable
- **transform_propagation.rs** (22.07): duplicate_blocks 26 — recursive tree traversal with necessarily similar structure

## Unscanned Files

(none — full scan complete)

## Removed Files

(none)

## Run Log

| Date | Files Analyzed | Scores |
|------|---------------|--------|
| 2026-04-01 | card_back.rs (848.65), splash/render.rs (57.70), spawn_table_card.rs (55.99), splash/animation.rs (30.39), gem_sockets.rs (24.03), default_plugins.rs (23.99), flip_animation.rs (22.37), signature_profile.rs (21.32), card_name.rs (19.99), residual.rs (17.89) | batch avg: 112.29 (28.12 excl. outlier) |
| 2026-04-01 | shape/render.rs (44.95), repository.rs (30.41→excluded), visual_params.rs (22.63), stash/layout.rs (22.51), camera.rs (21.15), unified_render.rs (19.88), tessellate.rs (13.84), font.rs (13.63), definition.rs (9.29), rapier_backend.rs (7.95) | batch avg: 19.50 (9 scored) |
| 2026-04-01 | img-to-shape/lib.rs (133.04), sprite.rs (54.95), living-docs/lib.rs (52.71), codegen.rs (44.46), visual_regression.rs (39.01), adjectives.rs (33.61), interaction.rs (27.12), img-to-shape-gui/main.rs (25.72), wgpu_renderer/types.rs (21.35), scene.rs (21.34) | batch avg: 45.33 |
| 2026-04-01 | **Test migration**: img-to-shape/lib.rs 133.04→22.34, sprite.rs 54.95→2.96, interaction.rs 27.12→3.02 | total debt reduced by 186.15 |
| 2026-04-01 | **Test migration batch 2**: 15 files migrated. | total debt reduced by 326.94 |
| 2026-04-02 | hover.rs (19.34), pages.rs (15.27), camera_drag.rs (15.04), bloom.rs (10.16), mixer_engine.rs (9.22), baked_render.rs (8.20), cards.rs (7.78), app.rs (7.27), reader/spawn.rs (7.05), art_shader.rs (4.86) | batch avg: 10.42 |
| 2026-04-02 | **Test migration batch 3**: 7 files migrated. | total debt reduced by 59.24 |
| 2026-04-02 | boundary.rs (3.95), flip.rs (3.60), bake.rs (6.67), plugin.rs (2.87), color.rs (2.81), drop_zone_glow.rs (1.91), pick/apply.rs (3.82), stash/render.rs (2.38), nouns.rs (1.89), spring.rs (1.33) | batch avg: 3.12 |
| 2026-04-02 | **Test migration batch 4–5**: 9 files migrated. | total debt reduced by 38.79 |
| 2026-04-02 | compound_parts.rs (10.41), glow.rs (3.01), signature/types.rs (4.71), card_data.rs (4.43), debug_spawn.rs (2.82), release/target.rs (6.63), main.rs (3.60), syllables.rs (1.53), release/apply.rs (3.08), grid.rs (4.08) | batch avg: 4.43 |
| 2026-04-02 | drag.rs (2.73), eject.rs (3.73), face_layout.rs (3.62), slots.rs (2.92), drag_preview.rs (2.31), insert.rs (2.28), algorithms.rs (2.27), hit_test.rs (1.83), geometry.rs (1.81), helpers.rs (0.33) | batch avg: 2.18 |
| 2026-04-02 | **Full scan**: 322 remaining files (166 prod, 156 test). Highest new: core_lib.rs test (116.00), base_type.rs (25.15), spatial.rs (23.28), shape/path.rs (23.58), scale_spring.rs (20.67). No new semantic findings. | batch avg: 6.16 |

