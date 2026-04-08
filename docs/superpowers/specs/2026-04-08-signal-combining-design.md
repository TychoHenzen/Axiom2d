# Signal Combining Design

**Date:** 2026-04-08
**Status:** Approved

## Overview

A combiner device merges two `SignatureSpace` signals into one. The combined signal is an 8D geometric shape — a capsule for 2 control points, a B-spline loop tube for 3+ — inflated to preserve a total volume equal to the sum of individual card sphere volumes. Combiners can chain: the output of one feeds into another, unioning control points into a single canonically-sorted set.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Device type | Dedicated 2-in 1-out combiner | Explicit wiring graph, composable |
| Signal type | Generalize `SignatureSpace` in-place | One type everywhere, no branching in propagation |
| Control point ordering | Canonical lexicographic sort on 8 axes | Same card set always produces identical signal regardless of wiring topology |
| Volume model | Sum of individual sphere volumes | Compensatory — more points = longer curve but total volume grows to prevent excessive thinning |
| Radius solve | Newton's method | Converges in 3-5 iterations for this polynomial |
| Chained combiner latency | One-frame delay per link | Zero complexity; perceptible but acceptable |
| Single-input behavior | Pass-through | Enables future toggle mechanics |
| Screen rendering (first pass) | Projected dots + line segments | Defer smooth B-spline rendering to a later iteration |

## Section 1: `SignatureSpace` Generalization

### Before

```rust
pub struct SignatureSpace {
    pub center: CardSignature,
    pub radius: f32,
}
```

### After

```rust
pub struct SignatureSpace {
    pub control_points: Vec<CardSignature>,
    pub radius: f32,
    pub volume: f32,
}
```

- `control_points` is canonically sorted: lexicographic comparison on the 8 axes, ascending.
- `radius` is the inflation radius computed to preserve total volume.
- 1 point = sphere (backward compatible with current behavior).
- 2 points = capsule (line segment Minkowski-summed with 8D ball).
- 3+ points = closed B-spline loop tube.

### Per-card radius variance

The reader emits a per-card sphere radius in the range [0.15, 0.25], deterministically derived from the card's signature (replacing the current fixed `SIGNATURE_SPACE_RADIUS` of 0.2). The derivation: compute the mean absolute intensity across all 8 axes, map linearly from [0.0, 1.0] → [0.15, 0.25]. Cards with stronger signatures get slightly larger spheres.

### Volume preservation

The reference volume for a single card with radius `R` is the 8D hypersphere volume:

```
V_sphere(R) = π⁴/24 × R⁸
```

When combining N cards with individual radii `R₁..Rₙ`, the total target volume is:

```
V_total = Σ V_sphere(Rᵢ)
```

For a tube of radius `r` around a curve of arc length `L` in 8D, the volume is approximately:

```
V_tube(r, L) = V_sphere(r) + L × C₇ × r⁷
```

where `C₇` is the volume of the 7D unit ball (`π³·⁵ / Γ(4.5)` = `π³/6 × 16/15`).

For 2 control points, `L` is the Euclidean distance between them in 8D. For 3+ points, `L` is the total arc length of the closed B-spline loop through the sorted control points.

Newton's method solves `f(r) = V_tube(r, L) - V_total = 0`:

```
r_{n+1} = rₙ - f(rₙ) / f'(rₙ)
```

Initial guess: `r₀ = (V_total × 24 / π⁴)^(1/8)` (the sphere-only radius). Converges in 3-5 iterations.

### Canonical sort

`CardSignature` gets an `Ord`-compatible comparison: lexicographic on `axes[0]` through `axes[7]`, using total-ordering float comparison (`f32::total_cmp`). After unioning control points, the combiner sorts the `Vec` and deduplicates (points closer than ε ≈ 1e-6 in 8D distance are merged).

### `contains` adaptation

The current `contains(&self, point: &CardSignature) -> bool` checks distance to center ≤ radius. The generalized version checks distance to the nearest point on the curve ≤ radius:

- 1 point: distance to the point (same as before).
- 2 points: distance to the line segment.
- 3+ points: distance to the nearest segment of the closed polyline through sorted points (exact B-spline distance can be deferred — polyline approximation is sufficient).

## Section 2: Combiner Device

### Component

```rust
pub struct CombinerDevice {
    pub input_a: Entity,
    pub input_b: Entity,
    pub output: Entity,
}
```

All three fields are jack entities.

### Physical layout

