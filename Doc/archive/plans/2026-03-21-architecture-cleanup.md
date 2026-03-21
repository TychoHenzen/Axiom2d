# Architecture Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Clean up the card game's architecture — remove false confidence from bad tests, add real confidence with end-to-end tests, fix API contracts, and update development rules — before entering the serious game design phase.

**Architecture:** The card game uses bevy_ecs standalone with trait-abstracted hardware (Renderer, PhysicsBackend, AudioBackend). The cleanup preserves this architecture but fixes contracts (void → Result), removes ~100 framework-guarantee tests, adds end-to-end schedule tests, and updates CLAUDE.md to prevent regression. No structural rewrites.

**Tech Stack:** Rust 2024, bevy_ecs 0.16, rapier2d, wgpu, tracing crate (new dependency)

**Source:** All findings and priorities come from `Doc/Codebase_Debate_2026-03-21.md` — a structured 4-expert debate analyzing the codebase.

---

## Phase 1: Fix What Guides Future Work

These changes are tiny but high-leverage — they prevent the cleanup work itself from reintroducing the problems we're fixing.

### Task 1: Update CLAUDE.md test rules to prevent test disease

The CLAUDE.md already bans derive tests but the ban doesn't cover serde roundtrips on derived impls. The LLM pattern-matches against existing tests, so bad examples propagate. Fix the rules first.

**Files:**
- Modify: `CLAUDE.md` (Testing Strategy → "What NOT to test" section, around line 118)

**Step 1: Add serde roundtrip ban and behavioral test requirement**

Add to the "What NOT to test" list in CLAUDE.md:

```markdown
- **Serde roundtrip tests on derived impls**: Don't test that `Serialize`/`Deserialize` roundtrips work on types that only use `#[derive(Serialize, Deserialize)]`. Serde's derive macros are not broken. Only test serialization when there is a custom `Serialize`/`Deserialize` impl or when the serialized format is part of a public contract (e.g., save files, network protocol).
- **PartialEq tests on derived impls**: Don't test that `PartialEq` correctly distinguishes enum variants or struct fields when using `#[derive(PartialEq)]`. Rust's derive macros are not broken.
- **Constructor-echo tests**: Don't test that `Foo::new(10, 10, 3)` produces `width=10, height=10, pages=3`. If the constructor stores its arguments, that's a language guarantee. Only test constructors that compute or validate.
```

Add a new subsection after "What NOT to test":

```markdown
### Required test categories

Every roadmap step or feature PR must include at least one **behavioral test** — a test that exercises the outcome a player would observe, not the internal method calls made to achieve it. Good behavioral tests:
- Assert on game state (card is in hand, card has no physics body) not on spy logs (remove_body was called)
- Don't care about implementation order — they survive refactors
- For system chains, test through the real schedule when the interaction between systems is the thing that matters

Spy-based tests (SpyRenderer, SpyPhysicsBackend captures) are acceptable for verifying rendering output and physics API usage, but the primary assertion should always be on the resulting game state.
```

**Step 2: Verify CLAUDE.md is well-formed**

Read the modified file and confirm the new sections are properly placed and formatted.

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md test rules — ban serde roundtrips, require behavioral tests"
```

---

### Task 2: Soften the release profile for diagnostics

The current release profile (`strip = true`, `panic = "abort"`, `opt-level = "z"`) makes any panic an untraceable process death. This is the #5 consensus finding from the debate.

**Files:**
- Modify: root `Cargo.toml` (the `[profile.release]` section)

**Step 1: Add a debuggable release profile and soften the default**

Change the existing `[profile.release]` and add a new `[profile.release-debuggable]`:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = "debuginfo"   # Changed: keeps symbol names for stack traces, strips DWARF

[profile.release-debuggable]
inherits = "release"
strip = false          # Full symbols
panic = "unwind"       # Backtraces work
opt-level = 2          # Speed over size, better debuggability
```

The key change: `strip = true` → `strip = "debuginfo"`. This keeps function names in the binary (so panics show symbol names) while still stripping the bulk of debug info. The `release-debuggable` profile is for when you actually need to diagnose something.

**Step 2: Verify it builds**

Run: `cargo.exe build --release`
Expected: Builds successfully.

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: soften release profile — keep symbol names, add release-debuggable profile"
```

---

## Phase 2: Test Hygiene — Remove Waste, Add Real Value

