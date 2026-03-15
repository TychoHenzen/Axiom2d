# Card Game Implementation Roadmap

This document tracks the implementation of a card game built on the Axiom2d engine. The core gameplay mechanic involves manipulating physical playing cards on a table surface, with inventory management for hand and stash storage.

## Core Gameplay

- **Table area**: Cards are Dynamic physics bodies on a poker table with friction (damping). Grab anywhere on a card to drag it — a spring force pulls the grab point toward the cursor, creating natural torque so the card rotates to trail behind the movement direction. On release, the physics engine continues with the card's current velocity and angular velocity — no artificial impulse needed.
- **Card flip**: Right-click a card to flip it over (local-space scale.x tween animation).
- **Hand inventory**: Bottom of screen, Uno-style. Cards in hand are not affected by physics. Drag cards between table and hand.
- **Stash inventory**: ARPG-style grid (10x10). Cards represented as inventory icons. Hover shows a card face preview near the cursor. Multiple pages (buy more later). Drag cards between stash, hand, and table.

## Physics Model — Drag & Flick

The drag system uses a **spring force at grab point** approach:

1. **Grab**: On click, record the grab point as a **local-space offset** from the card's center of mass.
2. **Drag**: Each frame, compute the grab point's current world position from body position + rotation + local offset. Apply a spring force (`stiffness * (cursor_pos - grab_world_pos)`) at that world point. Because the force is off-center, it creates torque — the card naturally rotates to align its center of mass with the direction of movement, with inertia.
3. **Release**: Stop applying the spring force. The card already has the correct linear velocity and angular velocity from the physics simulation — it continues gliding and spinning across the table, decelerating via damping (table friction).

No kinematic body switching. No velocity tracking. No impulse on release. The physics engine handles everything.

---

## Phase A: Engine Extensions

These extend `engine_physics` with capabilities the card game requires.

### Step A1 — Add Force at Point `[DONE]`
**Crate:** engine_physics
**Why:** The core drag mechanic. Applying a spring force at an off-center grab point each frame creates both linear acceleration and torque, producing the natural card-trailing-behind-cursor behavior.

- [x] Add `fn add_force_at_point(&mut self, entity: Entity, force: Vec2, world_point: Vec2)` to `PhysicsBackend` trait
- [x] Implement on `RapierBackend`: look up body handle, wake body, call rapier's `add_force_at_point(force, point, true)`
- [x] Implement on `NullPhysicsBackend`: no-op
- [x] Tests: force at center produces linear motion without rotation, force off-center produces rotation, unknown entity is no-op, zero force is no-op

### Step A2 — Configure Damping `[DONE]`
**Crate:** engine_physics
**Why:** Cards on the table need linear and angular damping to simulate table friction (poker felt). Without damping, cards slide forever. Must be configurable per body.

- [x] Add `fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32)` to `PhysicsBackend` trait
- [x] Implement on `RapierBackend`: look up body handle, set `linear_damping` and `angular_damping`
- [x] Implement on `NullPhysicsBackend`: no-op
- [x] Tests: high damping body stops faster than low damping body (step N times, compare positions), unknown entity is no-op

### Step A3 — Body World Transform Query `[DONE]`
**Crate:** engine_physics
**Why:** During drag, we need the body's current position AND rotation to compute where the local-space grab offset is in world space. `body_position` and `body_rotation` already exist on the trait, but we need to verify they work together for this use case. If they don't provide sufficient precision mid-step, add a combined query.

- [x] Verify existing `body_position(entity)` and `body_rotation(entity)` return current values after `step()`
- [x] Add `fn body_point_to_world(&self, entity: Entity, local_point: Vec2) -> Option<Vec2>` convenience method — computes `body_pos + rotate(local_point, body_rotation)`
- [x] Default trait implementation using `body_position` + `body_rotation` (no per-backend override needed unless rapier provides a more precise native method)
- [x] Tests: local origin maps to body position, rotated body transforms local offset correctly, unknown entity returns None

---

## Phase B: Core Card Data Model

### Step B1 — Card Component and Zone `[DONE]`
**Crate:** card_game
**Why:** Foundation data types that every other system references.

- [x] `Card` component: `face_texture: TextureId`, `back_texture: TextureId`, `face_up: bool`
- [x] `CardZone` component enum: `Table`, `Hand(usize)`, `Stash { page: u8, col: u8, row: u8 }`
- [x] Derive Component, Debug, Clone, PartialEq, Serialize, Deserialize
- [x] Tests: zone equality, card default face_up state

### Step B2 — Hand Resource `[DONE]`
**Crate:** card_game
**Why:** Ordered container for cards "in hand" at the bottom of the screen.

