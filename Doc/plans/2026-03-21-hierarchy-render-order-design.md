# Hierarchical Render Order via DFS Traversal

**Date:** 2026-03-21
**Status:** Approved

## Problem

All card roots spawn with `SortOrder(0)`. The existing `sort_propagation_system` computes `child_sort = parent_sort * SORT_STRIDE + local_sort`, but with identical parent values, children of different cards get the same effective sort values and interleave during rendering.

## Design

Adopt Unity-style scene hierarchy ordering: the depth-first traversal order of the scene graph determines render order. Every entity lives in the hierarchy. A single DFS walk assigns an incrementing `SortOrder` to each entity visited.

### Core Mechanism

`hierarchy_sort_system` replaces `sort_propagation_system`:

1. Collect root entities (no `ChildOf`), sorted by `LocalSortOrder` (default 0).
2. DFS walk — at each level, siblings ordered by `LocalSortOrder`.
3. Assign incrementing counter to each visited entity's `SortOrder`.

Example:
```
Hand (LocalSortOrder 0)
  CardA (LocalSortOrder 0)
    Border (LocalSortOrder 1)
    Art (LocalSortOrder 2)
  CardB (LocalSortOrder 1)
    Border (LocalSortOrder 1)
    Art (LocalSortOrder 2)
```
DFS assigns: Hand=0, CardA=1, Border=2, Art=3, CardB=4, Border=5, Art=6.
CardA's children (2-3) always render between CardA(1) and CardB(4). Interleaving impossible.

### Semantics

- `LocalSortOrder` = sibling index. Controls position among siblings. Default 0.
- `SortOrder` = computed global render order. Game code should not set this directly on entities in the hierarchy.
- `RenderLayer` = primary sort key (unchanged). Hierarchy ordering only affects `SortOrder` (secondary key).
- Entities without `LocalSortOrder` are treated as `LocalSortOrder(0)`.
- Roots (no `ChildOf`) are implicit top-level siblings.

### Bringing a card to front

Set its `LocalSortOrder` higher than its siblings. The next DFS pass recomputes all `SortOrder` values. Zone systems set base ordering; interaction (drag/click) overrides within zone by bumping `LocalSortOrder`.

## Scope of Changes

### engine_scene
- **New:** `hierarchy_sort_system` — DFS walk assigning `SortOrder` from traversal counter.
- **Remove:** `sort_propagation_system` (replaced, not modified).
- **Unchanged:** `hierarchy_maintenance_system`, `Children` vec ordering (Entity-ID sorted). DFS sorts by `LocalSortOrder` during traversal without mutating the vec.
- **Unchanged:** `LocalSortOrder` component definition (semantics shift from "stride offset" to "sibling order").

### engine_render
- **No changes.** Render systems sort by `(RenderLayer, SortOrder)` — they get correct values from the hierarchy system.

### card_game
- Replace `sort_propagation_system` registration with `hierarchy_sort_system`.
- Zone/interaction systems set `LocalSortOrder` on card roots to control front-to-back ordering.

### Backward compatibility
- Entities without `LocalSortOrder` default to 0 during DFS — existing code keeps working.
- `SORT_STRIDE` constant becomes unused and can be removed.
