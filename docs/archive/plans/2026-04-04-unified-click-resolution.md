# Unified Click Resolution — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace four independent polling pick systems with a single `click_resolve_system` that raycasts all `Clickable` entities, picks the topmost by `SortOrder`, and delivers a `ClickedEntity` observer trigger — eliminating double-pick bugs structurally.

**Architecture:** `click_resolve_system` runs in `Phase::Input` and queries `(Entity, &Clickable, &GlobalTransform2D, &SortOrder)` for all interactive entity types. The topmost hit gets `commands.trigger_targets(ClickedEntity { world_cursor }, entity)`. Each entity type registers its own observer at spawn time and reacts to the trigger.

**Tech Stack:** bevy_ecs 0.16 (observers + triggers), glam Affine2 for local-space hit tests, existing `engine_physics::hit_test::local_space_hit`.

---

## File Map

**Created:**
- `crates/card_game/src/card/interaction/click_resolve.rs` — `Clickable`, `ClickHitShape`, `ClickedEntity`, `click_resolve_system`, `on_card_clicked`
- `crates/card_game/tests/suite/card_interaction_click_resolve.rs` — behavioral tests

**Modified:**
- `crates/card_game/src/card/interaction/mod.rs` — add `pub mod click_resolve`
- `crates/card_game/src/card/interaction/pick.rs` — remove `card_pick_system`, `mod hit_test`, `mod source`, keep constants
- `crates/card_game/src/card/rendering/spawn_table_card.rs` — add `Clickable` + register observer
- `crates/card_game/src/card/reader/spawn.rs` — add `Clickable` to reader + socket + register observers
- `crates/card_game/src/card/reader/pick.rs` — replace system with `on_reader_clicked` observer fn
- `crates/card_game/src/card/reader.rs` — swap export from `reader_pick_system` to `on_reader_clicked`
- `crates/card_game/src/card/screen_device.rs` — add `Clickable` + `on_screen_clicked` + register observers
- `crates/card_game/src/card/jack_socket.rs` — add `on_socket_clicked`, remove `jack_socket_pick_system`
- `crates/card_game/src/plugin.rs` — add `click_resolve_system` to `Phase::Input`, remove 4 pick systems from `Phase::Update` chain
- `crates/card_game/src/prelude.rs` — swap `card_pick_system` export for `click_resolve_system`
- `crates/card_game/tests/suite/mod.rs` — register new test module

---

## Task 1: `Clickable` component, `ClickedEntity` event, and `click_resolve_system`

**Files:**
- Create: `crates/card_game/src/card/interaction/click_resolve.rs`
- Create: `crates/card_game/tests/suite/card_interaction_click_resolve.rs`
- Modify: `crates/card_game/src/card/interaction/mod.rs`
- Modify: `crates/card_game/tests/suite/mod.rs`

- [ ] **Step 1: Write the failing test**

```rust
// crates/card_game/tests/suite/card_interaction_click_resolve.rs
#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::prelude::{InputState, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::{Affine2, Vec2};

use card_game::card::component::{Card, CardZone};
use card_game::card::interaction::click_resolve::{Clickable, ClickHitShape, click_resolve_system};
use card_game::card::interaction::drag_state::DragState;
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::jack_socket::PendingCable;
use card_game::card::reader::components::{CardReader, ReaderDragState};
use card_game::card::screen_device::ScreenDragState;

fn run_click_resolve(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(click_resolve_system);
    schedule.run(world);
}

fn make_world() -> World {
    let mut world = World::new();
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(ScreenDragState::default());
    world.insert_resource(PendingCable::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
    world
}

fn make_mouse_at(pos: Vec2) -> MouseState {
    let mut mouse = MouseState::default();
    mouse.simulate_just_pressed_left_at(pos);
    mouse
}

/// When a card and reader overlap and the card has a higher SortOrder,
/// clicking the overlap picks the card and leaves the reader untouched.
#[test]
fn when_card_and_reader_overlap_then_topmost_card_picked() {
    // Arrange
    let mut world = make_world();
    let pos = Vec2::new(100.0, 100.0);

    let reader = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: Vec2::new(40.0, 55.0),
                jack_entity: Entity::PLACEHOLDER,
            },
            Transform2D { position: pos, rotation: 0.0, scale: Vec2::ONE },
            Clickable(ClickHitShape::Aabb(Vec2::new(40.0, 55.0))),
            SortOrder::new(1),
            GlobalTransform2D(Affine2::from_translation(pos)),
        ))
        .id();

    let card = world
        .spawn((
            Card { face_texture: engine_core::prelude::TextureId(0),
                   back_texture: engine_core::prelude::TextureId(0),
                   face_up: true,
                   signature: Default::default() },
            CardZone::Table,
            Collider::Aabb(Vec2::new(30.0, 42.0)),
            Clickable(ClickHitShape::Aabb(Vec2::new(30.0, 42.0))),
            SortOrder::new(5),  // higher than reader
            GlobalTransform2D(Affine2::from_translation(pos)),
        ))
        .id();

    world.insert_resource(make_mouse_at(pos));

    // Act
    run_click_resolve(&mut world);

    // Assert — card picked via intent, reader drag not started
    let intents = world.resource::<EventBus<InteractionIntent>>();
    assert_eq!(intents.len(), 1);
    let intent = intents.iter().next().unwrap();
    assert!(
        matches!(intent, InteractionIntent::PickCard { entity, .. } if *entity == card),
        "expected PickCard for card entity, got {intent:?}"
    );
    let reader_drag = world.resource::<ReaderDragState>();
    assert!(reader_drag.dragging.is_none(), "reader should not be dragged");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game when_card_and_reader_overlap_then_topmost_card_picked 2>&1 | tail -20
```

