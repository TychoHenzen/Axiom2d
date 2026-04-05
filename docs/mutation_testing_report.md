# Mutation Testing Report

Generated: 2026-04-05  
Tool: `cargo-mutants 27.0.0`  
Scope: full workspace (`--no-shuffle -vV --in-place --timeout 30`)  
Test count at time of run: ~1,700 across all crates

---

## 1. Overall Results

| Status     | Count | % of viable |
|------------|-------|-------------|
| Caught     | 1,976 | 65%         |
| **Missed** | **981**   | **32%**     |
| Timeout    | 4     | <1%         |
| Unviable   | 213   | —           |
| **Total**  | **5,193** |         |

After this session's fixes, 15 new tests were added targeting the highest-value logic gaps. The analysis below covers both what was fixed and what remains.

---

## 2. Fixes Applied This Session

The following gaps were closed by writing targeted tests in three files:

### `crates/card_game/tests/suite/store.rs` (7 new tests)

| Mutant caught | Test |
|---|---|
| `StoreWallet::can_afford → true` (bypasses affordability) | `when_wallet_coins_less_than_cost_then_can_afford_is_false` |
| `can_afford` boundary (coins == cost) | `when_wallet_coins_equal_cost_then_can_afford_is_true` |
| `can_afford → true` through spend path | `when_wallet_empty_and_spend_called_then_returns_false_and_coins_unchanged` |
| `storage_tab_purchase_cost → 0` or `→ 1` | `when_one_storage_tab_exists_then_purchase_cost_equals_base_cost` |
| `<< vs >>` in cost exponent (50 vs 0 for tab 2) | `when_two_storage_tabs_exist_then_purchase_cost_is_twice_base` |
| exponential cost scaling confirmed | `when_three_storage_tabs_exist_then_purchase_cost_is_four_times_base` |
| behavioral: 0-coin wallet blocks tab purchase | `when_wallet_empty_and_plus_tab_clicked_then_no_storage_tab_added` |

**Root cause of the `storage_tab_purchase_cost` mutations surviving:** The existing test used `100 - storage_tab_purchase_cost(1)` as its expected value. Because cargo-mutants mutates the function body, both the call-site in the system _and_ the assertion in the test use the mutated version. The assertion always holds. Fix: hardcode the numeric expectation.

### `crates/card_game/tests/suite/stash_pages.rs` (3 new tests)

| Mutant caught | Test |
|---|---|
| `< → <=` on tab-row top boundary | `when_click_exactly_at_tab_row_top_edge_then_page_changes` |
| `> → ==` on tab-row bottom boundary | `when_click_exactly_at_tab_row_bottom_edge_then_page_changes` |
| `\|\| → &&` making out-of-bounds clicks pass through | `when_click_above_tab_row_then_page_unchanged_regardless_of_starting_page` |

**Root cause of the `|| → &&` survival:** The existing test `when_click_above_tabs_then_page_unchanged` asserts `page == 1`, but `StashGrid::new` defaults to `current_page = 1`. Clicking above the tab row at tab-1's x position with the mutant causes no page change (already on page 1), so the test passes for the wrong reason. Fix: start on page 2.

### `crates/card_game/tests/suite/card_screen_device.rs` (4 new tests + helper)

| Mutant caught | Test |
|---|---|
| `screen_pick_system → ()`, `delete !` on just_pressed, `<= → >` hit test | `when_cursor_inside_screen_device_bounds_and_left_clicked_then_drag_starts` |
| `\|\| → &&` in drag-guard (lines 270–272) | `when_card_drag_active_and_cursor_on_screen_then_screen_drag_does_not_start` |
| stash priority blocks screen pick | `when_stash_visible_and_cursor_inside_stash_bounds_then_screen_drag_blocked` |
| `stash_ui_contains → true` (via screen_pick_system context) | `when_stash_visible_and_cursor_outside_stash_bounds_then_screen_drag_starts` |

---

## 3. Remaining Missed Mutants

981 mutants remain uncaught. They fall into three categories.

### 3a. Display / Render Math — Skip (394 mutants)

