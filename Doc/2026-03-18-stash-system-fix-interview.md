# Stash System Fix — Requirements Spec

> **For Claude:** This spec was produced by /interview. Use /writing-plans to expand into an implementation plan, or /tdd to implement directly.

**Goal:** Fix multiple visual bugs in the stash system and implement correct cursor-follow drag mechanics for cards originating from the stash.

**Date:** 2026-03-18

---

## Requirements

### Bug Fixes

**1. Render order**
`stash_render_system` runs after `shape_render_system`, so the stash background rect is drawn on top of card icon shapes. Fix: the stash background (and empty slot rects) must be drawn before shape_render_system runs so card icons appear on top of the grid. One approach: split stash rendering into two systems — `stash_background_render_system` (Phase::Render, before shape_render_system) and keep `stash_render_system` for any overlays. Or use a RenderLayer ordering trick. The stash grid background must never cover rendered card icons.

**2. Card not centered in slot**
`stash_layout_system` computes slot screen position as `GRID_MARGIN + col * SLOT_STRIDE_W` and `GRID_MARGIN + row * SLOT_STRIDE_H`. This places the card's center at the slot's top-left corner. Fix: add `SLOT_WIDTH * 0.5` to the x and `SLOT_HEIGHT * 0.5` to the y so the card is centered within its slot.

**3. Card icon too large**
`StashIcon` in `spawn_visual_card` is spawned with half-extents matching the full card dimensions `(CARD_WIDTH*0.5, CARD_HEIGHT*0.5) = (30, 45)`. The icon should match slot dimensions. Fix: spawn `StashIcon` with half-extents `(SLOT_WIDTH*0.5, SLOT_HEIGHT*0.5) = (25, 37.5)`.

**4. Rotation not reset on stash entry**
When `drop_on_stash` is called, the card's `Transform2D.rotation` is not reset. A card that was spinning on the table will appear rotated in the stash slot. Fix: in `drop_on_stash`, insert a `Transform2D` update that sets `.rotation = 0.0` (preserving position and scale).

### Slot Dimension Change

Change slots from 50×50 square to a shape matching the 2:3 card aspect ratio:

```
SLOT_WIDTH:    50.0   (was SLOT_SIZE: 50.0)
SLOT_HEIGHT:   75.0   (new)
SLOT_GAP:       4.0   (unchanged)
SLOT_STRIDE_W: 54.0   (SLOT_WIDTH + SLOT_GAP)
SLOT_STRIDE_H: 79.0   (SLOT_HEIGHT + SLOT_GAP)
```

Update `stash_render_system` to use `SLOT_WIDTH`/`SLOT_HEIGHT` when drawing slot rects and the background rect. Update `stash_layout_system` centering offsets. Update `find_stash_slot_at` hit-testing to use the new dimensions.

### Drag Behavior: Card Originating from Stash

When the player picks a card from a stash slot, it enters **cursor-follow mode** rather than physics-spring mode.

**Cursor-follow mode (over stash):**
- No `RigidBody` or physics body is created when the card is picked from stash.
- Each frame, the card's `Transform2D.position` is set directly to `mouse.world_pos()`.
- Card is shown in `CardItemForm` (slot-icon appearance).
- Card scale = slot scale (`Vec2::splat(SLOT_WIDTH / CARD_WIDTH)` ≈ 0.833 — see note on scale below).

**Physics activation (cursor exits stash area while dragging a stash card):**
- A `RigidBody::Dynamic` physics body is added via `physics.add_body`.
- Initial velocity is set from recent cursor movement (cursor delta × some factor, or cursor velocity estimate).
- Spring drag takes over (existing `card_drag_system`).
- A scale animation begins: card grows from slot scale → `Vec2::ONE` over ~0.2s.

**Cursor-follow resume (cursor re-enters stash area during drag with physics active):**
- Physics body is removed (`physics.remove_body`, `remove::<RigidBody>()`).
- Cursor-follow resumes: `Transform2D.position = mouse.world_pos()`.
- A scale animation begins: card shrinks from current scale → slot scale over ~0.2s.

**Track stash-drag state:**
`DragInfo` needs a field `stash_cursor_follow: bool` (true = cursor-follow mode, false = physics mode). Card drag system branches on this flag:
- `stash_cursor_follow = true`: set `Transform2D.position = mouse.world_pos()` directly; no spring force.
- `stash_cursor_follow = false`: existing spring drag logic.

A new system (or extended card_drag_system) detects the cursor crossing the stash boundary and performs the cursor-follow ↔ physics transition.

**Note on scale:** `ScaleSpring` (from `engine_core::scale_spring`, re-exported in `crate::scale_spring`) is already used in the codebase for smooth scale transitions. The stash grow/shrink animation should use `ScaleSpring` with target `Vec2::splat(SLOT_WIDTH / CARD_WIDTH)` (enter stash) or `Vec2::ONE` (leave stash). The existing `sync_scale_spring_lock_x` system already coordinates with `FlipAnimation`.

### Drag Behavior: Card Originating from Table or Hand

Unchanged: spring physics continues as normal. When the cursor enters the stash area:
- `stash_drag_hover_system` inserts `CardItemForm` (existing behavior — correct).
- On release over a stash slot, `drop_on_stash` snaps the card to the slot (existing behavior — correct after bug fixes above).

---

## Subtask Checklist

- [ ] **S1 — Change slot dimensions to card aspect ratio**: Replace `SLOT_SIZE: f32 = 50.0` with `SLOT_WIDTH: f32 = 50.0` and `SLOT_HEIGHT: f32 = 75.0`. Add `SLOT_STRIDE_W: f32 = 54.0` and `SLOT_STRIDE_H: f32 = 79.0`. Update all usages in `stash_render.rs`, `stash_layout.rs`, and `find_stash_slot_at` in `card_pick.rs`. Update tests that hard-code slot positions/sizes.

