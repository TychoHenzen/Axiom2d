# Hybrid Rope Physics Design

## Overview

Replace the current pure-Verlet cable simulation with a hybrid system: a **geometric wrapping wire** (source of truth for the cable's path) combined with **Verlet particle chains** (visual wobble and cable feel). The geometric layer ensures stability — no phantom loops, exact wrapping at polygon vertices — while the particle layer provides the floppy physical response that makes it feel like a real cable.

## Requirements

- **No excessive slack or stable loops** without obstacles blocking the path
- **Full collision wrapping** around obstacle polygons, including complete loops
- **Top-down table view** — no gravity, only friction/damping
- **Proportional retraction** — cable reels in toward its target length; fast when slack is large, slow as it approaches taut
- **Arbitrary convex polygon** obstacles from day one (not just AABBs)
- **Wrapping active during drag** — not just after connection
- **Cable-obstacle collision only** — no cable-cable collision

## Architecture: Two-Layer Component (Approach A)

Two components on each cable entity:
- **`WrapWire`** — geometric layer: ordered list of anchor points at polygon vertices, path length tracking, retraction state
- **`RopeWire`** (simplified) — single Verlet particle chain covering the full cable path, with particles pinned at anchor positions

The geometric layer is the source of truth. The particle chain is visual decoration that respects the anchor constraints.

## Data Model

### `CableCollider` (migrated from AABB to polygon)

```rust
#[derive(Component, Debug, Clone)]
pub struct CableCollider {
    /// Convex hull vertices in local space, wound counter-clockwise.
    pub vertices: Vec<Vec2>,
}
```

Existing AABB users (readers, screen devices) construct this from their half-extents as 4 corner vertices. New obstacle shapes provide arbitrary convex polygon vertices.

### `WrapAnchor`

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
}
```

### `WrapWire`

```rust
#[derive(Component, Debug, Clone)]
pub struct WrapWire {
    /// Ordered anchor points from source toward dest.
    pub anchors: Vec<WrapAnchor>,
    /// Current total path length (sum of spans through anchors).
    pub path_length: f32,
    /// Target length the cable is retracting toward.
    pub target_length: f32,
}
```

### `RopeWire` (simplified)

Keeps its current `particles: Vec<RopeParticle>` and `rest_length: f32`. Represents a single particle chain covering the full cable path.

**Anchor pinning**: When anchors change (added/removed), walk the particle chain and find the particle whose index is closest to the anchor's proportional position along the path (e.g. if the anchor is 40% of the way along the cable, pin particle at index `0.4 * (len - 1)`). Store the pinned particle index in the anchor. Each physics tick, force that particle's position to the anchor's world position. Segments between pins wobble freely.

## Wrap Detection Algorithm

### Adding anchors (wrap)

For each span (line segment between adjacent pin points — endpoints or existing anchors), check if the straight line intersects any `CableCollider` polygon edge. If it does:

1. Find the polygon vertex nearest to the intersection point on the "outside" of the cable's path
2. Compute the wrap direction using the 2D cross product: `sign((B - A) x (vertex - A))` where A->B is the span direction
3. Insert a new `WrapAnchor` at that vertex
4. Re-pin the nearest particle to the anchor position

### Removing anchors (unwrap)

For each existing anchor, check if the cable has swung past the wrap threshold:

1. Take the segments before and after the anchor (prev_pin -> anchor -> next_pin)
2. Compute the cross product of `(anchor - prev_pin) x (next_pin - anchor)`
3. Compare its sign against the anchor's stored `wrap_sign`
4. If the sign has flipped, the cable has swung back past the obstacle — remove the anchor

### Moving obstacles

Each frame, update anchor world positions from `CableCollider` entity transforms + vertex index. If an obstacle moves, the anchors follow automatically.

### Performance

Brute-force all spans against all obstacles. At card game scale (dozens of obstacles), no spatial acceleration needed.

## Retraction & Length Management

**Target length** = shortest path through all anchors (sum of distances between consecutive pin points) x a small slack factor (e.g. `1.05x`).

**When excess length exists** (after unwrapping, or endpoints moved closer):

```
target_length -= (target_length - shortest_path) * retraction_rate * dt
```

`retraction_rate` is a tunable constant (2.0-5.0 per second). Proportional decay: fast initial pull, exponential approach to target.

**When endpoints move apart** (cable pulled taut): `target_length` is clamped to never go below the actual shortest path through anchors. The Verlet constraints handle the visual tension.

**Rest length for Verlet particles** is derived from `target_length / (particle_count - 1)`. As the cable retracts, rest lengths shorten, and the constraint solver pulls particles inward — producing the visible "reeling in" effect.

**No maximum cable length** — the cable can stretch as far as the endpoints demand. Retraction only fights slack.

## System Order & Integration

### `Phase::Update` (once per frame)

```
pending_cable_drag_system
  -> wrap_update_system        (update anchor positions from transforms)
  -> wrap_detect_system        (add/remove anchors)
  -> retraction_system         (shrink target_length toward shortest path)
  -> rope_render_system        (reads particle positions)