- [x] `Hand` resource: `cards: Vec<Entity>`, `max_size: usize`
- [x] Methods: `add(entity) -> Result<usize, HandFull>`, `remove(entity) -> Option<usize>`, `cards() -> &[Entity]`, `len()`, `is_full()`
- [x] `HandFull` error type
- [x] Tests: add returns index, add to full hand errors, remove returns former index, remove unknown returns None, ordering preserved

### Step B3 — StashGrid Resource `[DONE]`
**Crate:** card_game
**Why:** 10x10 paged grid for card storage, ARPG inventory style.

- [x] `StashGrid` resource: `slots: HashMap<(u8, u8, u8), Entity>` (page, col, row key), `width: u8`, `height: u8`, `page_count: u8`, `current_page: u8`
- [x] Page-aware methods: `place(page, col, row, entity) -> Result<(), SlotOccupied>`, `take(page, col, row) -> Option<Entity>`, `get(page, col, row) -> Option<&Entity>`
- [x] `first_empty(page) -> Option<(u8, u8)>` — scans column-major for first unoccupied slot on a page
- [x] `SlotOccupied` error type
- [x] Tests: place and retrieve, place on occupied slot errors, take empties slot, first_empty finds gap, first_empty returns None when page full, out-of-bounds coordinates (15 tests)

---

## Phase C: Drag and Drop Foundation

### Step C1 — DragState Resource `[DONE]`
**Crate:** card_game
**Why:** Central state for the drag-and-drop system.

- [x] `DragState` resource: `dragging: Option<DragInfo>`
- [x] `DragInfo`: `entity: Entity`, `local_grab_offset: Vec2` (body-local-space offset from center to grab point), `origin_zone: CardZone`
- [x] Tests: default is None, start/clear cycle (tested implicitly via C2/C3/C4 system tests)

### Step C2 — Table Card Picking `[DONE]`
**Crate:** card_game
**Why:** Clicking on a card on the table starts a drag. Must pick the topmost (highest SortOrder) card under cursor.

- [x] `card_pick_system` in Phase::Update: on `mouse.just_pressed(MouseButton::Left)`, query all `(Entity, &Card, &CardZone, &GlobalTransform2D, &Sprite, &SortOrder)` where CardZone is Table
- [x] AABB hit test at `mouse.world_pos()` using sprite dimensions centered on GlobalTransform2D position (accounting for rotation)
- [x] Pick entity with highest SortOrder among hits
- [x] On pick: compute `local_grab_offset` by inverse-rotating `(cursor - body_center)` into body-local space, set DragState, bump SortOrder above all others
- [x] Tests: pick topmost when overlapping, miss when cursor outside all cards, no pick when no cards, no pick when already dragging, local_grab_offset computed correctly for rotated card

### Step C3 — Card Drag System (Spring Force) `[DONE]`
**Crate:** card_game
**Why:** While dragging, a spring force at the grab point pulls it toward the cursor. The card stays Dynamic — physics handles the rotation and inertia naturally.

- [x] `card_drag_system` in Phase::Update: while `DragState.dragging.is_some()` and `mouse.pressed(Left)`
- [x] Compute grab point's current world position: `physics.body_point_to_world(entity, local_grab_offset)`
- [x] Compute spring force: `DRAG_STIFFNESS * (cursor_world_pos - grab_world_pos)`
- [x] Apply via `physics.add_force_at_point(entity, force, grab_world_pos)`
- [x] `DRAG_STIFFNESS` as a tunable constant (high = snappy, low = laggy/swingy)
- [x] Tests: force direction is toward cursor, force magnitude proportional to distance, no force when grab point equals cursor, system no-ops when not dragging

### Step C4 — Card Release `[DONE]`
**Crate:** card_game
**Why:** Releasing a dragged card simply stops applying the spring force. The card continues with its current physics velocity and angular velocity.

- [x] `card_release_system` in Phase::Update (after card_drag_system): on `mouse.just_released(Left)` while dragging
- [x] If origin was Table and drop target is Table: just clear DragState — physics continues naturally
- [ ] If dropping onto hand/stash: handle zone transition (see Phase F/G)
- [x] Clear DragState
- [x] Tests: DragState cleared on release, no panic when not dragging, drag state preserved while held, zone unchanged on table release

---

## Phase D: Table Physics Configuration

### Step D1 — Card Body Setup `[DONE]`
**Crate:** card_game
**Why:** Cards need specific physics parameters for poker-table feel.