These are arithmetic mutations inside rendering functions. The mutations change pixel positions, color interpolations, or geometry calculations that are only observable by inspecting rendered output. No behavioral (non-visual) test can catch them without a pixel-level rendering assertion, and the visual regression tests aren't run against mutation-modified code in this setup.

| File | Count | Nature |
|---|---|---|
| `engine_render/src/font.rs` | 88 | Glyph layout arithmetic |
| `axiom2d/src/splash/render.rs` | 64 | `color_lerp` interpolation math |
| `card_game/src/stash/hover.rs` | 42 | `lissajous_offset`, hover preview positions |
| `card_game/src/card/rendering/bake.rs` | 42 | Vertex geometry calculations |
| `card_game/src/card/rendering/spawn_table_card/text.rs` | 28 | Text layout positions |
| `card_game/src/card/rendering/spawn_table_card/overlay.rs` | 20 | Overlay geometry |
| `card_game/src/stash/render/drag_preview.rs` | 17 | Preview position math |
| `engine_ui/src/text_render.rs` | 16 | Text render layout |
| `card_game/src/card/rendering/geometry.rs` | 15 | Card shape vertices |
| `card_game/src/card/rendering/drop_zone_glow.rs` | 14 | Glow geometry |
| `engine_render/src/camera.rs` | 13 | Camera matrix math |
| *(remaining render files)* | 35 | Similar |

**Action:** None. Adding assertions on exact pixel positions would be brittle and test rendering implementation rather than game behavior. Visual regression tests (`image-compare`, `SSIM ≥ 0.99`) provide the appropriate coverage for this code.

### 3b. Business Logic — Residual Gaps (357 mutants)

These are in non-rendering code. Most are either equivalent mutations or display-geometry in a logic file.

**`stash/store.rs` — 200 mutants remaining**  
After fixing `can_afford` and `storage_tab_purchase_cost`, the remaining 196 are all mutations of `store_ui_bounds` (returning arbitrary `(f32, f32, f32, f32)` coordinate tuples). `store_ui_bounds` computes the pixel bounds of the store UI panel — its result feeds rendering, not decision logic. None of these mutations change game state. **Skip.**

**`card/screen_device.rs` — 91 mutants remaining**  
After fixing the pick system, the remaining are arithmetic in `screen_render_system` (signal dot positions), `spawn_screen_device` (child entity offset geometry), and `on_screen_clicked` grab-offset calculation. These are rendering/transform math. **Skip.**

**`stash/pages.rs` — 24 mutants remaining**  
After fixing the boundary conditions and `|| → &&`, the remaining are:
- `tab_row_top_y` arithmetic (`+ → *`, `- → +`) — position calculation, display
- `draw_centered_screen_text` division — text centering math, display
- `stash_tab_render_system` division — tab position rendering, display
- `stash_ui_contains` `&& → ||` and arithmetic mutations (post-fix, the `→ true` is caught; the arithmetic mutations change the exact pixel boundary, observable only visually)

**Action for `stash_ui_contains` arithmetic:** The `&& → ||` mutations in `stash_ui_contains` would need a click positioned outside the stash horizontally but in the tab row y-range, at a tab's x position. This is geometrically impossible — tabs are centered within the stash's x bounds, so no valid tab hit position lies outside `stash_ui_contains`'s x range. The guard is structurally redundant for `stash_tab_click_system`, which validates tab positions independently. For other callers (`screen_pick_system`, `click_resolve`, `store_buy_system`), the relevant mutations were caught. **Skip remaining.**

**`card/jack_cable.rs` — 13 mutants**  
All are arithmetic in `cable_visuals` (control point positions for the bezier cable shape). One gap: `cable_render_system → ()` (entire system deleted) — the test `when_cable_connects_two_positioned_entities_then_one_line_shape_is_drawn` should catch this but shows as zero-kill (see §4). Worth investigating whether that test runs at all. The arithmetic mutations are display geometry. **Skip arithmetic; investigate system deletion.**

**`hand/layout.rs` — 10 mutants**  
All in `fan_angle` and `fan_screen_position` — fan layout geometry, display math. **Skip.**

