# Booster Pack System Design

## Overview

A booster pack system that consumes cards loaded in readers (via their SignatureSpace signal) and produces sealed packs containing 5-15 new cards sampled from the same bezier curve region. Packs are physical table entities that can be opened with a cinematic animation.

## Cross-Cutting Change: SignatureSpace Entity Back-References

`SignatureSpace` gains a `source_cards: Vec<Entity>` field tracking which card entities in readers produced the signal.

```rust
pub struct SignatureSpace {
    pub control_points: Vec<CardSignature>,
    pub radius: f32,
    pub volume: f32,
    pub source_cards: Vec<Entity>,  // NEW
}
```

- `from_single(center, radius, source)` — stores `vec![source]`
- `combine(a, b)` — merges both `source_cards` vecs, deduplicates
- Reader insert system passes the card entity when creating the space
- Propagation and combiner systems already clone `SignatureSpace` — the new field rides along

## BoosterMachine Device

A purchasable table device following the Combiner/Screen pattern.

### Component

```rust
pub struct BoosterMachine {
    pub signal_input: Entity,        // Jack<SignatureSpace> input
    pub button_entity: Entity,       // Clickable "Seal" button child
    pub output_pack: Option<Entity>, // Pack in the output slot
}
```

### Behavior

- **Store**: `StoreItemKind::BoosterMachine`, cost 35 coins
- **Body**: Rounded rect (~80x60 half-extents), gold/amber color scheme
- **Jack**: Single input jack on the left side (`JackDirection::Input`)
- **Output slot**: Visual slot area on the right side. Pack spawns here at reduced scale. Picking up the pack removes it from the slot. Nothing can be placed back into the slot.
- **Seal button**: Clickable child entity on the device body
- **Drag**: Own `BoosterDragState` resource, `booster_drag_system` / `booster_release_system` (Combiner pattern)

### Seal Button Logic

1. Check `output_pack.is_none()` — if occupied, no-op
2. Read `SignatureSpace` from the input jack
3. If `source_cards` is empty, no-op
4. Sample 5-15 `CardSignature` points from within the SignatureSpace region
5. Spawn a `BoosterPack` entity in the output slot
6. Destroy the source card entities: for each entity in `source_cards`, find the `CardReader` that has `loaded == Some(entity)`, set `reader.loaded = None`, clear the reader's output jack data (`jack.data = None`), then despawn the card entity

### Spawn Function

```rust
pub fn spawn_booster_machine(world: &mut World, position: Vec2) -> (Entity, Entity)
// Returns (device_entity, input_jack)
```

Follows `spawn_combiner_device` pattern: spawn jack entity, device entity, button child, register observers.

## BoosterPack Entity

### Component

```rust
pub struct BoosterPack {
    pub cards: Vec<CardSignature>,
    pub source_space: SignatureSpace,
}
```

### Physical Properties

- **Dimensions**: 1.1x card width, 1.3x card height
- **Shape**: Card-shaped body with jagged/serrated zigzag edges on top and bottom (the extra 0.2x height). Polygon points form the zigzag pattern.
- **Visual**: Dark wrapper fill, gold/amber stroke, card count badge as text overlay
- **Physics**: `RigidBody::Dynamic`, `Collider::Aabb` matching full bounds. Higher damping than regular cards (2-3x `BASE_LINEAR_DRAG` and `BASE_ANGULAR_DRAG`).
- **Zone**: `CardZone::Table` only — not stashable
- **Clickable**: `ClickHitShape::Aabb` for drag interaction
- **Drag**: Reuses existing card drag system (same drag components as cards)

### Lightweight Design

The pack stores only `Vec<CardSignature>`, not spawned card entities. Real card entities are created only when the pack is opened.

## Double-Click Detection

New reusable system, not currently in the codebase.

```rust
pub struct DoubleClickState {
    pub last_click: Option<(Entity, f32)>, // (entity, timestamp)
}

pub const DOUBLE_CLICK_WINDOW: f32 = 0.3;
```

- Runs in `Phase::Input` after `click_resolve_system`
- When a `ClickedEntity` fires on the same entity within the time window, emits `InteractionIntent::DoubleClick(Entity)`
- Movement threshold: if the entity moved significantly between clicks, treat as two separate drags, not a double-click
- Only acts on `BoosterPack` entities for now, but detection is generic

## Opening Animation

Multi-phase state machine triggered by double-clicking a `BoosterPack`.

### State Machine

```rust
pub enum BoosterOpenPhase {
    MovingToCenter { start_pos: Vec2, original_pos: Vec2, progress: f32 },
    Ripping { progress: f32 },
    LoweringPack { progress: f32 },
    RevealingCards { card_index: usize, card_progress: f32 },
    Completing { progress: f32 },
}

pub struct BoosterOpening {
    pub pack_entity: Entity,
    pub phase: BoosterOpenPhase,
    pub cards: Vec<CardSignature>,
    pub original_position: Vec2,
    pub spawned_cards: Vec<Entity>,
}
```

### Phase Sequence

1. **MovingToCenter** (~0.3s) — Pack animates from table position to screen center. Physics body removed.

2. **Ripping** (~0.4s) — Top jagged edge separates upward and fades out. Pack transitions to "open at top" look.

3. **LoweringPack** (~0.3s) — Pack slides down so its top edge is at screen center, bottom half off-screen. Creates space above for cards to emerge.

4. **RevealingCards** (~0.5s per card) — Each card spawns behind the pack (lower sort order), slides upward past the pack's top edge into view. Brief face-up pause at the reveal position above center, then the card animates to its fan position at the pack's original table location. Next card begins emerging while the previous is traveling.

5. **Completing** (~0.3s) — Pack wrapper slides off-screen downward, despawns. All cards are on the table in a fan arc around the original position with physics bodies.

### Fan Layout

Cards are placed in an arc/fan pattern centered on the pack's original table position. Angular spread based on card count. Cards receive physics bodies after placement.

### Input Suppression

While `BoosterOpening` resource exists, normal card/device interaction is suppressed (no dragging during the animation).

## Card Sampling

### Count Determination

- Base range: 5-15
- Seeded RNG: `ChaCha8Rng`, seed derived from `SignatureSpace` control points + monotonic counter
- Rarity bias: +1 card for each source card above Common rarity

### Sampling Algorithm

- **Single-point space** (1 control point): Random 8D direction scaled by `rng.gen_range(0.0..=radius)`
- **Multi-point space** (polyline tube): Pick random position along polyline (weighted by segment length), offset by random direction within tube radius
- All sampled values clamped to [-1.0, 1.0] via `CardSignature::new()`

## System Registration

### Phase Placement (in `card_game_bin/src/main.rs`)

- **`Phase::Input`**: `double_click_detect_system` (after `click_resolve_system`)
- **`Phase::Update`** (chained with interaction systems):
  - `booster_seal_system`
  - `booster_drag_system` / `booster_release_system`
  - `booster_pack_pickup_system`
- **`Phase::Animate`**: `booster_opening_system`

### New Resources

- `BoosterDragState`
- `DoubleClickState`

### Store Changes

- `StoreItemKind::BoosterMachine` (35 coins)
- Spawn handler in `store_buy_system`

## Module Structure

```
crates/card_game/src/booster/
  mod.rs          — pub uses, module declarations
  device.rs       — BoosterMachine component, spawn, drag, seal button
  pack.rs         — BoosterPack component, spawn, pickup, physics
  sampling.rs     — sample_signatures_from_space()
  opening.rs      — BoosterOpening state machine, animation system
  double_click.rs — DoubleClickState, double_click_detect_system
```