### Task 3: Delete framework-guarantee tests in card_game

These tests verify that Rust's derive macros and serde work. They create false confidence and serve as bad templates for the LLM. Target: ~17 tests across 4 files.

**Files:**
- Modify: `crates/card_game/src/card/definition.rs` — delete 10 serde roundtrip tests + the proptest
- Modify: `crates/card_game/src/card/zone.rs` — delete 3 PartialEq tests + 1 serde roundtrip
- Modify: `crates/card_game/src/card/component.rs` — delete 1 serde roundtrip test
- Modify: `crates/card_game/src/card/label.rs` — delete 1 serde roundtrip test
- Modify: `crates/card_game/src/plugin.rs` — delete 2 constructor-echo tests

**Step 1: Delete serde roundtrip tests from definition.rs**

Delete these test functions (keep the logic tests like `description_from_abilities` variants, `rarity_colors_are_distinct`, `art_descriptors_differ_by_type`, and the stats option tests):

- `when_keyword_serialized_to_ron_then_each_variant_roundtrips`
- `when_rarity_serialized_to_ron_then_each_variant_roundtrips`
- `when_card_type_serialized_to_ron_then_each_variant_roundtrips`
- `when_card_stats_serialized_to_ron_then_cost_attack_health_roundtrip`
- `when_gradient_serialized_to_ron_then_top_and_bottom_colors_roundtrip`
- `when_art_descriptor_serialized_to_ron_then_roundtrips`
- `when_card_abilities_with_no_keywords_serialized_to_ron_then_roundtrips`
- `when_card_abilities_with_multiple_keywords_serialized_to_ron_then_roundtrips`
- `when_creature_card_definition_serialized_to_ron_then_roundtrips`
- `when_spell_card_definition_serialized_to_ron_then_stats_none_roundtrips`
- The proptest `when_any_card_definition_serialized_to_ron_then_roundtrips`

Also remove `use ron;` and `use proptest::prelude::*;` if they become unused. Remove the `proptest!` macro invocation entirely.

**Step 2: Delete all tests from zone.rs**

All 4 tests in `zone.rs` are framework-guarantee tests:
- 3 PartialEq derive verification tests
- 1 serde roundtrip test

Delete the entire `#[cfg(test)] mod tests` block. Remove `#[allow(clippy::unwrap_used)]` that preceded it. Remove the unused `use` for `ron` if present.

**Step 3: Delete serde roundtrip test from component.rs**

Delete `when_card_serialized_to_ron_then_roundtrip_preserves_all_fields`. Check if other tests remain — if so, keep the test module. If this was the only test, delete the entire `#[cfg(test)] mod tests` block.

**Step 4: Delete serde roundtrip test from label.rs**

Delete `when_card_label_serialized_to_ron_then_roundtrip_preserves_name_and_description`. Same logic — delete the test module if it becomes empty.

**Step 5: Delete constructor-echo tests from plugin.rs**

Delete these two tests that verify constructor stores arguments:
- `when_plugin_built_then_clear_color_is_dark_gray` — tests that `Color { r: 0.1, ... }` is `Color { r: 0.1, ... }`
- `when_plugin_built_then_stash_grid_is_10x10_with_3_pages` — tests that `StashGrid::new(10, 10, 3)` has width 10, height 10, 3 pages

Keep these three plugin tests (they test actual behavior/integration):
- `when_plugin_built_then_hand_max_size_is_10` — tests hand capacity works through add()
- `when_plugin_built_then_physics_res_not_inserted` — tests an important negative: plugin doesn't insert physics (binary does)
- `when_plugin_built_then_card_art_shader_resolves_in_registry` — tests shader registration integration

**Step 6: Run tests to verify nothing broke**

Run: `cargo.exe test -p card_game`
Expected: All remaining tests pass. Test count should drop by ~17.

**Step 7: Commit**

```bash
git add crates/card_game/
git commit -m "test: delete ~17 framework-guarantee tests (serde roundtrips, derive checks, constructor echoes)

These tests verified that Rust's derive macros and serde work, not authored behavior.
CLAUDE.md now explicitly bans this category."
```

---

### Task 4: Audit and delete structural tests in spawn_table_card.rs

