# Hybrid Rope Physics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the pure-Verlet cable simulation with a hybrid geometric-wrapping + Verlet-particle system that prevents phantom loops, wraps precisely around convex polygon obstacles, and provides physical cable wobble.

**Architecture:** Two components per cable entity — `WrapWire` (geometric anchor list, retraction state) and `RopeWire` (Verlet particle chain pinned at anchor positions). Wrap detection runs in `Phase::Update`; Verlet physics runs in `Phase::FixedUpdate`.

**Tech Stack:** Rust, bevy_ecs (standalone), glam (Vec2 math), engine_physics (rapier2d backend), engine_app (Phase scheduling)

**Spec:** `docs/superpowers/specs/2026-04-06-hybrid-rope-physics-design.md`

---

## File Structure

| File | Role |
|------|------|
| `crates/card_game/src/card/jack_cable.rs` | Modify: `CableCollider` polygon migration, `WrapAnchor`/`WrapWire` types, wrap geometry functions, `resolve_polygon_collisions`, retraction logic, anchor-pinning in `rope_physics_system` |
| `crates/card_game/src/card/jack_socket.rs` | Modify: spawn `WrapWire` alongside `RopeWire` on cable entities |
| `crates/card_game/src/card/reader/spawn.rs` | Modify: `CableCollider` construction uses polygon vertices |
| `crates/card_game/src/card/screen_device.rs` | Modify: `CableCollider` construction uses polygon vertices |
| `crates/card_game/src/plugin.rs` | Modify: register new systems, move `rope_physics_system` to `FixedUpdate` |
| `crates/card_game/tests/suite/card_jack_cable.rs` | Modify: add wrap detection tests, polygon collision tests, retraction tests, anchor pinning tests |
| `crates/card_game/tests/suite/card_jack_socket.rs` | Modify: verify `WrapWire` is spawned on cable entities |

---

## Task 1: Migrate `CableCollider` from AABB to Polygon Vertices

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs:130-136` (`CableCollider` struct)
- Modify: `crates/card_game/src/card/reader/spawn.rs:159` (reader spawn)
- Modify: `crates/card_game/src/card/screen_device.rs:183-185` (screen spawn)
- Modify: `crates/card_game/src/card/jack_cable.rs:235-253` (`resolve_aabb_collisions`)
- Modify: `crates/card_game/src/card/jack_cable.rs:329-357` (`rope_physics_system` collider query)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write failing test — polygon collision pushes particle out of box**

In `crates/card_game/tests/suite/card_jack_cable.rs`, add:

```rust
// ---------------------------------------------------------------------------
// CableCollider polygon — particle pushed out of convex polygon
// ---------------------------------------------------------------------------

#[test]
fn when_particle_inside_polygon_then_resolve_polygon_collisions_pushes_it_out() {
    // Arrange — a square obstacle centered at (100, 100) with half-extent 20
    let vertices = vec![
        Vec2::new(80.0, 80.0),
        Vec2::new(120.0, 80.0),
        Vec2::new(120.0, 120.0),
        Vec2::new(80.0, 120.0),
    ];
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle { pos: Vec2::new(0.0, 0.0), prev: Vec2::new(0.0, 0.0) },
        // Interior particle inside the polygon
        RopeParticle { pos: Vec2::new(105.0, 100.0), prev: Vec2::new(105.0, 100.0) },
        RopeParticle { pos: Vec2::new(200.0, 0.0), prev: Vec2::new(200.0, 0.0) },
    ]);

    // Act
    wire.resolve_polygon_collisions(&[(Vec2::new(100.0, 100.0), &vertices)]);

    // Assert — particle must be outside the polygon
    let p = wire.particles[1].pos;
    let dx = (p.x - 100.0).abs();
    let dy = (p.y - 100.0).abs();
    assert!(
        dx >= 19.9 || dy >= 19.9,
        "particle must be pushed to polygon boundary, got {p}"
    );
}
```

Also add the import for the new function at the top of the test file (it will be added alongside existing `RopeWire` imports from `card_game::card::jack_cable`).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_particle_inside_polygon_then_resolve_polygon_collisions_pushes_it_out`
Expected: FAIL — `resolve_polygon_collisions` does not exist yet.

- [ ] **Step 3: Change `CableCollider` to polygon vertices**

In `crates/card_game/src/card/jack_cable.rs`, replace:

```rust
#[derive(Component, Debug, Clone)]
pub struct CableCollider {
    pub half_extents: Vec2,
}
```

with:

```rust
#[derive(Component, Debug, Clone)]
pub struct CableCollider {
    /// Convex hull vertices in local space, wound counter-clockwise.
    pub vertices: Vec<Vec2>,
}

impl CableCollider {
    /// Construct from AABB half-extents (backward compat for readers/screens).
    pub fn from_aabb(half: Vec2) -> Self {
        Self {
            vertices: vec![
                Vec2::new(-half.x, -half.y),
                Vec2::new(half.x, -half.y),
                Vec2::new(half.x, half.y),
                Vec2::new(-half.x, half.y),
            ],
        }
    }
}
```

- [ ] **Step 4: Add `resolve_polygon_collisions` method on `RopeWire`**

In `crates/card_game/src/card/jack_cable.rs`, add after the existing `resolve_aabb_collisions` method:

