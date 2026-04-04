# @doc Annotation Coverage Plan

This document tracks `/// @doc:` annotation coverage across the test suite. Annotations add human-readable context to the [Living Documentation](Living_Documentation.md) — explaining *why* a test exists, not *what* it does (the test name already covers that).

## Convention

```rust
/// @doc: Explanation of the design decision, algorithm, or invariant this test validates
#[test]
fn when_action_then_outcome() {
```

Place the annotation on the line directly above `#[test]`. The living-docs tool (`cargo.exe run -p living-docs`) picks it up automatically and renders it as an indented blockquote beneath the test description.

## What to annotate

Not every test needs an annotation. Target tests where the *name alone* doesn't convey the full picture:

| Annotate | Skip |
|----------|------|
| Design decisions (why this approach over alternatives) | Straightforward CRUD-like behavior |
| Algorithm explanations (spatial audio panning formula) | Obvious cause-and-effect (press key → key is pressed) |
| Non-obvious invariants (accumulator carry-forward) | Derive/construction tests |
| Cross-system contracts (physics→ECS sync rules) | Tests whose names are already fully descriptive |
| Edge cases with real-world motivation (silent-sound culling) | Snapshot or property-based tests (format is self-evident) |

Aim for roughly **20-30%** of tests annotated. Over-annotating dilutes value.

## Current coverage

49 annotations across 696 tests (~7%). Target: ~150 annotations (~22%).

## Priority tiers

Annotations are organized by impact — Tier 1 tests explain core engine contracts that every contributor needs to understand. Tier 3 tests are self-explanatory and rarely need annotation.

---

### Tier 1 — Core contracts and algorithms (annotate ~50%)

These tests document foundational behavior that other systems depend on. A contributor reading these annotations should understand the engine's key design decisions.

#### engine_core — time.rs (13 tests, 4 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_tick_large_delta_then_returns_multiple_steps_and_retains_remainder` | Done | Fix Your Timestep pattern |
| `when_tick_across_frames_then_accumulator_carries_forward` | Done | Accumulator carry-forward |
| `when_tick_below_step_size_then_returns_zero_steps` | Done | Sub-step deltas accumulate silently — no simulation steps fire until a full step_size is reached |
| `when_fake_clock_advanced_then_delta_returns_advancement` | Done | FakeClock enables deterministic testing — advance() accumulates, delta() drains |
| `when_system_clock_delta_called_twice_then_second_call_returns_small_delta` | N/A | Test does not exist in codebase |

#### engine_render — camera.rs (31 tests, 6 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_camera_uniform_from_camera_at_origin_zoom_one_then_origin_maps_to_ndc_center` | Done | Default camera produces pixel-perfect 1:1 mapping at viewport center |
| `when_world_point_matches_camera_center_then_world_to_screen_returns_screen_center` | Done | Camera position defines the world point that appears at screen center |
| `when_screen_to_world_after_world_to_screen_then_recovers_original_point` | Done | world_to_screen and screen_to_world are exact inverses |
| `when_world_point_at_zoom_two_then_world_to_screen_reflects_magnification` | Done | Zoom multiplies screen-space distances — zoom 2 means objects appear 2x larger |
| `when_camera_prepare_system_runs_without_camera_then_default_ortho_set` | Done | camera_prepare_system always sets a projection — defaults to viewport-centered ortho |
| `when_entity_completely_left_of_view_then_aabb_intersects_returns_false` | Done | Frustum culling AABB test — no edge overlap means no intersection |

#### engine_render — sprite.rs (42 tests, 5 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_sprite_fully_outside_camera_view_then_draw_sprite_not_called` | Done | Frustum culling skips draw calls |
| `when_sprite_straddles_camera_view_edge_then_draw_sprite_called` | Done | Edge-touching sprites are drawn — conservative culling avoids popping |
| `when_no_camera_entity_then_all_sprites_drawn_without_culling` | Done | Without a Camera2D entity, frustum culling is disabled entirely |
| `when_two_sprites_on_different_layers_then_background_drawn_before_world` | Done | RenderLayer is the primary sort key — Background draws before UI regardless of SortOrder |
| `when_entity_has_effective_visibility_false_then_draw_sprite_not_called` | Done | EffectiveVisibility(false) is the earliest cull — before sorting or frustum tests |