`spawn_table_card.rs` has 31 tests across 994 lines. Many verify structural properties ("front face has four shape children", "back center pattern uses thirty percent") that encode the current visual layout as specification. These will break on any visual redesign without catching behavioral bugs.

**Files:**
- Modify: `crates/card_game/src/card/spawn_table_card.rs`

**Step 1: Categorize tests**

Read all 31 test names and bodies. Categorize each as:
- **Keep**: Tests behavior/logic (visibility based on face_up, rarity color mapping, description text from abilities)
- **Delete**: Tests structural layout constants (shape child count, exact pixel positions, percentage sizes, half-extents matching slot dimensions)

Likely **delete** candidates (verify by reading):
- `when_spawn_visual_card_then_front_face_has_four_shape_children` — counts children
- `when_spawn_visual_card_then_back_face_has_at_least_two_shape_children` — counts children
- `when_spawn_visual_card_then_front_border_matches_card_dimensions` — asserts pixel values
- `when_spawn_visual_card_then_front_name_strip_position_and_size_correct` — asserts pixel positions
- `when_spawn_visual_card_then_front_art_area_position_and_size_correct` — asserts pixel positions
- `when_spawn_visual_card_then_front_description_strip_position_and_size_correct` — asserts pixel positions
- `when_spawn_visual_card_then_back_border_matches_card_half_size` — asserts pixel values
- `when_spawn_visual_card_then_back_center_pattern_uses_thirty_percent` — asserts percentage
- `when_spawn_visual_card_then_front_border_half_size_matches_card_half` — asserts pixel values
- `when_spawn_visual_card_then_stash_icon_half_extents_match_slot_dimensions` — asserts pixel values
- `when_spawn_visual_card_then_exactly_one_stash_icon_child_exists` — counts children

Likely **keep** candidates:
- `when_spawn_visual_card_then_root_has_card_component_face_down` — behavior
- `when_spawn_visual_card_then_root_has_card_label_with_name_from_definition` — behavior
- `when_spawn_visual_card_then_card_label_description_from_abilities` — behavior
- `when_spawn_visual_card_with_keywords_then_description_includes_keyword_names` — behavior
- `when_spawn_common_rarity_then_border_color_matches_rarity` — logic (color mapping)
- `when_spawn_legendary_rarity_then_border_color_is_golden_not_white` — logic (color mapping)
- `when_spawn_visual_card_face_down_then_front_children_not_visible` — behavior
- `when_spawn_visual_card_face_down_then_back_children_visible` — behavior
- `when_spawn_visual_card_face_up_then_front_visible_back_hidden` — behavior
- `when_spawn_visual_card_then_stash_icon_child_is_hidden` — behavior
- `when_card_art_shader_registered_then_art_area_has_material2d` — integration
- `when_no_card_art_shader_then_art_area_has_no_material2d` — integration
- `when_spawn_visual_card_then_two_text_children_with_front_side` — might be structural (review)
- `when_spawn_visual_card_then_name_text_matches_definition_name` — behavior
- `when_spawn_visual_card_then_description_text_matches_abilities` — behavior
- `when_spawn_visual_card_then_name_text_sort_higher_than_name_strip` — ordering constraint (keep)
- `when_spawn_visual_card_then_desc_text_sort_higher_than_desc_strip` — ordering constraint (keep)
- `when_spawn_visual_card_face_down_then_text_children_hidden` — behavior

**Step 2: Delete the structural tests**

Delete the ~11 structural layout tests identified above. Remove any unused imports.

**Step 3: Run tests**

Run: `cargo.exe test -p card_game`
Expected: All remaining tests pass.

**Step 4: Commit**

```bash
git add crates/card_game/src/card/spawn_table_card.rs
git commit -m "test: delete ~11 structural layout tests from spawn_table_card

These encoded pixel positions and child counts as specification — they would
break on any visual redesign without catching behavioral bugs."
```

---

### Task 5: Extract test builder for release.rs and collapse single-assertion families

`release.rs` has 41 tests across 2,060 lines. The `DragInfo` construction block is copy-pasted ~40 times. Four `when_release_in_hand_area_from_table_then_*` tests share identical setup and differ only in their assertion.

**Files:**
- Modify: `crates/card_game/src/card/release.rs` (test module, starting line 246)

**Step 1: Create a `ReleaseTestBuilder` in the test module**

Add a builder that encapsulates the common setup pattern:

```rust
struct ReleaseTestBuilder {
    viewport_h: u32,
    screen_pos: Vec2,
    stash_visible: bool,
    origin_zone: CardZone,
    spawn_components: Vec<Box<dyn FnOnce(&mut EntityWorldMut)>>,
    card_face_up: bool,
}

impl ReleaseTestBuilder {
    fn card_on_table() -> Self { /* defaults for table drop */ }
    fn card_in_hand(index: usize) -> Self { /* defaults for hand origin */ }
    fn card_in_stash(page: u8, col: u8, row: u8) -> Self { /* defaults for stash origin */ }
    fn screen_pos(mut self, x: f32, y: f32) -> Self { /* ... */ }
    fn viewport_height(mut self, h: u32) -> Self { /* ... */ }
    fn stash_visible(mut self) -> Self { /* ... */ }
    fn face_up(mut self) -> Self { /* ... */ }
    fn build(self) -> (World, Entity, RemoveBodyLog, AddBodyLog) { /* ... */ }
}
```

Note: The exact implementation should follow the existing `make_release_world` pattern but include entity spawning with the DragState setup. The builder's `build()` returns the entity so tests can query it.

**Step 2: Collapse single-assertion test families**

Merge these four tests into one:
- `when_release_in_hand_area_from_table_then_card_added_to_hand`
- `when_release_in_hand_area_from_table_then_zone_becomes_hand`
- `when_release_in_hand_area_from_table_then_render_layer_becomes_ui`
- `when_release_in_hand_area_from_table_then_physics_body_removed`

Into:
```rust
#[test]
fn when_card_released_into_hand_from_table_then_full_zone_transition() {
    // Arrange
    let (mut world, entity, remove_log, _) = ReleaseTestBuilder::card_on_table()
        .screen_pos(400.0, 550.0)  // in hand drop zone
        .build();

    // Act
    run_system(&mut world);

    // Assert
    let hand = world.resource::<Hand>();
    assert!(hand.contains(entity), "card should be in hand");
    assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Hand(0));
    assert_eq!(*world.get::<RenderLayer>(entity).unwrap(), RenderLayer::UI);
    assert!(!world.get::<RigidBody>(entity).is_some(), "physics body should be removed");
}
```

Do the same for any other single-assertion families you find (e.g., stash drop tests that share identical setup).

**Step 3: Migrate remaining tests to use the builder**

Convert the remaining tests to use `ReleaseTestBuilder` instead of manual `make_release_world` + `DragState` construction. The `make_release_world` helper can be deleted once all tests are migrated.

**Step 4: Run tests**

Run: `cargo.exe test -p card_game -- release`
Expected: All tests pass. Test count may drop (collapsed families), line count should drop significantly.

**Step 5: Commit**

```bash
git add crates/card_game/src/card/release.rs
git commit -m "test(release): extract ReleaseTestBuilder, collapse single-assertion families

Reduces ~2060 LOC of tests to ~800-1000 by eliminating copy-pasted DragInfo
construction and merging tests that share identical Arrange/Act."
```

---

### Task 6: Write end-to-end schedule tests

This is the #1 consensus finding: 22 systems chained in `register_systems()` with zero tests verifying the chain works. A misordering produces subtle bugs, not assertion failures.

**Files:**
- Create: `crates/card_game/src/integration_tests.rs`
- Modify: `crates/card_game/src/lib.rs` — add `#[cfg(test)] mod integration_tests;`

**Step 1: Create the integration test module**

This module exercises the real schedule (all systems registered by CardGamePlugin) across multiple frames. It needs a helper that builds a fully-wired World:

```rust
//! End-to-end tests that exercise the real CardGamePlugin schedule.
//! These verify system ordering and multi-system interactions — the most
//! dangerous untested property identified in the architecture review.

use std::sync::{Arc, Mutex};
use bevy_ecs::prelude::*;
use engine_app::prelude::{App, Phase};
use engine_core::prelude::*;
use engine_input::prelude::*;
use engine_physics::prelude::*;
use engine_render::prelude::*;
use engine_render::testing::SpyRenderer;
use engine_scene::prelude::*;
use glam::Vec2;

use crate::plugin::CardGamePlugin;
use crate::card::zone::CardZone;
use crate::card::component::Card;
use crate::card::spawn_table_card::spawn_visual_card;
use crate::card::definition::*;
use crate::hand::cards::Hand;
use crate::test_helpers::SpyPhysicsBackend;

/// Build a fully-wired App with CardGamePlugin, SpyRenderer, and SpyPhysicsBackend.
/// Returns the App plus handles needed for assertions.
fn make_game_app() -> (App, /* any spy handles */) {
    let mut app = App::new();
    // Insert prerequisites that card_game_bin normally provides
    let spy_physics = SpyPhysicsBackend::new()
        // ... with whatever logs needed
        ;
    app.world_mut().insert_resource(PhysicsRes::new(Box::new(spy_physics)));
    app.world_mut().insert_resource(ShaderRegistry::default());

    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_viewport(800, 600);
    app.world_mut().insert_resource(RendererRes::new(Box::new(spy)));

    // Engine resources that DefaultPlugins normally inserts
    app.world_mut().insert_resource(DeltaTime::default());
    app.world_mut().insert_resource(InputState::default());
    app.world_mut().insert_resource(MouseState::default());
    app.world_mut().insert_resource(MouseEventBuffer::default());
    app.world_mut().insert_resource(InputEventBuffer::default());
    // ... any other resources that the schedule's systems expect

    // Register the game plugin — this wires all 22 systems
    app.add_plugin(CardGamePlugin);

    // Register engine systems that DefaultPlugins would add
    // (input_system, mouse_input_system, physics_step_system, physics_sync_system,
    //  hierarchy_maintenance_system, transform_propagation_system, visibility_system,
    //  camera_prepare_system, etc.)
    // Only register what the card game systems actually depend on.

    (app, /* spy handles */)
}

/// Simulate one frame: run all phase schedules in order.
fn tick(app: &mut App) {
    for phase in Phase::ALL {
        app.world_mut()  // Need to run schedule — check App's API
        // The App may expose handle_redraw() or similar.
        // If not, run each schedule manually.
    }
}
```

Note: The exact wiring depends on how `App` exposes schedule execution for tests. Check if `App` has a `handle_redraw()` or if schedules need to be run directly. The key constraint is that the schedules must run in `Phase` order with the real system registrations.

**Step 2: Write test — pick-drag-release to hand**

```rust
#[test]
fn when_card_picked_dragged_to_hand_zone_and_released_then_card_is_in_hand() {
    // Arrange
    let (mut app, ..) = make_game_app();
    let entity = spawn_a_test_card(&mut app); // helper that spawns a card on the table

    // Frame 1: simulate mouse press on the card's position
    simulate_mouse_press(&mut app, card_screen_pos);
    tick(&mut app);

    // Frame 2: simulate mouse move to hand zone
    simulate_mouse_move(&mut app, hand_zone_pos);
    tick(&mut app);

    // Frame 3: simulate mouse release
    simulate_mouse_release(&mut app);
    tick(&mut app);

    // Assert: behavioral outcome, not implementation details
    let hand = app.world().resource::<Hand>();
    assert!(hand.contains(entity), "card should be in the hand after pick-drag-release");
    assert_eq!(*app.world().get::<CardZone>(entity).unwrap(), CardZone::Hand(0));
    // Card should have no physics body (hand cards are non-physical)
    assert!(app.world().get::<RigidBody>(entity).is_none());
}
```

**Step 3: Write test — pick-drag-release to stash**

Similar pattern: pick a card from table, open stash, drag to a stash slot, release. Assert the card is in the stash grid, has `CardItemForm`, and has `CardZone::Stash { ... }`.

**Step 4: Write test — pick-drag-release back to table**

Pick a card, drag it but release in the table area. Assert it's still on the table with `RigidBody::Dynamic` and `CardZone::Table`.

**Step 5: Write test — hand-to-table transition**

Pick a card from the hand, drag to table, release. Assert it has physics body reactivated and `CardZone::Table`.

**Step 6: Write test — stash-to-hand transition**

Pick a card from stash, drag to hand zone, release. Assert it's in the hand with no physics body.

These 5 tests cover the major zone transitions through the real schedule.

**Step 7: Run tests**

Run: `cargo.exe test -p card_game -- integration`
Expected: All 5 pass (or you discover real ordering bugs — which is the point).

**Step 8: Commit**

```bash
git add crates/card_game/src/integration_tests.rs crates/card_game/src/lib.rs
git commit -m "test: add 5 end-to-end schedule tests for zone transitions

Exercises pick-drag-release through the real CardGamePlugin schedule.
Verifies system ordering — the highest-consequence untested property."
```