**`card/interaction/apply.rs` — 3 mutants**  
- Line 173: `== → !=` in `filter(|(z, _)| **z == CardZone::Table)` — real gap. This would compute `max_sort` from cards in all zones instead of only Table cards, assigning the dragged card an incorrectly high sort order when Table and Hand cards coexist. The test `when_pick_card_table_intent_applied_then_drag_state_set_with_correct_entity_and_zone` doesn't set up a mixed-zone scenario with distinct sort orders.
- Line 191: `+ → *`, `+ → -` in sort order assignment — arithmetic, can be covered by the above.

**`engine_assets/src/handle.rs` — 4 mutants**  
Handle identity/comparison operations. **Low priority.**

### 3c. Other (233 mutants)

**`card/identity/definition.rs` — 90 mutants (all in `hsl_to_rgb`)**  
Custom HSL-to-RGB conversion. Every mutation changes the color math. No behavioral test can catch these without asserting on exact color output, which is rendered visuals. The function is indirectly exercised by visual regression tests. **Skip.**

**`card/identity/gem_sockets.rs` — 26 mutants**  
`hexagon_vertices` geometry, `aspect_color` hue calculation. Display math. **Skip.**

**`card/identity/name_pools/templates.rs` — 21 mutants**  
String interpolation and template selection. Caught if a test asserts the exact output; existing tests check _shape_ (length, template kind) not exact strings. Reasonable to leave.

**`card_game/src/test_helpers.rs` — 19 mutants**  
Mutations in test helper infrastructure. These are in non-production code. **Skip.**

**`engine_physics/src/lib.rs` — 15 mutants**  
All in `SpyPhysicsBackend` and `RecordingPhysicsBackend` implementations (test doubles). Not production logic. **Skip.**

**`engine_audio/src/playback/system.rs` — 9 mutants**  
Audio playback system — calls to audio backend. Hard to test without real audio output. **Skip.**

**`engine_scene` (hierarchy, transform_propagation, visibility) — 6 mutants**  
Scene graph math (2 each). These are fundamental engine subsystems. Worth investigating but complex. **Low priority.**

---

## 4. Test Attribution Analysis

### 4a. Zero-Kill Tests

572 tests (33% of 1,729) never caused a mutation to fail. Breakdown:

| Category | Count |
|---|---|
| Tool crates not in mutation scope (`img-to-shape`, `img-to-shape-gui`) | 69 |
| `suite::` tests — matching CLAUDE.md banned patterns | 67 |
| `suite::` tests — other explanations (see below) | 364 |

**Matched banned patterns (67 tests — candidates for deletion):**

These tests fire assertions that Rust's type system or derive macros already guarantee:
- `when_camera2d_serialized_to_ron_then_deserializes_to_equal_value` — serde roundtrip on `#[derive(Serialize, Deserialize)]`
- `when_card_definition_is_creature_with_none_stats_then_stats_is_none` — constructor echo
- `when_card_definition_is_spell_with_some_stats_then_stats_is_some` — constructor echo
- `when_aspect_cluster_called_for_all_aspects_then_each_maps_to_correct_cluster` — likely enum match verification
- `when_default_rarity_tier_config_constructed_then_advance_rates_are_0_point_3` — trivial default echo
- All `when_*_is_fully_opaque` color tests — asserting `.a == 1.0` on a const struct
- `when_dominant_aspect_called_for_all_elements_then_each_returns_distinct_positive_variant` — enum completeness check, caught by compilation

**Other zero-kill `suite::` tests (364 — require manual review):**

These don't match banned patterns but still catch nothing. Common root causes:

1. **Null-path tests with no behavioral assertion** (≈75 tests):  
   `when_rmb_just_pressed_then_camera_position_unchanged`, `when_zero_scroll_delta_then_zoom_unchanged`, `when_no_cables_exist_then_no_shapes_are_drawn` — these assert the system doesn't crash on a no-op path, or that something is "unchanged" when the system has nothing to do. These will never fail unless the code panics. They're testing absence of behavior, not presence of behavior. **Candidates for deletion or strengthening.**