Expected: compile error — `click_resolve` module does not exist yet.

- [ ] **Step 3: Create `click_resolve.rs`**

```rust
// crates/card_game/src/card/interaction/click_resolve.rs
use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut, Trigger};
use engine_core::prelude::EventBus;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::hit_test::local_space_hit;
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::Vec2;

use crate::card::component::CardZone;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::intent::InteractionIntent;
use crate::card::jack_socket::PendingCable;
use crate::card::reader::ReaderDragState;
use crate::card::screen_device::ScreenDragState;
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;

// ---------------------------------------------------------------------------
// Components + Events
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum ClickHitShape {
    Aabb(Vec2),
    Circle(f32),
}

#[derive(Component)]
pub struct Clickable(pub ClickHitShape);

#[derive(bevy_ecs::prelude::Event)]
pub struct ClickedEntity {
    pub world_cursor: Vec2,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn click_resolve_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    reader_drag: Res<ReaderDragState>,
    screen_drag: Res<ScreenDragState>,
    pending: Res<PendingCable>,
    stash_visible: Option<Res<StashVisible>>,
    grid: Option<Res<StashGrid>>,
    mut intents: ResMut<EventBus<InteractionIntent>>,
    clickables: Query<(Entity, &Clickable, &GlobalTransform2D, &SortOrder)>,
    mut commands: Commands,
) {
    if drag_state.dragging.is_some()
        || reader_drag.dragging.is_some()
        || screen_drag.dragging.is_some()
        || pending.source.is_some()
    {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Stash UI is screen-space — handle before world raycast.
    if let (Some(sv), Some(g)) = (&stash_visible, &grid) {
        if sv.0 {
            let screen = mouse.screen_pos();
            if crate::stash::pages::stash_ui_contains(screen, g) {
                if !g.is_store_page() {
                    if let Some((col, row)) = find_stash_slot_at(screen, g.width(), g.height()) {
                        let page = g.current_storage_page().unwrap_or(0);
                        if let Some(&entity) = g.get(page, col, row) {
                            intents.push(InteractionIntent::PickFromStash {
                                entity,
                                page,
                                col,
                                row,
                            });
                        }
                    }
                }
                return;
            }
        }
    }

    let cursor = mouse.world_pos();

    let winner = clickables
        .iter()
        .filter(|(_, clickable, global, _)| hit_test(cursor, clickable, global))
        .max_by_key(|(_, _, _, sort)| sort.value());

    if let Some((entity, _, _, _)) = winner {
        commands.trigger_targets(ClickedEntity { world_cursor: cursor }, entity);
    }
}

fn hit_test(cursor: Vec2, clickable: &Clickable, global: &GlobalTransform2D) -> bool {
    let cursor_local = global.0.inverse().transform_point2(cursor);
    match &clickable.0 {
        ClickHitShape::Aabb(half) => local_space_hit(cursor_local, *half),
        ClickHitShape::Circle(radius) => cursor_local.length() <= *radius,
    }
}

// ---------------------------------------------------------------------------
// Card observer — registered on every card entity at spawn time
// ---------------------------------------------------------------------------

pub fn on_card_clicked(
    trigger: Trigger<ClickedEntity>,
    cards: Query<(&CardZone, &GlobalTransform2D, &Collider)>,
    mut intents: ResMut<EventBus<InteractionIntent>>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok((zone, global, collider)) = cards.get(entity) else {
        return;
    };
    let cursor_delta = cursor - global.0.translation;
    let grab_offset = global.0.matrix2.inverse().mul_vec2(cursor_delta);
    intents.push(InteractionIntent::PickCard {
        entity,
        zone: *zone,
        collider: collider.clone(),
        grab_offset,
    });
}
```

