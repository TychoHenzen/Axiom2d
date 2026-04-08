# Booster Pack System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a booster pack system — a purchasable BoosterMachine device that consumes cards in readers via cable input and produces sealed booster packs containing 5-15 sampled cards, openable with a cinematic double-click animation.

**Architecture:** Cross-cutting `SignatureSpace` change adds entity back-references. New `booster/` module under `card_game/src/` with 5 files (device, pack, sampling, opening, double_click). Follows existing device patterns (Combiner/Screen/Reader). Opening animation is a resource-based state machine in `Phase::Animate`.

**Tech Stack:** Rust, bevy_ecs (standalone), rapier2d (physics), rand/rand_chacha (seeded RNG), glam (math)

**Spec:** `docs/superpowers/specs/2026-04-08-booster-pack-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `crates/card_game/src/card/reader/signature_space.rs` | Modify | Add `source_cards: Vec<Entity>` to `SignatureSpace`, update `from_single` and `combine` |
| `crates/card_game/src/card/reader/insert.rs` | Modify | Pass card entity to `from_single` |
| `crates/card_game/src/card/interaction/intent.rs` | Modify | Add `DoubleClick` and `OpenBoosterPack` variants to `InteractionIntent` |
| `crates/card_game/src/card/component.rs` | Modify | Add `BoosterSlot(Entity)` variant to `CardZone` |
| `crates/card_game/src/booster/mod.rs` | Create | Module declarations and pub re-exports |
| `crates/card_game/src/booster/device.rs` | Create | `BoosterMachine` component, spawn, drag, seal button click |
| `crates/card_game/src/booster/pack.rs` | Create | `BoosterPack` component, spawn pack entity, pickup from slot |
| `crates/card_game/src/booster/sampling.rs` | Create | `sample_signatures_from_space()` — RNG sampling within a `SignatureSpace` region |
| `crates/card_game/src/booster/opening.rs` | Create | `BoosterOpening` resource, `BoosterOpenPhase` enum, animation state machine system |
| `crates/card_game/src/booster/double_click.rs` | Create | `DoubleClickState` resource, `double_click_detect_system` |
| `crates/card_game/src/lib.rs` | Modify | Add `pub mod booster;` |
| `crates/card_game/src/prelude.rs` | Modify | Re-export booster public types |
| `crates/card_game/src/plugin.rs` | Modify | Insert new resources, register new systems |
| `crates/card_game/src/stash/store.rs` | Modify | Add `BoosterMachine` to `StoreItemKind`, spawn handler, preview, sell handler, drag check |
| `crates/card_game/tests/suite/mod.rs` | Modify | Register new test modules |
| `crates/card_game/tests/suite/booster_sampling.rs` | Create | Tests for signature sampling |
| `crates/card_game/tests/suite/booster_device.rs` | Create | Tests for seal logic, card destruction |
| `crates/card_game/tests/suite/booster_pack.rs` | Create | Tests for pack spawning, physics properties |
| `crates/card_game/tests/suite/booster_double_click.rs` | Create | Tests for double-click detection |
| `crates/card_game/tests/suite/booster_opening.rs` | Create | Tests for opening animation state machine |

---

## Task 1: Create feature branch

**Files:** None

- [ ] **Step 1: Create and switch to feature branch**

```bash
git checkout -b feature/booster-pack
```

- [ ] **Step 2: Verify clean state**

```bash
git status
```

Expected: `On branch feature/booster-pack`, nothing to commit.

---

## Task 2: Add entity back-references to SignatureSpace

**Files:**
- Modify: `crates/card_game/src/card/reader/signature_space.rs`
- Modify: `crates/card_game/src/card/reader/insert.rs`
- Test: `crates/card_game/tests/suite/card_reader.rs` (existing tests may need updating)

- [ ] **Step 1: Write failing test for `from_single` with source entity**

Create `crates/card_game/tests/suite/booster_sampling.rs`:

```rust
#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use card_game::card::reader::signature_space::SignatureSpace;
use card_game::prelude::CardSignature;

#[test]
fn when_from_single_then_source_cards_contains_entity() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let sig = CardSignature::new([0.5; 8]);

    // Act
    let space = SignatureSpace::from_single(sig, 0.2, entity);

    // Assert
    assert_eq!(space.source_cards, vec![entity]);
}

#[test]
fn when_combine_then_source_cards_merged() {
    // Arrange
    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();
    let sig1 = CardSignature::new([0.3; 8]);
    let sig2 = CardSignature::new([0.7; 8]);
    let space_a = SignatureSpace::from_single(sig1, 0.2, e1);
    let space_b = SignatureSpace::from_single(sig2, 0.2, e2);

    // Act
    let combined = SignatureSpace::combine(&space_a, &space_b);

    // Assert
    assert_eq!(combined.source_cards.len(), 2);
    assert!(combined.source_cards.contains(&e1));
    assert!(combined.source_cards.contains(&e2));
}
```

Register it in `crates/card_game/tests/suite/mod.rs`:

```rust
mod booster_sampling;
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game booster_sampling -- --nocapture
```

Expected: Compilation error — `from_single` doesn't accept 3 arguments yet.

- [ ] **Step 3: Update `SignatureSpace` to include `source_cards`**

In `crates/card_game/src/card/reader/signature_space.rs`, add `source_cards` field and update methods:

```rust
use bevy_ecs::prelude::Entity;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct SignatureSpace {
    pub control_points: Vec<CardSignature>,
    pub radius: f32,
    pub volume: f32,
    pub source_cards: Vec<Entity>,
}

impl SignatureSpace {
    pub fn from_single(center: CardSignature, radius: f32, source: Entity) -> Self {
        let volume = sphere_volume_8d(radius);
        Self {
            control_points: vec![center],
            radius,
            volume,
            source_cards: vec![source],
        }
    }

    pub fn combine(a: &Self, b: &Self) -> Self {
        let mut points = Vec::with_capacity(a.control_points.len() + b.control_points.len());
        points.extend_from_slice(&a.control_points);
        points.extend_from_slice(&b.control_points);
        points.sort();
        points.dedup_by(|x, y| x.distance_to(y) < 1e-6);

        let volume = a.volume + b.volume;
        let arc_length = polyline_arc_length(&points);
        let radius = solve_tube_radius(volume, arc_length);

        let mut source_cards = Vec::with_capacity(a.source_cards.len() + b.source_cards.len());
        source_cards.extend_from_slice(&a.source_cards);
        for &e in &b.source_cards {
            if !source_cards.contains(&e) {
                source_cards.push(e);
            }
        }

        Self {
            control_points: points,
            radius,
            volume,
            source_cards,
        }
    }
}
```

- [ ] **Step 4: Fix all existing `from_single` call sites**

In `crates/card_game/src/card/reader/insert.rs` (line 61), update:

```rust
jack.data = Some(SignatureSpace::from_single(
    card.signature,
    signature_radius(&card.signature),
    card_entity,
));
```

Search for any other `from_single` calls in tests and update them to pass a dummy entity. Use `World::new().spawn_empty().id()` or the existing entity from the test context.

```bash
cargo.exe grep "from_single" -- crates/card_game/
```

Fix each call site.

- [ ] **Step 5: Run all tests to verify nothing breaks**

```bash
cargo.exe test -p card_game -- --nocapture
```

Expected: All tests pass, including the two new ones.

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/reader/signature_space.rs crates/card_game/src/card/reader/insert.rs crates/card_game/tests/suite/booster_sampling.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card_game): add source_cards back-references to SignatureSpace

SignatureSpace::from_single now takes a source Entity parameter.
SignatureSpace::combine merges source_cards from both inputs.
Reader insert system passes the card entity as the source."
```

---

## Task 3: Add InteractionIntent variants and CardZone variant

**Files:**
- Modify: `crates/card_game/src/card/interaction/intent.rs`
- Modify: `crates/card_game/src/card/component.rs`

- [ ] **Step 1: Add new InteractionIntent variants**

In `crates/card_game/src/card/interaction/intent.rs`, add:

```rust
    OpenBoosterPack {
        entity: Entity,
    },
```

- [ ] **Step 2: Run build to verify compilation**

```bash
cargo.exe build -p card_game
```

