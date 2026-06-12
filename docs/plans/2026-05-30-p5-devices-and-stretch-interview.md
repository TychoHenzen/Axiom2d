# Devices, Simulation & Stretch Goals (Priority 5+) — Requirements Spec (Stub)

> **For Claude (/goal):** Work through each incomplete step below.
> 1. Mark a step `[>]` when you begin working on it.
> 2. Verify each proof by running the stated command/process and confirming the expected outcome.
> 3. Mark each proof `[x]` only when the claim has been tested and matches the expected value.
> 4. A step may only be marked `[x]` once ALL its proofs are `[x]` or `[~]`.
> 5. If a proof cannot be met because requirements changed or the original condition is unreasonable:
>    - Mark it `[~]` with the original condition struck through.
>    - Add a bullet underneath: `  - Met instead: [what was actually achieved]`
>    - The step can still be `[x]` once all proofs are resolved (either `[x]` or `[~]`).
> 6. Continue until every step is `[x]` — then stop and report done.
>
> **Self-contained.** No external context needed. Run the commands listed in proofs directly.
>
> ⚠️ **Stub spec.** Requirements are derived from backlog one-liners. Run `/interview` on individual items before implementing to fill in behavioral details, edge cases, and error handling.

---

## Goal

Implement device infrastructure for the card game and stretch-goal features. Devices are interactive game objects with physical connections (jacks, cables, signals) that process card data. Stretch goals extend the world generation pipeline with advanced terrain and gameplay systems.

---

## Priority 5 — Devices & Simulation

### Step 1: I30 — Jack and cable infrastructure `[ ]`

Implement a physical connection system between devices — jacks (connection points on entities), cables (visual + logical links between jacks), and signal routing (data flows along cables). Jacks are components on entities; cables are entities connecting two jacks. Signal propagation runs each frame.

**Cross-reference:** The rope/cable physics (visual simulation, wrapping, sag) is covered in `docs/plans/2026-05-30-rope-physics-interview.md`. This step owns the logical signal routing and jack component API. The rope spec owns how cables look and physically behave.

Module: `crates/card_game/src/device/jack.rs` and `crates/card_game/src/device/cable.rs`. Systems wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "pub struct Jack\|pub struct Cable\|SignalValue" crates/card_game/src/device/` matches at least 3 results
- [ ] `rtk cargo test -p card_game -- jack\|cable\|signal_rout` exits 0, at least 3 tests pass
- [ ] `rtk grep "jack\|cable\|signal" crates/card_game_bin/src/main.rs` matches (systems registered)

---

### Step 2: I31 — Card slot devices and signature chaining `[ ]`

Implement devices with card slots — a device can accept a card, read its signature, and output a signal derived from the card's properties. Multiple devices chained via cables compose their signature transformations. Depends on I30 (jack/cable infrastructure). Module: `crates/card_game/src/device/card_slot.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "CardSlot\|card_slot\|SignatureChain\|signature_chain" crates/card_game/src/device/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- card_slot\|signature_chain` exits 0, at least 2 tests pass

---

### Step 3: I32 — Screen and button devices `[ ]`

Implement interactive devices with visual displays (render text/numbers/status from signal input) and buttons (emit signal on player click). Screens read from input jacks; buttons write to output jacks on interaction. Depends on I30. Module: `crates/card_game/src/device/screen.rs` and `crates/card_game/src/device/button.rs`. Systems wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "ScreenDevice\|ButtonDevice\|pub struct Screen\|pub struct Button" crates/card_game/src/device/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- screen_device\|button_device` exits 0
- [ ] `rtk grep "screen\|button" crates/card_game_bin/src/main.rs` matches (systems registered)

---

### Step 4: I33 — Conveyor belt transport system `[ ]`

