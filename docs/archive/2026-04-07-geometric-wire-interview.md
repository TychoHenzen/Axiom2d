# Geometric Taut Wires — Requirements Spec

> **For Claude:** This spec was produced by /interview. Use /writing-plans to expand into an implementation plan, or /tdd to implement directly.

**Goal:** Replace the broken Verlet rope particle simulation with geometric taut wires — cables are always-taut polylines through wrap anchors, rendered as ribbons with slight corner rounding.

**Date:** 2026-04-07

---

## Requirements

### What it does
- Cables are always taut polylines: `src → anchor1 → ... → anchorN → dst`
- Rendered as thick ribbons (using `CABLE_HALF_THICKNESS`) with slight Catmull-Rom smoothing at wrap-anchor corners
- Wrap detection (`detect_wraps`/`detect_unwraps`) stays unchanged — cables still route around card obstacles
- Signal propagation through cables (`signature_space_propagation_system`) stays unchanged
- Pending cable drag shows a wire from socket to cursor, with wrapping during drag

### What it does NOT do
- No slack, no sag, no catenary — this is a top-down flat tabletop
- No particle simulation, no Verlet, no constraint solving
- No retraction system (no slack to retract)
- No `target_length` (cable length = shortest geometric path, always)

### Deletions (~800 lines)
- `RopeParticle` struct
- `RopeWire` component (struct + all methods: `verlet_step`, `relax_constraints`, `pin_endpoints`, `resolve_polygon_collisions`, `resolve_trajectory_collisions`, `apply_shrinkage`, `resize_for_path`, `resize_for_endpoints`, `resize_free_end`, `rebuild_along_path`, `for_distance`, `new`, `with_particles`, `rest_length`, `path_length`)
- `rope_physics_system`, `retraction_system`, `rope_render_system`
- `cable_render_system`, `cable_visuals` (dead code, not registered in plugin)
- `catmull_rom_subdivide`, `particles_to_ribbon`
- `point_inside_polygon`, `point_push_out_of_polygon`
- `WrapWire.target_length` field, `WrapWire::retract()` method
- `WrapAnchor.pinned_particle` field
- All rope constants: `ROPE_DAMPING`, `ROPE_CONSTRAINT_ITERATIONS`, `ROPE_SLACK`, `SEGMENT_LENGTH`, `SHRINKAGE_STRENGTH`, `RETRACTION_RATE`, `SUBDIVISIONS_PER_SEGMENT`
- `UNWRAP_THRESHOLD` stays (used by `detect_unwraps`)

### Renames
- `RopeWireEndpoints` → `WireEndpoints`

### New code (~80 lines)
- `polyline_to_ribbon(waypoints, half_thickness)` — builds a ribbon polygon from waypoints with per-span Catmull-Rom smoothing near corners for slight rounding
- `wire_render_system` — computes waypoints from `WireEndpoints` + `WrapWire`, builds ribbon via `polyline_to_ribbon`, sets `Shape`

### Files touched
- `crates/card_game/src/card/jack_cable.rs` — major rewrite (delete ~800 lines, add ~80)
- `crates/card_game/src/card/jack_socket.rs` — remove `RopeWire` usage from imports/spawn/drag, rename `RopeWireEndpoints` → `WireEndpoints`
- `crates/card_game/src/plugin.rs` — remove `rope_physics_system` from FixedUpdate, remove `retraction_system`/`rope_render_system` from Update chain, add `wire_render_system`, update imports
- `crates/card_game/tests/suite/card_jack_cable.rs` — delete all particle/rope physics tests, keep geometry/wrap tests, adapt system tests for new component names
- `crates/card_game/tests/suite/card_jack_socket.rs` — update for `WireEndpoints` rename

## Subtask Checklist