Expected: Passes. No match exhaustiveness errors since `InteractionIntent` is not matched exhaustively anywhere (it uses `EventBus`).

- [ ] **Step 3: Commit**

```bash
git add crates/card_game/src/card/interaction/intent.rs
git commit -m "feat(card_game): add OpenBoosterPack interaction intent"
```

---

## Task 4: Create booster module skeleton

**Files:**
- Create: `crates/card_game/src/booster/mod.rs`
- Create: `crates/card_game/src/booster/device.rs`
- Create: `crates/card_game/src/booster/pack.rs`
- Create: `crates/card_game/src/booster/sampling.rs`
- Create: `crates/card_game/src/booster/opening.rs`
- Create: `crates/card_game/src/booster/double_click.rs`
- Modify: `crates/card_game/src/lib.rs`

- [ ] **Step 1: Create all module files with minimal content**

`crates/card_game/src/booster/mod.rs`:

```rust
pub mod device;
pub mod double_click;
pub mod opening;
pub mod pack;
pub mod sampling;
```

`crates/card_game/src/booster/device.rs`:

```rust
// BoosterMachine device — spawn, drag, seal button
```

`crates/card_game/src/booster/pack.rs`:

```rust
// BoosterPack entity — spawn, pickup, physics
```

`crates/card_game/src/booster/sampling.rs`:

```rust
// Card signature sampling from SignatureSpace regions
```

`crates/card_game/src/booster/opening.rs`:

```rust
// Booster pack opening animation state machine
```

`crates/card_game/src/booster/double_click.rs`:

```rust
// Double-click detection system
```

- [ ] **Step 2: Register module in lib.rs**

In `crates/card_game/src/lib.rs`, add:

```rust
pub mod booster;
```

- [ ] **Step 3: Verify compilation**

```bash
cargo.exe build -p card_game
```

Expected: Passes.

- [ ] **Step 4: Commit**

```bash
git add crates/card_game/src/booster/ crates/card_game/src/lib.rs
git commit -m "feat(card_game): add booster module skeleton"
```

---

## Task 5: Implement card signature sampling

**Files:**
- Modify: `crates/card_game/src/booster/sampling.rs`
- Test: `crates/card_game/tests/suite/booster_sampling.rs`

- [ ] **Step 1: Write failing tests for sampling**

Append to `crates/card_game/tests/suite/booster_sampling.rs`:

```rust
use card_game::booster::sampling::sample_signatures_from_space;
use card_game::card::reader::signature_space::SignatureSpace;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

#[test]
fn when_sample_from_single_point_space_then_all_within_radius() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let center = CardSignature::new([0.5; 8]);
    let radius = 0.2;
    let space = SignatureSpace::from_single(center, radius, entity);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Act
    let signatures = sample_signatures_from_space(&space, 10, &mut rng);

    // Assert
    assert_eq!(signatures.len(), 10);
    for sig in &signatures {
        assert!(
            space.contains(sig),
            "sampled signature {sig:?} is outside the space"
        );
    }
}

#[test]
fn when_sample_from_polyline_space_then_all_within_radius() {
    // Arrange
    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();
    let sig1 = CardSignature::new([0.2; 8]);
    let sig2 = CardSignature::new([0.8; 8]);
    let space_a = SignatureSpace::from_single(sig1, 0.2, e1);
    let space_b = SignatureSpace::from_single(sig2, 0.2, e2);
    let space = SignatureSpace::combine(&space_a, &space_b);
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    // Act
    let signatures = sample_signatures_from_space(&space, 10, &mut rng);

    // Assert
    assert_eq!(signatures.len(), 10);
    for sig in &signatures {
        assert!(
            space.contains(sig),
            "sampled signature {sig:?} is outside the polyline space"
        );
    }
}

#[test]
fn when_sample_then_all_axes_clamped() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let center = CardSignature::new([0.95; 8]);
    let space = SignatureSpace::from_single(center, 0.3, entity);
    let mut rng = ChaCha8Rng::seed_from_u64(7);

    // Act
    let signatures = sample_signatures_from_space(&space, 20, &mut rng);

    // Assert
    for sig in &signatures {
        for &v in &sig.axes() {
            assert!((-1.0..=1.0).contains(&v), "axis value {v} out of range");
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo.exe test -p card_game booster_sampling -- --nocapture
```

Expected: Compilation error — `sample_signatures_from_space` doesn't exist.

- [ ] **Step 3: Implement sampling**

In `crates/card_game/src/booster/sampling.rs`:

```rust
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::card::identity::signature::CardSignature;
use crate::card::reader::signature_space::SignatureSpace;

/// Sample `count` signatures from within a `SignatureSpace` region.
///
/// For single-point spaces: random 8D offset within the sphere of `space.radius`.
/// For multi-point spaces: pick a random point along the polyline, then offset
/// within the tube radius.
pub fn sample_signatures_from_space(
    space: &SignatureSpace,
    count: usize,
    rng: &mut ChaCha8Rng,
) -> Vec<CardSignature> {
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let base = pick_base_point(space, rng);
        let offset = random_8d_offset(space.radius, rng);
        let mut axes = [0.0_f32; 8];
        let base_axes = base.axes();
        for i in 0..8 {
            axes[i] = base_axes[i] + offset[i];
        }
        result.push(CardSignature::new(axes));
    }
    result
}

/// Pick a random base point along the polyline.
fn pick_base_point(space: &SignatureSpace, rng: &mut ChaCha8Rng) -> CardSignature {
    let points = &space.control_points;
    match points.len() {
        0 => CardSignature::default(),
        1 => points[0],
        _ => {
            let segment_count = if points.len() == 2 { 1 } else { points.len() };
            let mut lengths = Vec::with_capacity(segment_count);
            let mut total = 0.0_f32;
            for i in 0..segment_count {
                let j = (i + 1) % points.len();
                let len = points[i].distance_to(&points[j]);
                lengths.push(len);
                total += len;
            }
            if total < f32::EPSILON {
                return points[0];
            }
            let target = rng.gen_range(0.0..total);
            let mut accum = 0.0_f32;
            for (i, &len) in lengths.iter().enumerate() {
                accum += len;
                if accum >= target {
                    let overshoot = accum - target;
                    let t = if len > f32::EPSILON {
                        1.0 - overshoot / len
                    } else {
                        0.0
                    };
                    let j = (i + 1) % points.len();
                    return lerp_signature(&points[i], &points[j], t);
                }
            }
            *points.last().expect("non-empty points")
        }
    }
}

fn lerp_signature(a: &CardSignature, b: &CardSignature, t: f32) -> CardSignature {
    let aa = a.axes();
    let ba = b.axes();
    let mut axes = [0.0_f32; 8];
    for i in 0..8 {
        axes[i] = aa[i] + t * (ba[i] - aa[i]);
    }
    CardSignature::new(axes)
}

/// Generate a random offset in 8D within a sphere of the given radius.
fn random_8d_offset(radius: f32, rng: &mut ChaCha8Rng) -> [f32; 8] {
    let mut direction = [0.0_f32; 8];
    let mut mag_sq = 0.0_f32;
    for d in &mut direction {
        *d = rng.gen_range(-1.0..1.0);
        mag_sq += *d * *d;
    }
    let mag = mag_sq.sqrt().max(f32::EPSILON);
    let r = radius * rng.gen_range(0.0_f32..1.0).powf(1.0 / 8.0);
    for d in &mut direction {
        *d = *d / mag * r;
    }
    direction
}
```

- [ ] **Step 4: Run tests**

```bash
cargo.exe test -p card_game booster_sampling -- --nocapture
```

Expected: All 5 tests pass (2 from Task 2 + 3 new).

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/booster/sampling.rs crates/card_game/tests/suite/booster_sampling.rs
git commit -m "feat(card_game): implement card signature sampling from SignatureSpace"
```

---

## Task 6: Implement BoosterPack component and spawn

**Files:**
- Modify: `crates/card_game/src/booster/pack.rs`
- Create: `crates/card_game/tests/suite/booster_pack.rs`
- Modify: `crates/card_game/tests/suite/mod.rs`

- [ ] **Step 1: Write failing test for pack spawning**

Create `crates/card_game/tests/suite/booster_pack.rs`:

```rust
#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use card_game::booster::pack::{BoosterPack, spawn_booster_pack};
use card_game::card::component::CardZone;
use card_game::prelude::CardSignature;
use engine_core::prelude::Transform2D;
use engine_physics::prelude::RigidBody;
use glam::Vec2;