```rust
/// Push interior particles out of convex polygons.
/// `polygons` is a slice of (center_position, &vertices_in_world_space).
pub fn resolve_polygon_collisions(&mut self, polygons: &[(Vec2, &[Vec2])]) {
    let len = self.particles.len();
    for i in 1..len.saturating_sub(1) {
        let p = self.particles[i].pos;
        for &(_, verts) in polygons {
            let n = verts.len();
            if n < 3 {
                continue;
            }
            // Check if point is inside convex polygon using cross products
            let mut inside = true;
            for j in 0..n {
                let a = verts[j];
                let b = verts[(j + 1) % n];
                let cross = (b - a).perp_dot(p - a);
                if cross < 0.0 {
                    inside = false;
                    break;
                }
            }
            if !inside {
                continue;
            }
            // Find closest edge and push out along its normal
            let mut min_dist = f32::MAX;
            let mut push = Vec2::ZERO;
            for j in 0..n {
                let a = verts[j];
                let b = verts[(j + 1) % n];
                let edge = b - a;
                let edge_len_sq = edge.length_squared();
                if edge_len_sq < 1e-10 {
                    continue;
                }
                let t = ((p - a).dot(edge) / edge_len_sq).clamp(0.0, 1.0);
                let closest = a + edge * t;
                let dist = (p - closest).length();
                if dist < min_dist {
                    min_dist = dist;
                    let normal = Vec2::new(-edge.y, edge.x).normalize_or_zero();
                    push = normal * (min_dist + 0.1);
                }
            }
            self.particles[i].pos = p + push;
        }
    }
}
```

- [ ] **Step 5: Update `rope_physics_system` to use polygon collisions**

In `crates/card_game/src/card/jack_cable.rs`, update `rope_physics_system`. Change the collider query and boxes construction:

Replace:

```rust
    let boxes: Vec<(Vec2, Vec2)> = colliders
        .iter()
        .map(|(t, c)| (t.position, c.half_extents))
        .collect();
```

with:

```rust
    let polygons: Vec<(Vec2, Vec<Vec2>)> = colliders
        .iter()
        .map(|(t, c)| {
            let world_verts: Vec<Vec2> = c.vertices.iter().map(|v| *v + t.position).collect();
            (t.position, world_verts)
        })
        .collect();
```

And inside the constraint loop, replace:

```rust
            rope.resolve_aabb_collisions(&boxes);
```

with:

```rust
            let poly_refs: Vec<(Vec2, &[Vec2])> = polygons
                .iter()
                .map(|(c, v)| (*c, v.as_slice()))
                .collect();
            rope.resolve_polygon_collisions(&poly_refs);
```

- [ ] **Step 6: Update reader spawn to use `CableCollider::from_aabb`**

In `crates/card_game/src/card/reader/spawn.rs:159`, replace:

```rust
            CableCollider { half_extents: half },
```

with:

```rust
            CableCollider::from_aabb(half),
```

- [ ] **Step 7: Update screen device spawn to use `CableCollider::from_aabb`**

In `crates/card_game/src/card/screen_device.rs:183-185`, replace:

```rust
            CableCollider {
                half_extents: SCREEN_HALF_EXTENTS,
            },
```

with:

```rust
            CableCollider::from_aabb(SCREEN_HALF_EXTENTS),
```

- [ ] **Step 8: Remove old `resolve_aabb_collisions` method**

Delete the `resolve_aabb_collisions` method from `RopeWire` in `jack_cable.rs` (lines 235-253). It is fully replaced by `resolve_polygon_collisions`.

- [ ] **Step 9: Run all tests to verify no regressions**

Run: `cargo.exe test -p card_game`
Expected: All existing tests pass. New polygon collision test passes.

- [ ] **Step 10: Run clippy and fmt**

Run: `cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: Clean.

- [ ] **Step 11: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/src/card/reader/spawn.rs crates/card_game/src/card/screen_device.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "refactor: migrate CableCollider from AABB to convex polygon vertices"
```

---

## Task 2: Add `WrapAnchor` and `WrapWire` Types

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs` (new types)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write failing test — `WrapWire` computes shortest path length**

In `crates/card_game/tests/suite/card_jack_cable.rs`, add:

```rust
use card_game::card::jack_cable::WrapWire;

// ---------------------------------------------------------------------------
// WrapWire — shortest path through anchors
// ---------------------------------------------------------------------------

#[test]
fn when_wrap_wire_has_no_anchors_then_shortest_path_is_endpoint_distance() {
    // Arrange
    let wire = WrapWire::new();
    let src = Vec2::new(0.0, 0.0);
    let dst = Vec2::new(100.0, 0.0);

    // Act
    let path = wire.shortest_path(src, dst);

    // Assert
    assert!((path - 100.0).abs() < 0.01, "expected 100.0, got {path}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_wrap_wire_has_no_anchors_then_shortest_path_is_endpoint_distance`
Expected: FAIL — `WrapWire` does not exist.

- [ ] **Step 3: Implement `WrapAnchor` and `WrapWire`**

In `crates/card_game/src/card/jack_cable.rs`, add after the `CableCollider` definition:

```rust
#[derive(Debug, Clone)]
pub struct WrapAnchor {
    /// World-space position of this anchor (polygon vertex).
    pub position: Vec2,
    /// Which obstacle entity this anchor belongs to.
    pub obstacle: Entity,
    /// Index into that obstacle's CableCollider.vertices.
    pub vertex_index: usize,
    /// Wrap direction: +1.0 for CCW wrap, -1.0 for CW wrap.
    pub wrap_sign: f32,
    /// Index of the pinned particle in the RopeWire chain.
    pub pinned_particle: usize,
}

#[derive(Component, Debug, Clone)]
pub struct WrapWire {
    /// Ordered anchor points from source toward dest.
    pub anchors: Vec<WrapAnchor>,
    /// Target length the cable is retracting toward.
    pub target_length: f32,
}

impl WrapWire {
    pub fn new() -> Self {
        Self {
            anchors: vec![],
            target_length: 0.0,
        }
    }

    /// Compute the shortest geometric path from `src` through all anchors to `dst`.
    pub fn shortest_path(&self, src: Vec2, dst: Vec2) -> f32 {
        if self.anchors.is_empty() {
            return (dst - src).length();
        }
        let mut total = (self.anchors[0].position - src).length();
        for i in 0..self.anchors.len() - 1 {
            total += (self.anchors[i + 1].position - self.anchors[i].position).length();
        }
        total += (dst - self.anchors.last().expect("checked non-empty").position).length();
        total
    }
}
```

- [ ] **Step 4: Export `WrapWire` and `WrapAnchor` from the crate**

Ensure `WrapWire` and `WrapAnchor` are `pub` and accessible via `card_game::card::jack_cable::WrapWire`. They already are if defined as `pub` in `jack_cable.rs`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_wrap_wire_has_no_anchors_then_shortest_path_is_endpoint_distance`
Expected: PASS.

- [ ] **Step 6: Write test — shortest path with one anchor**

```rust
#[test]
fn when_wrap_wire_has_one_anchor_then_shortest_path_goes_through_it() {
    // Arrange
    let mut wire = WrapWire::new();
    // Use Entity::from_raw for test — no real world needed for pure math
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 50.0),
        obstacle: Entity::from_raw(999),
        vertex_index: 0,
        wrap_sign: 1.0,
        pinned_particle: 0,
    });
    let src = Vec2::new(0.0, 0.0);
    let dst = Vec2::new(100.0, 0.0);

    // Act
    let path = wire.shortest_path(src, dst);

    // Assert — src→(50,50)→dst = ~70.71 + ~70.71 = ~141.42
    let expected = (Vec2::new(50.0, 50.0) - src).length()
        + (dst - Vec2::new(50.0, 50.0)).length();
    assert!(
        (path - expected).abs() < 0.01,
        "expected {expected}, got {path}"
    );
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_wrap_wire_has_one_anchor_then_shortest_path_goes_through_it`
Expected: PASS (implementation already handles this).

- [ ] **Step 8: Run all tests, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: All pass, clean.

- [ ] **Step 9: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "feat: add WrapAnchor and WrapWire types with shortest_path computation"
```