- [ ] **Step 4: Add `click_resolve` to `interaction/mod.rs`**

```rust
// crates/card_game/src/card/interaction/mod.rs
pub mod apply;
pub mod camera_drag;
pub mod click_resolve;   // ← add this line
pub mod damping;
pub mod drag;
pub mod drag_state;
pub mod flip;
pub mod flip_animation;
pub mod game_state_param;
pub mod intent;
pub(crate) mod physics_helpers;
pub mod pick;
pub mod release;
```

- [ ] **Step 5: Add test module to `tests/suite/mod.rs`**

Append to the file:
```rust
mod card_interaction_click_resolve;
```

- [ ] **Step 6: Implement `MouseState::simulate_just_pressed_left_at` if missing**

Check if `MouseState` in `engine_input` has a test helper for simulating button presses. Look at how existing pick tests set up mouse state:

```bash
cargo.exe grep -r "simulate_just_pressed\|set_just_pressed\|just_pressed" crates/engine_input/src/
```

If it doesn't exist, add a test helper method to `MouseState` in `engine_input`:

```rust
// In engine_input/src/mouse_state.rs (or wherever MouseState is defined)
#[cfg(test)]
impl MouseState {
    pub fn simulate_just_pressed_left_at(&mut self, world_pos: Vec2) {
        // Set world_pos and mark left button as just pressed
        // Look at how MouseState stores state and replicate
    }
}
```

Check first by looking at `crates/engine_input/src/`:

```bash
cargo.exe grep -rn "just_pressed\|world_pos" crates/engine_input/src/ 2>&1 | head -30
```

Use the existing pattern from tests like `card_reader.rs` that already exercise mouse state.

- [ ] **Step 7: Run test**

```bash
cargo.exe test -p card_game when_card_and_reader_overlap_then_topmost_card_picked 2>&1 | tail -30
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/card_game/src/card/interaction/click_resolve.rs \
        crates/card_game/src/card/interaction/mod.rs \
        crates/card_game/tests/suite/card_interaction_click_resolve.rs \
        crates/card_game/tests/suite/mod.rs
git commit -m "feat(card-game): add Clickable component, ClickedEntity event, and click_resolve_system"
```

---

## Task 2: Add `Clickable` to card spawn + register `on_card_clicked` observer

**Files:**
- Modify: `crates/card_game/src/card/rendering/spawn_table_card.rs`

- [ ] **Step 1: Add failing test for observer registration**

Add to `crates/card_game/tests/suite/card_interaction_click_resolve.rs`:

```rust
/// Clicking a card at its exact center fires the PickCard intent.
#[test]
fn when_card_clicked_then_pick_card_intent_pushed() {
    use card_game::card::identity::definition::CardDefinition;
    use card_game::card::rendering::geometry::{TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT};
    use card_game::card::rendering::spawn_table_card::spawn_visual_card;

    // Arrange
    let mut world = make_world();
    world.insert_resource(engine_core::prelude::ClockRes::new(
        Box::new(engine_core::time::SystemClock::default()),
    ));

    let pos = Vec2::new(0.0, 0.0);
    let def = CardDefinition::default();
    let sig = Default::default();
    let card = spawn_visual_card(
        &mut world,
        &def,
        pos,
        Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT),
        true,
        sig,
    );

    // Manually set GlobalTransform2D (normally set by transform_propagation_system in LateUpdate)
    world.entity_mut(card).insert(
        GlobalTransform2D(Affine2::from_translation(pos))
    );
    world.insert_resource(make_mouse_at(pos));

    // Act
    run_click_resolve(&mut world);
    // Flush commands so the trigger fires the observer
    world.flush();

    // Assert
    let intents = world.resource::<EventBus<InteractionIntent>>();
    assert_eq!(intents.len(), 1);
    assert!(matches!(
        intents.iter().next().unwrap(),
        InteractionIntent::PickCard { entity, .. } if *entity == card
    ));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game when_card_clicked_then_pick_card_intent_pushed 2>&1 | tail -20
```