#[test]
fn when_spawn_booster_pack_then_has_pack_component() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(engine_core::prelude::EventBus::<engine_physics::prelude::PhysicsCommand>::default());
    let sigs = vec![CardSignature::new([0.3; 8]); 5];

    // Act
    let entity = spawn_booster_pack(&mut world, Vec2::new(100.0, 200.0), sigs.clone());

    // Assert
    let pack = world.get::<BoosterPack>(entity).unwrap();
    assert_eq!(pack.cards.len(), 5);
    let zone = world.get::<CardZone>(entity).unwrap();
    assert_eq!(*zone, CardZone::Table);
    let rb = world.get::<RigidBody>(entity).unwrap();
    assert_eq!(*rb, RigidBody::Dynamic);
}
```

Register in `crates/card_game/tests/suite/mod.rs`:

```rust
mod booster_pack;
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game booster_pack -- --nocapture
```

Expected: Compilation error — `BoosterPack` and `spawn_booster_pack` don't exist.

- [ ] **Step 3: Implement BoosterPack and spawn function**

In `crates/card_game/src/booster/pack.rs`:

```rust
use bevy_ecs::prelude::{Component, Entity, World};
use engine_core::prelude::{EventBus, Transform2D};
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::component::CardZone;
use crate::card::identity::signature::CardSignature;
use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, on_card_clicked};
use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};

/// Friction multiplier for booster packs (heavier than cards).
const PACK_DRAG_MULTIPLIER: f32 = 2.5;

/// Pack is 1.1x card width, 1.3x card height.
pub const PACK_WIDTH: f32 = TABLE_CARD_WIDTH * 1.1;
pub const PACK_HEIGHT: f32 = TABLE_CARD_HEIGHT * 1.3;

/// The card-body portion (excluding jagged edges) is card-sized.
const BODY_HEIGHT: f32 = TABLE_CARD_HEIGHT * 1.1;

/// Height of each jagged edge strip (top and bottom).
const JAGGED_EDGE_HEIGHT: f32 = (PACK_HEIGHT - BODY_HEIGHT) / 2.0;

/// Number of zigzag teeth per edge.
const TEETH_COUNT: usize = 6;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BoosterPack {
    pub cards: Vec<CardSignature>,
}

/// Build polygon points for the pack shape: a rectangle with zigzag top and bottom edges.
fn pack_shape_points() -> Vec<Vec2> {
    let half_w = PACK_WIDTH / 2.0;
    let half_h = PACK_HEIGHT / 2.0;
    let body_half_h = BODY_HEIGHT / 2.0;

    let mut points = Vec::new();

    // Bottom-left corner
    points.push(Vec2::new(-half_w, -body_half_h));

    // Bottom jagged edge (left to right)
    let tooth_w = PACK_WIDTH / TEETH_COUNT as f32;
    for i in 0..TEETH_COUNT {
        let x_start = -half_w + i as f32 * tooth_w;
        let x_mid = x_start + tooth_w * 0.5;
        points.push(Vec2::new(x_mid, -half_h));
        if i < TEETH_COUNT - 1 {
            points.push(Vec2::new(x_start + tooth_w, -body_half_h));
        }
    }

    // Bottom-right corner to top-right corner
    points.push(Vec2::new(half_w, -body_half_h));
    points.push(Vec2::new(half_w, body_half_h));

    // Top jagged edge (right to left)
    for i in (0..TEETH_COUNT).rev() {
        let x_start = -half_w + i as f32 * tooth_w;
        let x_mid = x_start + tooth_w * 0.5;
        if i < TEETH_COUNT - 1 {
            points.push(Vec2::new(x_start + tooth_w, body_half_h));
        }
        points.push(Vec2::new(x_mid, half_h));
    }

    // Top-left corner (close the polygon)
    points.push(Vec2::new(-half_w, body_half_h));

    points
}

pub fn spawn_booster_pack(
    world: &mut World,
    position: Vec2,
    cards: Vec<CardSignature>,
) -> Entity {
    let half = Vec2::new(PACK_WIDTH / 2.0, PACK_HEIGHT / 2.0);

    let pack_fill = engine_core::color::Color {
        r: 0.15,
        g: 0.12,
        b: 0.20,
        a: 1.0,
    };
    let pack_stroke = engine_core::color::Color {
        r: 0.80,
        g: 0.65,
        b: 0.20,
        a: 1.0,
    };

    let entity = world
        .spawn((
            BoosterPack { cards },
            CardZone::Table,
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
            Collider::Aabb(half),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: pack_shape_points(),
                },
                color: pack_fill,
            },
            Stroke {
                color: pack_stroke,
                width: 2.0,
            },
            RenderLayer::World,
            SortOrder::default(),
            Clickable(ClickHitShape::Aabb(half)),
        ))
        .id();

    world.entity_mut(entity).observe(on_card_clicked);

    if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
        bus.push(PhysicsCommand::AddBody {
            entity,
            body_type: RigidBody::Dynamic,
            position,
        });
        bus.push(PhysicsCommand::AddCollider {
            entity,
            collider: Collider::Aabb(half),
        });
        bus.push(PhysicsCommand::SetDamping {
            entity,
            linear: BASE_LINEAR_DRAG * PACK_DRAG_MULTIPLIER,
            angular: BASE_ANGULAR_DRAG * PACK_DRAG_MULTIPLIER,
        });
    }

    entity
}
```

- [ ] **Step 4: Run tests**

```bash
cargo.exe test -p card_game booster_pack -- --nocapture
```

Expected: All pass.

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/booster/pack.rs crates/card_game/tests/suite/booster_pack.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card_game): implement BoosterPack component and spawn function"
```

---

## Task 7: Implement double-click detection

**Files:**
- Modify: `crates/card_game/src/booster/double_click.rs`
- Create: `crates/card_game/tests/suite/booster_double_click.rs`
- Modify: `crates/card_game/tests/suite/mod.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/card_game/tests/suite/booster_double_click.rs`:

```rust
#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use card_game::booster::double_click::{DoubleClickState, DOUBLE_CLICK_WINDOW};
use card_game::booster::pack::BoosterPack;
use card_game::prelude::CardSignature;

#[test]
fn when_same_entity_clicked_within_window_then_double_click_detected() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn(BoosterPack {
            cards: vec![CardSignature::default()],
        })
        .id();
    let mut state = DoubleClickState::default();

    // Act
    let first = state.register_click(entity, 1.0);
    let second = state.register_click(entity, 1.0 + DOUBLE_CLICK_WINDOW * 0.5);

    // Assert
    assert!(first.is_none());
    assert_eq!(second, Some(entity));
}

#[test]
fn when_same_entity_clicked_outside_window_then_no_double_click() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn(BoosterPack {
            cards: vec![CardSignature::default()],
        })
        .id();
    let mut state = DoubleClickState::default();

    // Act
    let first = state.register_click(entity, 1.0);
    let second = state.register_click(entity, 1.0 + DOUBLE_CLICK_WINDOW + 0.1);

    // Assert
    assert!(first.is_none());
    assert!(second.is_none());
}

#[test]
fn when_different_entities_clicked_then_no_double_click() {
    // Arrange
    let mut world = World::new();
    let e1 = world
        .spawn(BoosterPack {
            cards: vec![CardSignature::default()],
        })
        .id();
    let e2 = world
        .spawn(BoosterPack {
            cards: vec![CardSignature::default()],
        })
        .id();
    let mut state = DoubleClickState::default();

    // Act
    let first = state.register_click(e1, 1.0);
    let second = state.register_click(e2, 1.1);

    // Assert
    assert!(first.is_none());
    assert!(second.is_none());
}
```

Register in `crates/card_game/tests/suite/mod.rs`:

```rust
mod booster_double_click;
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo.exe test -p card_game booster_double_click -- --nocapture
```

Expected: Compilation error.

- [ ] **Step 3: Implement double-click detection**

In `crates/card_game/src/booster/double_click.rs`:

```rust
use bevy_ecs::prelude::{Entity, Query, Res, ResMut, Resource, With};
use engine_core::prelude::EventBus;
use engine_input::prelude::MouseState;

use crate::booster::pack::BoosterPack;
use crate::card::interaction::click_resolve::ClickedEntity;
use crate::card::interaction::intent::InteractionIntent;

pub const DOUBLE_CLICK_WINDOW: f32 = 0.3;

#[derive(Resource, Debug, Default)]
pub struct DoubleClickState {
    last_click: Option<(Entity, f32)>,
}

impl DoubleClickState {
    /// Register a click on an entity at the given timestamp.
    /// Returns `Some(entity)` if this constitutes a double-click.
    pub fn register_click(&mut self, entity: Entity, time: f32) -> Option<Entity> {
        if let Some((prev_entity, prev_time)) = self.last_click {
            if prev_entity == entity && (time - prev_time) <= DOUBLE_CLICK_WINDOW {
                self.last_click = None;
                return Some(entity);
            }
        }
        self.last_click = Some((entity, time));
        None
    }
}

pub fn double_click_detect_system(
    mouse: Res<MouseState>,
    mut state: ResMut<DoubleClickState>,
    packs: Query<Entity, With<BoosterPack>>,
    mut intents: ResMut<EventBus<InteractionIntent>>,
) {
    if !mouse.just_pressed(engine_input::mouse_button::MouseButton::Left) {
        return;
    }
    // Check if click_resolve_system found a BoosterPack entity this frame.
    // We piggyback on the existing click resolution by checking which pack
    // entities had ClickedEntity triggered. Since observers fire synchronously
    // during click_resolve, we use a simpler approach: check the mouse position
    // against pack transforms in the system. However, the cleaner approach is
    // to store the last-clicked entity from the observer and check it here.
    //
    // For now, the observer on_card_clicked already fires for BoosterPack entities
    // (they use on_card_clicked). We'll track the clicked entity via the
    // DragState — if a pack was just picked up, register the click.
    //
    // Implementation note: the actual wiring happens through the click observer.
    // The system reads from DoubleClickState which is populated by the observer.
    // If a double-click is detected, emit the intent and cancel the drag.
}
```

**Note to implementer:** The full system wiring (connecting the click observer to `DoubleClickState`) requires coordination with the drag system. The `register_click` method and `DoubleClickState` are the core testable logic. The system function will be refined when wiring into `plugin.rs` (Task 10). For now, export the state and method so tests pass.

- [ ] **Step 4: Run tests**

```bash
cargo.exe test -p card_game booster_double_click -- --nocapture
```

Expected: All 3 pass.

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/booster/double_click.rs crates/card_game/tests/suite/booster_double_click.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card_game): implement double-click detection state machine"
```

---

## Task 8: Implement BoosterMachine device (component, spawn, drag)

**Files:**
- Modify: `crates/card_game/src/booster/device.rs`
- Create: `crates/card_game/tests/suite/booster_device.rs`
- Modify: `crates/card_game/tests/suite/mod.rs`

- [ ] **Step 1: Write failing test for device spawn**

Create `crates/card_game/tests/suite/booster_device.rs`:

```rust
#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use card_game::booster::device::{BoosterMachine, spawn_booster_machine};
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::signature_space::SignatureSpace;
use engine_core::prelude::Transform2D;
use glam::Vec2;

#[test]
fn when_spawn_booster_machine_then_has_components() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(
        engine_core::prelude::EventBus::<engine_physics::prelude::PhysicsCommand>::default(),
    );

    // Act
    let (device, input_jack) = spawn_booster_machine(&mut world, Vec2::new(100.0, 200.0));

    // Assert
    let machine = world.get::<BoosterMachine>(device).unwrap();
    assert_eq!(machine.signal_input, input_jack);
    assert!(machine.output_pack.is_none());

    let jack = world.get::<Jack<SignatureSpace>>(input_jack).unwrap();
    assert_eq!(jack.direction, JackDirection::Input);

    let transform = world.get::<Transform2D>(device).unwrap();
    assert_eq!(transform.position, Vec2::new(100.0, 200.0));
}
```

Register in `crates/card_game/tests/suite/mod.rs`:

```rust
mod booster_device;
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game booster_device -- --nocapture
```

Expected: Compilation error.

- [ ] **Step 3: Implement BoosterMachine**

In `crates/card_game/src/booster/device.rs`:

```rust
use bevy_ecs::prelude::{
    Component, Entity, Query, Res, ResMut, Resource, Trigger, With, Without, World,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::prelude::{LocalSortOrder, SpawnChildExt};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, ClickedEntity};
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, on_socket_clicked};
use crate::card::reader::signature_space::SignatureSpace;

const BODY_HALF_W: f32 = 50.0;
const BODY_HALF_H: f32 = 35.0;
const BODY_CORNER_RADIUS: f32 = 4.0;

const BODY_FILL: Color = Color {
    r: 0.18,
    g: 0.14,
    b: 0.10,
    a: 1.0,
};
const BODY_STROKE: Color = Color {
    r: 0.80,
    g: 0.65,
    b: 0.20,
    a: 1.0,
};
const SOCKET_COLOR: Color = Color {
    r: 0.80,
    g: 0.65,
    b: 0.20,
    a: 1.0,
};
const BUTTON_FILL: Color = Color {
    r: 0.25,
    g: 0.20,
    b: 0.12,
    a: 1.0,
};
const BUTTON_STROKE: Color = Color {
    r: 0.90,
    g: 0.75,
    b: 0.25,
    a: 1.0,
};

const SOCKET_RADIUS: f32 = 8.0;
const DEVICE_LOCAL_SORT: i32 = -1;
const SOCKET_LOCAL_SORT: i32 = 1;
const BUTTON_LOCAL_SORT: i32 = 0;

pub const BOOSTER_HALF_EXTENTS: Vec2 = Vec2::new(BODY_HALF_W, BODY_HALF_H);
const INPUT_X: f32 = -(BODY_HALF_W + SOCKET_RADIUS + 4.0);
const BUTTON_HALF_W: f32 = 18.0;
const BUTTON_HALF_H: f32 = 10.0;
const BUTTON_OFFSET: Vec2 = Vec2::new(20.0, 0.0);