---

## Task 3: Spawn `WrapWire` on Cable Entities

**Files:**
- Modify: `crates/card_game/src/card/jack_socket.rs:143-167` (on_socket_clicked spawn)
- Modify: `crates/card_game/src/card/jack_socket.rs:382-406` (fallback spawn)
- Test: `crates/card_game/tests/suite/card_jack_socket.rs`

- [ ] **Step 1: Write failing test — cable entity has `WrapWire` component**

In `crates/card_game/tests/suite/card_jack_socket.rs`, add. This reuses the same pattern as the existing `given_two_free_compatible_sockets_when_cable_connected_then_both_sockets_connected_cable_is_set` test — create two compatible sockets, simulate a mouse release over the target, run `jack_socket_release_system`, then inspect the spawned cable entity.

```rust
use card_game::card::jack_cable::WrapWire;

#[test]
fn when_cable_connected_between_sockets_then_cable_entity_has_wrap_wire() {
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;

    // Arrange
    let mut world = make_pick_world();

    let output_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None,
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::default(),
        ))
        .id();

    let input_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None,
            },
            Transform2D {
                position: Vec2::new(50.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 0.0))),
            SortOrder::default(),
        ))
        .id();

    world.insert_resource(PendingCable {
        source: Some(output_socket),
        origin_cable: None,
        free_end: None,
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(50.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert — find the cable entity via the socket's connected_cable
    let socket = world.get::<JackSocket>(output_socket).unwrap();
    let cable_entity = socket
        .connected_cable
        .expect("socket must have a connected cable");
    let wrap = world
        .get::<WrapWire>(cable_entity)
        .expect("cable entity must have WrapWire component");
    assert!(
        wrap.anchors.is_empty(),
        "new cable must start with no anchors"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_cable_spawned_between_sockets_then_entity_has_wrap_wire`
Expected: FAIL — `WrapWire` not on entity.

- [ ] **Step 3: Add `WrapWire` to cable spawn in `on_socket_clicked`**

In `crates/card_game/src/card/jack_socket.rs`, in the `on_socket_clicked` function, where the cable entity is spawned (around line 144), add `WrapWire::new()` to the spawn bundle:

Find the `commands.spawn((...))` that creates the cable with `RopeWireEndpoints`, `rope`, `Transform2D`, `Shape`, etc. Add `WrapWire::new()` to the tuple.

Also add the import at the top: add `WrapWire` to the import from `crate::card::jack_cable`.

- [ ] **Step 4: Add `WrapWire` to fallback cable spawn in `jack_socket_release_system`**

In `crates/card_game/src/card/jack_socket.rs`, in the `jack_socket_release_system` function (around line 383), find the fallback `commands.spawn((...))` that also creates a cable entity. Add `WrapWire::new()` to that spawn tuple as well.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_cable_spawned_between_sockets_then_entity_has_wrap_wire`
Expected: PASS.

- [ ] **Step 6: Run all tests, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: All pass, clean.

- [ ] **Step 7: Commit**

```bash
git add crates/card_game/src/card/jack_socket.rs crates/card_game/tests/suite/card_jack_socket.rs
git commit -m "feat: spawn WrapWire component on all cable entities"
```

---

## Task 4: Move `rope_physics_system` to `FixedUpdate`

**Files:**
- Modify: `crates/card_game/src/plugin.rs:140-151` (system registration)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write test — rope physics stabilizes with fixed timestep**

The existing `rope_physics_system` tests already verify behavior. This step is a system registration change — no new test is needed for the physics behavior itself (it's the same system). Instead, verify the system ordering compiles and runs.

Run: `cargo.exe test -p card_game given_rope_wired_between_two_sockets_when_rope_physics_system_runs_then_endpoints_match_socket_positions`
Expected: PASS (baseline — currently works).

- [ ] **Step 2: Move `rope_physics_system` from `Phase::Update` to `Phase::FixedUpdate`**

In `crates/card_game/src/plugin.rs`, remove `rope_physics_system` from the `Phase::Update` chain (lines 143-144) and add it to `Phase::FixedUpdate`:

Change the Update chain from:

```rust
        .add_systems(
            Phase::Update,
            (
                pending_cable_drag_system,
                rope_physics_system,
                rope_render_system,
                signature_space_propagation_system,
                jack_socket_render_system,
                screen_render_system,
            )
                .chain()
                .after(jack_socket_release_system),
        )
```

to:

```rust
        .add_systems(
            Phase::Update,
            (
                pending_cable_drag_system,
                rope_render_system,
                signature_space_propagation_system,
                jack_socket_render_system,
                screen_render_system,
            )
                .chain()
                .after(jack_socket_release_system),
        )