Expected: FAIL — card entity has no `Clickable` component, so no hit registered.

- [ ] **Step 3: Add `Clickable` and register observer in `spawn_visual_card`**

In `crates/card_game/src/card/rendering/spawn_table_card.rs`:

Add import:
```rust
use crate::card::interaction::click_resolve::{Clickable, ClickHitShape, on_card_clicked};
```

In `spawn_visual_card`, after the `world.spawn(...)` block that creates `root`, add `Clickable` to the entity bundle. Find the spawn call:

```rust
    let root = world
        .spawn((
            card,
            def.clone(),
            label.clone(),
            CardZone::Table,
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
            Collider::Aabb(half),
            RenderLayer::World,
            SortOrder::default(),
            Clickable(ClickHitShape::Aabb(half)),   // ← add this
        ))
        .id();
```

After the `.id()`, register the observer:
```rust
    world.entity_mut(root).observe(on_card_clicked);
```

- [ ] **Step 4: Run test**

```bash
cargo.exe test -p card_game when_card_clicked_then_pick_card_intent_pushed 2>&1 | tail -20
```

Expected: PASS.

- [ ] **Step 5: Run all card_game tests to check for regressions**

```bash
cargo.exe test -p card_game 2>&1 | tail -30
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/rendering/spawn_table_card.rs \
        crates/card_game/tests/suite/card_interaction_click_resolve.rs
git commit -m "feat(card-game): add Clickable to card spawn and register on_card_clicked observer"
```

---

## Task 3: Add `Clickable` to reader + socket spawn, register observers

**Files:**
- Modify: `crates/card_game/src/card/reader/spawn.rs`
- Modify: `crates/card_game/src/card/reader/pick.rs` (repurpose: remove system, add observer fn)
- Modify: `crates/card_game/src/card/reader.rs` (update export)

- [ ] **Step 1: Write failing test**

Add to `crates/card_game/tests/suite/card_interaction_click_resolve.rs`:

```rust
/// Clicking a reader (with no card on top) starts reader drag.
#[test]
fn when_reader_clicked_alone_then_reader_drag_starts() {
    use card_game::card::reader::spawn::spawn_reader;

    // Arrange
    let mut world = make_world();
    // spawn_reader uses World directly, needs physics bus
    world.insert_resource(EventBus::<engine_physics::prelude::PhysicsCommand>::default());

    let pos = Vec2::new(200.0, 200.0);
    let (reader_entity, _jack_entity) = spawn_reader(&mut world, pos);

    // Manually set GlobalTransform2D
    world.entity_mut(reader_entity).insert(
        GlobalTransform2D(Affine2::from_translation(pos))
    );
    // SortOrder defaults to 0
    world.insert_resource(make_mouse_at(pos));

    // Act
    run_click_resolve(&mut world);
    world.flush();

    // Assert
    let reader_drag = world.resource::<ReaderDragState>();
    assert!(reader_drag.dragging.is_some(), "reader drag should start");
    assert_eq!(reader_drag.dragging.unwrap().entity, reader_entity);
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo.exe test -p card_game when_reader_clicked_alone_then_reader_drag_starts 2>&1 | tail -20
```

Expected: FAIL — reader has no `Clickable`.

- [ ] **Step 3: Replace `reader/pick.rs` with observer function**

Replace the entire content of `crates/card_game/src/card/reader/pick.rs`:

```rust
use bevy_ecs::prelude::{ResMut, Trigger, Query, With};
use engine_core::prelude::Transform2D;

use crate::card::interaction::click_resolve::ClickedEntity;
use crate::card::reader::components::{CardReader, ReaderDragInfo, ReaderDragState};

pub fn on_reader_clicked(
    trigger: Trigger<ClickedEntity>,
    readers: Query<&Transform2D, With<CardReader>>,
    mut reader_drag: ResMut<ReaderDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = readers.get(entity) else {
        return;
    };
    reader_drag.dragging = Some(ReaderDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}
```