#[derive(Component, Debug)]
pub struct BoosterMachine {
    pub signal_input: Entity,
    pub button_entity: Entity,
    pub output_pack: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct BoosterSealButton {
    pub machine_entity: Entity,
}

/// Spawns a booster machine device at `position`.
///
/// Returns `(device_entity, input_jack)`.
pub fn spawn_booster_machine(world: &mut World, position: Vec2) -> (Entity, Entity) {
    let input_jack = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: SOCKET_RADIUS,
                color: SOCKET_COLOR,
                connected_cable: None,
            },
            Transform2D {
                position: position + Vec2::new(INPUT_X, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Shape {
                variant: ShapeVariant::Circle {
                    radius: SOCKET_RADIUS,
                },
                color: SOCKET_COLOR,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id();

    // We need device_entity to set on BoosterSealButton, so reserve it
    let device_entity = world.spawn_empty().id();

    let button_entity = world
        .spawn((
            BoosterSealButton {
                machine_entity: device_entity,
            },
            Transform2D {
                position: position + BUTTON_OFFSET,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Shape {
                variant: rounded_rect_path(BUTTON_HALF_W, BUTTON_HALF_H, 3.0),
                color: BUTTON_FILL,
            },
            Stroke {
                color: BUTTON_STROKE,
                width: 1.5,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(BUTTON_LOCAL_SORT),
            Clickable(ClickHitShape::Aabb(Vec2::new(BUTTON_HALF_W, BUTTON_HALF_H))),
        ))
        .id();

    world.entity_mut(device_entity).insert((
        BoosterMachine {
            signal_input: input_jack,
            button_entity,
            output_pack: None,
        },
        Transform2D {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Shape {
            variant: rounded_rect_path(BODY_HALF_W, BODY_HALF_H, BODY_CORNER_RADIUS),
            color: BODY_FILL,
        },
        Stroke {
            color: BODY_STROKE,
            width: 2.0,
        },
        RenderLayer::World,
        SortOrder::default(),
        LocalSortOrder(DEVICE_LOCAL_SORT),
        Clickable(ClickHitShape::Aabb(BOOSTER_HALF_EXTENTS)),
        CableCollider::from_aabb(BOOSTER_HALF_EXTENTS),
    ));

    world.entity_mut(device_entity).observe(on_booster_clicked);
    world.entity_mut(input_jack).observe(on_socket_clicked);
    world.entity_mut(button_entity).observe(on_seal_button_clicked);

    (device_entity, input_jack)
}

// --- Drag ---

#[derive(Resource, Debug, Default)]
pub struct BoosterDragState {
    pub dragging: Option<BoosterDragInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoosterDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
}

fn on_booster_clicked(
    trigger: Trigger<ClickedEntity>,
    machines: Query<&Transform2D, With<BoosterMachine>>,
    mut drag: ResMut<BoosterDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = machines.get(entity) else {
        return;
    };
    drag.dragging = Some(BoosterDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}

pub fn booster_drag_system(
    mouse: Res<MouseState>,
    drag: Res<BoosterDragState>,
    mut machine_transforms: Query<&mut Transform2D, With<BoosterMachine>>,
    mut other_transforms: Query<&mut Transform2D, Without<BoosterMachine>>,
    machines: Query<&BoosterMachine>,
) {
    let Some(info) = &drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = machine_transforms.get_mut(info.entity) {
        transform.position = target;
    }
    if let Ok(machine) = machines.get(info.entity) {
        // Move input jack
        if let Ok(mut jack_t) = other_transforms.get_mut(machine.signal_input) {
            jack_t.position = target + Vec2::new(INPUT_X, 0.0);
        }
        // Move button
        if let Ok(mut btn_t) = other_transforms.get_mut(machine.button_entity) {
            btn_t.position = target + BUTTON_OFFSET;
        }
        // Move pack in slot if present
        if let Some(pack_entity) = machine.output_pack {
            if let Ok(mut pack_t) = other_transforms.get_mut(pack_entity) {
                pack_t.position = target + Vec2::new(-20.0, 0.0);
            }
        }
    }
}

pub fn booster_release_system(mouse: Res<MouseState>, mut drag: ResMut<BoosterDragState>) {
    if drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        drag.dragging = None;
    }
}

// --- Seal Button ---

fn on_seal_button_clicked(
    _trigger: Trigger<ClickedEntity>,
    _buttons: Query<&BoosterSealButton>,
    _machines: Query<&BoosterMachine>,
) {
    // Seal logic will be implemented in Task 9.
    // This observer is registered so the button entity is wired up.
}
```

- [ ] **Step 4: Run tests**

```bash
cargo.exe test -p card_game booster_device -- --nocapture
```

Expected: Pass.

- [ ] **Step 5: Run full test suite**

```bash
cargo.exe test -p card_game -- --nocapture
```

Expected: All pass.

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/booster/device.rs crates/card_game/tests/suite/booster_device.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card_game): implement BoosterMachine device with spawn and drag"
```

---

## Task 9: Implement seal button logic (card destruction + pack creation)

**Files:**
- Modify: `crates/card_game/src/booster/device.rs`
- Modify: `crates/card_game/tests/suite/booster_device.rs`

- [ ] **Step 1: Write failing test for seal logic**

Append to `crates/card_game/tests/suite/booster_device.rs`:

```rust
use bevy_ecs::prelude::{Schedule, IntoScheduleConfigs};
use card_game::booster::device::{BoosterDragState, booster_seal_system};
use card_game::booster::pack::BoosterPack;
use card_game::card::component::Card;
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::components::CardReader;
use card_game::card::reader::signature_space::{SignatureSpace, signature_radius};
use engine_core::prelude::EventBus;
use engine_physics::prelude::PhysicsCommand;

#[test]
fn when_seal_pressed_with_signal_then_pack_spawned_and_cards_destroyed() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    let sig = card_game::prelude::CardSignature::new([0.5; 8]);
    let card_entity = world
        .spawn(Card::face_down(
            engine_core::prelude::TextureId(1),
            engine_core::prelude::TextureId(2),
        ))
        .id();

    // Create a reader with the card loaded
    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: Some(SignatureSpace::from_single(
                sig,
                signature_radius(&sig),
                card_entity,
            )),
        })
        .id();
    let reader_entity = world
        .spawn(CardReader::new(Vec2::new(40.0, 55.0), jack_entity))
        .id();
    world.get_mut::<CardReader>(reader_entity).unwrap().loaded = Some(card_entity);

    // Create the booster machine with signal connected
    let (device, input_jack) = spawn_booster_machine(&mut world, Vec2::ZERO);
    // Simulate cable connection: copy signal data to input jack
    let signal = world
        .get::<Jack<SignatureSpace>>(jack_entity)
        .unwrap()
        .data
        .clone();
    world.get_mut::<Jack<SignatureSpace>>(input_jack).unwrap().data = signal;

    // Insert required resources
    world.insert_resource(card_game::booster::device::SealButtonPressed(Some(device)));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(booster_seal_system);
    schedule.run(&mut world);

    // Assert
    let machine = world.get::<BoosterMachine>(device).unwrap();
    assert!(machine.output_pack.is_some(), "pack should be in slot");

    let pack_entity = machine.output_pack.unwrap();
    let pack = world.get::<BoosterPack>(pack_entity).unwrap();
    assert!(!pack.cards.is_empty(), "pack should contain cards");
    assert!(pack.cards.len() >= 5);
    assert!(pack.cards.len() <= 15);

    // Source card should be despawned
    assert!(
        world.get_entity(card_entity).is_err(),
        "source card should be despawned"
    );

    // Reader should be cleared
    let reader = world.get::<CardReader>(reader_entity).unwrap();
    assert!(reader.loaded.is_none(), "reader should be unloaded");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game when_seal_pressed -- --nocapture
```

Expected: Compilation error — `booster_seal_system` and `SealButtonPressed` don't exist.

- [ ] **Step 3: Implement seal system**

Add to `crates/card_game/src/booster/device.rs`:

```rust
use crate::booster::pack::spawn_booster_pack;
use crate::booster::sampling::sample_signatures_from_space;
use crate::card::component::Card;
use crate::card::reader::components::CardReader;
use crate::card::identity::signature::Rarity;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand::Rng;

/// Resource set by the seal button observer to signal the seal system.
#[derive(Resource, Debug, Default)]
pub struct SealButtonPressed(pub Option<Entity>);

pub fn booster_seal_system(world: &mut World) {
    let Some(device_entity) = world
        .get_resource_mut::<SealButtonPressed>()
        .and_then(|mut pressed| pressed.0.take())
    else {
        return;
    };

    let Some(machine) = world.get::<BoosterMachine>(device_entity) else {
        return;
    };

    // Guard: slot must be empty
    if machine.output_pack.is_some() {
        return;
    }

    let input_jack = machine.signal_input;
    let Some(jack) = world.get::<Jack<SignatureSpace>>(input_jack) else {
        return;
    };
    let Some(space) = jack.data.clone() else {
        return;
    };
    if space.source_cards.is_empty() {
        return;
    }

    // Determine card count (5-15, biased by rarity)
    let source_cards = space.source_cards.clone();
    let mut rarity_bonus: usize = 0;
    for &card_entity in &source_cards {
        if let Some(card) = world.get::<Card>(card_entity) {
            let rarity = card.signature.rarity();
            rarity_bonus += match rarity {
                Rarity::Common => 0,
                Rarity::Uncommon => 1,
                Rarity::Rare => 2,
                Rarity::Epic => 3,
                Rarity::Legendary => 4,
            };
        }
    }

    // Seed RNG from space control points for variety
    let seed_bytes: u64 = space
        .control_points
        .iter()
        .flat_map(|cp| cp.axes())
        .fold(0u64, |acc, v| acc.wrapping_add(v.to_bits() as u64));
    let mut rng = ChaCha8Rng::seed_from_u64(seed_bytes);
    let base_count = rng.gen_range(5..=15_usize);
    let count = base_count.saturating_add(rarity_bonus).min(15);

    // Sample signatures
    let signatures = sample_signatures_from_space(&space, count, &mut rng);

    // Spawn pack in the output slot
    let machine_pos = world
        .get::<Transform2D>(device_entity)
        .map(|t| t.position)
        .unwrap_or(Vec2::ZERO);
    let pack_pos = machine_pos + Vec2::new(-20.0, 0.0);
    let pack_entity = spawn_booster_pack(world, pack_pos, signatures);

    // Scale down in slot
    world
        .entity_mut(pack_entity)
        .insert(engine_core::scale_spring::ScaleSpring::new(0.5));

    // Remove physics from pack while in slot
    if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
        bus.push(PhysicsCommand::RemoveBody {
            entity: pack_entity,
        });
    }
    world.entity_mut(pack_entity).remove::<RigidBody>();

    // Set machine output_pack
    if let Some(mut machine) = world.get_mut::<BoosterMachine>(device_entity) {
        machine.output_pack = Some(pack_entity);
    }

    // Destroy source cards and clear readers
    for &card_entity in &source_cards {
        // Find the reader that has this card loaded
        let reader_to_clear: Option<(Entity, Entity)> = {
            let mut query = world.query::<(Entity, &CardReader)>();
            query
                .iter(&world)
                .find(|(_, reader)| reader.loaded == Some(card_entity))
                .map(|(re, reader)| (re, reader.jack_entity))
        };

        if let Some((reader_entity, jack_entity)) = reader_to_clear {
            if let Some(mut reader) = world.get_mut::<CardReader>(reader_entity) {
                reader.loaded = None;
            }
            if let Some(mut jack) = world.get_mut::<Jack<SignatureSpace>>(jack_entity) {
                jack.data = None;
            }
        }

        world.despawn(card_entity);
    }
}
```

Update `on_seal_button_clicked` observer:

```rust
fn on_seal_button_clicked(
    trigger: Trigger<ClickedEntity>,
    buttons: Query<&BoosterSealButton>,
    mut pressed: ResMut<SealButtonPressed>,
) {
    let entity = trigger.target();
    let Ok(button) = buttons.get(entity) else {
        return;
    };
    pressed.0 = Some(button.machine_entity);
}
```

- [ ] **Step 4: Run tests**

```bash
cargo.exe test -p card_game booster_device -- --nocapture
```

Expected: All pass.

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/booster/device.rs crates/card_game/tests/suite/booster_device.rs
git commit -m "feat(card_game): implement seal button logic with card destruction"
```

---

## Task 10: Implement opening animation state machine

**Files:**
- Modify: `crates/card_game/src/booster/opening.rs`
- Create: `crates/card_game/tests/suite/booster_opening.rs`
- Modify: `crates/card_game/tests/suite/mod.rs`

- [ ] **Step 1: Write failing test for phase transitions**

Create `crates/card_game/tests/suite/booster_opening.rs`:

```rust
#![allow(clippy::unwrap_used)]

use card_game::booster::opening::{BoosterOpenPhase, BoosterOpening};
use card_game::prelude::CardSignature;
use glam::Vec2;

#[test]
fn when_opening_advances_then_phases_progress_in_order() {
    // Arrange
    let cards = vec![CardSignature::new([0.3; 8]); 3];
    let mut opening = BoosterOpening::new(
        bevy_ecs::prelude::Entity::PLACEHOLDER,
        cards,
        Vec2::new(100.0, 200.0),
        Vec2::ZERO,
    );

    // Act & Assert — advance through all phases
    assert!(matches!(opening.phase, BoosterOpenPhase::MovingToCenter { .. }));

    // Advance past MovingToCenter
    opening.advance(0.4);
    assert!(matches!(opening.phase, BoosterOpenPhase::Ripping { .. }));

    // Advance past Ripping
    opening.advance(0.5);
    assert!(matches!(opening.phase, BoosterOpenPhase::LoweringPack { .. }));

    // Advance past LoweringPack
    opening.advance(0.4);
    assert!(matches!(opening.phase, BoosterOpenPhase::RevealingCards { .. }));

    // Advance past all card reveals (3 cards × ~0.5s each)
    for _ in 0..3 {
        opening.advance(0.6);
    }
    assert!(matches!(opening.phase, BoosterOpenPhase::Completing { .. }));

    // Advance past Completing
    opening.advance(0.4);
    assert!(matches!(opening.phase, BoosterOpenPhase::Done));
}

#[test]
fn when_opening_reveals_cards_then_spawned_cards_populated() {
    // Arrange
    let cards = vec![CardSignature::new([0.3; 8]); 2];
    let mut opening = BoosterOpening::new(
        bevy_ecs::prelude::Entity::PLACEHOLDER,
        cards,
        Vec2::new(100.0, 200.0),
        Vec2::ZERO,
    );

    // Act — advance to RevealingCards
    opening.advance(0.4); // past MovingToCenter
    opening.advance(0.5); // past Ripping
    opening.advance(0.4); // past LoweringPack

    // Now in RevealingCards — each advance should increment card_index
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::RevealingCards { card_index: 0, .. }
    ));
    opening.advance(0.6);
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::RevealingCards { card_index: 1, .. }
    ));
}
```

Register in `crates/card_game/tests/suite/mod.rs`:

```rust
mod booster_opening;
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo.exe test -p card_game booster_opening -- --nocapture
```

Expected: Compilation error.

- [ ] **Step 3: Implement opening state machine**

In `crates/card_game/src/booster/opening.rs`:

```rust
use bevy_ecs::prelude::{Entity, Resource, World};
use engine_core::prelude::Transform2D;
use glam::Vec2;

use crate::card::identity::signature::CardSignature;

const MOVE_TO_CENTER_DURATION: f32 = 0.3;
const RIPPING_DURATION: f32 = 0.4;
const LOWERING_DURATION: f32 = 0.3;
const REVEAL_DURATION: f32 = 0.5;
const COMPLETING_DURATION: f32 = 0.3;

/// Fan arc angle (radians) for placing cards on the table.
const FAN_ARC: f32 = std::f32::consts::PI * 0.6;
/// Distance from pack center for fan placement.
const FAN_RADIUS: f32 = 80.0;

#[derive(Debug, Clone)]
pub enum BoosterOpenPhase {
    MovingToCenter {
        start_pos: Vec2,
        progress: f32,
    },
    Ripping {
        progress: f32,
    },
    LoweringPack {
        progress: f32,
    },
    RevealingCards {
        card_index: usize,
        card_progress: f32,
    },
    Completing {
        progress: f32,
    },
    Done,
}

#[derive(Resource, Debug, Clone)]
pub struct BoosterOpening {
    pub pack_entity: Entity,
    pub phase: BoosterOpenPhase,
    pub cards: Vec<CardSignature>,
    pub original_position: Vec2,
    pub screen_center: Vec2,
    pub spawned_cards: Vec<Entity>,
}

impl BoosterOpening {
    pub fn new(
        pack_entity: Entity,
        cards: Vec<CardSignature>,
        original_position: Vec2,
        screen_center: Vec2,
    ) -> Self {
        Self {
            pack_entity,
            phase: BoosterOpenPhase::MovingToCenter {
                start_pos: original_position,
                progress: 0.0,
            },
            cards,
            original_position,
            screen_center,
            spawned_cards: Vec::new(),
        }
    }

    /// Advance the state machine by `dt` seconds.
    /// Returns phase transitions for the caller to act on (spawn cards, move entities, etc.).
    pub fn advance(&mut self, dt: f32) {
        match &mut self.phase {
            BoosterOpenPhase::MovingToCenter { progress, .. } => {
                *progress += dt / MOVE_TO_CENTER_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::Ripping { progress: 0.0 };
                }
            }
            BoosterOpenPhase::Ripping { progress } => {
                *progress += dt / RIPPING_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::LoweringPack { progress: 0.0 };
                }
            }
            BoosterOpenPhase::LoweringPack { progress } => {
                *progress += dt / LOWERING_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::RevealingCards {
                        card_index: 0,
                        card_progress: 0.0,
                    };
                }
            }
            BoosterOpenPhase::RevealingCards {
                card_index,
                card_progress,
            } => {
                *card_progress += dt / REVEAL_DURATION;
                if *card_progress >= 1.0 {
                    let next_index = *card_index + 1;
                    if next_index >= self.cards.len() {
                        self.phase = BoosterOpenPhase::Completing { progress: 0.0 };
                    } else {
                        *card_index = next_index;
                        *card_progress = 0.0;
                    }
                }
            }
            BoosterOpenPhase::Completing { progress } => {
                *progress += dt / COMPLETING_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::Done;
                }
            }
            BoosterOpenPhase::Done => {}
        }
    }

    /// Compute the fan position for card at `index` out of `total` cards.
    pub fn fan_position(&self, index: usize, total: usize) -> Vec2 {
        if total <= 1 {
            return self.original_position;
        }
        let angle_start = -FAN_ARC / 2.0;
        let angle_step = FAN_ARC / (total - 1) as f32;
        let angle = angle_start + angle_step * index as f32;
        self.original_position + Vec2::new(angle.cos(), angle.sin()) * FAN_RADIUS
    }

    pub fn is_done(&self) -> bool {
        matches!(self.phase, BoosterOpenPhase::Done)
    }
}

