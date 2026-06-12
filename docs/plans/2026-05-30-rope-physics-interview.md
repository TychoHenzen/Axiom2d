# Rope/Cable Physics Overhaul — Requirements Spec

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

## Goal

Replace the current Verlet particle chain rope physics with a **Geometric Wrapping Wire** system that prevents stable loops without obstacles and enables proper wrapping around objects.

## Current State

The rope simulation lives in `crates/card_game/src/card/jack_cable.rs` with this loop:

```
verlet_step(ROPE_DAMPING=0.95) → relax_constraints × 8 → resolve_aabb_collisions → pin_endpoints
```

**Known issues:**

1. Runs in `Phase::Update`, not `FixedUpdate` — frame-rate dependent behavior
2. `ROPE_CONSTRAINT_ITERATIONS = 8` — too few for stiff cable, segments stretch
3. `apply_shrinkage` exists but is never called (dead code)
4. `ROPE_SLACK = 1.0` (no extra length) — zero natural droop
5. AABB collision is single-pass push-out, not interleaved with constraints — tunneling
6. `resize_for_endpoints` only shrinks at `wrap_ratio < 1.4` — excess particles accumulate
7. Rapier joints exist in engine but are unused for rope

## Recommended Approach: Geometric Wrapping Wire

The rope is NOT a particle chain — it's a list of anchor points plus one active segment. When the straight line from current anchor to endpoint crosses an obstacle corner, a new anchor is inserted. When endpoint swings back past anchor, it's removed (unwrap).

**Benefits:**

- No slack by construction — rope is always straight line segments between anchors
- Perfect wrapping — conforms exactly to obstacle vertices
- Pendulum physics on active segment only
- Visual smoothing via existing `catmull_rom_subdivide`

---

## Steps

### Step 1: Move rope simulation to FixedUpdate `[ ]`

Change `rope_physics_system` registration from `Phase::Update` to `Phase::FixedUpdate`. This alone dramatically improves stability by making simulation frame-rate independent.

**Quick win — do this regardless of the full overhaul.**

#### Proofs

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "FixedUpdate" crates/card_game_bin/src/main.rs` shows rope system registered under `FixedUpdate`
- [ ] `rtk grep "Phase::Update" crates/card_game/src/card/jack_cable.rs` returns no matches (removed from Update)
- [ ] `rtk cargo test -p card_game -- cable\|rope\|jack` exits 0

---

### Step 2: Define geometric wire data structures `[ ]`

Replace the Verlet particle chain with new data structures:

- `WrapPoint` struct: position (`Vec2`), obstacle entity reference, corner index
- `GeometricWire` component: list of `WrapPoint`s (anchor chain), active segment endpoint positions, total rest length
- Remove or deprecate the old particle-based rope data

#### Proofs

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "pub struct WrapPoint\|pub struct GeometricWire" crates/card_game/src/` shows both exist
- [ ] `rtk cargo test -p card_game -- geometric_wire\|wrap_point` exits 0 with at least 2 tests passing

---

### Step 3: Implement wrap/unwrap detection `[ ]`

Core algorithm using 2D cross products:

- For each segment from current anchor to endpoint, test against `CableCollider` corners
- If straight line crosses an obstacle corner, insert a new `WrapPoint` (wrap)
- If endpoint swings back past the most recent anchor's angle, remove it (unwrap)
- Handle edge cases: simultaneous contacts, high angular velocity

#### Proofs

- [ ] `rtk cargo test -p card_game -- wrap_detect\|unwrap` exits 0 with at least 4 tests passing
- [ ] Tests cover: basic wrap around single corner, unwrap when swinging back, no wrap when line doesn't cross corner, multiple sequential wraps
- [ ] `rtk grep "cross\|cross_product\|perp_dot" crates/card_game/src/` shows 2D cross product used in detection logic

---

### Step 4: Implement active segment physics `[ ]`

The segment between the last wrap point and the free endpoint gets pendulum physics:

- Constrained to rest length from last anchor
- Gravity applied
- Uses engine's `FixedUpdate` timestep for stability

#### Proofs

- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- active_segment\|pendulum` exits 0 with at least 2 tests passing
- [ ] Test: active segment length stays within tolerance of rest length after simulation steps

---

### Step 5: Visual rendering with Catmull-Rom smoothing `[ ]`

Use existing `catmull_rom_subdivide` to smooth the visual appearance of the wire between anchor points. The wire should render as smooth curves, not hard angles at wrap points.

#### Proofs

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk grep "catmull_rom" crates/card_game/src/` shows `catmull_rom_subdivide` used in wire rendering
- [ ] `rtk cargo test -p card_game -- cable_render\|wire_render` exits 0

---

### Step 6: Remove dead code and clean up `[ ]`

- Remove `apply_shrinkage` if truly dead code (or integrate it if the geometric wire uses it)
- Remove or feature-gate the old Verlet particle chain code
- Update any tests that relied on the old rope behavior

#### Proofs

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk cargo test -p card_game` exits 0 (full test suite)
- [ ] `rtk cargo clippy -p card_game` produces no new warnings
- [ ] `rtk grep "apply_shrinkage" crates/card_game/src/` shows either removed or has a caller (no dead code)

---

### Step 7: Integration validation `[ ]`

Verify the wire system works end-to-end: cables between jack sockets wrap around card obstacles, unwrap when dragged away, and render smoothly.

#### Proofs

- [ ] `rtk cargo build -p card_game_bin` exits 0
- [ ] `rtk cargo test -p card_game` exits 0 (full suite)
- [ ] `rtk cargo clippy -p card_game -p card_game_bin` produces no new warnings

---

## Research Notes

- Current rope code: `crates/card_game/src/card/jack_cable.rs`
- Engine has rapier2d but rope doesn't use it — Rapier joint chains are explicitly **not recommended** (fragile for stiff rope)
- Existing `catmull_rom_subdivide` function is available for visual smoothing
- `CableCollider` is the component marking entities as obstacles for the cable
- Schedule phases: `FixedUpdate` runs N times per frame based on `FixedTimestep` accumulator
- Engine uses `bevy_ecs` standalone, systems are plain functions with `Query` parameters
- All systems must be wired into `crates/card_game_bin/src/main.rs`
- Full research report: `docs/Rope_dynamics.md`

### References

- [Box2D Ninja Rope (wrapping algorithm)](http://antonior-software.blogspot.com/2016/12/box2d-ninja-rope.html)
- [Grappling Hook Dev Log 5: Wrapping and Slacking](https://www.pentadact.com/2013-12-23-the-grappling-hook-game-dev-log-5-wrapping-and-slacking/)

## Open Questions

- Should the old Verlet code be kept behind a feature flag or fully removed?
- How many `CableCollider` entities are expected in a typical game scene? (affects wrap detection performance)
- Should wrap detection use rapier's collision detection or a custom corner-list per collider?
- What happens to cables when a card (obstacle) is picked up and moved while a cable wraps around it?
- Should there be a maximum number of wrap points per cable?
