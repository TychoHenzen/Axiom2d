# Micro-Mutation Tracking

Stochastic mutation testing — one random source file per daily CI run.
Selection weighted by **staleness** (50%), **file size** (30%), and **git churn** (20%).
Over weeks, covers the codebase without combinatorial explosion.

**Cumulative (all runs)**: 4227 mutants | 1477 caught | 2573 missed | 12 timeout | 165 unviable | 53 zero-mutant | 4 errors | **catch rate: 36.5%** | 184 runs | 149 files tested

**Last run**: 2026-09-10 (`869e022`)

---

## File Inventory

All 256 eligible source files. Sorted by selection priority (staleness × size × churn).

| Priority | File | Lines | Churn | Stale | Last Tested | Result | Status |
|----------|------|-------|-------|-------|-------------|--------|--------|
| 91% | `crates/particle_poc/src/lib.rs` | 1632 | 6 | 90d | never | — | ⬜ |
| 81% | `crates/terrain_viewer/src/main.rs` | 594 | 3 | 90d | never | — | ⬜ |
| 80% | `crates/engine_render/src/testing/visual_regression.rs` | 815 | 2 | 90d | never | — | ⬜ |
| 79% | `crates/particle_poc/src/state.rs` | 2246 | 9 | 56d | 2026-07-16 | 0/1238 (0%) | ⚠️ |
| 78% | `crates/engine_ui/src/unified_render.rs` | 448 | 2 | 90d | never | — | ⬜ |
| 78% | `crates/card_game/src/booster/opening.rs` | 421 | 2 | 90d | never | — | ⬜ |
| 77% | `crates/card_game/src/card/identity/name_pools/adjectives.rs` | 865 | 0 | 90d | never | — | ⬜ |
| 76% | `crates/card_game/src/card/jack_socket.rs` | 303 | 2 | 90d | never | — | ⬜ |
| 76% | `crates/card_game/src/card/jack_cable/render.rs` | 284 | 2 | 90d | never | — | ⬜ |
| 75% | `crates/card_game/src/stash/pages.rs` | 232 | 2 | 90d | never | — | ⬜ |
| 74% | `crates/card_game/src/card/screen_geometry.rs` | 175 | 2 | 90d | never | — | ⬜ |
| 74% | `crates/engine_render/src/shape/tessellate.rs` | 275 | 1 | 90d | never | — | ⬜ |
| 73% | `crates/engine_physics/src/rapier_backend.rs` | 336 | 0 | 90d | never | — | ⬜ |
| 73% | `crates/card_game/src/stash/grid.rs` | 113 | 2 | 90d | never | — | ⬜ |
| 73% | `crates/terrain/src/wfc.rs` | 188 | 1 | 90d | never | — | ⬜ |
| 72% | `crates/card_game/src/card/jack_cable/geom.rs` | 182 | 1 | 90d | never | — | ⬜ |
| 72% | `crates/axiom2d/src/splash/render.rs` | 279 | 0 | 90d | never | — | ⬜ |
| 72% | `crates/card_game/src/card/combiner_device.rs` | 253 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/rendering/spawn_table_card/overlay.rs` | 227 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/engine_input/src/key_code.rs` | 213 | 0 | 90d | never | — | ⬜ |
| 71% | `crates/card_game/src/card/jack_cable/mod.rs` | 112 | 1 | 90d | never | — | ⬜ |
| 70% | `crates/terrain/src/tile_def/example.rs` | 165 | 0 | 90d | never | — | ⬜ |
| 70% | `crates/card_game/src/card/identity/base_type.rs` | 156 | 0 | 90d | never | — | ⬜ |
| 70% | `crates/axiom2d/src/splash/types.rs` | 92 | 1 | 90d | never | — | ⬜ |
| 70% | `crates/engine_render/src/shape/path.rs` | 148 | 0 | 90d | never | — | ⬜ |
| 70% | `crates/card_game/src/stash/render.rs` | 144 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/identity/gem_sockets.rs` | 138 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/engine_render/src/renderer.rs` | 138 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/interaction/click_resolve.rs` | 136 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/hand/layout.rs` | 132 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/booster/sampling.rs` | 124 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/stash/render/slots.rs` | 122 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/booster/pack.rs` | 118 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/terrain/src/dual_grid.rs` | 116 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/card/rendering/art_shader.rs` | 113 | 0 | 90d | never | — | ⬜ |
| 69% | `crates/card_game/src/booster/double_click.rs` | 112 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/card_game/src/card/reader/signature_space.rs` | 106 | 0 | 90d | never | — | ⬜ |
| 68% | `crates/engine_render/benches/tessellation.rs` | 106 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_audio/src/spatial.rs` | 80 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_ui/src/widget/panel.rs` | 76 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_render/src/shape/geometry.rs` | 75 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/build.rs` | 74 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/card_game/src/card/rendering/face_layout.rs` | 74 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_assets/src/asset_server.rs` | 72 | 0 | 90d | never | — | ⬜ |
| 67% | `crates/engine_input/src/mouse/state.rs` | 67 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_core/benches/stress.rs` | 65 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/reader/insert.rs` | 61 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/reader/glow.rs` | 60 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_scene/src/sort_propagation.rs` | 58 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_core/src/color.rs` | 57 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/benches/bake.rs` | 56 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/rendering/debug_sleep_indicator.rs` | 56 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_ui/src/text_render.rs` | 55 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/card_game/src/card/interaction/drag.rs` | 54 | 0 | 90d | never | — | ⬜ |
| 66% | `crates/engine_physics/src/physics_command.rs` | 54 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/rendering/drop_zone_glow.rs` | 51 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/interaction/flip.rs` | 49 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_render/src/lib.rs` | 28 | 1 | 90d | never | — | ⬜ |
| 65% | `crates/engine_core/src/types.rs` | 44 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/card_game/src/card/interaction/camera_drag.rs` | 43 | 0 | 90d | never | — | ⬜ |
| 65% | `crates/engine_input/src/keyboard/state.rs` | 43 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/stash/layout.rs` | 39 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_ui/src/layout/flex.rs` | 39 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/interaction/damping.rs` | 36 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/engine_audio/src/backend/traits.rs` | 36 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/card_game/src/card/interaction/sleep.rs` | 34 | 0 | 90d | never | — | ⬜ |
| 64% | `crates/particle_poc/src/capture.rs` | 1288 | 8 | 35d | 2026-08-06 | 0/531 (0%) | ⚠️ |
| 64% | `crates/engine_app/src/profiler_plugin.rs` | 31 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_audio/src/sound/effect.rs` | 30 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/card_game/src/card/rendering/geometry.rs` | 29 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_ui/src/layout/anchor.rs` | 28 | 0 | 90d | never | — | ⬜ |
| 63% | `crates/engine_render/src/image_data.rs` | 25 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_input/src/mouse/system.rs` | 24 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_ui/src/theme.rs` | 24 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/mod.rs` | 14 | 1 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/rendering/baked_mesh.rs` | 23 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_render/src/sprite.rs` | 22 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_ui/src/widget/text.rs` | 22 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/card_game/src/card/reader/pick.rs` | 21 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_physics/src/physics_res.rs` | 21 | 0 | 90d | never | — | ⬜ |
| 62% | `crates/engine_render/src/shape/mod.rs` | 19 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_core/src/lib.rs` | 11 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/engine_ui/src/plugin.rs` | 11 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/engine_app/src/mouse_world_pos_system.rs` | 18 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_input/src/mouse/buffer.rs` | 18 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/card_game/src/stash/mod.rs` | 10 | 1 | 90d | never | — | ⬜ |
| 61% | `crates/card_game/src/card/reader/rotation_lock.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_app/src/window_size.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/terrain/src/lib.rs` | 15 | 0 | 90d | never | — | ⬜ |
| 61% | `crates/engine_audio/src/lib.rs` | 9 | 1 | 90d | never | — | ⬜ |
| 60% | `crates/engine_physics/src/collision_event.rs` | 14 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/engine_input/src/button_state.rs` | 13 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/card_game/src/card/rendering/mod.rs` | 12 | 0 | 90d | never | — | ⬜ |
| 60% | `crates/card_game/src/stash/render/models.rs` | 12 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_ui/src/ui_event.rs` | 11 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_ui/src/widget/mod.rs` | 11 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/card_game/src/card/rendering/gpu_card_mesh.rs` | 10 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/card_game/src/card/screen_device.rs` | 389 | 4 | 49d | 2026-07-23 | 24/86 (28%) | ⚠️ |
| 59% | `crates/engine_input/src/keyboard/buffer.rs` | 9 | 0 | 90d | never | — | ⬜ |
| 59% | `crates/engine_physics/src/collider.rs` | 9 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_audio/src/test_helpers.rs` | 8 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/card_game/src/lib.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 58% | `crates/engine_scene/src/lib.rs` | 7 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/card_game/src/card/interaction/pick.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/engine_audio/src/sound/mod.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/engine_input/src/keyboard/mod.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 57% | `crates/terrain/src/shader.rs` | 6 | 0 | 90d | never | — | ⬜ |
| 56% | `crates/card_game/src/booster/mod.rs` | 5 | 0 | 90d | never | — | ⬜ |
| 56% | `crates/particle_poc/src/main.rs` | 134 | 32 | 30d | 2026-08-11 | 0/14 (0%) | ⚠️ |
| 54% | `crates/engine_assets/src/lib.rs` | 3 | 0 | 90d | never | — | ⬜ |
| 54% | `crates/axiom2d/src/default_plugins.rs` | 203 | 1 | 56d | 2026-07-16 | 5/6 (83%) | ⚠️ |
| 53% | `crates/card_game/src/card/identity/card_name.rs` | 113 | 0 | 62d | 2026-07-10 | 0 mutants | ✅ |
| 53% | `crates/engine_assets/src/prelude.rs` | 2 | 0 | 90d | never | — | ⬜ |
| 52% | `crates/card_game/src/booster/device.rs` | 377 | 2 | 44d | 2026-07-28 | 7/49 (14%) | ⚠️ |
| 51% | `crates/engine_scene/src/transform_propagation.rs` | 74 | 0 | 62d | 2026-07-10 | 0 mutants | ✅ |
| 51% | `crates/card_game/src/card/rendering/debug_spawn.rs` | 79 | 0 | 61d | 2026-07-11 | 3/3 (100%) | ✅ |
| 50% | `crates/engine_render/src/shader.rs` | 66 | 0 | 61d | 2026-07-11 | 19/22 (86%) | ✅ |
| 50% | `crates/engine_render/src/atlas.rs` | 163 | 2 | 47d | 2026-07-25 | 57/60 (95%) | ✅ |
| 50% | `crates/card_game/src/card/reader/spawn.rs` | 229 | 0 | 51d | 2026-07-21 | 0/32 (0%) | ⚠️ |
| 49% | `crates/axiom2d/src/splash/mod.rs` | 15 | 3 | 59d | 2026-07-13 | 0 mutants | ➖ |
| 48% | `crates/card_game/src/hand/cards.rs` | 52 | 1 | 54d | 2026-07-18 | 13/16 (81%) | ⚠️ |
| 47% | `crates/card_game/src/stash/store.rs` | 555 | 5 | 21d | 2026-08-20 | error | ❌ |
| 46% | `crates/card_game/src/card/identity/name_pools/templates.rs` | 87 | 0 | 52d | 2026-07-20 | 18/29 (62%) | ⚠️ |
| 46% | `crates/engine_input/src/mouse_button.rs` | 22 | 0 | 61d | 2026-07-11 | 0/1 (0%) | ⚠️ |
| 46% | `crates/engine_core/benches/spring.rs` | 97 | 0 | 50d | 2026-07-22 | 0 mutants | ➖ |
| 46% | `crates/engine_physics/benches/stress.rs` | 45 | 0 | 55d | 2026-07-17 | 0 mutants | ➖ |
| 45% | `crates/card_game/src/card/identity/signature/types.rs` | 204 | 0 | 43d | 2026-07-29 | 39/46 (85%) | ✅ |
| 44% | `crates/card_game/src/card/identity/name_pools/syllables.rs` | 33 | 0 | 55d | 2026-07-17 | 2/2 (100%) | ✅ |
| 43% | `crates/card_game/src/card/reader.rs` | 22 | 0 | 56d | 2026-07-16 | 0 mutants | ➖ |
| 43% | `crates/engine_render/src/shape/components.rs` | 125 | 0 | 43d | 2026-07-29 | 5/10 (50%) | ⚠️ |
| 43% | `crates/card_game/src/card/art/mod.rs` | 49 | 0 | 49d | 2026-07-23 | 17/18 (94%) | ⚠️ |
| 42% | `crates/engine_input/src/keyboard/system.rs` | 14 | 0 | 57d | 2026-07-15 | 1/1 (100%) | ✅ |
| 42% | `crates/engine_scene/src/render_order.rs` | 29 | 0 | 51d | 2026-07-21 | 4/4 (100%) | ✅ |
| 41% | `crates/card_game/src/stash/toggle.rs` | 10 | 0 | 58d | 2026-07-14 | 2/2 (100%) | ✅ |
| 41% | `crates/engine_render/src/testing/helpers.rs` | 10 | 0 | 58d | 2026-07-14 | 0/1 (0%) | ⚠️ |
| 41% | `crates/engine_core/src/error.rs` | 8 | 0 | 59d | 2026-07-13 | 0 mutants | ➖ |
| 41% | `crates/engine_render/src/shape/cache.rs` | 16 | 0 | 54d | 2026-07-18 | 1/1 (100%) | ✅ |
| 41% | `crates/card_game/src/card/identity/name_pools/mod.rs` | 28 | 0 | 50d | 2026-07-22 | 0/1 (0%) | ⚠️ |
| 41% | `crates/card_game/src/card/component.rs` | 47 | 0 | 46d | 2026-07-26 | 0/1 (0%) | ⚠️ |
| 40% | `crates/engine_audio/src/prelude.rs` | 10 | 1 | 52d | 2026-07-20 | 0 mutants | ➖ |
| 40% | `crates/card_game/src/card/interaction/apply.rs` | 314 | 0 | 31d | 2026-08-10 | 12/13 (92%) | ⚠️ |
| 40% | `crates/engine_audio/src/playback/buffer.rs` | 43 | 0 | 45d | 2026-07-27 | 0/2 (0%) | ⚠️ |
| 40% | `crates/card_game/src/card/interaction/physics_helpers.rs` | 35 | 0 | 46d | 2026-07-26 | 1/1 (100%) | ✅ |
| 39% | `crates/card_game/src/stash/boundary.rs` | 68 | 0 | 41d | 2026-07-31 | 12/12 (100%) | ✅ |
| 39% | `crates/engine_audio/src/audio_res.rs` | 21 | 0 | 48d | 2026-07-24 | 0/2 (0%) | ⚠️ |
| 39% | `crates/engine_physics/src/physics_command_apply_system.rs` | 65 | 0 | 40d | 2026-08-01 | 1/1 (100%) | ✅ |
| 38% | `crates/card_game/src/card/identity/residual.rs` | 77 | 0 | 38d | 2026-08-03 | 26/28 (93%) | ✅ |
| 37% | `crates/engine_render/benches/stress.rs` | 73 | 0 | 37d | 2026-08-04 | 0 mutants | ➖ |
| 37% | `crates/engine_render/src/font.rs` | 352 | 3 | 15d | 2026-08-26 | 54/173 (31%) | ⚠️ |
| 37% | `crates/card_game/src/card/reader/eject.rs` | 54 | 0 | 39d | 2026-08-02 | 1/2 (50%) | ✅ |
| 37% | `crates/card_game/src/card/interaction/drag_state.rs` | 24 | 0 | 44d | 2026-07-28 | 0 mutants | ➖ |
| 37% | `crates/engine_render/src/camera.rs` | 139 | 0 | 31d | 2026-08-10 | 48/70 (69%) | ⚠️ |
| 36% | `crates/engine_core/src/event_bus.rs` | 37 | 0 | 40d | 2026-08-01 | 6/14 (43%) | ✅ |
| 36% | `crates/engine_core/src/window.rs` | 19 | 1 | 41d | 2026-07-31 | 0 mutants | ➖ |
| 36% | `crates/engine_ui/src/interaction.rs` | 84 | 0 | 34d | 2026-08-07 | 26/26 (100%) | ✅ |
| 36% | `crates/engine_ui/src/layout/system.rs` | 30 | 0 | 41d | 2026-07-31 | 3/3 (100%) | ✅ |
| 36% | `crates/engine_physics/src/hit_test.rs` | 11 | 0 | 48d | 2026-07-24 | 8/8 (100%) | ✅ |
| 36% | `crates/engine_app/src/app.rs` | 337 | 1 | 20d | 2026-08-21 | 30/35 (86%) | ⚠️ |
| 36% | `crates/card_game/src/stash/constants.rs` | 25 | 0 | 42d | 2026-07-30 | 4/4 (100%) | ✅ |
| 36% | `crates/card_game/benches/stress.rs` | 50 | 0 | 37d | 2026-08-04 | 0 mutants | ➖ |
| 36% | `crates/engine_render/benches/font.rs` | 133 | 0 | 30d | 2026-08-11 | 0 mutants | ➖ |
| 35% | `crates/axiom2d/src/splash/animation.rs` | 68 | 0 | 34d | 2026-08-07 | 9/9 (100%) | ✅ |
| 35% | `crates/card_game/src/card/interaction/mod.rs` | 14 | 0 | 45d | 2026-07-27 | 0 mutants | ➖ |
| 35% | `crates/engine_ui/src/widget/node.rs` | 37 | 0 | 38d | 2026-08-03 | 3/3 (100%) | ✅ |
| 35% | `crates/engine_ui/src/lib.rs` | 15 | 1 | 40d | 2026-08-01 | 3/3 (100%) | ✅ |
| 35% | `crates/engine_ecs/src/schedule.rs` | 72 | 0 | 32d | 2026-08-09 | 4/4 (100%) | ✅ |
| 34% | `crates/card_game/src/card/interaction/release.rs` | 2 | 0 | 57d | 2026-07-15 | 0 mutants | ➖ |
| 34% | `crates/engine_render/src/culling.rs` | 58 | 0 | 33d | 2026-08-08 | 36/41 (88%) | ⚠️ |
| 34% | `crates/card_game/src/card/rendering/spawn_table_card.rs` | 135 | 0 | 27d | 2026-08-14 | 3/4 (75%) | ✅ |
| 34% | `crates/card_game/src/stash/render/drag_preview.rs` | 97 | 0 | 29d | 2026-08-12 | 12/29 (41%) | ⚠️ |
| 34% | `crates/engine_render/src/testing/mod.rs` | 385 | 0 | 19d | 2026-08-22 | 28/59 (47%) | ⚠️ |
| 34% | `crates/engine_input/src/prelude.rs` | 7 | 0 | 47d | 2026-07-25 | 0 mutants | ➖ |
| 33% | `crates/engine_audio/src/backend/cpal.rs` | 118 | 0 | 26d | 2026-08-15 | 2/8 (25%) | ⚠️ |
| 33% | `crates/card_game/src/prelude.rs` | 73 | 0 | 29d | 2026-08-12 | 0 mutants | ➖ |
| 33% | `crates/card_game/src/card/rendering/bake.rs` | 151 | 0 | 23d | 2026-08-18 | 9/51 (18%) | ⚠️ |
| 32% | `crates/engine_physics/src/physics_step_system.rs` | 20 | 0 | 37d | 2026-08-04 | 1/1 (100%) | ✅ |
| 32% | `crates/engine_core/src/time.rs` | 123 | 0 | 23d | 2026-08-18 | 13/20 (65%) | ✅ |
| 31% | `crates/engine_physics/benches/physics.rs` | 66 | 0 | 27d | 2026-08-14 | 0 mutants | ➖ |
| 31% | `crates/engine_audio/src/plugin.rs` | 14 | 1 | 34d | 2026-08-07 | 0 mutants | ➖ |
| 31% | `crates/card_game/src/stash/store_render.rs` | 319 | 2 | 8d | 2026-09-02 | error | ❌ |
| 31% | `crates/card_game/src/card/identity/name_pools/nouns.rs` | 319 | 0 | 15d | 2026-08-26 | 3/3 (100%) | ✅ |
| 31% | `crates/engine_input/src/lib.rs` | 7 | 0 | 42d | 2026-07-30 | 0 mutants | ➖ |
| 31% | `crates/card_game/src/card/interaction/release/target.rs` | 121 | 0 | 21d | 2026-08-20 | 0 mutants | ✅ |
| 30% | `crates/card_game/src/card/reader/volume.rs` | 68 | 0 | 25d | 2026-08-16 | 63/78 (81%) | ⚠️ |
| 30% | `crates/card_game/src/card/identity/card_description.rs` | 42 | 0 | 28d | 2026-08-13 | 10/14 (71%) | ⚠️ |
| 30% | `crates/axiom2d/src/splash/letters.rs` | 270 | 1 | 11d | 2026-08-30 | 0 mutants | ➖ |
| 29% | `crates/engine_ecs/src/prelude.rs` | 7 | 0 | 39d | 2026-08-02 | 0 mutants | ➖ |
| 29% | `crates/card_game/src/card/identity/name_pools/compound_parts.rs` | 354 | 0 | 11d | 2026-08-30 | 8/8 (100%) | ✅ |
| 29% | `crates/engine_ui/src/prelude.rs` | 15 | 0 | 33d | 2026-08-08 | 0 mutants | ➖ |
| 29% | `crates/card_game/src/terrain/mod.rs` | 4 | 0 | 42d | 2026-07-30 | 0 mutants | ➖ |
| 29% | `crates/engine_core/src/prelude.rs` | 11 | 1 | 31d | 2026-08-10 | 0 mutants | ➖ |
| 29% | `crates/terrain/src/tile_def.rs` | 298 | 0 | 11d | 2026-08-30 | 106/200 (53%) | ⚠️ |
| 28% | `crates/card_game/src/card/identity/mod.rs` | 10 | 0 | 35d | 2026-08-06 | 0 mutants | ➖ |
| 28% | `crates/card_game/src/card/art_selection.rs` | 119 | 0 | 17d | 2026-08-24 | 45/63 (71%) | ⚠️ |
| 28% | `crates/engine_ui/src/render.rs` | 19 | 0 | 30d | 2026-08-11 | 1/1 (100%) | ✅ |
| 28% | `crates/terrain/src/material.rs` | 130 | 0 | 16d | 2026-08-25 | 2/5 (40%) | ⚠️ |
| 28% | `crates/engine_render/src/plugin.rs` | 23 | 1 | 24d | 2026-08-17 | 0/1 (0%) | ⚠️ |
| 28% | `crates/engine_physics/src/rigid_body.rs` | 8 | 0 | 35d | 2026-08-06 | 0 mutants | ➖ |
| 28% | `crates/card_game/src/plugin.rs` | 203 | 0 | 12d | 2026-08-29 | 2/2 (100%) | ✅ |
| 27% | `crates/engine_render/src/material.rs` | 73 | 0 | 18d | 2026-08-23 | 2/8 (25%) | ⚠️ |
| 26% | `crates/engine_physics/src/lib.rs` | 245 | 1 | 5d | 2026-09-05 | 27/51 (53%) | ⚠️ |
| 26% | `crates/engine_audio/src/sound/library.rs` | 15 | 0 | 28d | 2026-08-13 | 2/3 (67%) | ✅ |
| 26% | `crates/axiom2d/src/lib.rs` | 3 | 0 | 39d | 2026-08-02 | 0 mutants | ➖ |
| 26% | `crates/card_game/src/card/identity/signature_profile.rs` | 56 | 0 | 18d | 2026-08-23 | 14/16 (88%) | ⚠️ |
| 25% | `crates/engine_audio/src/mixer.rs` | 44 | 0 | 19d | 2026-08-22 | 6/6 (100%) | ✅ |
| 25% | `crates/card_game/src/card/identity/signature/algorithms.rs` | 29 | 0 | 21d | 2026-08-20 | 14/15 (93%) | ⚠️ |
| 25% | `crates/card_game/src/stash/hover.rs` | 135 | 0 | 10d | 2026-08-31 | 16/58 (28%) | ⚠️ |
| 25% | `crates/engine_scene/src/prelude.rs` | 6 | 0 | 32d | 2026-08-09 | 0 mutants | ➖ |
| 25% | `crates/engine_app/src/prelude.rs` | 6 | 1 | 28d | 2026-08-13 | 0 mutants | ➖ |
| 24% | `crates/card_game/src/card/reader/drag.rs` | 67 | 0 | 14d | 2026-08-27 | 5/9 (56%) | ⚠️ |
| 24% | `crates/engine_physics/src/prelude.rs` | 11 | 1 | 23d | 2026-08-18 | 0 mutants | ➖ |
| 24% | `crates/engine_render/src/clear.rs` | 13 | 0 | 25d | 2026-08-16 | 1/1 (100%) | ✅ |
| 24% | `crates/engine_audio/src/backend/mod.rs` | 4 | 0 | 33d | 2026-08-08 | 0 mutants | ➖ |
| 24% | `crates/card_game/src/card/zone_config.rs` | 43 | 0 | 16d | 2026-08-25 | 0/1 (0%) | ⚠️ |
| 24% | `crates/card_game/src/card/identity/visual_params.rs` | 55 | 0 | 14d | 2026-08-27 | 6/11 (55%) | ⚠️ |
| 24% | `crates/engine_core/src/profiler.rs` | 111 | 0 | 9d | 2026-09-01 | 10/11 (91%) | ✅ |
| 23% | `crates/engine_physics/src/physics_backend.rs` | 155 | 0 | 6d | 2026-09-04 | 36/36 (100%) | ✅ |
| 23% | `crates/card_game/src/card/reader/components.rs` | 20 | 0 | 20d | 2026-08-21 | 7/7 (100%) | ✅ |
| 23% | `crates/card_game/src/card/rendering/baked_render.rs` | 46 | 0 | 14d | 2026-08-27 | 1/1 (100%) | ✅ |
| 23% | `crates/engine_scene/src/hierarchy.rs` | 37 | 0 | 15d | 2026-08-26 | 4/6 (67%) | ⚠️ |
| 22% | `crates/engine_render/src/prelude.rs` | 29 | 1 | 13d | 2026-08-28 | 0 mutants | ➖ |
| 22% | `crates/engine_assets/src/handle.rs` | 47 | 0 | 13d | 2026-08-28 | 2/9 (22%) | ⚠️ |
| 22% | `crates/axiom2d/src/prelude.rs` | 20 | 0 | 18d | 2026-08-23 | 0 mutants | ➖ |
| 22% | `crates/engine_ui/src/widget/progress_bar.rs` | 59 | 0 | 10d | 2026-08-31 | 14/14 (100%) | ✅ |
| 21% | `crates/engine_render/src/bloom.rs` | 43 | 0 | 12d | 2026-08-29 | 22/22 (100%) | ✅ |
| 21% | `crates/engine_scene/src/visibility.rs` | 86 | 0 | 7d | 2026-09-03 | 4/6 (67%) | ⚠️ |
| 21% | `crates/terrain/src/prelude.rs` | 12 | 0 | 20d | 2026-08-21 | 0 mutants | ➖ |
| 20% | `crates/card_game/src/hand/mod.rs` | 3 | 0 | 29d | 2026-08-12 | 0 mutants | ➖ |
| 20% | `crates/card_game/src/card/rendering/spawn_table_card/text.rs` | 45 | 0 | 9d | 2026-09-01 | 0/28 (0%) | ⚠️ |
| 20% | `crates/card_game/src/card/interaction/intent.rs` | 40 | 0 | 9d | 2026-09-01 | 0 mutants | ➖ |
| 19% | `crates/card_game/src/card/identity/definition.rs` | 141 | 0 | today | 2026-09-10 | 60/152 (39%) | ⚠️ |
| 18% | `crates/engine_app/src/lib.rs` | 6 | 1 | 17d | 2026-08-24 | 0 mutants | ➖ |
| 18% | `crates/engine_ui/src/draw_command.rs` | 84 | 0 | 1d | 2026-09-09 | 2/7 (29%) | ⚠️ |
| 18% | `crates/card_game/src/card/interaction/flip_animation.rs` | 47 | 0 | 5d | 2026-09-05 | 21/23 (91%) | ⚠️ |
| 18% | `crates/engine_core/src/scale_spring.rs` | 70 | 0 | 2d | 2026-09-08 | 14/19 (74%) | ⚠️ |
| 17% | `crates/engine_audio/src/playback/system.rs` | 63 | 0 | 2d | 2026-09-08 | 8/18 (44%) | ⚠️ |
| 17% | `crates/engine_input/src/mouse/mod.rs` | 7 | 0 | 16d | 2026-08-25 | 0 mutants | ➖ |
| 16% | `crates/engine_audio/src/playback/mod.rs` | 6 | 0 | 17d | 2026-08-24 | 0 mutants | ➖ |
| 16% | `crates/engine_core/src/transform.rs` | 23 | 0 | 7d | 2026-09-03 | 1/1 (100%) | ✅ |
| 16% | `crates/engine_audio/src/playback/id.rs` | 2 | 0 | 24d | 2026-08-17 | 0 mutants | ➖ |
| 16% | `crates/engine_audio/src/mixer_engine.rs` | 31 | 0 | 4d | 2026-09-06 | 16/16 (100%) | ✅ |
| 15% | `crates/card_game/src/card/interaction/game_state_param.rs` | 16 | 0 | 7d | 2026-09-03 | 0 mutants | ➖ |
| 15% | `crates/engine_ui/src/widget/button.rs` | 42 | 0 | today | 2026-09-10 | 1/1 (100%) | ✅ |
| 14% | `crates/engine_physics/src/physics_sync_system.rs` | 17 | 0 | 6d | 2026-09-04 | 1/1 (100%) | ✅ |
| 14% | `crates/engine_render/src/shape/render.rs` | 24 | 0 | 3d | 2026-09-07 | 0/24 (0%) | ⚠️ |
| 14% | `crates/engine_input/src/action_map.rs` | 23 | 0 | 3d | 2026-09-07 | 4/6 (67%) | ✅ |
| 14% | `crates/card_game/src/card/rendering/render_layer.rs` | 29 | 0 | 1d | 2026-09-09 | 2/3 (67%) | ✅ |
| 13% | `crates/engine_physics/src/plugin.rs` | 14 | 1 | 2d | 2026-09-08 | 0 mutants | ➖ |
| 13% | `crates/engine_core/src/spring.rs` | 15 | 0 | 5d | 2026-09-05 | 25/25 (100%) | ✅ |
| 13% | `crates/engine_ecs/src/lib.rs` | 2 | 0 | 19d | 2026-08-22 | 0 mutants | ➖ |
| 13% | `crates/engine_audio/src/sound/data.rs` | 12 | 0 | 6d | 2026-09-04 | 4/4 (100%) | ✅ |
| 13% | `crates/engine_render/src/rect.rs` | 22 | 0 | 1d | 2026-09-09 | 0 mutants | ➖ |
| 11% | `crates/card_game/src/card/identity/signature.rs` | 4 | 0 | 10d | 2026-08-31 | 0 mutants | ➖ |
| 10% | `crates/engine_ui/src/layout/margin.rs` | 8 | 0 | 4d | 2026-09-06 | 0 mutants | ➖ |
| 10% | `crates/engine_ui/src/layout/mod.rs` | 8 | 0 | 4d | 2026-09-06 | 0 mutants | ➖ |
| 9% | `crates/engine_scene/src/spawn_child.rs` | 10 | 0 | today | 2026-09-10 | 0/1 (0%) | ⚠️ |

---

## Recent Runs

| Date | Commit | File | Total | Caught | Missed | Timeout | Unviable | Status |
|------|--------|------|-------|--------|--------|---------|----------|--------|
| 2026-09-10 | `869e022` | `crates/engine_ui/src/widget/button.rs` | 1 | 1 | 0 | 0 | 0 | ✅ |
| 2026-09-10 | `869e022` | `crates/engine_scene/src/spawn_child.rs` | 1 | 0 | 0 | 0 | 1 | ✅ |
| 2026-09-10 | `869e022` | `crates/card_game/src/card/identity/definition.rs` | 152 | 60 | 90 | 0 | 2 | ✅ |
| 2026-09-09 | `c181df8` | `crates/engine_ui/src/draw_command.rs` | 7 | 2 | 4 | 0 | 1 | ✅ |
| 2026-09-09 | `c181df8` | `crates/card_game/src/card/rendering/render_layer.rs` | 3 | 2 | 0 | 0 | 1 | ✅ |
| 2026-09-09 | `c181df8` | `crates/engine_render/src/rect.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-09-08 | `d841b70` | `crates/engine_physics/src/plugin.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-09-08 | `d841b70` | `crates/engine_audio/src/playback/system.rs` | 18 | 8 | 9 | 0 | 1 | ✅ |
| 2026-09-08 | `d841b70` | `crates/engine_core/src/scale_spring.rs` | 19 | 14 | 1 | 4 | 0 | ✅ |
| 2026-09-07 | `62c2c80` | `crates/engine_input/src/action_map.rs` | 6 | 4 | 0 | 0 | 2 | ✅ |
| 2026-09-07 | `62c2c80` | `crates/engine_render/src/shape/render.rs` | 24 | 0 | 24 | 0 | 0 | ✅ |
| 2026-09-07 | `62c2c80` | `crates/engine_audio/src/playback/system.rs` | 18 | 8 | 9 | 0 | 1 | ✅ |
| 2026-09-06 | `9d29fa1` | `crates/engine_ui/src/layout/margin.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-09-06 | `9d29fa1` | `crates/engine_ui/src/layout/mod.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-09-06 | `9d29fa1` | `crates/engine_audio/src/mixer_engine.rs` | 16 | 16 | 0 | 0 | 0 | ✅ |
| 2026-09-05 | `1282a0d` | `crates/engine_core/src/spring.rs` | 25 | 25 | 0 | 0 | 0 | ✅ |
| 2026-09-05 | `1282a0d` | `crates/card_game/src/card/interaction/flip_animation.rs` | 23 | 21 | 1 | 0 | 1 | ✅ |
| 2026-09-05 | `1282a0d` | `crates/engine_physics/src/lib.rs` | 51 | 27 | 20 | 0 | 4 | ✅ |
| 2026-09-04 | `7b6d24c` | `crates/engine_physics/src/physics_sync_system.rs` | 1 | 1 | 0 | 0 | 0 | ✅ |
| 2026-09-04 | `7b6d24c` | `crates/engine_physics/src/physics_backend.rs` | 36 | 36 | 0 | 0 | 0 | ✅ |
| 2026-09-04 | `7b6d24c` | `crates/engine_audio/src/sound/data.rs` | 4 | 4 | 0 | 0 | 0 | ✅ |
| 2026-09-03 | `9968731` | `crates/card_game/src/card/interaction/game_state_param.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |
| 2026-09-03 | `9968731` | `crates/engine_scene/src/visibility.rs` | 6 | 4 | 2 | 0 | 0 | ✅ |
| 2026-09-03 | `9968731` | `crates/engine_core/src/transform.rs` | 1 | 1 | 0 | 0 | 0 | ✅ |
| 2026-09-02 | `c4681be` | `crates/card_game/src/stash/store_render.rs` | 0 | 0 | 0 | 0 | 0 | ❌ cargo-mutants timed out after 3000s (per-mutant timeout was  |
| 2026-09-02 | `c4681be` | `crates/engine_audio/src/mixer_engine.rs` | 16 | 16 | 0 | 0 | 0 | ✅ |
| 2026-09-02 | `c4681be` | `crates/engine_ui/src/draw_command.rs` | 7 | 2 | 4 | 0 | 1 | ✅ |
| 2026-09-01 | `e7afed7` | `crates/card_game/src/card/rendering/spawn_table_card/text.rs` | 28 | 0 | 28 | 0 | 0 | ✅ |
| 2026-09-01 | `e7afed7` | `crates/engine_core/src/profiler.rs` | 11 | 10 | 0 | 0 | 1 | ✅ |
| 2026-09-01 | `e7afed7` | `crates/card_game/src/card/interaction/intent.rs` | 0 | 0 | 0 | 0 | 0 | 0 mutants |

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