/// System that drives the opening animation each frame.
/// Reads `BoosterOpening` resource, advances the state machine,
/// and applies visual changes to entities.
pub fn booster_opening_system(world: &mut World) {
    let Some(dt) = world
        .get_resource::<engine_core::prelude::FrameTime>()
        .map(|ft| ft.delta_seconds())
    else {
        return;
    };

    let Some(mut opening) = world.remove_resource::<BoosterOpening>() else {
        return;
    };

    let prev_phase = std::mem::discriminant(&opening.phase);
    opening.advance(dt);

    // Apply visual effects based on current phase
    match &opening.phase {
        BoosterOpenPhase::MovingToCenter {
            start_pos,
            progress,
        } => {
            let t = progress.clamp(0.0, 1.0);
            let pos = *start_pos + (opening.screen_center - *start_pos) * t;
            if let Some(mut transform) = world.get_mut::<Transform2D>(opening.pack_entity) {
                transform.position = pos;
            }
        }
        BoosterOpenPhase::LoweringPack { progress } => {
            let t = progress.clamp(0.0, 1.0);
            let lowered_y = opening.screen_center.y + 200.0 * t;
            if let Some(mut transform) = world.get_mut::<Transform2D>(opening.pack_entity) {
                transform.position.y = lowered_y;
            }
        }
        BoosterOpenPhase::RevealingCards {
            card_index,
            card_progress,
        } => {
            let idx = *card_index;
            let t = card_progress.clamp(0.0, 1.0);

            // Spawn card entity if this is the start of a new reveal
            if t < 0.05 && idx >= opening.spawned_cards.len() {
                if let Some(sig) = opening.cards.get(idx).copied() {
                    let pack_pos = world
                        .get::<Transform2D>(opening.pack_entity)
                        .map(|t| t.position)
                        .unwrap_or(opening.screen_center);

                    let card_def =
                        crate::card::identity::definition::CardDefinition::from_signature(&sig);
                    let card_entity = crate::card::rendering::spawn_table_card::spawn_visual_card(
                        world,
                        &card_def,
                        pack_pos,
                        crate::card::rendering::geometry::TABLE_CARD_SIZE,
                        true,
                        sig,
                    );
                    // Remove physics during reveal animation
                    if let Some(mut bus) =
                        world.get_resource_mut::<engine_core::prelude::EventBus<
                            engine_physics::prelude::PhysicsCommand,
                        >>()
                    {
                        bus.push(engine_physics::prelude::PhysicsCommand::RemoveBody {
                            entity: card_entity,
                        });
                    }
                    world
                        .entity_mut(card_entity)
                        .remove::<engine_physics::prelude::RigidBody>();
                    opening.spawned_cards.push(card_entity);
                }
            }

            // Animate the current card upward
            if let Some(&card_entity) = opening.spawned_cards.get(idx) {
                let pack_pos = world
                    .get::<Transform2D>(opening.pack_entity)
                    .map(|tr| tr.position)
                    .unwrap_or(opening.screen_center);
                let reveal_y = pack_pos.y - 120.0 * t;
                if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                    transform.position = Vec2::new(pack_pos.x, reveal_y);
                }
            }
        }
        BoosterOpenPhase::Completing { progress } => {
            let t = progress.clamp(0.0, 1.0);
            let total = opening.spawned_cards.len();

            // Move cards to fan positions
            for (i, &card_entity) in opening.spawned_cards.iter().enumerate() {
                let target = opening.fan_position(i, total);
                if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                    let current = transform.position;
                    transform.position = current + (target - current) * t;
                }
            }

            // Slide pack off-screen
            if let Some(mut transform) = world.get_mut::<Transform2D>(opening.pack_entity) {
                transform.position.y += 300.0 * t;
            }
        }
        BoosterOpenPhase::Done => {
            // Give all cards physics bodies
            for &card_entity in &opening.spawned_cards {
                if let Some(transform) = world.get::<Transform2D>(card_entity) {
                    let pos = transform.position;
                    world
                        .entity_mut(card_entity)
                        .insert(engine_physics::prelude::RigidBody::Dynamic);
                    if let Some(collider) =
                        world.get::<engine_physics::prelude::Collider>(card_entity).copied()
                    {
                        if let Some(mut bus) =
                            world.get_resource_mut::<engine_core::prelude::EventBus<
                                engine_physics::prelude::PhysicsCommand,
                            >>()
                        {
                            bus.push(engine_physics::prelude::PhysicsCommand::AddBody {
                                entity: card_entity,
                                body_type: engine_physics::prelude::RigidBody::Dynamic,
                                position: pos,
                            });
                            bus.push(engine_physics::prelude::PhysicsCommand::AddCollider {
                                entity: card_entity,
                                collider,
                            });
                            bus.push(engine_physics::prelude::PhysicsCommand::SetDamping {
                                entity: card_entity,
                                linear: crate::card::interaction::damping::BASE_LINEAR_DRAG,
                                angular: crate::card::interaction::damping::BASE_ANGULAR_DRAG,
                            });
                        }
                    }
                }
            }

            // Despawn the pack wrapper
            world.despawn(opening.pack_entity);

            // Don't re-insert the resource — animation is complete
            return;
        }
        _ => {}
    }

    world.insert_resource(opening);
}
```

**Note:** The `CardDefinition::from_signature` method may not exist. Check if there's a way to construct a `CardDefinition` from a signature. If not, the implementer should create a minimal constructor or use `CardDefinition::default()` and adjust. The existing `spawn_visual_card` already handles generating name/description/art from the signature — the `CardDefinition` is used for stats lookup. A default `CardDefinition` is sufficient.

- [ ] **Step 4: Run tests**

```bash
cargo.exe test -p card_game booster_opening -- --nocapture
```

Expected: All pass.

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/booster/opening.rs crates/card_game/tests/suite/booster_opening.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card_game): implement booster pack opening animation state machine"
```