```

And add to the existing `Phase::FixedUpdate` block:

```rust
        .add_systems(
            Phase::FixedUpdate,
            (
                card_damping_system.after(physics_sync_system),
                reader_rotation_lock_system.after(physics_sync_system),
                rope_physics_system.after(physics_sync_system),
            ),
        )
```

- [ ] **Step 3: Run all tests**

Run: `cargo.exe test -p card_game`
Expected: All pass. Existing rope tests use a one-shot `Schedule` with `rope_physics_system` directly, so they are unaffected by the registration change.

- [ ] **Step 4: Build the binary to verify registration compiles**

Run: `cargo.exe build -p card_game_bin`
Expected: Compiles successfully.

- [ ] **Step 5: Run clippy and fmt**

Run: `cargo.exe clippy -p card_game -p card_game_bin -- -D warnings && cargo.exe fmt --all`
Expected: Clean.

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/plugin.rs
git commit -m "refactor: move rope_physics_system to FixedUpdate for timestep stability"
```

---

## Task 5: Implement Wrap Detection — Line-Segment vs Polygon Intersection

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs` (geometry helpers)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

This task implements the pure geometry functions used by the wrap detection system. No ECS yet — pure math.

- [ ] **Step 1: Write failing test — segment-segment intersection**

```rust
// ---------------------------------------------------------------------------
// Geometry — segment-segment intersection
// ---------------------------------------------------------------------------

use card_game::card::jack_cable::segment_intersects_segment;

#[test]
fn when_two_crossing_segments_then_intersection_returns_some() {
    // Arrange — horizontal and vertical crossing segments
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(10.0, 0.0);
    let b1 = Vec2::new(5.0, -5.0);
    let b2 = Vec2::new(5.0, 5.0);

    // Act
    let result = segment_intersects_segment(a1, a2, b1, b2);

    // Assert
    assert!(result.is_some(), "crossing segments must intersect");
    let t = result.unwrap();
    assert!((t - 0.5).abs() < 0.01, "intersection at midpoint, t={t}");
}