```

### `Phase::FixedUpdate` (fixed timestep)

```
rope_physics_system            (verlet step + pin at anchors + constraints + collision)
  .after(physics_sync_system)
```

The Verlet sim reads anchor positions set during `Update` and steps at a fixed rate. Render reads particle positions that may be one tick behind the anchors — imperceptible at 60fps.

### What changes in existing systems

- **`rope_physics_system`**: Gains anchor-pinning logic. After `verlet_step` and during constraint iterations, particles nearest to anchor positions are pinned. Rest length comes from `WrapWire::target_length`.
- **`pending_cable_drag_system`**: Spawns `WrapWire` alongside `RopeWire` on cable entities. Wrap detection is active during drag.
- **`jack_socket_release_system`**: No change.
- **`resize_for_endpoints`**: Still needed for particle count management. The wrap_ratio guard simplifies since `WrapWire` explicitly tracks whether the cable is wrapped.
- **`CableCollider`**: Migrated from `half_extents: Vec2` to `vertices: Vec<Vec2>`. Reader and screen device spawn code updated to emit 4-corner vertex lists.

### New components on cable entities

`WrapWire` added alongside existing `RopeWire`, `RopeWireEndpoints`, `Cable`.

## Testing Strategy

### WrapWire unit tests (geometric layer — pure math, no ECS)

- `when_line_crosses_polygon_edge_then_anchor_inserted`
- `when_cable_swings_past_anchor_then_anchor_removed`
- `when_endpoint_moves_closer_then_target_length_retracts`
- `when_cable_wraps_full_loop_around_obstacle_then_all_four_corners_anchored`
- `when_obstacle_moves_then_anchor_positions_follow`

### RopeWire integration with anchors (Verlet + pinning)

- `when_anchors_present_then_nearest_particles_pinned`
- `when_target_length_shrinks_then_rest_length_decreases`

### System-level behavioral tests (World + Schedule)

- `when_cable_dragged_across_obstacle_then_wraps_around_it`
- `when_cable_unwraps_then_particles_retract`

### CableCollider migration

Existing tests for `resolve_aabb_collisions` adapt to polygon vertices — same behavior, different input format.

All tests in `tests/suite/` per project convention.

## Migration Path

Each step is independently shippable and testable:

1. **Migrate `CableCollider` to polygon vertices** — update component, update reader/screen_device spawn code. Update `resolve_aabb_collisions` to work with polygon vertices. Existing behavior preserved.

2. **Add `WrapWire` component** — initially empty anchors (no wrapping). Cable entities get `WrapWire` alongside `RopeWire`. No behavior change.

3. **Move `rope_physics_system` to `FixedUpdate`** — stabilizes existing Verlet sim. Visual improvement, no feature change.

4. **Implement wrap detection** — `wrap_detect_system` populates anchors. `rope_physics_system` pins particles at anchors. Wrapping is live.

5. **Implement retraction** — `retraction_system` drives `target_length` toward shortest path. Cable reels in after unwrapping.

6. **Wire into pending cable drag** — spawn `WrapWire` during drag so wrapping works before connection.