- [ ] **Step 4: Update `reader.rs` exports**

In `crates/card_game/src/card/reader.rs`, change:
```rust
pub use pick::reader_pick_system;
```
to:
```rust
pub use pick::on_reader_clicked;
```

- [ ] **Step 5: Add `Clickable` to reader + socket in `spawn_reader`, register observers**

In `crates/card_game/src/card/reader/spawn.rs`, add imports:
```rust
use crate::card::interaction::click_resolve::{Clickable, ClickHitShape};
use crate::card::jack_socket::on_socket_clicked;
use crate::card::reader::pick::on_reader_clicked;
```

In the `jack_entity` spawn, add `Clickable`:
```rust
    let jack_entity = world
        .spawn((
            Jack::<SignatureSpace> { direction: JackDirection::Output, data: None },
            JackSocket { radius: READER_SOCKET_RADIUS, color: READER_SOCKET_COLOR },
            Transform2D { position: position + READER_JACK_OFFSET, rotation: 0.0, scale: Vec2::ONE },
            Shape { variant: ShapeVariant::Circle { radius: READER_SOCKET_RADIUS }, color: READER_SOCKET_COLOR },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(READER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(READER_SOCKET_RADIUS)),   // ← add
        ))
        .id();
```

In the `reader_entity` spawn, add `Clickable`:
```rust
    let reader_entity = world
        .spawn((
            CardReader { loaded: None, half_extents: half, jack_entity },
            Transform2D { position, rotation: 0.0, scale: Vec2::ONE },
            RigidBody::Kinematic,
            Collider::Aabb(half),
            Shape { variant: rounded_rect_path(READER_HALF_W, READER_HALF_H, BASE_CORNER_RADIUS), color: BASE_FILL },
            Stroke { color: BASE_STROKE, width: 2.0 },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(-1),
            Clickable(ClickHitShape::Aabb(half)),   // ← add
        ))
        .id();
```

After `.id()`, register observers:
```rust
    world.entity_mut(reader_entity).observe(on_reader_clicked);
    world.entity_mut(jack_entity).observe(on_socket_clicked);
```

Note: `on_socket_clicked` does not exist yet — it will be added in Task 4. For now, stub it out or add Task 4 first. (Complete Task 4's `on_socket_clicked` before running this task's test.)

- [ ] **Step 6: Run test**

```bash
cargo.exe test -p card_game when_reader_clicked_alone_then_reader_drag_starts 2>&1 | tail -20
```

Expected: PASS.

- [ ] **Step 7: Run all card_game tests**

```bash
cargo.exe test -p card_game 2>&1 | tail -30
```

Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add crates/card_game/src/card/reader/pick.rs \
        crates/card_game/src/card/reader/spawn.rs \
        crates/card_game/src/card/reader.rs \
        crates/card_game/tests/suite/card_interaction_click_resolve.rs