- [ ] **S2 — Fix StashIcon size**: In `spawn_visual_card` (`spawn_table_card.rs`), change StashIcon polygon half-extents from `(half.x, half.y)` (= card half-size) to `(SLOT_WIDTH * 0.5, SLOT_HEIGHT * 0.5)`. Import slot constants from `stash_render`. Update spawn tests that check icon shape dimensions.

- [ ] **S3 — Fix slot centering in stash_layout_system**: In `stash_layout_system`, after converting slot screen position to world coords, the position must be the CENTER of the slot: `screen_x = GRID_MARGIN + col as f32 * SLOT_STRIDE_W + SLOT_WIDTH * 0.5`, `screen_y = GRID_MARGIN + row as f32 * SLOT_STRIDE_H + SLOT_HEIGHT * 0.5`. Update layout tests that assert slot positions.

- [ ] **S4 — Fix render order (background before icons)**: Split `stash_render_system` so the background and empty slot rects are drawn in a new `stash_background_system` that runs before `shape_render_system` in Phase::Render. The existing `stash_render_system` can be removed or repurposed. Update registration in `main.rs` / plugin. Update render tests.

- [ ] **S5 — Reset rotation on stash entry**: In `drop_on_stash` (`card_release.rs`), after updating `CardZone`, reset rotation by inserting a `Transform2D` change that sets `.rotation = 0.0` (keep position from current transform or set from slot position). Update card_release tests to assert rotation is 0 after stash drop.

- [ ] **S6 — Add `stash_cursor_follow` flag to DragInfo**: Add `stash_cursor_follow: bool` to `DragInfo` (`drag_state.rs`). Set to `true` when picking from stash (in `card_pick_system`), `false` otherwise. When picking from stash, do NOT add a physics body (remove `physics.add_body` call for stash origin). Update all DragInfo construction sites and tests.

- [ ] **S7 — Cursor-follow drag for stash cards**: In `card_drag_system`, branch on `drag_info.stash_cursor_follow`: if `true`, set `Transform2D.position = mouse.world_pos()` directly (no spring force applied). If `false`, existing spring logic runs unchanged. Add tests: cursor-follow positions card at cursor, no physics force applied when stash_cursor_follow is true.

- [ ] **S8 — Stash boundary transition system** (`stash_boundary_system`, Phase::Update, after card_drag_system): While dragging with `stash_cursor_follow = true` and cursor exits stash → add physics body with velocity estimate, set `stash_cursor_follow = false`, insert `ScaleSpring` targeting `Vec2::ONE`. While dragging with `stash_cursor_follow = false` and cursor enters stash → remove physics body, set `stash_cursor_follow = true`, insert/update `ScaleSpring` targeting `Vec2::splat(SLOT_WIDTH / CARD_WIDTH)`. Add tests for both transitions.

- [ ] **S9 — Reset scale on stash entry/exit**: Ensure `ScaleSpring` target is set correctly on stash transitions (S8). On `drop_on_stash`, reset `Transform2D.scale = Vec2::ONE` (the slot icon is already the right size; scale is for the "live card" mode). On stash pick-up, insert `ScaleSpring` at slot scale. Ensure `sync_scale_spring_lock_x` is registered and runs after the scale spring system.

- [ ] **S10 — Wire new systems and update registration in main.rs/plugin**: Register `stash_background_system` (Phase::Render, before shape_render_system), `stash_boundary_system` (Phase::Update, after card_drag_system). Update system ordering comments. Run `cargo.exe test` and `cargo.exe build`.

---

## Research Notes

- **Slot constants** are currently in `stash_render.rs` (`SLOT_SIZE`, `SLOT_GAP`, `SLOT_STRIDE`, `GRID_MARGIN`, `SLOT_COLOR`, `BACKGROUND_COLOR`). After S1, `SLOT_WIDTH` and `SLOT_HEIGHT` replace `SLOT_SIZE`; `SLOT_STRIDE_W` and `SLOT_STRIDE_H` replace `SLOT_STRIDE`.
- **`find_stash_slot_at`** is in `card_pick.rs` (pub, also used by `card_release.rs`). It currently uses `SLOT_SIZE`/`SLOT_STRIDE` for hit-testing; update to use `SLOT_WIDTH`/`SLOT_HEIGHT`/`SLOT_STRIDE_W`/`SLOT_STRIDE_H`.
- **`ScaleSpring`** lives in `engine_core::scale_spring`, re-exported via `crate::scale_spring`. `sync_scale_spring_lock_x` in `crate::scale_spring` coordinates with `FlipAnimation` (locks x during flip). Use this for stash grow/shrink animation.
- **`drop_on_stash`** is a private fn in `card_release.rs`. It already calls `physics.remove_body` and `remove::<RigidBody>()` — correct. Add rotation reset and scale reset here.
- **`card_pick_system`** in `card_pick.rs` currently adds a physics body when picking from stash. Remove this for stash-origin picks; set `stash_cursor_follow = true` instead.
- **Render system order** in `card_game_bin/src/main.rs`: currently `shape_render_system` runs then `stash_render_system`. After S4, `stash_background_system` must precede `shape_render_system`.

## Open Questions

- Velocity estimate for physics activation on stash exit (S8): simple approach is to track last-two-frame cursor positions in a resource, or use a fixed default velocity of zero (card placed at rest). Confirm which feels right during implementation.
- Whether the stash area grid highlight (showing which slot will be targeted) should be added now or deferred to H2 (polish).