---

## Task 11: Integrate into store, plugin, and prelude

**Files:**
- Modify: `crates/card_game/src/stash/store.rs`
- Modify: `crates/card_game/src/plugin.rs`
- Modify: `crates/card_game/src/prelude.rs`

- [ ] **Step 1: Add BoosterMachine to StoreItemKind**

In `crates/card_game/src/stash/store.rs`:

Add `BoosterMachine` variant to `StoreItemKind`:

```rust
pub enum StoreItemKind {
    Reader,
    Screen,
    Combiner,
    BoosterMachine,
}
```

Add match arms to all `StoreItemKind` methods:

```rust
// label
Self::BoosterMachine => "Booster Machine",

// cost
Self::BoosterMachine => 35,

// preview_color
Self::BoosterMachine => Color {
    r: 0.56,
    g: 0.44,
    b: 0.12,
    a: 1.0,
},
```

Add to `StoreCatalog::default()`:

```rust
items: vec![
    StoreItemKind::Reader,
    StoreItemKind::Screen,
    StoreItemKind::Combiner,
    StoreItemKind::BoosterMachine,
],
```

Add store preview drawing function and spawn handler — follow the pattern of `draw_combiner_preview` and `spawn_combiner_purchase`:

```rust
// In the match in draw_store_item:
StoreItemKind::BoosterMachine => {
    draw_booster_preview(renderer, camera, viewport_w, viewport_h, left, top);
}

// In the match in store_buy_system:
StoreItemKind::BoosterMachine => {
    let entity = spawn_booster_purchase(world, spawn_pos);
    world.resource_mut::<BoosterDragState>().dragging = Some(BoosterDragInfo {
        entity,
        grab_offset: Vec2::ZERO,
    });
}
```