- [x] `spawn_table_card` helper: creates entity with Card, CardZone::Table, Sprite, Transform2D, RigidBody::Dynamic, Collider::Aabb(half), RenderLayer::World, SortOrder(0)
- [x] After spawn: register with physics backend via `add_body` + `add_collider`
- [x] Configure damping: initial set_damping with BASE_LINEAR_DRAG (8.0) and BASE_ANGULAR_DRAG (5.0), then card_damping_system adjusts per-frame based on angular velocity
- [x] Gravity should be zero (top-down table view) — RapierBackend initialized with `Vec2::ZERO` gravity (deferred to H1 game plugin)
- [x] Tests: physics body registered, collider registered, initial damping applied (3 tests)
- [x] Constants: CARD_WIDTH=60.0, CARD_HEIGHT=90.0 (matching existing half-extents in card_pick tests)

### Step D2 — Camera Drag (replaces Table Boundaries) `[DONE]`
**Crate:** card_game
**Why:** Instead of constraining cards with walls, the camera follows the action. Players pan with right-click drag and zoom with scroll wheel, giving an infinite playspace.

- [x] `CameraDragState` resource with anchor_screen_pos tracking
- [x] `camera_drag_system` in Phase::Update: right-click drag pans Camera2D by delta
- [x] `camera_zoom_system` in Phase::Update: scroll wheel zooms Camera2D with ZOOM_MIN floor
- [x] Tests: 22 tests covering drag start/move/release, zoom in/out/clamping, no-camera edge cases

---

## Phase E: Card Flip

### Step E1 — Flip Detection `[NOT STARTED]`
**Crate:** card_game
**Why:** Right-clicking a table card initiates a flip.

- [ ] `card_flip_system` in Phase::Update: on `mouse.just_pressed(MouseButton::Right)`, hit-test table cards (same AABB logic as C2)
- [ ] On hit: toggle `Card.face_up`, begin flip animation
- [ ] Tests: right-click on card toggles face_up, right-click on empty space does nothing, no flip during drag

### Step E2 — Flip Animation (Scale.x Tween) `[NOT STARTED]`
**Crate:** card_game
**Why:** Smooth visual flip in the card's local space — scale.x shrinks to 0, texture swaps, scale.x grows back.

- [ ] `FlipAnimation` component: `progress: f32` (0.0 → 1.0), `duration: Seconds`, `swapped: bool`
- [ ] `flip_animation_system` in Phase::Update: advance progress by dt/duration each frame
  - progress < 0.5: `scale.x = 1.0 - progress * 2.0` (shrink to 0)
  - At progress crossing 0.5 (first frame past midpoint): swap Sprite.uv_rect between face and back, set `swapped = true`
  - progress >= 0.5: `scale.x = (progress - 0.5) * 2.0` (grow back to 1)
  - At progress >= 1.0: remove FlipAnimation component, ensure scale.x = 1.0
- [ ] Tests: scale.x at midpoint is 0, texture swaps at midpoint, animation completes and component removed, multiple flips queue correctly

---

## Phase F: Hand Inventory

### Step F1 — Hand Layout System `[NOT STARTED]`
**Crate:** card_game
**Why:** Cards in the player's hand are displayed in a row at the bottom of the screen, unaffected by physics.

- [ ] `hand_layout_system` in Phase::PostUpdate: reads Hand resource, positions each card entity using compute_flex_offsets (horizontal, centered at screen bottom)
- [ ] Hand cards: RenderLayer::UI, no RigidBody, Visible(true)
- [ ] Card sprites sized down or kept at table size (design decision)
- [ ] Tests: cards positioned horizontally with spacing, empty hand produces no positioning, card order matches Hand.cards order

### Step F2 — Hand Interaction `[NOT STARTED]`
**Crate:** card_game
**Why:** Drag cards from hand to table and from table to hand.

- [ ] Extend card_pick_system: also check hand card hit-testing (screen-space, RenderLayer::UI)
- [ ] On pick from hand: set DragState with origin_zone = Hand(index), remove from Hand resource, remove physics body if present, begin drag
- [ ] Extend card_release_system drop targets:
  - Drop on table area: add physics body (Dynamic), set CardZone::Table, set RenderLayer::World, configure damping
  - Drop on hand area (screen bottom region): remove physics body (if from table), add to Hand, set CardZone::Hand, set RenderLayer::UI
- [ ] Tests: pick from hand starts drag, drop on table adds physics body, drop on hand area removes physics body, hand capacity respected

---

## Phase G: Stash Grid Inventory

### Step G1 — Stash Grid Rendering `[DONE]`
**Crate:** card_game
**Why:** Visual grid of inventory slots for storing cards.

- [x] Render system draws width×height slot rects positioned by (col * SLOT_SIZE, row * SLOT_SIZE) offset from grid origin
- [x] Occupied slots render with the card's Shape color
- [x] Empty slots render with SLOT_COLOR constant (dark grey)
- [x] Stash visibility: togglable via Tab key press (StashVisible resource + stash_toggle_system)
- [x] Tests: 6 render tests (hidden=no draw, count, empty slot color, occupied slot color, column spacing, row spacing) + 4 toggle tests (default hidden, open, close, no-op without keypress) + 3 accessor tests (width, height, page_count)

