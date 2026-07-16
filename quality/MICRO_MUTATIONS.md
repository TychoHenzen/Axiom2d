# Micro-Mutation Tracking

Stochastic mutation testing — one random source file per daily CI run.
Selection weighted by **staleness** (50%), **file size** (30%), and **git churn** (20%).
Over weeks, covers the codebase without combinatorial explosion.

**Cumulative (all runs)**: 1375 mutants | 93 caught | 1252 missed | 0 timeout | 30 unviable | 7 zero-mutant | 0 errors | **catch rate: 6.9%** | 19 runs | 20 files tested

**Last run**: 2026-07-16 (`b867689`)

---

## File Inventory

All 256 eligible source files. Sorted by selection priority (staleness × size × churn).

| Priority | File | Lines | Churn | Stale | Last Tested | Result | Status |
|----------|------|-------|-------|-------|-------------|--------|--------|
| 94% | `crates/particle_poc/src/capture.rs` | 1288 | 8 | 90d | never | — | ⬜ |
| 91% | `crates/particle_poc/src/lib.rs` | 1632 | 6 | 90d | never | — | ⬜ |
| 89% | `crates/particle_poc/src/main.rs` | 134 | 32 | 90d | never | — | ⬜ |
| 87% | `crates/card_game/src/stash/store.rs` | 555 | 6 | 90d | never | — | ⬜ |
| 83% | `crates/card_game/src/card/screen_device.rs` | 389 | 5 | 90d | never | — | ⬜ |
| 81% | `crates/terrain_viewer/src/main.rs` | 594 | 3 | 90d | never | — | ⬜ |
| 80% | `crates/engine_render/src/testing/visual_regression.rs` | 815 | 2 | 90d | never | — | ⬜ |
| 79% | `crates/card_game/src/booster/device.rs` | 377 | 3 | 90d | never | — | ⬜ |
| 79% | `crates/engine_render/src/font.rs` | 352 | 3 | 90d | never | — | ⬜ |
| 78% | `crates/card_game/src/card/jack_socket.rs` | 303 | 3 | 90d | never | — | ⬜ |
| 78% | `crates/engine_ui/src/unified_render.rs` | 448 | 2 | 90d | never | — | ⬜ |
| 78% | `crates/card_game/src/booster/opening.rs` | 421 | 2 | 90d | never | — | ⬜ |
| 77% | `crates/card_game/src/stash/store_render.rs` | 319 | 2 | 90d | never | — | ⬜ |
| 77% | `crates/card_game/src/card/interaction/apply.rs` | 314 | 2 | 90d | never | — | ⬜ |
| 77% | `crates/card_game/src/card/identity/name_pools/adjectives.rs` | 865 | 0 | 90d | never | — | ⬜ |
| 76% | `crates/terrain/src/tile_def.rs` | 298 | 2 | 90d | never | — | ⬜ |
| 76% | `crates/card_game/src/card/jack_cable/render.rs` | 284 | 2 | 90d | never | — | ⬜ |
| 76% | `crates/engine_physics/src/lib.rs` | 245 | 2 | 90d | never | — | ⬜ |
| 75% | `crates/card_game/src/stash/pages.rs` | 232 | 2 | 90d | never | — | ⬜ |
| 75% | `crates/card_game/src/plugin.rs` | 203 | 2 | 90d | never | — | ⬜ |
| 75% | `crates/engine_app/src/app.rs` | 337 | 1 | 90d | never | — | ⬜ |
| 75% | `crates/engine_physics/src/rapier_backend.rs` | 336 | 1 | 90d | never | — | ⬜ |
| 74% | `crates/card_game/src/card/screen_geometry.rs` | 175 | 2 | 90d | never | — | ⬜ |
| 74% | `crates/engine_render/src/shape/tessellate.rs` | 275 | 1 | 90d | never | — | ⬜ |
| 74% | `crates/terrain/src/tile_def/example.rs` | 165 | 2 | 90d | never | — | ⬜ |
| 74% | `crates/engine_render/src/atlas.rs` | 163 | 2 | 90d | never | — | ⬜ |
| 74% | `crates/card_game/src/card/combiner_device.rs` | 253 | 1 | 90d | never | — | ⬜ |
| 73% | `crates/card_game/src/card/identity/name_pools/compound_parts.rs` | 354 | 0 | 90d | never | — | ⬜ |
| 73% | `crates/card_game/src/card/identity/name_pools/nouns.rs` | 319 | 0 | 90d | never | — | ⬜ |
| 73% | `crates/card_game/src/stash/grid.rs` | 113 | 2 | 90d | never | — | ⬜ |
| 73% | `crates/terrain/src/wfc.rs` | 188 | 1 | 90d | never | — | ⬜ |
| 72% | `crates/card_game/src/card/jack_cable/geom.rs` | 182 | 1 | 90d | never | — | ⬜ |
| 72% | `crates/axiom2d/src/splash/render.rs` | 279 | 0 | 90d | never | — | ⬜ |
| 72% | `crates/engine_physics/src/physics_backend.rs` | 155 | 1 | 90d | never | — | ⬜ |
| 72% | `crates/card_game/src/stash/render.rs` | 144 | 1 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/reader/spawn.rs` | 229 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/rendering/spawn_table_card/overlay.rs` | 227 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/interaction/click_resolve.rs` | 136 | 1 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/stash/hover.rs` | 135 | 1 | 90d | never | — | ⬜ |
| 71% | `crates/terrain/src/material.rs` | 130 | 1 | 90d | never | — | ⬜ |
| 71% | `crates/engine_input/src/key_code.rs` | 213 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/identity/signature/types.rs` | 204 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/interaction/release/target.rs` | 121 | 1 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/jack_cable/mod.rs` | 112 | 1 | 90d | never | — | ⬜ |
| 70% | `crates/card_game/src/card/reader/signature_space.rs` | 106 | 1 | 90d | never | — | ⬜ |
| 70% | `crates/card_game/src/card/identity/base_type.rs` | 156 | 0 | 90d | never | — | ⬜ |
| 70% | `crates/card_game/src/card/rendering/debug_sleep_indicator.rs` | 56 | 2 | 90d | never | — | ⬜ |
| 70% | `crates/axiom2d/src/splash/types.rs` | 92 | 1 | 90d | never | — | ⬜ |
| 70% | `crates/card_game/src/card/rendering/bake.rs` | 151 | 0 | 90d | never | — | ⬜ |
| 70% | `crates/engine_render/src/shape/path.rs` | 148 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/identity/definition.rs` | 141 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_ui/src/draw_command.rs` | 84 | 1 | 90d | never | — | ⬜ |
| 69% | `crates/engine_render/src/camera.rs` | 139 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/identity/gem_sockets.rs` | 138 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_render/src/renderer.rs` | 138 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/rendering/spawn_table_card.rs` | 135 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_audio/src/spatial.rs` | 80 | 1 | 90d | never | — | ⬜ |
| 69% | `crates/engine_render/benches/font.rs` | 133 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/hand/layout.rs` | 132 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_render/src/shape/components.rs` | 125 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/booster/sampling.rs` | 124 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/build.rs` | 74 | 1 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/stash/render/slots.rs` | 122 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/prelude.rs` | 73 | 1 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/art_selection.rs` | 119 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/booster/pack.rs` | 118 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_audio/src/backend/cpal.rs` | 118 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/terrain/src/dual_grid.rs` | 116 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/rendering/art_shader.rs` | 113 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/booster/double_click.rs` | 112 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_core/src/profiler.rs` | 111 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/engine_physics/src/physics_command_apply_system.rs` | 65 | 1 | 90d | never | — | ⬜ |
| 68% | `crates/engine_render/benches/tessellation.rs` | 106 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/engine_audio/src/playback/system.rs` | 63 | 1 | 90d | never | — | ⬜ |
| 68% | `crates/card_game/src/stash/render/drag_preview.rs` | 97 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/engine_core/benches/spring.rs` | 97 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/engine_physics/src/physics_command.rs` | 54 | 1 | 90d | never | — | ⬜ |
| 68% | `crates/card_game/src/card/identity/name_pools/templates.rs` | 87 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/card_game/src/hand/cards.rs` | 52 | 1 | 90d | never | — | ⬜ |
| 68% | `crates/engine_scene/src/visibility.rs` | 86 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_ui/src/interaction.rs` | 84 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/src/card/identity/residual.rs` | 77 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_ui/src/widget/panel.rs` | 76 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_render/src/shape/geometry.rs` | 75 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/src/card/rendering/face_layout.rs` | 74 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_render/benches/stress.rs` | 73 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_render/src/material.rs` | 73 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_assets/src/asset_server.rs` | 72 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_ecs/src/schedule.rs` | 72 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_audio/src/playback/buffer.rs` | 43 | 1 | 90d | never | — | ⬜ |
| 67% | `crates/engine_core/src/scale_spring.rs` | 70 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/axiom2d/src/splash/animation.rs` | 68 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/src/card/reader/volume.rs` | 68 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/src/stash/boundary.rs` | 68 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/src/card/reader/drag.rs` | 67 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_input/src/mouse/state.rs` | 67 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_physics/benches/physics.rs` | 66 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_core/benches/stress.rs` | 65 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/reader/insert.rs` | 61 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/reader/glow.rs` | 60 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_ui/src/widget/progress_bar.rs` | 59 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_render/src/culling.rs` | 58 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_scene/src/sort_propagation.rs` | 58 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_core/src/color.rs` | 57 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/interaction/sleep.rs` | 34 | 1 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/benches/bake.rs` | 56 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/identity/signature_profile.rs` | 56 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/terrain/src/prelude.rs` | 12 | 3 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/identity/visual_params.rs` | 55 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_ui/src/text_render.rs` | 55 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/interaction/drag.rs` | 54 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/reader/eject.rs` | 54 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/rendering/drop_zone_glow.rs` | 51 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/benches/stress.rs` | 50 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/art/mod.rs` | 49 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/interaction/flip.rs` | 49 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_render/src/prelude.rs` | 29 | 1 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/component.rs` | 47 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/interaction/flip_animation.rs` | 47 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_assets/src/handle.rs` | 47 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_render/src/lib.rs` | 28 | 1 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/rendering/baked_render.rs` | 46 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/rendering/spawn_table_card/text.rs` | 45 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_physics/benches/stress.rs` | 45 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_audio/src/mixer.rs` | 44 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_core/src/types.rs` | 44 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/interaction/camera_drag.rs` | 43 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/zone_config.rs` | 43 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_input/src/keyboard/state.rs` | 43 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/identity/card_description.rs` | 42 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_ui/src/widget/button.rs` | 42 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/interaction/intent.rs` | 40 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/interaction/drag_state.rs` | 24 | 1 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/stash/layout.rs` | 39 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_ui/src/layout/flex.rs` | 39 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_render/src/plugin.rs` | 23 | 1 | 90d | never | — | ⬜ |
| 64% | `crates/engine_core/src/event_bus.rs` | 37 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_scene/src/hierarchy.rs` | 37 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_ui/src/widget/node.rs` | 37 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/interaction/damping.rs` | 36 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_audio/src/backend/traits.rs` | 36 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/interaction/physics_helpers.rs` | 35 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/reader/pick.rs` | 21 | 1 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/reader/components.rs` | 20 | 1 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/identity/name_pools/syllables.rs` | 33 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_core/src/window.rs` | 19 | 1 | 90d | never | — | ⬜ |
| 64% | `crates/engine_app/src/profiler_plugin.rs` | 31 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_audio/src/mixer_engine.rs` | 31 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_audio/src/sound/effect.rs` | 30 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_ui/src/layout/system.rs` | 30 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/card_game/src/card/identity/signature/algorithms.rs` | 29 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/card_game/src/card/rendering/geometry.rs` | 29 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/card_game/src/card/rendering/render_layer.rs` | 29 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_scene/src/render_order.rs` | 29 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/card_game/src/card/identity/name_pools/mod.rs` | 28 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_ui/src/layout/anchor.rs` | 28 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_audio/src/prelude.rs` | 10 | 2 | 90d | never | — | ⬜ |
| 63% | `crates/card_game/src/stash/constants.rs` | 25 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_render/src/image_data.rs` | 25 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_ui/src/lib.rs` | 15 | 1 | 90d | never | — | ⬜ |
| 63% | `crates/terrain/src/lib.rs` | 15 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/engine_input/src/mouse/system.rs` | 24 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_render/src/shape/render.rs` | 24 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_ui/src/theme.rs` | 24 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/interaction/mod.rs` | 14 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/mod.rs` | 14 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/engine_audio/src/plugin.rs` | 14 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/engine_physics/src/plugin.rs` | 14 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/rendering/baked_mesh.rs` | 23 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_core/src/transform.rs` | 23 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_input/src/action_map.rs` | 23 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_render/src/rect.rs` | 22 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_render/src/sprite.rs` | 22 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_ui/src/widget/text.rs` | 22 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_audio/src/audio_res.rs` | 21 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_physics/src/physics_res.rs` | 21 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/axiom2d/src/prelude.rs` | 20 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_physics/src/physics_step_system.rs` | 20 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/rendering/mod.rs` | 12 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/engine_render/src/shape/mod.rs` | 19 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_ui/src/render.rs` | 19 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_core/src/lib.rs` | 11 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/engine_core/src/prelude.rs` | 11 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/engine_physics/src/prelude.rs` | 11 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/engine_ui/src/plugin.rs` | 11 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/engine_app/src/mouse_world_pos_system.rs` | 18 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_input/src/mouse/buffer.rs` | 18 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_physics/src/physics_sync_system.rs` | 17 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/card_game/src/stash/mod.rs` | 10 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/card_game/src/card/interaction/game_state_param.rs` | 16 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_render/src/shape/cache.rs` | 16 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/card_game/src/card/reader/rotation_lock.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_app/src/window_size.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_audio/src/sound/library.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_core/src/spring.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_ui/src/prelude.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_audio/src/lib.rs` | 9 | 1 | 90d | never | — | ⬜ |
| 60% | `crates/engine_physics/src/collision_event.rs` | 14 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/engine_input/src/button_state.rs` | 13 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/engine_render/src/clear.rs` | 13 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/card_game/src/stash/render/models.rs` | 12 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/engine_audio/src/sound/data.rs` | 12 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/card_game/src/lib.rs` | 7 | 1 | 90d | never | — | ⬜ |
| 59% | `crates/engine_physics/src/hit_test.rs` | 11 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_ui/src/ui_event.rs` | 11 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_ui/src/widget/mod.rs` | 11 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/card_game/src/card/identity/mod.rs` | 10 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/card_game/src/card/rendering/gpu_card_mesh.rs` | 10 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_scene/src/spawn_child.rs` | 10 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_app/src/lib.rs` | 6 | 1 | 90d | never | — | ⬜ |
| 59% | `crates/engine_app/src/prelude.rs` | 6 | 1 | 90d | never | — | ⬜ |
| 59% | `crates/engine_audio/src/playback/mod.rs` | 6 | 1 | 90d | never | — | ⬜ |
| 59% | `crates/engine_input/src/keyboard/buffer.rs` | 9 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_physics/src/collider.rs` | 9 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_audio/src/test_helpers.rs` | 8 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_physics/src/rigid_body.rs` | 8 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_ui/src/layout/margin.rs` | 8 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_ui/src/layout/mod.rs` | 8 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_ecs/src/prelude.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_input/src/lib.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_input/src/mouse/mod.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_input/src/prelude.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_scene/src/lib.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/card_game/src/terrain/mod.rs` | 4 | 1 | 90d | never | — | ⬜ |
| 57% | `crates/card_game/src/card/interaction/pick.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/engine_audio/src/sound/mod.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/engine_input/src/keyboard/mod.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/engine_scene/src/prelude.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/terrain/src/shader.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 56% | `crates/card_game/src/booster/mod.rs` | 5 | 0 | 90d | never | — | ⬜ |
| 55% | `crates/engine_audio/src/backend/mod.rs` | 4 | 0 | 90d | never | — | ⬜ |
| 54% | `crates/axiom2d/src/lib.rs` | 3 | 0 | 90d | never | — | ⬜ |
| 54% | `crates/card_game/src/hand/mod.rs` | 3 | 0 | 90d | never | — | ⬜ |
| 54% | `crates/engine_assets/src/lib.rs` | 3 | 0 | 90d | never | — | ⬜ |
| 53% | `crates/engine_assets/src/prelude.rs` | 2 | 0 | 90d | never | — | ⬜ |
| 53% | `crates/engine_ecs/src/lib.rs` | 2 | 0 | 90d | never | — | ⬜ |
| 48% | `crates/particle_poc/src/state.rs` | 2246 | 9 | today | 2026-07-16 | 0/1238 (0%) | ⚠️ |
| 26% | `crates/engine_render/src/testing/mod.rs` | 385 | 0 | 4d | 2026-07-12 | 28/59 (47%) | ⚠️ |
| 25% | `crates/axiom2d/src/splash/letters.rs` | 270 | 1 | 1d | 2026-07-15 | 0 mutants | ➖ |
| 23% | `crates/axiom2d/src/default_plugins.rs` | 203 | 1 | today | 2026-07-16 | 5/6 (83%) | ⚠️ |
| 22% | `crates/card_game/src/card/identity/card_name.rs` | 113 | 0 | 6d | 2026-07-10 | 0 mutants | ✅ |
| 21% | `crates/engine_core/src/time.rs` | 123 | 0 | 3d | 2026-07-13 | 13/20 (65%) | ✅ |
| 20% | `crates/engine_scene/src/transform_propagation.rs` | 74 | 0 | 6d | 2026-07-10 | 0 mutants | ✅ |
| 20% | `crates/card_game/src/card/rendering/debug_spawn.rs` | 79 | 0 | 5d | 2026-07-11 | 3/3 (100%) | ✅ |
| 19% | `crates/engine_render/src/shader.rs` | 66 | 0 | 5d | 2026-07-11 | 19/22 (86%) | ✅ |
| 18% | `crates/axiom2d/src/splash/mod.rs` | 15 | 3 | 3d | 2026-07-13 | 0 mutants | ➖ |
| 16% | `crates/engine_render/src/bloom.rs` | 43 | 0 | 2d | 2026-07-14 | 22/22 (100%) | ✅ |
| 15% | `crates/engine_input/src/mouse_button.rs` | 22 | 0 | 5d | 2026-07-11 | 0/1 (0%) | ⚠️ |
| 14% | `crates/card_game/src/card/reader.rs` | 22 | 1 | today | 2026-07-16 | 0 mutants | ➖ |
| 11% | `crates/engine_input/src/keyboard/system.rs` | 14 | 0 | 1d | 2026-07-15 | 1/1 (100%) | ✅ |
| 10% | `crates/card_game/src/stash/toggle.rs` | 10 | 0 | 2d | 2026-07-14 | 2/2 (100%) | ✅ |
| 10% | `crates/engine_render/src/testing/helpers.rs` | 10 | 0 | 2d | 2026-07-14 | 0/1 (0%) | ⚠️ |
| 10% | `crates/engine_core/src/error.rs` | 8 | 0 | 3d | 2026-07-13 | 0 mutants | ➖ |
| 8% | `crates/card_game/src/card/identity/signature.rs` | 4 | 0 | 4d | 2026-07-12 | 0 mutants | ➖ |
| 5% | `crates/engine_audio/src/playback/id.rs` | 2 | 0 | 4d | 2026-07-12 | 0 mutants | ➖ |
| 3% | `crates/card_game/src/card/interaction/release.rs` | 2 | 0 | 1d | 2026-07-15 | 0 mutants | ➖ |

