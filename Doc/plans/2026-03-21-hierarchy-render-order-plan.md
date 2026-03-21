# Hierarchical Render Order Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace stride-based sort propagation with DFS traversal so scene hierarchy determines render order — children of one parent never interleave with children of another.

**Architecture:** A new `hierarchy_sort_system` walks the entire scene graph depth-first, sorting siblings by `LocalSortOrder` at each level, and assigns an incrementing `SortOrder` to every visited entity. This replaces `sort_propagation_system`. Render systems already sort by `(RenderLayer, SortOrder)` and need no changes.

**Tech Stack:** Rust, bevy_ecs (standalone), engine_scene crate, card_game crate.

---

### Task 1: Write `hierarchy_sort_system` with tests

**Files:**
- Modify: `crates/engine_scene/src/sort_propagation.rs` (rewrite the system, keep `LocalSortOrder`)
- Modify: `crates/engine_scene/src/lib.rs:39-41` (update test helper)

**Step 1: Write the failing test — single root gets SortOrder 0**

In `crates/engine_scene/src/sort_propagation.rs`, replace the entire `#[cfg(test)] mod tests` block. Start with one test:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;

    use super::*;
    use crate::hierarchy::ChildOf;
    use crate::test_helpers::run_hierarchy_sort_system as run_system;

    fn run_with_hierarchy(world: &mut World) {
        crate::test_helpers::run_hierarchy_system(world);
        run_system(world);
    }

    #[test]
    fn when_single_root_then_sort_order_is_zero() {
        // Arrange
        let mut world = World::new();
        let root = world.spawn(SortOrder(99)).id();

        // Act
        run_with_hierarchy(&mut world);

        // Assert
        assert_eq!(world.entity(root).get::<SortOrder>().unwrap().0, 0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo.exe test -p engine_scene when_single_root_then_sort_order_is_zero`
Expected: FAIL — `run_hierarchy_sort_system` does not exist yet.

**Step 3: Write minimal `hierarchy_sort_system`**

In `crates/engine_scene/src/sort_propagation.rs`, replace `sort_propagation_system` and `SORT_STRIDE` with:

```rust
use bevy_ecs::prelude::{Component, Entity, Query, With, Without};
use serde::{Deserialize, Serialize};

use crate::hierarchy::{ChildOf, Children};
use crate::render_order::SortOrder;

/// Sibling order within a parent. Controls position among siblings during
/// depth-first traversal. Lower values render first (behind higher values).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalSortOrder(pub i32);

/// Walks the scene graph depth-first, sorting siblings by `LocalSortOrder`,
/// and assigns an incrementing `SortOrder` to each entity visited.
pub fn hierarchy_sort_system(
    roots: Query<(Entity, Option<&LocalSortOrder>), (With<SortOrder>, Without<ChildOf>)>,
    children_query: Query<&Children>,
    local_sort_query: Query<Option<&LocalSortOrder>>,
    mut sort_query: Query<&mut SortOrder>,
) {
    let mut root_list: Vec<_> = roots.iter().collect();
    root_list.sort_by_key(|(_, local)| local.map_or(0, |l| l.0));

    let mut counter: i32 = 0;
    for (entity, _) in &root_list {
        assign_sort(*entity, &mut counter, &children_query, &local_sort_query, &mut sort_query);
    }
}

fn assign_sort(
    entity: Entity,
    counter: &mut i32,
    children_query: &Query<&Children>,
    local_sort_query: &Query<Option<&LocalSortOrder>>,
    sort_query: &mut Query<&mut SortOrder>,
) {
    if let Ok(mut sort) = sort_query.get_mut(entity) {
        sort.0 = *counter;
    }
    *counter += 1;

    if let Ok(children) = children_query.get(entity) {
        let mut sorted_children: Vec<Entity> = children.0.clone();
        sorted_children.sort_by_key(|&child| {
            local_sort_query
                .get(child)
                .ok()
                .flatten()
                .map_or(0, |l| l.0)
        });
        for child in sorted_children {
            assign_sort(child, counter, children_query, local_sort_query, sort_query);
        }
    }
}
```

Also update the test helper in `crates/engine_scene/src/lib.rs` — rename `run_sort_propagation_system` to `run_hierarchy_sort_system`:

```rust
pub(crate) fn run_hierarchy_sort_system(world: &mut World) {
    run_system(world, crate::sort_propagation::hierarchy_sort_system);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo.exe test -p engine_scene when_single_root_then_sort_order_is_zero`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_scene/src/sort_propagation.rs crates/engine_scene/src/lib.rs
git commit -m "feat(engine_scene): replace sort_propagation with DFS hierarchy_sort_system"
```

---

### Task 2: Add remaining hierarchy_sort tests

**Files:**
- Modify: `crates/engine_scene/src/sort_propagation.rs` (add tests to existing test module)

**Step 1: Add all remaining tests to the test module**

Append these tests inside the `mod tests` block:

```rust
#[test]
fn when_two_roots_then_sorted_by_local_sort_order() {
    // Arrange
    let mut world = World::new();
    let a = world.spawn((SortOrder(0), LocalSortOrder(1))).id();
    let b = world.spawn((SortOrder(0), LocalSortOrder(0))).id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert — b has lower LocalSortOrder, so gets 0; a gets 1
    assert_eq!(world.entity(b).get::<SortOrder>().unwrap().0, 0);
    assert_eq!(world.entity(a).get::<SortOrder>().unwrap().0, 1);
}

#[test]
fn when_parent_with_children_then_dfs_order_parent_child_a_child_b() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(SortOrder(0)).id();
    let child_a = world
        .spawn((ChildOf(parent), LocalSortOrder(0), SortOrder(0)))
        .id();
    let child_b = world
        .spawn((ChildOf(parent), LocalSortOrder(1), SortOrder(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    let p = world.entity(parent).get::<SortOrder>().unwrap().0;
    let a = world.entity(child_a).get::<SortOrder>().unwrap().0;
    let b = world.entity(child_b).get::<SortOrder>().unwrap().0;
    assert_eq!(p, 0);
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn when_two_parents_with_children_then_no_interleaving() {
    // Arrange
    let mut world = World::new();
    let card_a = world.spawn((SortOrder(0), LocalSortOrder(0))).id();
    let a_border = world
        .spawn((ChildOf(card_a), LocalSortOrder(1), SortOrder(0)))
        .id();
    let a_art = world
        .spawn((ChildOf(card_a), LocalSortOrder(2), SortOrder(0)))
        .id();

    let card_b = world.spawn((SortOrder(0), LocalSortOrder(1))).id();
    let b_border = world
        .spawn((ChildOf(card_b), LocalSortOrder(1), SortOrder(0)))
        .id();
    let b_art = world
        .spawn((ChildOf(card_b), LocalSortOrder(2), SortOrder(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert — DFS: card_a(0), a_border(1), a_art(2), card_b(3), b_border(4), b_art(5)
    let sorts: Vec<i32> = [card_a, a_border, a_art, card_b, b_border, b_art]
        .iter()
        .map(|&e| world.entity(e).get::<SortOrder>().unwrap().0)
        .collect();
    assert_eq!(sorts, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn when_grandchildren_then_dfs_visits_recursively() {
    // Arrange
    let mut world = World::new();
    let root = world.spawn(SortOrder(0)).id();
    let child = world
        .spawn((ChildOf(root), LocalSortOrder(0), SortOrder(0)))
        .id();
    let grandchild = world
        .spawn((ChildOf(child), LocalSortOrder(0), SortOrder(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    let r = world.entity(root).get::<SortOrder>().unwrap().0;
    let c = world.entity(child).get::<SortOrder>().unwrap().0;
    let g = world.entity(grandchild).get::<SortOrder>().unwrap().0;
    assert_eq!((r, c, g), (0, 1, 2));
}

#[test]
fn when_children_reordered_by_local_sort_then_sort_order_reflects_new_order() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(SortOrder(0)).id();
    let child_a = world
        .spawn((ChildOf(parent), LocalSortOrder(1), SortOrder(0)))
        .id();
    let child_b = world
        .spawn((ChildOf(parent), LocalSortOrder(0), SortOrder(0)))
        .id();
    run_with_hierarchy(&mut world);
    // b=1, a=2 initially

    // Act — swap order: a goes first
    world.entity_mut(child_a).get_mut::<LocalSortOrder>().unwrap().0 = 0;
    world.entity_mut(child_b).get_mut::<LocalSortOrder>().unwrap().0 = 1;
    run_with_hierarchy(&mut world);

    // Assert
    let a = world.entity(child_a).get::<SortOrder>().unwrap().0;
    let b = world.entity(child_b).get::<SortOrder>().unwrap().0;
    assert!(a < b, "a ({a}) should sort before b ({b})");
}

#[test]
fn when_entity_has_no_local_sort_then_treated_as_zero() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(SortOrder(0)).id();
    let child_no_local = world
        .spawn((ChildOf(parent), SortOrder(0)))
        .id();
    let child_with_local = world
        .spawn((ChildOf(parent), LocalSortOrder(1), SortOrder(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert — no LocalSortOrder = 0, so renders before LocalSortOrder(1)
    let no_local = world.entity(child_no_local).get::<SortOrder>().unwrap().0;
    let with_local = world.entity(child_with_local).get::<SortOrder>().unwrap().0;
    assert!(no_local < with_local);
}

#[test]
fn when_root_has_no_sort_order_then_not_visited() {
    // Arrange — entity without SortOrder should be ignored
    let mut world = World::new();
    let _no_sort = world.spawn_empty().id();
    let with_sort = world.spawn(SortOrder(99)).id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    assert_eq!(world.entity(with_sort).get::<SortOrder>().unwrap().0, 0);
}
```

**Step 2: Run all tests**

Run: `cargo.exe test -p engine_scene`
Expected: All PASS (the implementation from Task 1 already handles these cases).

**Step 3: Commit**

```bash
git add crates/engine_scene/src/sort_propagation.rs
git commit -m "test(engine_scene): add comprehensive DFS hierarchy sort tests"
```

---

### Task 3: Update prelude and remove `SORT_STRIDE`

**Files:**
- Modify: `crates/engine_scene/src/prelude.rs:3`
- Modify: `crates/engine_scene/src/lib.rs:39-41` (keep old helper as deprecated alias if needed)

**Step 1: Update the prelude export**

In `crates/engine_scene/src/prelude.rs`, change line 3 from:
```rust
pub use crate::sort_propagation::{LocalSortOrder, SORT_STRIDE, sort_propagation_system};
```
to:
```rust
pub use crate::sort_propagation::{LocalSortOrder, hierarchy_sort_system};
```

**Step 2: Verify engine_scene builds**

Run: `cargo.exe build -p engine_scene`
Expected: PASS (no external consumers of `SORT_STRIDE` or `sort_propagation_system` within this crate).

**Step 3: Commit**

```bash
git add crates/engine_scene/src/prelude.rs
git commit -m "refactor(engine_scene): export hierarchy_sort_system, remove SORT_STRIDE from prelude"
```

---

### Task 4: Update card_game to use `hierarchy_sort_system`

**Files:**
- Modify: `crates/card_game/src/plugin.rs:30,83`
- Modify: `crates/card_game/src/prelude.rs:35`

**Step 1: Update plugin.rs imports and registration**

In `crates/card_game/src/plugin.rs`:

Change line 30 from:
```rust
use engine_scene::sort_propagation::sort_propagation_system;
```
to:
```rust
use engine_scene::sort_propagation::hierarchy_sort_system;
```

Change line 83 from:
```rust
            sort_propagation_system,
```
to:
```rust
            hierarchy_sort_system,
```

**Step 2: Update card_game prelude**

In `crates/card_game/src/prelude.rs`, change line 35 from:
```rust
pub use engine_scene::sort_propagation::sort_propagation_system;
```
to:
```rust
pub use engine_scene::sort_propagation::hierarchy_sort_system;
```

**Step 3: Build the full workspace**

Run: `cargo.exe build`
Expected: PASS. If `card_game_bin` or `demo` reference `sort_propagation_system` directly, fix those too.

**Step 4: Run all tests**

Run: `cargo.exe test`
Expected: All PASS.

**Step 5: Commit**

```bash
git add crates/card_game/src/plugin.rs crates/card_game/src/prelude.rs
git commit -m "refactor(card_game): switch from sort_propagation_system to hierarchy_sort_system"
```

---

### Task 5: Remove old `sort_propagation_system` and `SORT_STRIDE`

**Files:**
- Modify: `crates/engine_scene/src/sort_propagation.rs` (remove dead code)
- Modify: `crates/engine_scene/src/lib.rs` (remove old test helper alias if present)

**Step 1: Remove the old function and constant**

In `crates/engine_scene/src/sort_propagation.rs`, delete the `sort_propagation_system` function and `SORT_STRIDE` constant (they should be unused after Task 4).

**Step 2: Verify the workspace builds and tests pass**

Run: `cargo.exe build && cargo.exe test`
Expected: PASS. If anything still references them, fix the reference.

**Step 3: Run clippy**

Run: `cargo.exe clippy --workspace`
Expected: No new warnings.

**Step 4: Commit**

```bash
git add crates/engine_scene/src/sort_propagation.rs crates/engine_scene/src/lib.rs
git commit -m "chore(engine_scene): remove dead sort_propagation_system and SORT_STRIDE"
```

---

### Task 6: Format and final verification

**Step 1: Format all code**

Run: `cargo.exe fmt --all`

**Step 2: Run full test suite**

Run: `cargo.exe test`
Expected: All pass.

**Step 3: Build the binary**

Run: `cargo.exe build -p card_game_bin`
Expected: PASS — hierarchy_sort_system is registered in the plugin, so it runs in the game.

**Step 4: Commit if formatting changed anything**

```bash
git add -A && git commit -m "style: format after hierarchy sort refactor"
```