Add `spawn_booster_purchase` function:

```rust
fn spawn_booster_purchase(world: &mut World, position: Vec2) -> Entity {
    let (device_entity, _jack_entity) = spawn_booster_machine(world, position);
    device_entity
}
```

Add `draw_booster_preview` function (simple rounded rect with gold stroke):

```rust
fn draw_booster_preview(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    left: f32,
    top: f32,
) {
    let cx = left + STORE_ITEM_WIDTH * 0.5;
    let cy = top + STORE_HEADER_HEIGHT + 50.0;
    draw_screen_rect(
        renderer, camera, viewport_w, viewport_h,
        cx - 30.0, cy - 20.0, 60.0, 40.0,
        Color { r: 0.18, g: 0.14, b: 0.10, a: 1.0 },
    );
}
```

Add drag check for booster in `store_buy_system` drag_active block:

```rust
|| world
    .get_resource::<BoosterDragState>()
    .is_some_and(|drag| drag.dragging.is_some())
```

Add sell handler for BoosterMachine — follow the pattern of `store_sell_system` for Combiner. Find the device, despawn connected cables and entities.

Add required imports at the top of `store.rs`:

```rust
use crate::booster::device::{
    BoosterDragInfo, BoosterDragState, BoosterMachine, spawn_booster_machine,
};
```

- [ ] **Step 2: Update plugin.rs**

In `crates/card_game/src/plugin.rs`, add imports and registration:

```rust
use crate::booster::device::{
    BoosterDragState, SealButtonPressed, booster_drag_system, booster_release_system,
    booster_seal_system,
};
use crate::booster::double_click::DoubleClickState;
use crate::booster::opening::booster_opening_system;
```

In `CardGamePlugin::build`, insert new resources:

```rust
world.insert_resource(BoosterDragState::default());
world.insert_resource(DoubleClickState::default());
world.insert_resource(SealButtonPressed::default());
```

In `register_systems`, add systems:

```rust
// In Phase::Input, after click_resolve_system:
app.add_systems(
    Phase::Input,
    (click_resolve_system, double_click_detect_system).chain(),
);
// (Remove the existing standalone click_resolve_system registration)

// In the chained Phase::Update block, add:
booster_drag_system,
booster_release_system,
booster_seal_system,

// In Phase::Animate:
app.add_systems(Phase::Animate, booster_opening_system);
```

- [ ] **Step 3: Update prelude.rs**

Add key booster exports to `crates/card_game/src/prelude.rs`:

```rust
pub use crate::booster::device::{
    BoosterDragState, BoosterMachine, booster_drag_system, booster_release_system,
    booster_seal_system, spawn_booster_machine,
};
pub use crate::booster::double_click::{DoubleClickState, double_click_detect_system};
pub use crate::booster::opening::{BoosterOpening, booster_opening_system};
pub use crate::booster::pack::{BoosterPack, spawn_booster_pack};
pub use crate::booster::sampling::sample_signatures_from_space;
```

- [ ] **Step 4: Build everything**

```bash
cargo.exe build -p card_game && cargo.exe build -p card_game_bin
```

Expected: Both compile successfully. Fix any compilation errors.

- [ ] **Step 5: Run full test suite**

```bash
cargo.exe test -p card_game -- --nocapture
```

Expected: All tests pass.

- [ ] **Step 6: Format and lint**

```bash
cargo.exe fmt --all && cargo.exe clippy -p card_game -p card_game_bin
```

Fix any issues.

- [ ] **Step 7: Commit**

```bash
git add crates/card_game/src/stash/store.rs crates/card_game/src/plugin.rs crates/card_game/src/prelude.rs crates/card_game/src/booster/mod.rs
git commit -m "feat(card_game): integrate booster machine into store, plugin, and prelude"
```

---

## Task 12: Wire double-click to opening animation

**Files:**
- Modify: `crates/card_game/src/booster/double_click.rs`
- Modify: `crates/card_game/src/card/interaction/intent.rs` (if needed)
- Modify: `crates/card_game/src/plugin.rs` (if needed)

- [ ] **Step 1: Implement full double_click_detect_system**

Update `double_click_detect_system` in `crates/card_game/src/booster/double_click.rs` to:
1. When a card drag starts on a `BoosterPack` entity, register the click in `DoubleClickState`
2. If double-click detected, cancel the drag, remove physics, and insert `BoosterOpening` resource

```rust
pub fn double_click_detect_system(
    mouse: Res<MouseState>,
    time: Res<engine_core::prelude::FrameTime>,
    mut state: ResMut<DoubleClickState>,
    mut drag: ResMut<crate::card::interaction::drag_state::DragState>,
    packs: Query<&BoosterPack>,
    transforms: Query<&engine_core::prelude::Transform2D>,
    existing_opening: Option<Res<BoosterOpening>>,
    mut commands: bevy_ecs::prelude::Commands,
) {
    // Don't allow opening while another opening is in progress
    if existing_opening.is_some() {
        return;
    }

    if !mouse.just_pressed(engine_input::mouse_button::MouseButton::Left) {
        return;
    }

    let Some(drag_info) = drag.dragging.as_ref() else {
        return;
    };
    let entity = drag_info.entity;

    let Ok(pack) = packs.get(entity) else {
        return;
    };

    let now = time.total_seconds();
    if let Some(double_clicked) = state.register_click(entity, now) {
        // Cancel the drag
        drag.dragging = None;

        let original_pos = transforms
            .get(entity)
            .map(|t| t.position)
            .unwrap_or(glam::Vec2::ZERO);

        let screen_center = mouse.world_pos(); // approximate; will be refined by camera

        let opening = crate::booster::opening::BoosterOpening::new(
            entity,
            pack.cards.clone(),
            original_pos,
            screen_center,
        );
        commands.insert_resource(opening);
    }
}
```

- [ ] **Step 2: Build and test**

```bash
cargo.exe build -p card_game_bin && cargo.exe test -p card_game -- --nocapture
```

Expected: All pass.

- [ ] **Step 3: Commit**

```bash
git add crates/card_game/src/booster/double_click.rs
git commit -m "feat(card_game): wire double-click detection to booster opening animation"
```

---

## Task 13: End-to-end build verification and cleanup

**Files:** All modified files

- [ ] **Step 1: Full build**

```bash
cargo.exe build -p card_game -p card_game_bin
```

- [ ] **Step 2: Full test suite**

```bash
cargo.exe test -p card_game -- --nocapture
```

- [ ] **Step 3: Format and lint**

```bash
cargo.exe fmt --all && cargo.exe clippy -p card_game -p card_game_bin -- -W clippy::pedantic
```

Fix any warnings.

- [ ] **Step 4: Run tests one more time after fixes**

```bash
cargo.exe test -p card_game -- --nocapture
```

- [ ] **Step 5: Commit any cleanup**

```bash
git add -A && git commit -m "chore: fmt and clippy cleanup for booster pack feature"
```

---

## Implementation Notes

- **`CardDefinition::from_signature`** may not exist. Check `crates/card_game/src/card/identity/definition.rs` for how to construct one. The `spawn_visual_card` function handles most of the card identity generation from the signature — the `CardDefinition` provides base type data. Use `CardDefinition::default()` or create a constructor if needed.
- **`FrameTime`** resource: check what the actual type name is in `engine_core`. It might be `Time`, `DeltaTime`, or similar. Search with `cargo.exe grep "delta_seconds\|FrameTime" -- crates/engine_core/`.
- **The opening animation's `screen_center`** should ideally come from the camera's viewport center in world coordinates. Use `resolve_viewport_camera` + `screen_to_world` to convert screen center to world position. The exact wiring depends on what's available in the schedule phase.
- **Store sell handler** for BoosterMachine: follow the Combiner sell pattern in `store_sell_system`. Despawn device, jack, button, and any pack in the slot.
- **Input suppression during opening**: The simplest approach is to check `world.contains_resource::<BoosterOpening>()` at the top of drag/interaction systems and early-return if true.