- Small rectangle body with `CableCollider` for wrap detection.
- Two input `JackSocket`s on the left edge (vertically stacked).
- One output `JackSocket` on the right edge (centered).
- Static S-curve shape drawn between the socket positions as a merge indicator.
- `Clickable` for drag interaction (same pattern as ScreenDevice).

### `combiner_system`

```
fn combiner_system(
    devices: Query<&CombinerDevice>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
)
```

Logic:

1. For each `CombinerDevice`, read data from `input_a` and `input_b` jacks.
2. If both are `None` → set output to `None`.
3. If exactly one has data → pass-through: clone that signal to the output.
4. If both have data → combine:
   a. Union all control points from both inputs.
   b. Sort lexicographically, deduplicate.
   c. Compute `V_total` = sum of `V_sphere(Rᵢ)` for each input signal's original contributing volumes. (See "Volume tracking" below.)
   d. Compute arc length `L` of the polyline through sorted points.
   e. Solve for `r` via Newton's method with `V_total` and `L`.
   f. Set output jack data to `SignatureSpace { control_points, radius: r }`.

### Volume tracking

Each `SignatureSpace` carries its total volume so that chained combiners can accumulate correctly. When a reader creates a 1-point signal, `volume = V_sphere(radius)`. When a combiner merges two signals, `volume = vol_a + vol_b`. The radius is then derived from this stored volume and the curve length.

This means `SignatureSpace` gains one more field:

```rust
pub struct SignatureSpace {
    pub control_points: Vec<CardSignature>,
    pub radius: f32,
    pub volume: f32,
}
```

The `radius` field is derived (could be recomputed from `volume` + curve length), but storing it avoids redundant Newton solves downstream.

## Section 3: ScreenDevice Adaptation

### `screen_render_system` changes

For each panel (2D axis pair), project all control points onto the panel's axis pair:

- 1 point → circle (dot), same as current behavior.
- 2+ points → draw each projected point as a small dot, connect them with line segments forming the polyline. The visual thickness is derived from `radius × PANEL_HALF`.

`ScreenSignalDot` is renamed to `ScreenSignalShape`. Instead of one circle per panel, it renders a `ShapeVariant::Polygon` built from the projected curve outline. For the first pass, this is straight line segments between projected points with dots at vertices.

Smooth B-spline curve rendering on the panels is deferred to a later iteration.

## Section 4: Propagation

**No changes** to `signature_space_propagation_system` — it copies `Jack<SignatureSpace>` data from cable source to cable dest, type-agnostic.

**No changes** to cable rendering, wrap detection, or socket interaction.

**System ordering:** `combiner_system` runs after `signature_space_propagation_system`. Chained combiners incur one-frame latency per link (output arrives at downstream consumers next frame). Acceptable.

## Section 5: File Changes

### Modified files

| File | Change |
|------|--------|
| `card/reader/signature_space.rs` | Generalize to `control_points` + `radius` + `volume`, add canonical sort, Newton solver, polyline distance |
| `card/reader/mod.rs` | Reader emits 1-point `SignatureSpace` with per-card radius (0.15–0.25 from signature) |
| `card/screen_device.rs` | `screen_render_system` handles multi-point projection; `ScreenSignalDot` → `ScreenSignalShape` |

### New files

| File | Content |
|------|---------|
| `card/combiner_device.rs` | `CombinerDevice` component, `combiner_system`, `spawn_combiner_device` |

### Binary wiring

`card_game_bin/src/main.rs` — register `combiner_system` in schedule after propagation, spawn a combiner device on the table.

## Section 6: Tests

| Test | Behavior |
|------|----------|
| Combiner with two 1-point inputs | Produces 2-point capsule output with volume = sum of input volumes |
| Combiner with one input | Pass-through: output equals the single input signal unchanged |
| Combiner with no inputs | Output is `None` |
| Chained combiner | Combiner fed by another combiner's output: merged point set is re-sorted, volume is sum of all leaf contributions |
| Newton solver accuracy | Computed radius produces tube volume within 1e-6 of target volume |
| Canonical sort stability | Same points in different insertion order produce identical `SignatureSpace` |
| Per-card radius derivation | Radius is deterministic from signature, within [0.15, 0.25] |
| `contains` on capsule | Point near the segment midpoint (within radius) returns true; point far away returns false |
| Screen rendering multi-point | ScreenDevice with 2-point signal produces visible polygon shape on panels |