git commit -m "feat(card-game): add Clickable to reader spawn and replace reader_pick_system with on_reader_clicked observer"
```

---

## Task 4: Add `Clickable` to screen + socket, register observers

**Files:**
- Modify: `crates/card_game/src/card/screen_device.rs`
- Modify: `crates/card_game/src/card/jack_socket.rs`

- [ ] **Step 1: Write failing test**

Add to `crates/card_game/tests/suite/card_interaction_click_resolve.rs`:

```rust
/// Clicking a socket starts cable drag (sets PendingCable.source).
#[test]
fn when_socket_clicked_then_pending_cable_source_set() {
    use card_game::card::screen_device::spawn_screen_device;

    // Arrange
    let mut world = make_world();
    world.insert_resource(EventBus::<engine_physics::prelude::PhysicsCommand>::default());

    let pos = Vec2::new(50.0, 50.0);
    let (device_entity, jack_entity) = spawn_screen_device(&mut world, pos);

    // Set GlobalTransform2D for the socket (jack_entity)
    let socket_pos = pos + Vec2::new(129.0, 0.0); // BODY_HALF_W + SOCKET_RADIUS + 4
    world.entity_mut(jack_entity).insert(
        GlobalTransform2D(Affine2::from_translation(socket_pos))
    );
    // Give the socket a higher SortOrder than device
    world.entity_mut(jack_entity).insert(SortOrder::new(3));
    world.entity_mut(device_entity).insert(
        GlobalTransform2D(Affine2::from_translation(pos))
    );
    world.entity_mut(device_entity).insert(SortOrder::new(1));

    world.insert_resource(make_mouse_at(socket_pos));

    // Act
    run_click_resolve(&mut world);
    world.flush();

    // Assert
    let pending = world.resource::<PendingCable>();
    assert_eq!(pending.source, Some(jack_entity));
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo.exe test -p card_game when_socket_clicked_then_pending_cable_source_set 2>&1 | tail -20
```

Expected: FAIL — no `Clickable` on socket, no `on_socket_clicked`.

- [ ] **Step 3: Add `on_socket_clicked` to `jack_socket.rs`**

In `crates/card_game/src/card/jack_socket.rs`, add after existing imports:

```rust
use crate::card::interaction::click_resolve::ClickedEntity;
```

Add function (before `jack_socket_pick_system` — do NOT delete `jack_socket_pick_system` yet, that happens in Task 5):

```rust
pub fn on_socket_clicked(
    trigger: Trigger<ClickedEntity>,
    mut pending: ResMut<PendingCable>,
) {
    pending.source = Some(trigger.target());
}
```

- [ ] **Step 4: Add `on_screen_clicked` to `screen_device.rs` and add `Clickable` + observers to `spawn_screen_device`**

In `crates/card_game/src/card/screen_device.rs`, add import:
```rust
use crate::card::interaction::click_resolve::{Clickable, ClickHitShape, ClickedEntity};
use crate::card::jack_socket::on_socket_clicked;
```

Add observer function (near the other pick/release functions):
```rust
pub fn on_screen_clicked(
    trigger: Trigger<ClickedEntity>,
    screens: Query<&Transform2D, With<ScreenDevice>>,
    mut screen_drag: ResMut<ScreenDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = screens.get(entity) else {
        return;
    };
    screen_drag.dragging = Some(ScreenDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}
```

In `spawn_screen_device`, add `Clickable` to the jack entity spawn:
```rust
    let jack_entity = world
        .spawn((
            Jack::<SignatureSpace> { direction: JackDirection::Input, data: None },
            JackSocket { radius: SOCKET_RADIUS, color: SOCKET_COLOR },
            Transform2D { position: position + JACK_OFFSET, rotation: 0.0, scale: Vec2::ONE },
            Shape { variant: ShapeVariant::Circle { radius: SOCKET_RADIUS }, color: SOCKET_COLOR },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(SCREEN_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),   // ← add
        ))
        .id();
```

Add `Clickable` to the device entity spawn:
```rust
    let device_entity = world
        .spawn((
            ScreenDevice { signature_input: jack_entity },
            Transform2D { position, rotation: 0.0, scale: Vec2::ONE },
            Shape { variant: rounded_rect_path(BODY_HALF_W, BODY_HALF_H, BODY_CORNER_RADIUS), color: BODY_FILL },
            Stroke { color: BODY_STROKE, width: 2.0 },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(SCREEN_LOCAL_SORT),
            Clickable(ClickHitShape::Aabb(SCREEN_HALF_EXTENTS)),   // ← add
        ))
        .id();
```

After the device entity `.id()`, register observers:
```rust
    world.entity_mut(device_entity).observe(on_screen_clicked);
    world.entity_mut(jack_entity).observe(on_socket_clicked);
```

- [ ] **Step 5: Run tests**

```bash
cargo.exe test -p card_game when_socket_clicked_then_pending_cable_source_set 2>&1 | tail -20
```

Expected: PASS.

```bash
cargo.exe test -p card_game 2>&1 | tail -30
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/screen_device.rs \
        crates/card_game/src/card/jack_socket.rs \
        crates/card_game/tests/suite/card_interaction_click_resolve.rs
git commit -m "feat(card-game): add Clickable to screen/socket spawn and register on_screen_clicked/on_socket_clicked observers"
```

---

## Task 5: Wire `click_resolve_system` into `Phase::Input`, remove old pick systems

**Files:**
- Modify: `crates/card_game/src/plugin.rs`
- Modify: `crates/card_game/src/prelude.rs`

- [ ] **Step 1: Write the integration test that exercises the full fix**

Add to `crates/card_game/tests/suite/card_interaction_click_resolve.rs`:

```rust
/// The original double-pick bug: clicking where a card sits on a reader
/// must NOT start reader drag while also picking the card.
/// This test verifies the structural fix: only one entity reacts to a click.
#[test]
fn when_card_on_reader_clicked_only_card_picked_not_reader() {
    use card_game::card::identity::definition::CardDefinition;
    use card_game::card::rendering::geometry::{TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT};
    use card_game::card::rendering::spawn_table_card::spawn_visual_card;
    use card_game::card::reader::spawn::spawn_reader;

    // Arrange
    let mut world = make_world();
    world.insert_resource(EventBus::<engine_physics::prelude::PhysicsCommand>::default());

    let pos = Vec2::new(0.0, 0.0);

    let (reader_entity, _) = spawn_reader(&mut world, pos);
    world.entity_mut(reader_entity).insert((
        GlobalTransform2D(Affine2::from_translation(pos)),
        SortOrder::new(1),
    ));

    let card = spawn_visual_card(
        &mut world,
        &CardDefinition::default(),
        pos,
        Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT),
        true,
        Default::default(),
    );
    world.entity_mut(card).insert((
        GlobalTransform2D(Affine2::from_translation(pos)),
        SortOrder::new(10),  // card is visually on top
    ));

    world.insert_resource(make_mouse_at(pos));

    // Act
    run_click_resolve(&mut world);
    world.flush();

    // Assert — card intent fired, reader drag did NOT start
    let intents = world.resource::<EventBus<InteractionIntent>>();
    assert_eq!(intents.len(), 1, "exactly one pick intent expected");
    assert!(matches!(
        intents.iter().next().unwrap(),
        InteractionIntent::PickCard { entity, .. } if *entity == card
    ));
    let reader_drag = world.resource::<ReaderDragState>();
    assert!(reader_drag.dragging.is_none(), "reader must not be dragged");
}
```

- [ ] **Step 2: Run test to verify it currently fails (before wiring)**

```bash
cargo.exe test -p card_game when_card_on_reader_clicked_only_card_picked_not_reader 2>&1 | tail -20
```

Expected: FAIL — `card_pick_system` and `reader_pick_system` both still fire.

- [ ] **Step 3: Update `plugin.rs`**

In `crates/card_game/src/plugin.rs`:

Add import:
```rust
use crate::card::interaction::click_resolve::click_resolve_system;
```

Remove imports:
```rust
use crate::card::interaction::pick::card_pick_system;    // ← remove
// and:
use crate::card::reader::reader_pick_system;             // ← remove (or rename below)
// and from screen_device:
use crate::card::screen_device::screen_pick_system;      // ← remove
// and from jack_socket:
use crate::card::jack_socket::jack_socket_pick_system;   // ← remove
```

Add `click_resolve_system` to `Phase::Input`:
```rust
app.add_systems(Phase::Input, click_resolve_system);
```

In the `Phase::Update` `.chain()`, remove `card_pick_system`, `reader_pick_system`, `screen_pick_system`, `jack_socket_pick_system`. The chain becomes:

```rust
.add_systems(
    Phase::Update,
    (
        store_buy_system,
        card_reader_eject_system,
        card_drag_system,
        reader_drag_system,
        screen_drag_system,
        store_sell_system,
        stash_boundary_system,
        card_reader_insert_system,
        card_release_system,
        interaction_apply_system,
        reader_release_system,
        screen_release_system,
        jack_socket_release_system,
        card_flip_system,
        flip_animation_system,
    )
        .chain(),
)
```

- [ ] **Step 4: Run the integration test**

```bash
cargo.exe test -p card_game when_card_on_reader_clicked_only_card_picked_not_reader 2>&1 | tail -20
```

Expected: PASS.

- [ ] **Step 5: Run full test suite**

```bash
cargo.exe test -p card_game 2>&1 | tail -40
```

Expected: all pass. If card_game_bin doesn't compile, fix import errors.

- [ ] **Step 6: Update `prelude.rs`**

In `crates/card_game/src/prelude.rs`, replace:
```rust
pub use crate::card::interaction::pick::{
    CARD_COLLISION_FILTER, CARD_COLLISION_GROUP, card_pick_system,
};
```
with:
```rust
pub use crate::card::interaction::click_resolve::click_resolve_system;
pub use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
```

- [ ] **Step 7: Build binary to verify compilation**

```bash
cargo.exe build -p card_game_bin 2>&1 | tail -30
```

Expected: no errors. Fix any remaining import references to deleted systems.

- [ ] **Step 8: Commit**

```bash
git add crates/card_game/src/plugin.rs \
        crates/card_game/src/prelude.rs \
        crates/card_game/tests/suite/card_interaction_click_resolve.rs
git commit -m "feat(card-game): wire click_resolve_system into Phase::Input and remove four polling pick systems"
```

---

## Task 6: Cleanup — remove dead system bodies, update reader.rs

**Files:**
- Modify: `crates/card_game/src/card/interaction/pick.rs`
- Modify: `crates/card_game/src/card/jack_socket.rs`

- [ ] **Step 1: Remove `card_pick_system` and submodule declarations from `pick.rs`**

The file currently has `card_pick_system`, `mod hit_test`, `mod source`, and the constants. Keep only the constants. Replace the content of `crates/card_game/src/card/interaction/pick.rs` with:

```rust
// Physics collision group constants shared by interaction and physics helpers.
pub const CARD_COLLISION_GROUP: u32 = 0b0001;
pub const CARD_COLLISION_FILTER: u32 = 0b0010;
pub(crate) const DRAGGED_COLLISION_GROUP: u32 = 0;
pub(crate) const DRAGGED_COLLISION_FILTER: u32 = 0;
pub const DRAG_SCALE: f32 = 1.05;
```

Delete the submodule files (they are no longer referenced):
- `crates/card_game/src/card/interaction/pick/hit_test.rs`
- `crates/card_game/src/card/interaction/pick/source.rs`

These can be deleted with:
```bash
rm crates/card_game/src/card/interaction/pick/hit_test.rs
rm crates/card_game/src/card/interaction/pick/source.rs
rmdir crates/card_game/src/card/interaction/pick 2>/dev/null || true
```

- [ ] **Step 2: Remove `jack_socket_pick_system` from `jack_socket.rs`**

In `crates/card_game/src/card/jack_socket.rs`, delete the `jack_socket_pick_system` function entirely (the one that iterated over all sockets to find the one under the cursor — now replaced by `on_socket_clicked`).

Remove any imports that were only used by `jack_socket_pick_system` (e.g. `DragState`, `ReaderDragState`, `ScreenDragState` if only used there).

- [ ] **Step 3: Run full build and tests**

```bash
cargo.exe build -p card_game_bin 2>&1 | tail -20
cargo.exe test -p card_game 2>&1 | tail -30
```

Expected: no errors, all pass.

- [ ] **Step 4: Format**

```bash
cargo.exe fmt --all
```

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/card/interaction/pick.rs \
        crates/card_game/src/card/jack_socket.rs
git commit -m "chore(card-game): remove dead pick system bodies and unused submodule files"
```

---

## Self-Review Checklist

**Spec coverage:**
- ✅ `Clickable` component with `ClickHitShape` enum — Task 1
- ✅ `click_resolve_system` raycasts all clickable entities — Task 1
- ✅ Stash pick remains as special case in `click_resolve_system` — Task 1
- ✅ `trigger_targets(ClickedEntity, entity)` on topmost hit — Task 1
- ✅ `on_card_clicked` — Task 2
- ✅ `on_reader_clicked` — Task 3
- ✅ `on_screen_clicked` — Task 4
- ✅ `on_socket_clicked` — Task 4
- ✅ `click_resolve_system` in `Phase::Input` — Task 5
- ✅ 4 old pick systems removed — Tasks 5 + 6
- ✅ `SortOrder` topmost always wins (visually on top) — Task 1 hit_test uses max_by_key on SortOrder
- ✅ Behavioral test for double-pick bug — Task 5

**Type consistency:**
- `ClickHitShape` — defined Task 1, used Tasks 2-4 ✅
- `ClickedEntity` — defined Task 1, used Tasks 2-4 ✅
- `on_socket_clicked` — defined Task 4, referenced in Task 3 (reader spawn) — **Order dependency**: complete Task 4's `on_socket_clicked` definition before compiling Task 3's spawn changes.
- `trigger.target()` — bevy_ecs 0.16 API. If compile error, try `trigger.entity()` instead.

**Potential issue — `MouseState::simulate_just_pressed_left_at`:** Step 6 of Task 1 flags this. Check how existing pick tests simulate mouse clicks before writing this helper. Look at `crates/card_game/tests/suite/card_reader.rs` for the existing pattern.