---

## Phase 3: API Contracts — Make Traits Honest

### Task 7: Add Result returns to PhysicsBackend mutation methods

The void-returning methods silently swallow failures. Start with `remove_body` (the most dangerous — Bryan's `drop_on_hand` walkthrough shows a silent no-op leading to downstream bugs).

**Files:**
- Modify: `crates/engine_physics/src/physics_backend.rs` — trait definition
- Modify: `crates/engine_physics/src/rapier_backend.rs` (or wherever RapierBackend lives) — real implementation
- Modify: `crates/engine_physics/src/null_backend.rs` — NullPhysicsBackend
- Modify: `crates/card_game/src/test_helpers.rs` — SpyPhysicsBackend
- Modify: all call sites of the changed methods

**Step 1: Define PhysicsError**

Add to `engine_physics`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PhysicsError {
    #[error("entity {0:?} not found in physics world")]
    EntityNotFound(Entity),
    #[error("physics operation failed: {0}")]
    OperationFailed(String),
}
```

**Step 2: Change `remove_body` signature**

```rust
// Before:
fn remove_body(&mut self, entity: Entity);

// After:
fn remove_body(&mut self, entity: Entity) -> Result<(), PhysicsError>;
```

Update all three implementations:
- `RapierBackend`: return `Err(PhysicsError::EntityNotFound(entity))` when entity not in map
- `NullPhysicsBackend`: return `Ok(())`
- `SpyPhysicsBackend`: push to log and return `Ok(())`

**Step 3: Change `set_linear_velocity`, `set_angular_velocity`, `set_damping`, `set_collision_group` signatures**

Same pattern — return `Result<(), PhysicsError>`. These all operate on entities that may not exist in the physics world.

```rust
fn set_linear_velocity(&mut self, entity: Entity, velocity: Vec2) -> Result<(), PhysicsError>;
fn set_angular_velocity(&mut self, entity: Entity, angular_velocity: f32) -> Result<(), PhysicsError>;
fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32) -> Result<(), PhysicsError>;
fn set_collision_group(&mut self, entity: Entity, membership: u32, filter: u32) -> Result<(), PhysicsError>;
fn add_force_at_point(&mut self, entity: Entity, force: Vec2, world_point: Vec2) -> Result<(), PhysicsError>;
```

**Step 4: Fix all call sites**

Search for all callers of these methods across the codebase. For each call site, decide:
- **Game logic (card_game)**: Use `.expect("reason")` for true invariants, or handle the error with a fallback (like `drop_on_hand` already does for `hand.add()`). Log-and-continue is acceptable for non-critical operations.
- **Engine systems**: Propagate or log-and-continue.
- **Tests**: Use `.unwrap()` (tests have `#[allow(clippy::unwrap_used)]`).
- **Binary (card_game_bin)**: Use `.expect()` for setup code.

**Step 5: Run full test suite**

Run: `cargo.exe test`
Expected: All tests pass (may need to update spy-based assertions that previously asserted void calls).

**Step 6: Commit**

```bash
git add crates/engine_physics/ crates/card_game/ crates/card_game_bin/
git commit -m "feat(engine_physics): return Result from PhysicsBackend mutation methods

remove_body, set_linear_velocity, set_angular_velocity, set_damping,
set_collision_group, and add_force_at_point now return Result<(), PhysicsError>.
Eliminates the silent-failure class for physics operations."
```

---

### Task 8: Add Result returns to critical Renderer methods

The Renderer trait has 16 void-returning methods. Don't change all of them — focus on the methods where silent failure has concrete downstream consequences.

**Files:**
- Modify: `crates/engine_render/src/renderer.rs` — trait definition
- Modify: `crates/engine_render/src/wgpu_renderer/` — WgpuRenderer implementation
- Modify: `crates/engine_render/src/testing.rs` — SpyRenderer + NullRenderer
- Modify: call sites

**Step 1: Define RenderError (if not already present)**

```rust
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("atlas upload failed: {0}")]
    AtlasUpload(String),
    #[error("shader compilation failed: {0}")]
    ShaderCompilation(String),
    #[error("surface error: {0}")]
    Surface(String),
}
```

**Step 2: Change `upload_atlas` and `compile_shader` signatures**