#### engine_audio — spatial.rs (16 tests, 6 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_emitter_right_of_listener_then_right_gain_one` | Done | Constant-power stereo panning |
| `when_emitter_beyond_max_distance_then_gains_are_zero` | Done | Linear distance attenuation culling |
| `when_emitter_ahead_of_listener_then_gains_equal` | Done | Centered panning when emitter is on listener's forward axis (no left/right bias) |
| `when_emitter_at_listener_then_gains_equal_no_nan` | Done | Coincident positions must not produce NaN — atan2(0,0) edge case |
| `when_emitter_is_child_entity_then_world_position_used` | Done | Spatial audio uses GlobalTransform2D (world space), not local Transform2D |
| `when_no_listener_then_system_runs_without_panic` | Done | Without an AudioListener entity, spatial processing is a no-op |

#### engine_physics — rapier_backend.rs (13 tests, 5 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_dynamic_body_added_then_position_is_queryable` | Done | Body type mapping: ECS Dynamic → rapier Dynamic (free motion under forces) |
| `when_body_type_mapping_then_static_is_fixed_and_kinematic_is_position_based` | Done | Body type mapping: ECS Static → rapier Fixed, ECS Kinematic → rapier KinematicPositionBased |
| `when_two_overlapping_circles_step_then_started_event_with_correct_entities` | Done | Collision events flow: rapier ChannelEventCollector → drain → CollisionEventBuffer |
| `when_remove_body_on_rapier_then_position_returns_none` | Done | Entity removal must clean up both rapier RigidBody and the entity↔handle map |

#### engine_physics — physics_sync_system.rs (12 tests, 3 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_backend_returns_position_then_transform_position_is_updated` | Done | One-way sync: physics backend → Transform2D. ECS is the read side, rapier is the authority |
| `when_backend_returns_position_only_then_rotation_field_is_unchanged` | Done | Position and rotation are synced independently — either can be None without affecting the other |
| `when_entity_has_no_rigid_body_then_its_transform_is_not_touched` | Done | Only entities with RigidBody participate in physics sync — plain transforms are untouched |

---

### Tier 2 — System behavior and integration (annotate ~20%)

These tests validate how systems interact. Annotations explain the ordering or contract, not the mechanics.

#### engine_scene — hierarchy.rs (10 tests, 2 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_child_of_is_removed_then_parent_children_no_longer_contains_that_child` | Done | hierarchy_maintenance_system rebuilds Children from scratch each frame — reparenting is automatic |
| `when_last_child_of_is_removed_then_parent_children_component_is_removed` | Done | Stale Children components are cleaned up when no ChildOf references remain |

#### engine_scene — visibility.rs (12 tests, 2 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_parent_is_hidden_and_child_is_visible_then_child_effective_visibility_is_false` | Done | AND-logic propagation: EffectiveVisibility = parent_effective AND child_visible |
| `when_root_entity_has_no_visible_component_then_visibility_system_inserts_effective_visibility_true` | Done | Visible is opt-in — entities without it default to visible (no component = no hiding) |

#### engine_scene — transform_propagation.rs (12 tests, 2 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_child_has_translation_and_parent_has_translation_then_both_accumulate` | Done | GlobalTransform2D = parent.global * child.local — standard affine composition |
| `when_root_entity_has_identity_transform_then_global_transform_equals_affine2_identity` | Done | Root entities (no ChildOf) copy Transform2D directly to GlobalTransform2D |

#### engine_render — material.rs (12 tests, 1 annotated) + sprite.rs dedup test (1 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_two_sprites_with_same_blend_mode_then_set_blend_mode_called_once` (sprite.rs) | Done | apply_material deduplicates — set_blend_mode only called when mode actually changes |
| `when_preprocessing_with_define_present_then_ifdef_block_included` | Done | #ifdef preprocessor conditionally includes shader blocks for feature-based shader variants |

#### engine_render — bloom.rs (9 tests, 3 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_gaussian_weights_radius3_then_sum_is_one` | Done | Normalized kernel ensures bloom doesn't change overall image brightness |
| `when_gaussian_weights_computed_then_kernel_is_symmetric` | Done | Symmetry allows separable (H+V) blur — same kernel for both passes |
| `when_no_bloom_settings_then_post_process_system_skips` | Done | BloomSettings is opt-in — no resource insertion means zero post-process overhead |

#### engine_render — shape.rs (37 tests, 2 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_tessellating_circle_then_produces_nonempty_vertices_and_indices` | Done | Lyon FillTessellator generates triangle fan — all vertices lie at radius distance from origin |
| `when_tessellating_polygon_with_fewer_than_three_points_then_returns_empty_mesh` | Done | Degenerate polygons (< 3 vertices) produce empty mesh — no GPU draw call issued |

