# Observer-Driven Interaction Architecture

## Problem

The current interaction system is built on polling chains:

- `handle_key_event` / `handle_cursor_moved` push raw OS events into `EventBus<KeyInputEvent>` / `EventBus<MouseInputEvent>`
- Input systems drain those buses and accumulate state into `MouseState` / `KeyState` resources
- Pick systems poll `MouseState.just_pressed()` every frame, and also poll 3–4 drag-state resources (`DragState`, `ReaderDragState`, `ScreenDragState`, `PendingCable`) as mutual-exclusion guards
- A 20-system `.chain()` enforces ordering that only exists to sequence the polling
- Release systems poll mouse state and drag state resources to know when to clean up

The immediate bug: `card_pick_system` pushes a deferred `InteractionIntent` instead of setting `DragState.dragging` immediately, so `reader_pick_system` (which runs next in the chain) doesn't see the card as "dragged" yet, and both fire on the same click.

The root cause is structural: systems ask shared state whether something happened, instead of reacting when it does.

## Target Architecture

Replace the polling chain with three layers:

1. **App bridge** — `world.trigger()` directly from winit callbacks
2. **Click resolution** — single raycast, `trigger_targets` on the topmost hit entity
3. **Entity observers** — each entity type handles its own events; drag state lives as components on entities

The 18 `Phase` schedule labels are unchanged. The `.chain()` of 20 interaction systems collapses.

---

## Layer 1: App Bridge

`App::handle_key_event`, `handle_cursor_moved`, `handle_mouse_button`, `handle_mouse_wheel` currently push to `EventBus<KeyInputEvent>` / `EventBus<MouseInputEvent>` and return. Replace with direct `world.trigger()`:

```rust
// Before
bus.push(KeyInputEvent { key, state });

// After
self.world.trigger(KeyInputEvent { key, state });
```

`MouseState` and `KeyState` resources are updated by observers on these trigger types (one observer each, registered by `InputPlugin`). This preserves per-frame accumulated state (current position, held buttons) that drag systems need.

The `handle_*` methods on `App` remain as private helpers called from `window_event`; their bodies shrink to a single `trigger` call each.

---

## Layer 2: Click Resolution

A single `click_resolve_system` replaces all four per-type hit tests. It runs in the `Input` phase.

### `Clickable` component

```rust
pub enum ClickHitShape {
    Aabb(Vec2),   // half-extents in local space
    Circle(f32),  // radius
}

#[derive(Component)]
pub struct Clickable(pub ClickHitShape);
```

Added at spawn time:
- **Card**: `Clickable(ClickHitShape::Aabb(half))` — same half-extents as current `Collider`
- **CardReader**: `Clickable(ClickHitShape::Aabb(reader.half_extents))`
- **ScreenDevice**: `Clickable(ClickHitShape::Aabb(SCREEN_HALF_EXTENTS))`
- **JackSocket**: `Clickable(ClickHitShape::Circle(socket.radius))`

### `ClickedEntity` trigger event

```rust
#[derive(Event)]
pub struct ClickedEntity {
    pub world_cursor: Vec2,
}
```

### `click_resolve_system`

Guards (return early if any active drag component exists, or stash UI is over cursor):
- `Query<(), Or<(With<BeingDragged>, With<ReaderDragging>, With<ScreenDragging>)>>` is non-empty
- `PendingCable.source.is_some()`
- Mouse not just-pressed
- Stash UI visible and cursor over it

Hit test: query `(Entity, &Clickable, &GlobalTransform2D, &SortOrder)`. For each entity, test cursor against the hit shape in local space. Collect hits, pick max `SortOrder`. Call:

```rust
commands.trigger_targets(ClickedEntity { world_cursor: cursor }, winner);
```

One entity is picked per frame. The entity visually on top (highest `SortOrder`) always wins.

### Stash picking

Stash is screen-space UI, not a world entity. Handled as a special case inside `click_resolve_system` before the world-space raycast: if stash visible and cursor over stash UI, push `InteractionIntent::PickFromStash` directly and return.

---

## Layer 3: Entity Observers

Each entity type registers observers at spawn time. No entity asks "was I clicked?" — the trigger arrives at the entity.

### Card

