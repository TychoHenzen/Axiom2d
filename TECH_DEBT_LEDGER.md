# Tech Debt Ledger

**Project:** Axiom2d
**Total files tracked:** 272
**Total files scanned:** 20
**Average badness score:** 25.88
**Total badness score:** 517.56
**Last run:** 2026-04-01

## Summary

| File Path | Badness | Top Issue | Last Reviewed | Trend |
|-----------|---------|-----------|---------------|-------|
| crates/card_game/src/stash/render.rs | 63.99 | duplicate_blocks (120 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/card/interaction/pick.rs | 59.57 | duplicate_blocks (104 duplicate blocks) | 2026-04-01 | -- |
| crates/engine_render/src/shape/tessellate.rs | 48.86 | duplicate_blocks (71 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/card/reader.rs | 46.84 | duplicate_blocks (77 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/card/interaction/release.rs | 46.39 | duplicate_blocks (45 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/hand/layout.rs | 40.49 | duplicate_blocks (54 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/card/identity/signature.rs | 34.63 | magic_literals (248 magic literals) | 2026-04-01 | -- |
| crates/card_game/src/card/interaction/drag.rs | 33.70 | duplicate_blocks (31 duplicate blocks) | 2026-04-01 | -- |
| crates/engine_physics/src/rapier_backend.rs | 33.58 | duplicate_blocks (26 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/integration_tests.rs | 26.01 | duplicate_blocks (35 duplicate blocks) | 2026-04-01 | -- |
| crates/engine_render/src/atlas.rs | 22.65 | magic_literals (202 magic literals) | 2026-04-01 | -- |
| crates/engine_render/src/font.rs | 21.65 | duplicate_blocks (15 duplicate blocks) | 2026-04-01 | -- |
| crates/engine_app/src/app.rs | 19.52 | duplicate_blocks (22 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/stash/grid.rs | 17.93 | max_function_length (longest function: 325 lines) | 2026-04-01 | -- |
| crates/card_game/src/card/rendering/bake.rs | 13.47 | duplicate_blocks (11 duplicate blocks) | 2026-04-01 | -- |
| crates/card_game/src/card/reader/spawn.rs | 7.77 | magic_literals (37 magic literals) | 2026-04-01 | -- |
| crates/engine_render/src/wgpu_renderer/renderer.rs | 6.08 | magic_literals (39 magic literals) | 2026-04-01 | -- |
| crates/card_game/src/plugin.rs | 5.15 | max_function_length (longest function: 60 lines) | 2026-04-01 | -- |
| crates/card_game/src/card/rendering/face_layout.rs | 3.62 | magic_literals (19 magic literals) | 2026-04-01 | -- |
| crates/card_game_bin/src/main.rs | 3.60 | max_nesting_depth (max nesting depth: 5) | 2026-04-01 | -- |

## Unscanned Files

### Engine Crates
- crates/axiom2d/src/default_plugins.rs
- crates/axiom2d/src/lib.rs
- crates/axiom2d/src/prelude.rs
- crates/axiom2d/src/splash/animation.rs
- crates/axiom2d/src/splash/letters.rs
- crates/axiom2d/src/splash/mod.rs
- crates/axiom2d/src/splash/render.rs
- crates/axiom2d/src/splash/types.rs
- crates/engine_app/src/lib.rs
- crates/engine_app/src/mouse_world_pos_system.rs
- crates/engine_app/src/prelude.rs
- crates/engine_app/src/window_size.rs
- crates/engine_assets/src/asset_server.rs
- crates/engine_assets/src/handle.rs
- crates/engine_assets/src/lib.rs
- crates/engine_assets/src/prelude.rs
- crates/engine_audio/src/audio_res.rs
- crates/engine_audio/src/backend/cpal.rs
- crates/engine_audio/src/backend/mod.rs
- crates/engine_audio/src/backend/traits.rs
- crates/engine_audio/src/lib.rs
- crates/engine_audio/src/mixer.rs
- crates/engine_audio/src/mixer_engine.rs
- crates/engine_audio/src/playback/buffer.rs
- crates/engine_audio/src/playback/id.rs
- crates/engine_audio/src/playback/mod.rs
- crates/engine_audio/src/playback/system.rs
- crates/engine_audio/src/prelude.rs
- crates/engine_audio/src/sound/data.rs
- crates/engine_audio/src/sound/effect.rs
- crates/engine_audio/src/sound/library.rs
- crates/engine_audio/src/sound/mod.rs
- crates/engine_audio/src/spatial.rs
- crates/engine_audio/src/test_helpers.rs
- crates/engine_core/src/color.rs
- crates/engine_core/src/error.rs
- crates/engine_core/src/event_bus.rs
- crates/engine_core/src/lib.rs
- crates/engine_core/src/prelude.rs
- crates/engine_core/src/scale_spring.rs
- crates/engine_core/src/spring.rs
- crates/engine_core/src/time.rs
- crates/engine_core/src/transform.rs
- crates/engine_core/src/types.rs
- crates/engine_ecs/src/lib.rs
- crates/engine_ecs/src/prelude.rs
- crates/engine_ecs/src/schedule.rs
- crates/engine_input/src/action_map.rs
- crates/engine_input/src/button_state.rs
- crates/engine_input/src/key_code.rs
- crates/engine_input/src/keyboard/buffer.rs
- crates/engine_input/src/keyboard/mod.rs
- crates/engine_input/src/keyboard/state.rs
- crates/engine_input/src/keyboard/system.rs
- crates/engine_input/src/lib.rs
- crates/engine_input/src/mouse/buffer.rs
- crates/engine_input/src/mouse/mod.rs
- crates/engine_input/src/mouse/state.rs
- crates/engine_input/src/mouse/system.rs
- crates/engine_input/src/mouse_button.rs
- crates/engine_input/src/prelude.rs
- crates/engine_physics/src/collider.rs
- crates/engine_physics/src/collision_event.rs
- crates/engine_physics/src/hit_test.rs
- crates/engine_physics/src/lib.rs
- crates/engine_physics/src/physics_backend.rs
- crates/engine_physics/src/physics_res.rs
- crates/engine_physics/src/physics_step_system.rs
- crates/engine_physics/src/physics_sync_system.rs
- crates/engine_physics/src/prelude.rs
- crates/engine_physics/src/rigid_body.rs
- crates/engine_render/src/bloom.rs
- crates/engine_render/src/camera.rs
- crates/engine_render/src/clear.rs
- crates/engine_render/src/culling.rs
- crates/engine_render/src/image_data.rs
- crates/engine_render/src/lib.rs
- crates/engine_render/src/material.rs
- crates/engine_render/src/prelude.rs
- crates/engine_render/src/rect.rs
- crates/engine_render/src/renderer.rs
- crates/engine_render/src/shader.rs
- crates/engine_render/src/shape/cache.rs
- crates/engine_render/src/shape/components.rs
- crates/engine_render/src/shape/geometry.rs
- crates/engine_render/src/shape/mod.rs
- crates/engine_render/src/shape/path.rs
- crates/engine_render/src/shape/render.rs
- crates/engine_render/src/sprite.rs
- crates/engine_render/src/testing/helpers.rs
- crates/engine_render/src/testing/mod.rs
- crates/engine_render/src/testing/visual_regression.rs
- crates/engine_render/src/wgpu_renderer/bloom.rs
- crates/engine_render/src/wgpu_renderer/gpu_init.rs
- crates/engine_render/src/wgpu_renderer/mod.rs
- crates/engine_render/src/wgpu_renderer/renderer_trait.rs
- crates/engine_render/src/wgpu_renderer/shaders.rs
- crates/engine_render/src/wgpu_renderer/types.rs
- crates/engine_render/src/window.rs
- crates/engine_scene/src/hierarchy.rs
- crates/engine_scene/src/lib.rs
- crates/engine_scene/src/prelude.rs
- crates/engine_scene/src/render_order.rs
- crates/engine_scene/src/sort_propagation.rs
- crates/engine_scene/src/spawn_child.rs
- crates/engine_scene/src/transform_propagation.rs
- crates/engine_scene/src/visibility.rs
- crates/engine_ui/src/interaction.rs
- crates/engine_ui/src/layout/anchor.rs
- crates/engine_ui/src/layout/flex.rs
- crates/engine_ui/src/layout/margin.rs
- crates/engine_ui/src/layout/mod.rs
- crates/engine_ui/src/layout/system.rs
- crates/engine_ui/src/lib.rs
- crates/engine_ui/src/prelude.rs
- crates/engine_ui/src/render.rs
- crates/engine_ui/src/text_render.rs
- crates/engine_ui/src/theme.rs
- crates/engine_ui/src/ui_event.rs
- crates/engine_ui/src/unified_render.rs
- crates/engine_ui/src/widget/button.rs
- crates/engine_ui/src/widget/mod.rs
- crates/engine_ui/src/widget/node.rs
- crates/engine_ui/src/widget/panel.rs
- crates/engine_ui/src/widget/progress_bar.rs
- crates/engine_ui/src/widget/text.rs

### Card Game
- crates/card_game/src/card/art/card_back.rs
- crates/card_game/src/card/art/hydrate.rs
- crates/card_game/src/card/art/mod.rs
- crates/card_game/src/card/art/repository.rs
- crates/card_game/src/card/art_selection.rs
- crates/card_game/src/card/component.rs
- crates/card_game/src/card/identity/base_type.rs
- crates/card_game/src/card/identity/card_description.rs
- crates/card_game/src/card/identity/card_name.rs
- crates/card_game/src/card/identity/definition.rs
- crates/card_game/src/card/identity/gem_sockets.rs
- crates/card_game/src/card/identity/mod.rs
- crates/card_game/src/card/identity/name_pools/adjectives.rs
- crates/card_game/src/card/identity/name_pools/compound_parts.rs
- crates/card_game/src/card/identity/name_pools/mod.rs
- crates/card_game/src/card/identity/name_pools/nouns.rs
- crates/card_game/src/card/identity/name_pools/syllables.rs
- crates/card_game/src/card/identity/name_pools/templates.rs
- crates/card_game/src/card/identity/residual.rs
- crates/card_game/src/card/identity/signature/algorithms.rs
- crates/card_game/src/card/identity/signature/types.rs
- crates/card_game/src/card/identity/signature_profile.rs
- crates/card_game/src/card/identity/visual_params.rs
- crates/card_game/src/card/interaction/camera_drag.rs
- crates/card_game/src/card/interaction/damping.rs
- crates/card_game/src/card/interaction/drag_state.rs
- crates/card_game/src/card/interaction/flip.rs
- crates/card_game/src/card/interaction/flip_animation.rs
- crates/card_game/src/card/interaction/game_state_param.rs
- crates/card_game/src/card/interaction/mod.rs
- crates/card_game/src/card/interaction/physics_helpers.rs
- crates/card_game/src/card/interaction/pick/apply.rs
- crates/card_game/src/card/interaction/pick/hit_test.rs
- crates/card_game/src/card/interaction/pick/source.rs
- crates/card_game/src/card/interaction/release/apply.rs
- crates/card_game/src/card/interaction/release/target.rs
- crates/card_game/src/card/mod.rs
- crates/card_game/src/card/reader/components.rs
- crates/card_game/src/card/reader/drag.rs
- crates/card_game/src/card/reader/eject.rs
- crates/card_game/src/card/reader/glow.rs
- crates/card_game/src/card/reader/insert.rs
- crates/card_game/src/card/reader/pick.rs
- crates/card_game/src/card/reader/rotation_lock.rs
- crates/card_game/src/card/rendering/art_shader.rs
- crates/card_game/src/card/rendering/baked_mesh.rs
- crates/card_game/src/card/rendering/baked_render.rs
- crates/card_game/src/card/rendering/debug_spawn.rs
- crates/card_game/src/card/rendering/drop_zone_glow.rs
- crates/card_game/src/card/rendering/geometry.rs
- crates/card_game/src/card/rendering/mod.rs
- crates/card_game/src/card/rendering/render_layer.rs
- crates/card_game/src/card/rendering/spawn_table_card.rs
- crates/card_game/src/card/rendering/spawn_table_card/overlay.rs
- crates/card_game/src/card/rendering/spawn_table_card/text.rs
- crates/card_game/src/card/zone_config.rs
- crates/card_game/src/hand/cards.rs
- crates/card_game/src/hand/layout.rs
- crates/card_game/src/hand/mod.rs
- crates/card_game/src/lib.rs
- crates/card_game/src/prelude.rs
- crates/card_game/src/stash/boundary.rs
- crates/card_game/src/stash/constants.rs
- crates/card_game/src/stash/hover.rs
- crates/card_game/src/stash/layout.rs
- crates/card_game/src/stash/mod.rs
- crates/card_game/src/stash/pages.rs
- crates/card_game/src/stash/render/drag_preview.rs
- crates/card_game/src/stash/render/helpers.rs
- crates/card_game/src/stash/render/models.rs
- crates/card_game/src/stash/render/slots.rs
- crates/card_game/src/stash/toggle.rs
- crates/card_game/src/test_helpers.rs
- crates/card_game_bin/src/card_data.rs

### Generated Art (codegen output - excluded from analysis)
- crates/card_game/src/card/art/generated/*.rs (30 files)

### Demo
- crates/demo/src/main.rs
- crates/demo/src/scene.rs
- crates/demo/src/systems.rs
- crates/demo/src/types.rs

### Tools
- tools/img-to-shape/src/bezier_fit.rs
- tools/img-to-shape/src/boundary_graph.rs
- tools/img-to-shape/src/codegen.rs
- tools/img-to-shape/src/lib.rs
- tools/img-to-shape/src/manifest.rs
- tools/img-to-shape/src/scale2x.rs
- tools/img-to-shape/src/segment.rs
- tools/img-to-shape/src/simplify.rs
- tools/img-to-shape-gui/src/lib.rs
- tools/img-to-shape-gui/src/loader.rs
- tools/img-to-shape-gui/src/main.rs
- tools/img-to-shape-gui/src/preview.rs
- tools/img-to-shape-gui/src/state.rs
- tools/living-docs/src/lib.rs
- tools/living-docs/src/main.rs

## Removed Files

(none)

## Run Log

| Date | Files Analyzed | Scores |
|------|---------------|--------|
| 2026-04-01 | signature.rs (34.63), drag.rs (33.70), rapier_backend.rs (33.58), grid.rs (17.93), bake.rs (13.47), spawn.rs (7.77), renderer.rs (6.08), plugin.rs (5.15), face_layout.rs (3.62), main.rs (3.60) | batch avg: 15.95 |
| 2026-04-01 | render.rs (63.99), pick.rs (59.57), tessellate.rs (48.86), reader.rs (46.84), release.rs (46.39), layout.rs (40.49), font.rs (21.65), atlas.rs (22.65), app.rs (19.52), integration_tests.rs (26.01) | batch avg: 39.60 |