Implement conveyor belts that move cards/items along a defined path. Conveyors are entities with a path (sequence of waypoints); items placed on a conveyor move toward the output end at a configurable speed. Integrates with physics for collision and with the jack system for triggering signals when items pass checkpoints. Module: `crates/card_game/src/device/conveyor.rs`. System wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "Conveyor\|ConveyorBelt\|conveyor_system" crates/card_game/src/device/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- conveyor` exits 0, at least 2 tests pass
- [ ] `rtk grep "conveyor" crates/card_game_bin/src/main.rs` matches (system registered)

---

## Stretch Goals

### Step 5: I34 — Irregular quad mesh generation `[ ]`

Generate non-rectangular quad meshes for organic terrain shapes — subdivide a region into irregular quads that conform to terrain boundaries, biome edges, and elevation contours. Depends on world generation pipeline (P4: I25, I26, I29). Module: `crates/card_game/src/world/irregular_mesh.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "IrregularMesh\|QuadMesh\|irregular_quad" crates/card_game/src/world/` matches at least 1 result
- [ ] `rtk cargo test -p card_game -- irregular_mesh\|quad_mesh` exits 0

---

### Step 6: I35 — Structure placement on maps `[ ]`

Place buildings, landmarks, and other structures on generated terrain. Structures occupy tile regions, have placement rules (valid terrain types, minimum spacing, road adjacency), and affect gameplay (resource production, defense, etc.). Depends on world generation pipeline (P4: I25, I26). Module: `crates/card_game/src/world/structure.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "Structure\|StructurePlacement\|placement_rule" crates/card_game/src/world/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- structure_place\|structure` exits 0

---

### Step 7: I36 — Enemy spawning and management `[ ]`

Spawn enemies on the world map based on biome type and distance from the player's starting position. Manage enemy lifecycle (spawn, patrol, engage, despawn). Enemies interact with the turn-based combat system (P4: I13). Depends on world generation (P4) and combat (I13). Module: `crates/card_game/src/combat/enemy.rs`. System wired into `crates/card_game_bin/src/main.rs`.

**Proofs:**

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "EnemySpawner\|EnemyType\|spawn_enemies\|enemy_system" crates/card_game/src/combat/` matches at least 2 results
- [ ] `rtk cargo test -p card_game -- enemy\|spawner` exits 0
- [ ] `rtk grep "enemy" crates/card_game_bin/src/main.rs` matches (system registered)

---

## Dependency Graph

```
Priority 5 — Devices:
  I30 (jacks/cables) ──► I31 (card slots)
                    ──► I32 (screens/buttons)
                    ──► I33 (conveyors)

  Rope physics spec ◄── I30 (visual cable behavior)

Stretch Goals:
  P4 world gen ──► I34 (irregular mesh)
              ──► I35 (structure placement)
              ──► I36 (enemy spawning) ◄── I13 (combat, also P4)
```

## Open Questions

- **Signal routing architecture (I30):** What is the signal type? Untyped `f32`? Typed enum (`SignalValue::Number(f32) | SignalValue::Signature(...)`)? How does signal propagation handle cycles in the cable graph — skip, cap iterations, or error? This is the foundational design decision for the entire device system. Run `/interview` before implementing.
- **I30 vs rope-physics boundary:** The rope physics spec owns visual cable simulation (Verlet/geometric wire, wrapping, sag). I30 owns logical signal routing. Where does "cable entity spawning" live? Who creates the entity — the device system or the physics system? Run `/interview` to clarify ownership.
- **Card slot interaction model (I31):** How does a card enter a slot — drag-and-drop (reuse existing pick/release)? Auto-insert on proximity? Can the player remove a card from a slot? Does inserting a card consume it? Run `/interview` to scope.
- **Screen rendering (I32):** Screens need to render dynamic text/numbers. This must integrate with the existing `unified_render_system` and font rendering — no separate render pass. Implementation details need `/interview`.
- **Conveyor physics (I33):** Do items on conveyors use the physics system (forces applied by conveyor) or kinematic movement (position set directly)? Kinematic is simpler but doesn't handle collisions between conveyor items. Run `/interview` to decide.
- **I34 (irregular mesh) algorithm:** What algorithm — Voronoi relaxation, Catmull-Clark subdivision, custom? What's the input — biome boundaries, heightmap contours, manual control points? Run `/interview` to scope.
- **I35 (structures) and I36 (enemies) depend on the full P4 world generation pipeline.** Do not start these until P4 items are complete and stable.
- **All items need full `/interview` before implementation.** These are the least-specified items in the backlog. The one-liner descriptions cover intent but not behavior, edge cases, or API surface.