### Step G2 — Stash Drag and Drop `[NOT STARTED]`
**Crate:** card_game
**Why:** Move cards in and out of the stash grid.

- [ ] Extend card_pick_system: check stash slot hit-testing when stash is visible
- [ ] On pick from stash: `StashGrid.take(page, col, row)`, set DragState with origin_zone = Stash { page, col, row }
- [ ] Extend card_release_system drop targets:
  - Drop on stash slot: `StashGrid.place(page, col, row, entity)`, set CardZone::Stash
  - Return to origin stash slot if drop target invalid
- [ ] Cross-zone drops: stash↔hand, stash↔table all supported
- [ ] Tests: pick from stash removes from grid, drop on empty slot places, drop on occupied slot returns to origin, cross-zone drops work

### Step G3 — Stash Hover Preview `[NOT STARTED]`
**Crate:** card_game
**Why:** Hovering over a stash icon shows a larger card face preview near the cursor.

- [ ] Spawn a preview entity: Sprite (face texture), Visible(false), RenderLayer::UI, high SortOrder
- [ ] `stash_hover_system` in Phase::Update: on hover over occupied stash slot, set preview sprite to that card's face texture, position near cursor (offset so it doesn't overlap the slot), set Visible(true)
- [ ] On hover exit: set Visible(false)
- [ ] Tests: hover on occupied slot shows preview, hover on empty slot does nothing, hover exit hides preview, preview follows cursor position

### Step G4 — Stash Pages `[NOT STARTED]`
**Crate:** card_game
**Why:** Multiple pages of stash storage, eventually purchasable.

- [ ] UI buttons or key bindings (e.g. arrow keys when stash is open) to switch `StashGrid.current_page`
- [ ] Grid rendering updates to show only current page's contents
- [ ] Page indicator (text or dots showing current/total pages)
- [ ] Tests: page switch updates displayed slots, cards on other pages preserved, page bounds respected

---

## Phase H: Integration and Polish

### Step H1 — Game Plugin and Setup `[NOT STARTED]`
**Crate:** card_game
**Why:** Wire everything together into a playable game.

- [ ] `CardGamePlugin` implementing Plugin trait: registers all systems in correct phases, inserts resources (DragState, Hand, StashGrid, PhysicsRes with gravity=ZERO)
- [ ] System ordering within phases:
  - Phase::PreUpdate: physics_step_system, physics_sync_system (chained)
  - Phase::Update: card_pick_system, card_drag_system, card_release_system, card_flip_system, flip_animation_system (chained)
  - Phase::PostUpdate: hand_layout_system (after hierarchy systems)
  - Phase::Render: ui_render_system, stash render systems (after sprite_render_system)
- [ ] Setup function: spawn initial deck of cards on table, spawn hand area, spawn stash grid
- [ ] Tests: plugin registers expected system count, all resources inserted

### Step H2 — Drag Visual Feedback `[NOT STARTED]`
**Crate:** card_game
**Why:** Visual cues during drag for better UX.

- [ ] Dragged card: elevated SortOrder, slight scale-up (e.g. 1.05x), optional shadow/offset
- [ ] Valid drop target highlighting: hand area glow, stash slot border color change
- [ ] Invalid drop feedback: card snaps back to origin on invalid drop
- [ ] Tests: dragged card SortOrder higher than table cards, scale restored on drop, snap-back on invalid target

---

## Progress Legend

- `[NOT STARTED]` — No work done
- `[IN PROGRESS]` — Actively being implemented
- `[DONE]` — Implemented, tested, merged

## Dependency Graph

```
A1 (add_force_at_point)──┐
A2 (set_damping)──────────┼──→ C3 (drag spring force)
A3 (body_point_to_world)──┘         │
                                    ↓
B1 (Card/Zone)──→ C1 (DragState) → C2 (pick) → C3 → C4 (release)
                                                        │
                                                        ↓
                                                  D1 (body setup) ✓
                                                  D2 (camera drag) ✓
B2 (Hand)───────→ F1 (hand layout) → F2 (hand interaction)
B3 (StashGrid)──→ G1 (grid render) → G2 (stash drag) → G3 (preview) → G4 (pages)

E1 (flip detect) → E2 (flip animation)  ← independent after C2 pattern

H1 (plugin) ← after all systems exist
H2 (polish) ← after H1
```

**Critical path:** A1–A3 → B1 → C1 → C2 → C3 → C4 → H1

**Parallelizable after C4:** E1/E2, F1/F2, G1–G4 (D1/D2 done)