These are the methods where silent failure is most dangerous:

```rust
// Before:
fn upload_atlas(&mut self, atlas: &TextureAtlas);
fn compile_shader(&mut self, handle: ShaderHandle, source: &str);

// After:
fn upload_atlas(&mut self, atlas: &TextureAtlas) -> Result<(), RenderError>;
fn compile_shader(&mut self, handle: ShaderHandle, source: &str) -> Result<(), RenderError>;
```

Leave the draw methods (`draw_rect`, `draw_sprite`, `draw_shape`) as void for now — they are called per-frame in hot loops, and the failure mode (dropped draw call) is visually obvious.

**Step 3: Fix all call sites and implementations**

- `NullRenderer`: return `Ok(())`
- `SpyRenderer`: return `Ok(())`
- `WgpuRenderer`: return actual errors
- `upload_atlas_system`: handle or propagate the Result
- Plugin/setup code calling `compile_shader`: use `.expect()` for startup-time compilation

**Step 4: Run full test suite**

Run: `cargo.exe test`

**Step 5: Commit**

```bash
git add crates/engine_render/ crates/card_game/ crates/card_game_bin/ crates/axiom2d/ crates/demo/
git commit -m "feat(engine_render): return Result from upload_atlas and compile_shader

Eliminates silent failure for atlas upload and shader compilation.
Draw methods remain void — their failures are visually obvious."
```

---

### Task 9: Add minimal tracing at hardware boundaries

The codebase has zero runtime instrumentation. Add the `tracing` crate and instrument the hardware trait implementations (not the traits themselves) at critical points.

**Files:**
- Modify: root `Cargo.toml` — add `tracing` workspace dependency
- Modify: `crates/engine_audio/Cargo.toml` — add tracing dep
- Modify: `crates/engine_render/Cargo.toml` — add tracing dep
- Modify: `crates/engine_physics/Cargo.toml` — add tracing dep
- Modify: `crates/engine_audio/src/backend/cpal.rs` — instrument CpalBackend
- Modify: `crates/engine_render/src/wgpu_renderer/` — instrument surface errors
- Modify: `crates/engine_physics/src/rapier_backend.rs` — instrument step timing

**Step 1: Add tracing workspace dependency**

In root `Cargo.toml` under `[workspace.dependencies]`:
```toml
tracing = "0.1"
```

Add `tracing.workspace = true` to the three crates' `[dependencies]`.

**Step 2: Instrument CpalBackend**

Replace the `eprintln!` calls with `tracing::error!`. Add `tracing::warn!` when:
- No output device found
- Sample format isn't F32
- Stream build fails
- Stream play fails

Add `tracing::info!` when audio stream opens successfully.

```rust
// In open_stream():
let host = cpal::default_host();
let device = host.default_output_device();
let Some(device) = device else {
    tracing::warn!("no audio output device found — audio will be silent");
    return None;
};
```

**Step 3: Instrument WgpuRenderer surface errors**

Replace the `eprintln!` with `tracing::error!`. Add `tracing::debug!` for resize events and atlas uploads.

**Step 4: Instrument RapierBackend**

Add `tracing::warn!` when operations target unknown entities (now returning `Err`). Add `tracing::debug!` for physics step timing (optional, only if frame timing debugging is needed).

**Step 5: Wire up subscriber in card_game_bin**

In `main.rs`, before `app.run()`:
```rust
tracing_subscriber::fmt()
    .with_env_filter("warn")  // default to warn, RUST_LOG=debug for more
    .init();
```

Add `tracing-subscriber` to `card_game_bin/Cargo.toml`.

**Step 6: Run full test suite and build**

Run: `cargo.exe test && cargo.exe build -p card_game_bin`

**Step 7: Commit**

```bash
git add Cargo.toml crates/engine_audio/ crates/engine_render/ crates/engine_physics/ crates/card_game_bin/
git commit -m "feat: add tracing instrumentation at hardware boundaries

CpalBackend, WgpuRenderer, and RapierBackend now emit tracing events for
failures and significant operations. card_game_bin initializes a subscriber."
```

---

## Phase 4: Architecture Prep — Set Up Patterns for New Code

### Task 10: Introduce ZoneConfig reconciliation pattern