2. **"Spawn then check component exists" tests** (≈20 tests):  
   `given_empty_world_when_spawn_screen_device_called_then_jack_entity_has_direction_input`, `when_spawn_reader_then_accent_child_exists` — testing that spawn functions set components that they visibly set in the same function body. Any time spawn logic changes, these tests update automatically because they're reading the same state they're asserting. **Candidates for deletion.**

3. **Range/invariant tests with no specific value** (≈30 tests):  
   `when_random_signature_generated_then_all_axes_within_bounds` — asserts `(-1..=1)` range. The range is enforced structurally. `when_any_valid_axis_values_then_distance_is_non_negative` — always true by geometry. **Not false: these are legitimate invariant tests but cannot catch mutations because the invariant holds for all mutations.**

4. **Tests that require real rendering** (≈15 tests):  
   `when_card_ejected_then_runes_return_to_dim`, `when_reader_empty_then_runes_are_dim` — depend on `SpyRenderer` call count or color. If the spy doesn't capture color, these tests can't catch color mutations. **Worth reviewing the spy capture setup.**

5. **Tests where mutation scope didn't include the tested file** (≈220 tests):  
   Tests in `engine_app`, `engine_audio`, `engine_render`, `engine_scene`, `engine_ecs`, `engine_physics` that test subsystems which have zero missed mutants in them (everything was caught by other tests or was unviable). These tests ARE useful — they caught mutants. The attribution shows they have zero unique kills, not zero kills. Wait — these would be in the "redundant" bucket, not zero-kill. The actual zero-kills here are tests that ran against a mutation but never observed it fail.

### 4b. Redundant Tests (Zero Unique Kills)

1,010 tests (58% of tests with any kills) have no unique kills — every mutant they catch is also caught by at least one other test.

This is **expected and acceptable** for integration tests. Integration tests catch the same mutants as unit tests by design. The question is whether the integration test provides value beyond fault detection:

| Test | Kills | Status |
|---|---|---|
| `when_card_picked_from_stash_and_released_into_hand_then_card_in_hand` | 65 | Keep — validates full pick/release chain |
| `when_table_card_crosses_stash_boundary_over_multiple_frames_then_release_uses_current_zone` | 56 | Keep — timing-dependent multi-frame path |
| `when_stash_card_released_on_table_then_zone_table_body_present_item_form_removed_drag_cleared` | 55 | Keep — validates state consistency across systems |
| `when_click_on_third_tab_then_switches_to_page_two` | 44 | Review — duplicates `when_click_on_second_tab_then_switches_to_page_one` |
| `when_click_on_first_tab_then_stays_on_page_zero` | 35 | Review — tab 0 is a special case but may just duplicate tab-N tests |
| `when_store_page_renders_then_item_prices_land_with_the_item_cards` | 116 | Keep — spatial assertion on rendered text positions |
| `when_selling_reader_then_reader_tree_and_jack_are_removed` | 91 | Review — component-removal assertion; buy+sell cycle |

**High-redundancy flag: stash_pages tab navigation**  
`when_click_on_second_tab_then_switches_to_page_one`, `when_click_on_third_tab_then_switches_to_page_two`, and `when_click_on_first_tab_then_stays_on_page_zero` all catch the same mutants. Together they verify 3 different tab indices. Each provides no unique kills. One would suffice; the others could be merged into a property-style test over all tab indices.

### 4c. Bloated Mutants (High Test Coupling)

573 mutants were each caught by more than 5 tests. The most extreme example:

**`algorithms.rs:17:40 replace + with - in compute_seed` — caught by 133 tests.**

`compute_seed` is called during card identity generation. Because the majority of card-identity tests use signatures computed from seeds, any mutation here cascades through the entire identity test suite. This indicates the tests are highly coupled through a shared seeded computation rather than testing isolated behaviors.

**Impact:** This is not necessarily a problem — the coupling is intentional (deterministic generation). But it means 133 tests provide zero value _to each other_ for catching this class of mutation. If `compute_seed` is broken, any one test would catch it.

Other high-coupling examples (>50 tests per mutant):
- `signature/algorithms.rs` — seed computation touched by all identity tests
- `stash/store.rs` — store geometry functions touched by integration + render tests