- [ ] 1. Rewrite `jack_cable.rs`: delete `RopeParticle`, `RopeWire`, all rope methods/systems/constants, `cable_visuals`, `cable_render_system`, `catmull_rom_subdivide`, `particles_to_ribbon`, `point_inside_polygon`, `point_push_out_of_polygon`, `retraction_system`. Simplify `WrapWire` (remove `target_length`, `retract()`). Remove `pinned_particle` from `WrapAnchor`. Add `polyline_to_ribbon` and `wire_render_system`. Rename `RopeWireEndpoints` → `WireEndpoints`. Keep: `Cable`, `Jack`, `JackDirection`, `CableCollider`, `WrapWire` (detect_wraps/detect_unwraps/shortest_path), `WrapAnchor` (minus pinned_particle), `segment_intersects_segment`, `find_wrap_vertex`, `wrap_update_system`, `wrap_detect_system` (simplified), `signature_space_propagation_system`, `particles_to_bezier_path`.
- [ ] 2. Update `jack_socket.rs`: replace `RopeWire` import/usage with new types. In `on_socket_clicked`, spawn cables with `WireEndpoints` instead of `RopeWire` + `RopeWireEndpoints`. In `pending_cable_drag_system`, remove rope resize/target_length logic — just move the free end. In `jack_socket_release_system`, update cable spawn to use `WireEndpoints` without `RopeWire`.
- [ ] 3. Update `plugin.rs`: remove `rope_physics_system` from `FixedUpdate`. Remove `retraction_system` and `rope_render_system` from `Update` chain. Add `wire_render_system` in the chain where `rope_render_system` was. Update imports.
- [ ] 4. Build check: `cargo.exe build -p card_game` and `cargo.exe build -p card_game_bin` must compile.
- [ ] 5. Update tests in `card_jack_cable.rs`: delete all tests that exercise `RopeParticle`, `RopeWire`, `verlet_step`, `relax_constraints`, `apply_shrinkage`, `pin_endpoints`, `rope_physics_system`, `rope_render_system`, `retraction_system`, `resolve_polygon_collisions`, `resize_free_end`. Keep/adapt tests for `segment_intersects_segment`, `find_wrap_vertex`, `detect_wraps`, `detect_unwraps`, `shortest_path`, `wrap_detect_system`, signal propagation, `particles_to_bezier_path`. Update `wrap_detect_system` tests to not depend on `RopeWire`/`pinned_particle`.
- [ ] 6. Update tests in `card_jack_socket.rs`: rename `RopeWireEndpoints` → `WireEndpoints`, remove `RopeWire` from test spawns.
- [ ] 7. Full test pass: `cargo.exe test -p card_game` must pass.
- [ ] 8. Clippy + fmt: `cargo.exe clippy -p card_game` and `cargo.exe fmt --all` must pass.

## Research Notes

### Entity structure for a cable
A cable entity has: `Cable`, `RopeWireEndpoints` (→ `WireEndpoints`), `RopeWire` (→ deleted), `WrapWire`, `Transform2D`, `Shape`, `Visible`, `RenderLayer`, `SortOrder`, `LocalSortOrder`

### System execution order (current)
- `FixedUpdate`: `rope_physics_system` (after `physics_sync_system`) — **DELETE**
- `Update` chain: `pending_cable_drag_system → wrap_update_system → wrap_detect_system → retraction_system → rope_render_system → signature_space_propagation_system`
- New chain: `pending_cable_drag_system → wrap_update_system → wrap_detect_system → wire_render_system → signature_space_propagation_system`

### wrap_detect_system simplification
Currently queries `Option<&RopeWire>` for pinned_particle assignment (lines 1080-1104). Without RopeWire, this entire block is deleted. The system becomes: unwrap, wrap, snap target_length → unwrap, wrap (no target_length).

### pending_cable_drag_system simplification
Currently: moves free end, updates `wrap.target_length`, calls `rope.resize_free_end()` or `rope.resize_for_endpoints()`. New: just moves the free end. The wire renderer will pick up the new position.

### Test triage (~48 tests in card_jack_cable.rs)
- **Keep as-is**: signal propagation (2), segment_intersects_segment (2), find_wrap_vertex (1), detect_wraps (4), detect_unwraps (2), shortest_path (2), particles_to_bezier_path (2)
- **Adapt**: wrap_detect_system (2) — remove RopeWire dependency, cable_render (2) — might need rewrite for new renderer
- **Delete**: all RopeWire/RopeParticle/physics tests (~30)

## Open Questions

None — all requirements confirmed.