```rust
commands.entity(card).observe(on_card_clicked);

fn on_card_clicked(
    trigger: Trigger<ClickedEntity>,
    mut commands: Commands,
    cards: Query<(&CardZone, &GlobalTransform2D, &Collider, &SortOrder)>,
    mut hand: Option<ResMut<Hand>>,
    mut physics: ResMut<EventBus<PhysicsCommand>>,
) {
    let entity = trigger.target();
    // compute grab offset from world_cursor and GlobalTransform2D
    // insert BeingDragged component, handle zone-specific setup
    commands.entity(entity).insert(BeingDragged { ... });
}
```

### CardReader

```rust
commands.entity(reader).observe(on_reader_clicked);

fn on_reader_clicked(
    trigger: Trigger<ClickedEntity>,
    mut commands: Commands,
    readers: Query<&Transform2D, With<CardReader>>,
) {
    let entity = trigger.target();
    let transform = readers.get(entity).unwrap();
    commands.entity(entity).insert(ReaderDragging {
        grab_offset: trigger.event().world_cursor - transform.position,
    });
}
```

### ScreenDevice

```rust
commands.entity(screen).observe(on_screen_clicked);
// same pattern as CardReader
```

### JackSocket

```rust
commands.entity(socket).observe(on_socket_clicked);

fn on_socket_clicked(trigger: Trigger<ClickedEntity>, mut pending: ResMut<PendingCable>) {
    pending.source = Some(trigger.target());
}
```

---

## Drag State: Components on Entities

Replace the three drag-state resources with components on the dragged entity:

| Before | After |
|--------|-------|
| `Res<DragState>` with `Option<DragInfo>` | `BeingDragged { grab_offset, origin_zone, origin_position }` component on card |
| `Res<ReaderDragState>` with `Option<ReaderDragInfo>` | `ReaderDragging { grab_offset }` component on reader |
| `Res<ScreenDragState>` with `Option<ScreenDragInfo>` | `ScreenDragging { grab_offset }` component on screen |
| `Res<PendingCable>` | Keep as resource (only one cable drag at a time, not entity-scoped) |

Drag systems query the component instead of checking a resource:

```rust
// Before
fn card_drag_system(drag_state: Res<DragState>, ...) {
    let Some(drag_info) = drag_state.dragging.as_ref() else { return; };
    // ...
}

// After
fn card_drag_system(
    mouse: Res<MouseState>,
    mut dragged: Query<(Entity, &mut Transform2D, &BeingDragged)>,
) {
    for (entity, mut transform, drag) in &mut dragged {
        // update position from mouse.world_pos() + grab_offset
    }
}
```

---

## Release

Release triggers are fired in `click_resolve_system` (or a companion `release_resolve_system` in the `Input` phase) when the left mouse button is just-released.

```rust
#[derive(Event)]
pub struct ReleasedEntity {
    pub world_cursor: Vec2,
}
```

If any entity has `BeingDragged`, `ReaderDragging`, or `ScreenDragging`, trigger `ReleasedEntity` on it. Entity observers handle cleanup (remove component, restore physics, handle drop targets).

`PendingCable` release is handled by an observer on mouse-release that checks `pending.source`.

---

## System Ordering After

The 20-system `.chain()` in `Phase::Update` becomes:

```
Phase::Input:
  click_resolve_system    (raycast + trigger)
  release_resolve_system  (detect release + trigger)

Phase::Update:
  card_drag_system        (queries BeingDragged — no ordering dependency)
  reader_drag_system      (queries ReaderDragging — no ordering dependency)
  screen_drag_system      (queries ScreenDragging — no ordering dependency)
  card_reader_insert_system
  card_reader_eject_system
  store_buy_system
  store_sell_system
  stash_boundary_system
  card_flip_system
  flip_animation_system
```

Most of these can run in parallel (no `.chain()`). `interaction_apply_system` is deleted — its logic moves into the card's click and release observers.

---

## What Does Not Change

- 18 `Phase` labels and their order — frozen
- `MouseState` and `KeyState` as per-frame accumulated state resources
- `InteractionIntent` for stash picks (screen-space, no entity target)
- `PhysicsCommand` event bus
- Render pipeline
- All release and drag logic — only the triggering mechanism changes