#### engine_app — app.rs (35 tests, 2 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_handle_redraw_called_then_pre_update_runs_before_update` | Done | Phase execution order is fixed: Input → PreUpdate → Update → PostUpdate → Render |
| `when_handle_resize_called_then_window_size_resource_is_updated` | Done | Resize updates both the WindowSize resource and calls renderer.resize() — dual sync |

#### engine_audio — cpal_backend.rs (14 tests, 3 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_two_active_sounds_then_output_is_sum` | Done | Audio mixing is additive — all active sounds summed into output buffer, scaled by volume |
| `when_sound_shorter_than_buffer_then_removed_after_last_sample` | Done | Sounds auto-evict when cursor reaches end — no explicit stop() needed for one-shots |
| `when_global_and_track_volume_both_half_then_output_quarter` | Done | Effective volume = global_volume * track_volume — multiplicative stacking |

#### engine_ui — interaction.rs (16 tests, 3 annotated)

| Test | Status | Suggested annotation |
|------|--------|---------------------|
| `when_cursor_inside_node_then_interaction_becomes_hovered` | Done | AABB hit-test uses anchor_offset to compute top-left from node position + size |
| `when_disabled_button_then_interaction_stays_none` | Done | Disabled buttons are excluded from hit-testing entirely — not just visually dimmed |
| `when_node_clicked_then_focus_state_updated` | Done | Click sets FocusState.focused — only one entity has focus at a time |

---

### Tier 3 — Self-explanatory tests (annotate sparingly, ~5%)

These tests have descriptive names and straightforward Arrange/Act/Assert. Only annotate if there's a non-obvious edge case.

| Module | Tests | Notes |
|--------|-------|-------|
| engine_core/color.rs | 5 | from_u8 conversion math is simple — skip |
| engine_core/transform.rs | 11 | Affine math is standard — skip unless non-obvious |
| engine_core/types.rs | 6 | Newtype arithmetic — skip |
| engine_input/input_state.rs | 17 | Press/release semantics are intuitive — skip |
| engine_input/mouse_state.rs | 18 | Same as input_state — skip |
| engine_assets/asset_server.rs | 14 | Add/get/remove/ref-count is standard — skip |
| engine_assets/scene.rs | 12 | RON serialization roundtrips — skip |
| engine_render/atlas.rs | 31 | Builder pattern + packing — mostly skip |
| engine_render/testing.rs | 17 | SpyRenderer wiring — skip |
| engine_render/visual_regression.rs | 17 | SSIM + row padding — annotate padded_row_bytes alignment logic only |
| engine_ui/anchor.rs | 7 | Pure math — skip |
| engine_ui/flex_layout.rs | 7 | Row/Column offset math — skip |
| engine_ui/button.rs | 7 | Theme color lookup — skip |
| engine_ui/panel.rs | 4 | Border rect math — skip |
| engine_ui/progress_bar.rs | 6 | Fill width clamping — skip |
| axiom2d/default_plugins.rs | 15 | Integration smoke tests — skip |
| demo/main.rs | 41 | Scene setup assertions — skip |
| engine_ecs/schedule.rs | 1 | Phase ordering — skip |

---

## Tracking progress

Run the following to check current annotation count vs total tests:

```bash
# Count annotations
grep -rc "@doc:" crates/ tools/ | grep -v ":0$" | awk -F: '{sum += $2} END {print "Annotations:", sum}'

# Count tests
grep -rc "#\[test\]" crates/ tools/ | grep -v ":0$" | awk -F: '{sum += $2} END {print "Tests:", sum}'
```

Target coverage by tier:

| Tier | Test count | Target % | Target annotations |
|------|-----------|----------|-------------------|
| Tier 1 | ~130 | 50% | ~65 |
| Tier 2 | ~220 | 20% | ~44 |
| Tier 3 | ~340 | 5% | ~17 |
| **Total** | **~690** | **~18%** | **~126** |

## Workflow

When adding new tests, consider adding a `@doc:` annotation if the test falls into Tier 1 or Tier 2 and the name alone doesn't explain the *why*. The annotation should be a single line — if you need more, the test name or surrounding code probably needs refactoring instead.

Regenerate the living documentation after adding annotations:

```bash
cargo.exe run -p living-docs
```