---

## Recent Runs

| Date | Commit | File | Total | Caught | Missed | Timeout | Unviable | Status |
|------|--------|------|-------|--------|--------|---------|----------|--------|
| 2026-07-16 | `b867689` | `crates/card_game/src/card/reader.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-16 | `b867689` | `crates/axiom2d/src/default_plugins.rs` | 6 | 5 | 1 | 0 | 0 | ✅ |
| 2026-07-16 | `b867689` | `crates/particle_poc/src/state.rs` | 1238 | 0 | 1236 | 0 | 2 | ✅ |
| 2026-07-15 | `582977f` | `crates/card_game/src/card/interaction/release.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-15 | `582977f` | `crates/engine_input/src/keyboard/system.rs` | 1 | 1 | 0 | 0 | 0 | ✅ |
| 2026-07-15 | `582977f` | `crates/axiom2d/src/splash/letters.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-14 | `7aa1b9a` | `crates/engine_render/src/bloom.rs` | 22 | 22 | 0 | 0 | 0 | ✅ |
| 2026-07-14 | `7aa1b9a` | `crates/engine_render/src/testing/helpers.rs` | 1 | 0 | 0 | 0 | 1 | ✅ |
| 2026-07-14 | `7aa1b9a` | `crates/card_game/src/stash/toggle.rs` | 2 | 2 | 0 | 0 | 0 | ✅ |
| 2026-07-13 | `ce526fd` | `crates/axiom2d/src/splash/mod.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-13 | `ce526fd` | `crates/engine_core/src/error.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-13 | `ce526fd` | `crates/engine_core/src/time.rs` | 20 | 13 | 0 | 0 | 7 | ✅ |
| 2026-07-12 | `e918008` | `crates/engine_audio/src/playback/id.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-12 | `e918008` | `crates/engine_render/src/testing/mod.rs` | 59 | 28 | 15 | 0 | 16 | ✅ |
| 2026-07-12 | `e918008` | `crates/card_game/src/card/identity/signature.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-07-11 | `daecf46` | `crates/card_game/src/card/rendering/debug_spawn.rs` | 3 | 3 | 0 | 0 | 0 | ✅ |
| 2026-07-11 | `daecf46` | `crates/engine_input/src/mouse_button.rs` | 1 | 0 | 0 | 0 | 1 | ✅ |
| 2026-07-11 | `daecf46` | `crates/engine_render/src/shader.rs` | 22 | 19 | 0 | 0 | 3 | ✅ |
| 2026-07-10 | `60616b7` | `crates/engine_scene/src/transform_propagation.rs` | 0 | 0 | 0 | 0 | 0 | ✅ |

---

## Excluded Paths

- `*/demo/*`
- `*/card_game_bin/*`
- `*/wgpu_renderer/*`
- `*/art/generated/*`
- `*/card_back.rs`
- `*/repository.rs`
- `*/hydrate.rs`
- `*/tests/*`

<!-- Generated by scripts/micro-mutations.sh -->