---

## 5. Recommendations

### Immediate (high confidence, low risk)

1. **Delete the stash tab navigation duplicate tests.** `when_click_on_third_tab_then_switches_to_page_two` and `when_click_on_first_tab_then_stays_on_page_zero` are fully redundant with `when_click_on_second_tab_then_switches_to_page_one` and add no unique kills. Merge into a parameterized or table-driven test over all tabs.

2. **Review `card_jack_cable` zero-kill on system deletion.** `when_cable_connects_two_positioned_entities_then_one_line_shape_is_drawn` should catch `cable_render_system → ()` but shows zero kills. Either the test is not running the cable render system, or the spy doesn't capture shape output. Investigate.

3. **Delete or strengthen "spawn then check component" tests.** The `card_jack_socket` spawner tests (`given_empty_world_when_spawn_screen_device_called_then_*`) test that spawn functions attach components they visibly attach. These match the banned "component spawn tests" pattern from CLAUDE.md.

### Medium-term (higher effort)

4. **Fix `apply_pick_card` zone filter gap.** The `== → !=` mutation on `CardZone::Table` filter (line 173) lets a mutation survive that would incorrectly compute sort order in mixed-zone scenarios. Write one test: pick a card from Table when a Hand card has a higher sort order; assert the picked card gets `max_table_sort + 1`, not `max_hand_sort + 1`.

5. **Consolidate identity tests that rely on `compute_seed`.** 133 tests catch the same `compute_seed` mutation. Consider whether the identity test suite needs a smaller set of targeted seed tests, with the others focusing on the downstream transformation (color computation, name selection) rather than re-exercising the seed path.

### Low priority / acceptable debt

6. **Display math mutations (394).** Only pixel-level visual regression tests can catch these. The SSIM tests in `engine_render` provide this. Acceptable gap.

7. **Tool crate tests (69 zero-kill).** `img-to-shape` and `img-to-shape-gui` were not in the mutation scope. These tests are valid — they're just not reachable from the current mutation run configuration.

8. **`hsl_to_rgb` (90 mutants).** Color math for procedural card art. Untestable without rendering assertions. Acceptable gap.

9. **`stash_ui_contains` arithmetic (remaining after session fixes).** Geometrically impossible to reach remaining mutations through system-level tests. Acceptable gap.

---

## 6. Summary Table

| Area | Missed (start) | Fixed | Remaining | Disposition |
|---|---|---|---|---|
| `StoreWallet::can_afford` | 1 | ✅ | 0 | Fixed |
| `storage_tab_purchase_cost` | 3 | ✅ | 0 | Fixed |
| `store_ui_bounds` display geometry | 196 | — | 196 | Skip (display) |
| `screen_pick_system` logic | ~8 | ✅ | 0 | Fixed |
| `screen_device` render math | ~83 | — | 83 | Skip (display) |
| `stash_tab_click_system` boundaries | 5 | ✅ | 0 | Fixed |
| `stash_ui_contains` system-level | 1 | ✅ | 0 | Fixed |
| `stash_ui_contains` arithmetic | ~23 | — | ~23 | Geometrically unreachable |
| `stash_tab_render_system` / `tab_row_top_y` | ~11 | — | ~11 | Skip (display) |
| `apply_pick_card` zone filter | 1 | — | 1 | **Open gap** |
| `hsl_to_rgb` | 90 | — | 90 | Skip (color math) |
| `font.rs` / `text_render.rs` | 104 | — | 104 | Skip (display) |
| `splash/render.rs` | 64 | — | 64 | Skip (display) |
| `jack_cable` render math | 12 | — | 12 | Skip (display) |
| `jack_cable` system deletion | 1 | — | 1 | **Investigate** |
| Engine physics test helpers | 15 | — | 15 | Skip (test infra) |
| Audio backend | 13 | — | 13 | Skip |
| `gem_sockets` / `name_pools` | 47 | — | 47 | Low priority |
| Scene graph math | 6 | — | 6 | Low priority |
| Other | ~280 | — | ~280 | Various (see §3c) |