Don't rewrite existing `drop_on_*` functions — they work. Instead, create the `ZoneConfig` pattern as the recommended approach for any new zone-related code. This lets us evaluate the approach before migrating existing code.

**Files:**
- Create: `crates/card_game/src/card/zone_config.rs`
- Modify: `crates/card_game/src/card/mod.rs` — add `pub mod zone_config;`
- Modify: `crates/card_game/src/lib.rs` — re-export if needed

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::zone::CardZone;

    #[test]
    fn when_zone_is_hand_then_config_has_no_physics_and_ui_layer() {
        let config = ZoneConfig::for_zone(&CardZone::Hand(0));

        assert!(!config.has_physics);
        assert_eq!(config.render_layer, RenderLayer::UI);
        assert!(!config.has_item_form);
    }

    #[test]
    fn when_zone_is_table_then_config_has_physics_and_world_layer() {
        let config = ZoneConfig::for_zone(&CardZone::Table);

        assert!(config.has_physics);
        assert_eq!(config.render_layer, RenderLayer::World);
        assert!(!config.has_item_form);
    }

    #[test]
    fn when_zone_is_stash_then_config_has_item_form_and_ui_layer() {
        let config = ZoneConfig::for_zone(&CardZone::Stash { page: 0, col: 0, row: 0 });

        assert!(!config.has_physics);
        assert_eq!(config.render_layer, RenderLayer::UI);
        assert!(config.has_item_form);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo.exe test -p card_game -- zone_config`
Expected: FAIL — module doesn't exist yet.

**Step 3: Implement ZoneConfig**

```rust
use engine_scene::prelude::RenderLayer;
use crate::card::zone::CardZone;

/// Describes the properties a card should have when in a given zone.
/// Used by reconciliation logic to compute the delta between current
/// state and desired state after a zone transition.
///
/// This is a pure data description — no ECS types, no Commands, no side effects.
pub struct ZoneConfig {
    pub has_physics: bool,
    pub render_layer: RenderLayer,
    pub has_item_form: bool,
}

impl ZoneConfig {
    pub fn for_zone(zone: &CardZone) -> Self {
        match zone {
            CardZone::Table => Self {
                has_physics: true,
                render_layer: RenderLayer::World,
                has_item_form: false,
            },
            CardZone::Hand(_) => Self {
                has_physics: false,
                render_layer: RenderLayer::UI,
                has_item_form: false,
            },
            CardZone::Stash { .. } => Self {
                has_physics: false,
                render_layer: RenderLayer::UI,
                has_item_form: true,
            },
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo.exe test -p card_game -- zone_config`
Expected: PASS.

**Step 5: Document the pattern**

Add a doc comment to the module explaining that this is the recommended approach for new zone-related logic, and that existing `drop_on_*` functions will be migrated incrementally.

**Step 6: Commit**

```bash
git add crates/card_game/src/card/zone_config.rs crates/card_game/src/card/mod.rs
git commit -m "feat(card_game): introduce ZoneConfig data-driven zone transition pattern

Pure data description of per-zone properties (physics, render layer, item form).
Recommended approach for new zone logic — existing drop_on_* functions unchanged."
```

---

## Summary

| Phase | Tasks | What it achieves |
|-------|-------|-----------------|
| 1. Rules | T1 (CLAUDE.md), T2 (release profile) | Prevents regressions during cleanup |
| 2. Tests | T3-T6 (delete ~28 bad tests, refactor release.rs, add 5 e2e tests) | Replaces false confidence with real confidence |
| 3. Contracts | T7-T9 (Result returns, tracing) | Eliminates silent failure class |
| 4. Architecture | T10 (ZoneConfig) | Establishes data-driven pattern for future work |

**Estimated test count change:** Start ~1255 total, ~357 card_game. Delete ~28 framework-guarantee tests, collapse ~12 single-assertion families into ~3 combined tests, add ~5 end-to-end tests. Net: ~-32 card_game tests, but the remaining tests are dramatically more valuable.

**What this plan does NOT do** (deliberate trade-offs from the debate):
- Does not remove bevy_ecs — contracts and tests matter more than framework choice right now
- Does not rewrite `drop_on_*` functions — they work, and we lack e2e tests to catch regressions (Task 6 fixes that, future migration can follow)
- Does not reduce crate count or public API surface — low-priority given stable structure
- Does not address gameplay design — that's the "serious design phase" that comes after this cleanup
