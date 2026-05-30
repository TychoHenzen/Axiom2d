# Screen Panel Catmull-Rom Spline Rendering — Requirements Spec

> **For Claude:** This spec was produced by /interview. Use /writing-plans to expand into an implementation plan, or /tdd to implement directly.

**Goal:** Replace straight-segment signal rendering on screen panels with Catmull-Rom spline curves that have constant width and rounded endcaps.

**Date:** 2026-04-08

---

## Requirements

### Scope
- Screen panel signal rendering only (`screen_device.rs`). Cable rendering is unchanged.

### Behavior by signal type

**1-point signals:** Unchanged — rendered as clipped circles via `clipped_signal_circle`.

**2-point signals (open ribbon):**
- Straight ribbon with constant width (`space.radius * PANEL_HALF`)
- Semicircular endcaps at both endpoints (capsule shape)
- Clipped to panel bounds (±PANEL_HALF)

**3+ point signals (closed loop):**
- Catmull-Rom spline through all projected control points
- Smooth closed loop — the spline wraps from last point back to first with continuous curvature
- Constant width annular band (outer ring + inner ring from perpendicular offsets)
- 8 subdivisions per segment for smooth curves on the 100×100 px panels
- Clipped to panel bounds

### Constants
- `PANEL_SPLINE_SUBDIVISIONS = 8` (vs cable's 4 — higher for visual quality on small panels)
- Width = `space.radius * PANEL_HALF` (same formula as today, constant along the spline)

### What it does NOT do
- Does not change cable rendering (already has Catmull-Rom)
- Does not vary width along the spline
- No densification step (panel control points are sparse; subdivision handles smoothness)

## Subtask Checklist

- [ ] Subtask 1: Extract or share `catmull_rom_subdivide` — make the existing function in `jack_cable.rs` accessible from `screen_device.rs` (either move to a shared module or duplicate with adaptation for closed loops)
- [ ] Subtask 2: Add `catmull_rom_subdivide_closed` — variant that wraps indices so the spline closes smoothly (P[n-1]→P[0]→P[1] and P[n-2]→P[n-1]→P[0] use correct neighbors)
- [ ] Subtask 3: Add `semicircle_cap` helper — generates semicircular vertices at a point given a direction and radius, for endcaps on 2-point ribbons
- [ ] Subtask 4: Refactor `build_signal_polyline` for 3+ points — subdivide centerline with `catmull_rom_subdivide_closed`, compute perpendicular offsets at each subdivided sample, form annular polygon, clip to panel rect
- [ ] Subtask 5: Refactor `build_signal_polyline` for 2 points — straight ribbon with `semicircle_cap` endcaps at both ends, clip to panel rect
- [ ] Subtask 6: Unit tests for `catmull_rom_subdivide_closed` — verify smooth closure, correct number of output samples, passes through control points
- [ ] Subtask 7: Unit tests for `semicircle_cap` — verify correct arc generation and orientation
- [ ] Subtask 8: Verify existing integration tests pass — `cargo.exe test -p card_game`

## Research Notes

### Existing Catmull-Rom in `jack_cable.rs` (lines 513-543)
- `catmull_rom_subdivide(points, subdivisions)` — open spline, clamps boundary points
- Standard uniform Catmull-Rom formula: `P(t) = 0.5 * [2P₁ + (-P₀+P₂)t + (2P₀-5P₁+4P₂-P₃)t² + (-P₀+3P₁-3P₂+P₃)t³]`
- Produces `(n-1)*subdivisions + 1` output points

### Closed-loop adaptation needed
- For closed loops, neighbor lookup wraps: `P[i-1]` when `i=0` → `P[n-1]`, `P[i+2]` when `i=n-1` → `P[1]`
- Output: `n * subdivisions` points (no duplicate endpoint since the loop closes)

### Screen panel geometry
- Panel coordinate space: [-50, 50] × [-50, 50] (PANEL_HALF = 50.0)
- Control points are projected from 8D signature space via `project_signal_points`, then scaled by `PANEL_HALF`
- Clipping via Sutherland-Hodgman algorithm already implemented (`clip_polygon_to_rect`)

### Key file: `crates/card_game/src/card/screen_device.rs`
- `build_signal_polyline` (line 332) — the function to refactor
- `clipped_signal_circle` (line 390) — unchanged
- `screen_render_system` (line 110) — calls `build_signal_polyline`, unchanged interface

## Open Questions

None — all requirements confirmed.