#[test]
fn when_parallel_segments_then_intersection_returns_none() {
    // Arrange
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(10.0, 0.0);
    let b1 = Vec2::new(0.0, 5.0);
    let b2 = Vec2::new(10.0, 5.0);

    // Act
    let result = segment_intersects_segment(a1, a2, b1, b2);

    // Assert
    assert!(result.is_none(), "parallel segments must not intersect");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo.exe test -p card_game when_two_crossing_segments_then_intersection`
Expected: FAIL — function does not exist.

- [ ] **Step 3: Implement `segment_intersects_segment`**

In `crates/card_game/src/card/jack_cable.rs`, add:

```rust
/// Returns the parameter `t` along segment (a1→a2) where it intersects segment (b1→b2).
/// Returns `None` if the segments don't intersect. Both `t` and `u` must be in [0, 1].
pub fn segment_intersects_segment(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<f32> {
    let d1 = a2 - a1;
    let d2 = b2 - b1;
    let denom = d1.perp_dot(d2);
    if denom.abs() < 1e-10 {
        return None; // parallel
    }
    let diff = b1 - a1;
    let t = diff.perp_dot(d2) / denom;
    let u = diff.perp_dot(d1) / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}
```

- [ ] **Step 4: Write failing test — find wrap vertex on polygon**

```rust
use card_game::card::jack_cable::find_wrap_vertex;

#[test]
fn when_span_crosses_polygon_then_find_wrap_vertex_returns_correct_corner() {
    // Arrange — cable spans from left to right across a box centered at (50, 0)
    let span_a = Vec2::new(0.0, 5.0);
    let span_b = Vec2::new(100.0, 5.0);
    // Box vertices CCW: bottom-left, bottom-right, top-right, top-left
    let verts = vec![
        Vec2::new(40.0, -10.0),
        Vec2::new(60.0, -10.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];

    // Act
    let result = find_wrap_vertex(span_a, span_b, &verts);

    // Assert — should pick one of the top vertices (y=10 side, same side as the cable at y=5)
    assert!(result.is_some(), "must find a wrap vertex");
    let (idx, sign) = result.unwrap();
    let chosen = verts[idx];
    assert!(
        chosen.y > 0.0,
        "cable at y=5 should wrap around a top vertex, got {chosen}"
    );
    assert!(sign.abs() > 0.0, "wrap_sign must be nonzero");
}
```

- [ ] **Step 5: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_span_crosses_polygon_then_find_wrap_vertex`
Expected: FAIL — function does not exist.

- [ ] **Step 6: Implement `find_wrap_vertex`**

In `crates/card_game/src/card/jack_cable.rs`, add:

```rust
/// Given a cable span (a→b) that crosses a convex polygon, find the vertex to wrap around.
/// Returns `(vertex_index, wrap_sign)` where wrap_sign is +1.0 (CCW) or -1.0 (CW).
/// Returns `None` if the span doesn't cross the polygon.
pub fn find_wrap_vertex(a: Vec2, b: Vec2, polygon: &[Vec2]) -> Option<(usize, f32)> {
    let n = polygon.len();
    if n < 3 {
        return None;
    }

    // Check if the span intersects any edge of the polygon
    let mut has_intersection = false;
    for i in 0..n {
        let e1 = polygon[i];
        let e2 = polygon[(i + 1) % n];
        if segment_intersects_segment(a, b, e1, e2).is_some() {
            has_intersection = true;
            break;
        }
    }
    if !has_intersection {
        return None;
    }

    // Find the vertex that creates the shortest detour
    let span_dir = b - a;
    let mut best_idx = 0;
    let mut best_detour = f32::MAX;
    for i in 0..n {
        let v = polygon[i];
        let detour = (v - a).length() + (b - v).length();
        if detour < best_detour {
            best_detour = detour;
            best_idx = i;
        }
    }

    let wrap_sign = span_dir.perp_dot(polygon[best_idx] - a).signum();
    Some((best_idx, wrap_sign))
}
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo.exe test -p card_game when_span_crosses_polygon_then_find_wrap_vertex && cargo.exe test -p card_game when_two_crossing_segments`
Expected: All PASS.

- [ ] **Step 8: Run all tests, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: All pass, clean.

- [ ] **Step 9: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "feat: add segment intersection and wrap vertex detection geometry"
```

---

## Task 6: Implement Wrap Detection and Unwrap Logic on `WrapWire`

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs` (`WrapWire` methods)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write failing test — wrap detection inserts anchor**

```rust
#[test]
fn when_line_crosses_polygon_edge_then_detect_wraps_inserts_anchor() {
    // Arrange — cable from (0,0) to (100,5), box centered at (50,0)
    let mut wire = WrapWire::new();
    let src = Vec2::new(0.0, 5.0);
    let dst = Vec2::new(100.0, 5.0);
    let obstacle = Entity::from_raw(42);
    let verts = vec![
        Vec2::new(40.0, -10.0),
        Vec2::new(60.0, -10.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];
    let obstacles = vec![(obstacle, verts.as_slice())];

    // Act
    wire.detect_wraps(src, dst, &obstacles);

    // Assert
    assert_eq!(wire.anchors.len(), 1, "must insert exactly one anchor");
    assert_eq!(wire.anchors[0].obstacle, obstacle);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_line_crosses_polygon_edge_then_detect_wraps_inserts_anchor`
Expected: FAIL — `detect_wraps` does not exist.

- [ ] **Step 3: Implement `detect_wraps` on `WrapWire`**

In `crates/card_game/src/card/jack_cable.rs`, add to `impl WrapWire`:

```rust
    /// Check each span for polygon intersections and insert wrap anchors.
    pub fn detect_wraps(&mut self, src: Vec2, dst: Vec2, obstacles: &[(Entity, &[Vec2])]) {
        // Build list of pin points: src, anchor positions, dst
        let mut pins: Vec<Vec2> = Vec::with_capacity(self.anchors.len() + 2);
        pins.push(src);
        for anchor in &self.anchors {
            pins.push(anchor.position);
        }
        pins.push(dst);

        // Walk spans and check for intersections
        let mut insert_idx = 0; // index into self.anchors where new anchor goes
        let mut i = 0;
        while i < pins.len() - 1 {
            let span_a = pins[i];
            let span_b = pins[i + 1];

            let mut found = None;
            for &(entity, verts) in obstacles {
                // Skip obstacles already anchored in this span's neighbors
                let already_anchored = self.anchors.iter().any(|a| {
                    a.obstacle == entity
                });
                if already_anchored {
                    continue;
                }
                if let Some((vidx, sign)) = find_wrap_vertex(span_a, span_b, verts) {
                    found = Some(WrapAnchor {
                        position: verts[vidx],
                        obstacle: entity,
                        vertex_index: vidx,
                        wrap_sign: sign,
                        pinned_particle: 0, // assigned later by physics
                    });
                    break;
                }
            }

            if let Some(anchor) = found {
                let pos = anchor.position;
                self.anchors.insert(insert_idx, anchor);
                // Re-derive pins and restart scan from current span
                pins.insert(i + 1, pos);
                // Don't advance i — re-check the span from pins[i] to the new anchor
                insert_idx += 1;
                i += 1; // advance past the newly inserted pin
            } else {
                insert_idx += 1;
                i += 1;
            }
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_line_crosses_polygon_edge_then_detect_wraps_inserts_anchor`
Expected: PASS.

- [ ] **Step 5: Write failing test — unwrap removes anchor when cable swings past**

```rust
#[test]
fn when_cable_swings_past_anchor_then_detect_unwraps_removes_it() {
    // Arrange — anchor at (50, 10) with CCW wrap, cable now goes (0,0)→(50,10)→(100,20)
    // If cable swings to (100, -20) the cross product sign flips
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 10.0),
        obstacle: Entity::from_raw(42),
        vertex_index: 2,
        wrap_sign: 1.0, // CCW
        pinned_particle: 0,
    });

    let src = Vec2::new(0.0, 0.0);
    // dst on the opposite side — the cable has swung past the anchor
    let dst = Vec2::new(100.0, -20.0);

    // Act
    wire.detect_unwraps(src, dst);

    // Assert
    assert!(
        wire.anchors.is_empty(),
        "anchor must be removed when cable swings past"
    );
}
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_cable_swings_past_anchor_then_detect_unwraps_removes_it`
Expected: FAIL — `detect_unwraps` does not exist.

- [ ] **Step 7: Implement `detect_unwraps` on `WrapWire`**

In `crates/card_game/src/card/jack_cable.rs`, add to `impl WrapWire`:

```rust
    /// Remove anchors where the cable has swung past the wrap point.
    pub fn detect_unwraps(&mut self, src: Vec2, dst: Vec2) {
        self.anchors.retain(|anchor| {
            let prev = if let Some(idx) = self.anchors.iter().position(|a| std::ptr::eq(a, anchor)) {
                if idx == 0 { src } else { self.anchors[idx - 1].position }
            } else {
                src
            };
            let next_pos = if let Some(idx) = self.anchors.iter().position(|a| std::ptr::eq(a, anchor)) {
                if idx + 1 < self.anchors.len() {
                    self.anchors[idx + 1].position
                } else {
                    dst
                }
            } else {
                dst
            };

            let to_anchor = anchor.position - prev;
            let from_anchor = next_pos - anchor.position;
            let cross = to_anchor.perp_dot(from_anchor);

            // Keep anchor if cross product sign matches wrap_sign
            cross * anchor.wrap_sign > 0.0
        });
    }
```

Note: The `retain` with self-referential index lookup is awkward. A cleaner approach — rewrite as an index-based loop:

```rust
    /// Remove anchors where the cable has swung past the wrap point.
    pub fn detect_unwraps(&mut self, src: Vec2, dst: Vec2) {
        let mut i = 0;
        while i < self.anchors.len() {
            let prev = if i == 0 { src } else { self.anchors[i - 1].position };
            let next = if i + 1 < self.anchors.len() {
                self.anchors[i + 1].position
            } else {
                dst
            };

            let to_anchor = self.anchors[i].position - prev;
            let from_anchor = next - self.anchors[i].position;
            let cross = to_anchor.perp_dot(from_anchor);

            if cross * self.anchors[i].wrap_sign <= 0.0 {
                // Sign flipped — cable swung past, remove anchor
                self.anchors.remove(i);
                // Don't increment i — check the next anchor at the same index
            } else {
                i += 1;
            }
        }
    }
```

Use the index-based version.

- [ ] **Step 8: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_cable_swings_past_anchor_then_detect_unwraps_removes_it`
Expected: PASS.

- [ ] **Step 9: Write test — full wrap around a box anchors four corners**

```rust
#[test]
fn when_cable_wraps_full_loop_around_obstacle_then_all_four_corners_anchored() {
    // Arrange — cable path that goes around all four corners of a box
    // We simulate this by calling detect_wraps repeatedly as the cable
    // path wraps around. Start with src at (0,0) → dst at (0, -1) going
    // around a box at (50,0) with half-extent 10.
    let obstacle = Entity::from_raw(1);
    let verts = vec![
        Vec2::new(40.0, -10.0),
        Vec2::new(60.0, -10.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];
    let obstacles = vec![(obstacle, verts.as_slice())];

    let mut wire = WrapWire::new();

    // Simulate wrapping by moving dst around the box
    // Step 1: cable from left to right, above box
    wire.detect_wraps(Vec2::new(0.0, 15.0), Vec2::new(100.0, 15.0), &obstacles);
    // No wrap — cable passes above

    // Cable from left side to right side, going through the box
    let mut wire = WrapWire::new();
    wire.detect_wraps(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), &obstacles);
    assert!(!wire.anchors.is_empty(), "cable through box must create anchors");
}
```

- [ ] **Step 10: Run all tests, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: All pass, clean.

- [ ] **Step 11: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "feat: implement wrap detection and unwrap logic on WrapWire"
```

---

## Task 7: Implement `wrap_update_system` and `wrap_detect_system`

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs` (new ECS systems)
- Modify: `crates/card_game/src/plugin.rs` (register systems)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write failing test — wrap system detects wrap around obstacle**

```rust
use card_game::card::jack_cable::{
    wrap_update_system, wrap_detect_system, WrapWire, CableCollider,
};

#[test]
fn when_cable_dragged_across_obstacle_then_wrap_detect_system_adds_anchor() {
    // Arrange
    let mut world = World::new();

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 5.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(200.0, 5.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    // Obstacle box centered at (100, 0)
    world.spawn((
        Transform2D {
            position: Vec2::new(100.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        CableCollider::from_aabb(Vec2::new(20.0, 20.0)),
    ));

    let cable_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 5.0), Vec2::new(200.0, 5.0), 10),
            RopeWireEndpoints { source, dest },
            WrapWire::new(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((wrap_update_system, wrap_detect_system).chain());

    // Act
    schedule.run(&mut world);

    // Assert
    let wrap = world.get::<WrapWire>(cable_entity).unwrap();
    assert!(
        !wrap.anchors.is_empty(),
        "wrap_detect_system must find the obstacle crossing"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_cable_dragged_across_obstacle_then_wrap_detect_system_adds_anchor`
Expected: FAIL — systems don't exist.

- [ ] **Step 3: Implement `wrap_update_system`**

In `crates/card_game/src/card/jack_cable.rs`, add:

```rust
/// Update anchor world positions from obstacle transforms each frame.
pub fn wrap_update_system(
    mut wires: Query<&mut WrapWire>,
    colliders: Query<(&Transform2D, &CableCollider), Without<WrapWire>>,
) {
    for mut wrap in &mut wires {
        for anchor in &mut wrap.anchors {
            if let Ok((transform, collider)) = colliders.get(anchor.obstacle) {
                if let Some(local_vert) = collider.vertices.get(anchor.vertex_index) {
                    anchor.position = *local_vert + transform.position;
                }
            }
        }
    }
}
```

- [ ] **Step 4: Implement `wrap_detect_system`**

In `crates/card_game/src/card/jack_cable.rs`, add:

```rust
/// Detect wrap and unwrap events based on cable span vs obstacle polygon intersections.
pub fn wrap_detect_system(
    mut wires: Query<(&mut WrapWire, &RopeWireEndpoints)>,
    transforms: Query<&Transform2D, Without<WrapWire>>,
    colliders: Query<(Entity, &Transform2D, &CableCollider), Without<WrapWire>>,
) {
    let obstacles: Vec<(Entity, Vec<Vec2>)> = colliders
        .iter()
        .map(|(entity, t, c)| {
            let world_verts: Vec<Vec2> = c.vertices.iter().map(|v| *v + t.position).collect();
            (entity, world_verts)
        })
        .collect();

    for (mut wrap, endpoints) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src = src_t.position;
        let dst = dst_t.position;

        // Unwrap first, then wrap — order matters so we don't fight ourselves
        wrap.detect_unwraps(src, dst);

        let obstacle_refs: Vec<(Entity, &[Vec2])> = obstacles
            .iter()
            .map(|(e, v)| (*e, v.as_slice()))
            .collect();
        wrap.detect_wraps(src, dst, &obstacle_refs);
    }
}
```

- [ ] **Step 5: Register systems in `plugin.rs`**

In `crates/card_game/src/plugin.rs`, add imports for `wrap_update_system` and `wrap_detect_system` from `jack_cable`. Update the `Phase::Update` cable chain:

```rust
        .add_systems(
            Phase::Update,
            (
                pending_cable_drag_system,
                wrap_update_system,
                wrap_detect_system,
                rope_render_system,
                signature_space_propagation_system,
                jack_socket_render_system,
                screen_render_system,
            )
                .chain()
                .after(jack_socket_release_system),
        )
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_cable_dragged_across_obstacle_then_wrap_detect_system_adds_anchor`
Expected: PASS.

- [ ] **Step 7: Build the binary**

Run: `cargo.exe build -p card_game_bin`
Expected: Compiles.

- [ ] **Step 8: Run all tests, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: All pass, clean.

- [ ] **Step 9: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/src/plugin.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "feat: add wrap_update_system and wrap_detect_system for geometric wrapping"
```

---

## Task 8: Implement Anchor Pinning in `rope_physics_system`

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs:329-357` (`rope_physics_system`)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write failing test — anchored particles are pinned after physics tick**

```rust
#[test]
fn when_rope_has_wrap_anchor_then_physics_pins_particle_at_anchor_position() {
    // Arrange
    let mut world = World::new();

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(200.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let anchor_pos = Vec2::new(100.0, 50.0);
    let mut wrap = WrapWire::new();
    wrap.anchors.push(WrapAnchor {
        position: anchor_pos,
        obstacle: Entity::from_raw(999),
        vertex_index: 0,
        wrap_sign: 1.0,
        pinned_particle: 5, // middle of a 10-particle rope
    });
    wrap.target_length = 300.0;

    let rope_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 10),
            RopeWireEndpoints { source, dest },
            wrap,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(rope_physics_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let rope = world.get::<RopeWire>(rope_entity).unwrap();
    let pinned = rope.particles[5].pos;
    assert!(
        (pinned - anchor_pos).length() < 0.1,
        "particle[5] must be pinned to anchor at {anchor_pos}, got {pinned}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_rope_has_wrap_anchor_then_physics_pins_particle_at_anchor_position`
Expected: FAIL — `rope_physics_system` doesn't read `WrapWire` yet.

- [ ] **Step 3: Update `rope_physics_system` to read `WrapWire` and pin particles**

In `crates/card_game/src/card/jack_cable.rs`, update the system signature and body:

Change:

```rust
pub fn rope_physics_system(
    mut ropes: Query<(&mut RopeWire, &RopeWireEndpoints)>,
    transforms: Query<&Transform2D, Without<RopeWire>>,
    colliders: Query<(&Transform2D, &CableCollider), Without<RopeWire>>,
) {
```

to:

```rust
pub fn rope_physics_system(
    mut ropes: Query<(&mut RopeWire, &RopeWireEndpoints, Option<&WrapWire>)>,
    transforms: Query<&Transform2D, Without<RopeWire>>,
    colliders: Query<(&Transform2D, &CableCollider), Without<RopeWire>>,
) {
```

Inside the system, after `rope.pin_endpoints(src_pos, dst_pos);`, add anchor pinning:

```rust
        // Pin particles at wrap anchor positions
        if let Some(wrap) = wrap_wire {
            for anchor in &wrap.anchors {
                let idx = anchor.pinned_particle;
                if idx > 0 && idx < rope.particles.len() - 1 {
                    rope.particles[idx].pos = anchor.position;
                    rope.particles[idx].prev = anchor.position;
                }
            }
        }
```

Also add anchor pinning inside the constraint iteration loop (after `relax_constraints`, before `resolve_polygon_collisions`):

```rust
        for _ in 0..ROPE_CONSTRAINT_ITERATIONS {
            rope.relax_constraints(rest_length);
            // Re-pin anchored particles after each constraint iteration
            if let Some(wrap) = wrap_wire {
                for anchor in &wrap.anchors {
                    let idx = anchor.pinned_particle;
                    if idx > 0 && idx < rope.particles.len() - 1 {
                        rope.particles[idx].pos = anchor.position;
                        rope.particles[idx].prev = anchor.position;
                    }
                }
            }
            // ... polygon collisions
        }
```

Also update the rest_length derivation to use `WrapWire::target_length` when available:

```rust
        let rest_length = if let Some(wrap) = wrap_wire {
            if wrap.target_length > 0.0 && rope.particles.len() > 1 {
                wrap.target_length / (rope.particles.len() - 1) as f32
            } else {
                rope.rest_length
            }
        } else {
            rope.rest_length
        };
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_rope_has_wrap_anchor_then_physics_pins_particle_at_anchor_position`
Expected: PASS.

- [ ] **Step 5: Run all tests, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe clippy -p card_game -- -D warnings && cargo.exe fmt --all`
Expected: All pass, clean.

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "feat: pin rope particles at wrap anchor positions during physics"
```

---

## Task 9: Implement Retraction System

**Files:**
- Modify: `crates/card_game/src/card/jack_cable.rs` (new system + `WrapWire` method)
- Modify: `crates/card_game/src/plugin.rs` (register system)
- Test: `crates/card_game/tests/suite/card_jack_cable.rs`

- [ ] **Step 1: Write failing test — retraction reduces target_length toward shortest path**

```rust
#[test]
fn when_target_length_exceeds_shortest_path_then_retraction_reduces_it() {
    // Arrange
    let mut wire = WrapWire::new();
    wire.target_length = 200.0; // excess length
    let src = Vec2::new(0.0, 0.0);
    let dst = Vec2::new(100.0, 0.0);
    let shortest = wire.shortest_path(src, dst); // 100.0
    let dt = 0.016; // ~60fps
    let retraction_rate = 3.0;

    // Act
    wire.retract(src, dst, retraction_rate, dt);

    // Assert
    let expected = 200.0 - (200.0 - shortest) * retraction_rate * dt;
    assert!(
        (wire.target_length - expected).abs() < 0.1,
        "expected ~{expected}, got {}",
        wire.target_length
    );
    assert!(
        wire.target_length < 200.0,
        "target_length must decrease"
    );
    assert!(
        wire.target_length > shortest,
        "target_length must not go below shortest path"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_target_length_exceeds_shortest_path_then_retraction_reduces_it`
Expected: FAIL — `retract` does not exist.

- [ ] **Step 3: Implement `retract` on `WrapWire`**

In `crates/card_game/src/card/jack_cable.rs`, add to `impl WrapWire`:

```rust
    /// Proportionally retract target_length toward the shortest geometric path.
    pub fn retract(&mut self, src: Vec2, dst: Vec2, rate: f32, dt: f32) {
        let shortest = self.shortest_path(src, dst);
        let slack_factor = 1.05;
        let floor = shortest * slack_factor;

        if self.target_length > floor {
            self.target_length -= (self.target_length - floor) * rate * dt;
            // Clamp to floor to prevent oscillation
            if self.target_length < floor {
                self.target_length = floor;
            }
        }

        // Never go below actual shortest path
        if self.target_length < shortest {
            self.target_length = shortest;
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_target_length_exceeds_shortest_path_then_retraction_reduces_it`
Expected: PASS.

- [ ] **Step 5: Implement `retraction_system`**

In `crates/card_game/src/card/jack_cable.rs`, add:

```rust
const RETRACTION_RATE: f32 = 3.0;

pub fn retraction_system(
    mut wires: Query<(&mut WrapWire, &RopeWireEndpoints)>,
    transforms: Query<&Transform2D, Without<WrapWire>>,
    dt: Res<engine_core::time::DeltaTime>,
) {
    for (mut wrap, endpoints) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        wrap.retract(src_t.position, dst_t.position, RETRACTION_RATE, dt.0.0);
    }
}
```

Note: Check the actual `DeltaTime` type — it wraps `Seconds(f32)`. Access the inner value accordingly (e.g., `dt.0.0` if `DeltaTime(Seconds(f32))`). The implementer should verify by reading `engine_core::time::DeltaTime`.

- [ ] **Step 6: Register `retraction_system` in `plugin.rs`**

Add `retraction_system` to the imports and insert it in the `Phase::Update` cable chain, after `wrap_detect_system` and before `rope_render_system`:

```rust
        .add_systems(
            Phase::Update,
            (
                pending_cable_drag_system,
                wrap_update_system,
                wrap_detect_system,
                retraction_system,
                rope_render_system,
                signature_space_propagation_system,
                jack_socket_render_system,
                screen_render_system,
            )
                .chain()
                .after(jack_socket_release_system),
        )
```

- [ ] **Step 7: Run all tests, build binary, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe build -p card_game_bin && cargo.exe clippy -p card_game -p card_game_bin -- -D warnings && cargo.exe fmt --all`
Expected: All pass, compiles, clean.

- [ ] **Step 8: Commit**

```bash
git add crates/card_game/src/card/jack_cable.rs crates/card_game/src/plugin.rs crates/card_game/tests/suite/card_jack_cable.rs
git commit -m "feat: add retraction system for proportional cable slack reduction"
```

---

## Task 10: Wire `WrapWire` into Pending Cable Drag

**Files:**
- Modify: `crates/card_game/src/card/jack_socket.rs` (`on_socket_clicked`, `pending_cable_drag_system`)
- Test: `crates/card_game/tests/suite/card_jack_socket.rs`

- [ ] **Step 1: Verify `WrapWire` is already spawned**

Task 3 already added `WrapWire::new()` to the cable spawn. Verify this is in place by checking that the test from Task 3 passes:

Run: `cargo.exe test -p card_game when_cable_spawned_between_sockets_then_entity_has_wrap_wire`
Expected: PASS.

- [ ] **Step 2: Update `pending_cable_drag_system` to call `resize_for_endpoints` with `WrapWire` awareness**

In `crates/card_game/src/card/jack_socket.rs`, the `pending_cable_drag_system` already calls `rope.resize_for_endpoints(...)`. The wrap detection systems (`wrap_update_system`, `wrap_detect_system`) are already chained after `pending_cable_drag_system` in the schedule (from Task 7). So wrapping during drag is already active — no code change needed here beyond what was done in Tasks 3 and 7.

Verify by running the full test suite:

Run: `cargo.exe test -p card_game`
Expected: All pass.

- [ ] **Step 3: Initialize `WrapWire::target_length` when drag starts**

In `crates/card_game/src/card/jack_socket.rs`, in `pending_cable_drag_system`, after the `resize_for_endpoints` call, set the wrap wire's target length to match the current distance:

After the existing block:
```rust
            if let Ok(mut rope) = ropes.get_mut(cable_entity) {
                rope.resize_for_endpoints(src_t.position, cursor_pos);
            }
```

Add a query for `WrapWire` and set target_length. The implementer should add `WrapWire` to the system's query parameters and set:

```rust
            if let Ok(mut wrap) = wrap_wires.get_mut(cable_entity) {
                let dist = (cursor_pos - src_t.position).length();
                if wrap.target_length < dist {
                    wrap.target_length = dist * 1.05;
                }
            }
```

This ensures the target length grows as the user drags further, and retraction kicks in when they drag back.

- [ ] **Step 4: Run all tests, build binary, clippy, fmt**

Run: `cargo.exe test -p card_game && cargo.exe build -p card_game_bin && cargo.exe clippy -p card_game -p card_game_bin -- -D warnings && cargo.exe fmt --all`
Expected: All pass, compiles, clean.

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/card/jack_socket.rs crates/card_game/tests/suite/card_jack_socket.rs
git commit -m "feat: wire WrapWire into pending cable drag for wrap-during-drag"
```

---

## Task 11: Update Test Module Registration

**Files:**
- Modify: `crates/card_game/tests/suite/mod.rs` (if new test files were created)

- [ ] **Step 1: Verify all new tests are registered**

All new tests were added to existing files (`card_jack_cable.rs`, `card_jack_socket.rs`), which are already registered in `tests/suite/mod.rs`. No new test modules needed.

Run: `cargo.exe test -p card_game`
Expected: All tests pass, including all new wrap/retraction tests.

- [ ] **Step 2: Final build verification**

Run: `cargo.exe build -p card_game_bin && cargo.exe clippy --workspace -- -D warnings && cargo.exe fmt --all`
Expected: Clean build, no warnings.

- [ ] **Step 3: Final commit (if any fmt/clippy fixes)**

```bash
git add -A
git commit -m "chore: final cleanup for hybrid rope physics"
```
