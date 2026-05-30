# Signal Combining Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a combiner device that merges two `SignatureSpace` signals into a volume-preserving 8D curve, generalizing the signal type from a single sphere to a multi-point shape.

**Architecture:** Generalize `SignatureSpace` from `{center, radius}` to `{control_points, radius, volume}` where control points are canonically sorted. A new `CombinerDevice` (2-in, 1-out) unions control points and recomputes the inflation radius via Newton's method. `ScreenDevice` adapts to render multi-point signals as connected dots on its panels.

**Tech Stack:** Rust, bevy_ecs, glam, existing card_game crate patterns

**Spec:** `docs/superpowers/specs/2026-04-08-signal-combining-design.md`

---

### Task 1: Add canonical ordering to `CardSignature`

**Files:**
- Modify: `crates/card_game/src/card/identity/signature/types.rs`
- Test: `crates/card_game/tests/suite/card_identity_signature.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/card_game/tests/suite/card_identity_signature.rs`:

```rust
#[test]
fn when_signatures_sorted_then_lexicographic_on_axes() {
    // Arrange
    let a = CardSignature::new([0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let b = CardSignature::new([0.1, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let c = CardSignature::new([0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let mut sigs = vec![c, a, b];
    sigs.sort();

    // Assert
    assert_eq!(sigs, vec![a, b, c]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test when_signatures_sorted_then_lexicographic_on_axes -p card_game`
Expected: FAIL — `CardSignature` doesn't implement `Ord`

- [ ] **Step 3: Implement `Eq`, `Ord`, `PartialOrd` on `CardSignature`**

In `crates/card_game/src/card/identity/signature/types.rs`, add after the existing `#[derive]` and `impl` blocks:

```rust
impl Eq for CardSignature {}

impl PartialOrd for CardSignature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardSignature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (a, b) in self.axes.iter().zip(other.axes.iter()) {
            let ord = f32::total_cmp(a, b);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo.exe test when_signatures_sorted_then_lexicographic_on_axes -p card_game`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/card/identity/signature/types.rs crates/card_game/tests/suite/card_identity_signature.rs
git commit -m "feat(card): add canonical Ord to CardSignature via lexicographic f32::total_cmp"
```

---

### Task 2: Volume math helpers

**Files:**
- Create: `crates/card_game/src/card/reader/volume.rs`
- Modify: `crates/card_game/src/card/reader.rs` (add `pub mod volume;`)
- Test: `crates/card_game/tests/suite/card_reader_volume.rs`
- Modify: `crates/card_game/tests/suite/mod.rs` (add `mod card_reader_volume;`)

- [ ] **Step 1: Write failing tests**

Create `crates/card_game/tests/suite/card_reader_volume.rs`:

```rust
#![allow(clippy::unwrap_used)]

use card_game::card::reader::volume::{
    polyline_arc_length, solve_tube_radius, sphere_volume_8d, tube_volume_8d,
};
use card_game::card::identity::signature::CardSignature;

#[test]
fn when_sphere_radius_is_zero_then_volume_is_zero() {
    assert_eq!(sphere_volume_8d(0.0), 0.0);
}

#[test]
fn when_sphere_radius_is_02_then_volume_matches_formula() {
    // Arrange
    let r = 0.2_f32;
    let pi4 = std::f32::consts::PI.powi(4);
    let expected = pi4 / 24.0 * r.powi(8);

    // Act
    let v = sphere_volume_8d(r);

    // Assert
    assert!((v - expected).abs() < 1e-12, "got {v}, expected {expected}");
}

#[test]
fn when_arc_length_is_zero_then_tube_volume_equals_sphere_volume() {
    // Arrange
    let r = 0.2;

    // Act
    let tube_v = tube_volume_8d(r, 0.0);
    let sphere_v = sphere_volume_8d(r);

    // Assert
    assert!((tube_v - sphere_v).abs() < 1e-12);
}

#[test]
fn when_newton_solver_given_sphere_volume_and_zero_length_then_returns_original_radius() {
    // Arrange
    let r = 0.2;
    let v = sphere_volume_8d(r);

    // Act
    let solved = solve_tube_radius(v, 0.0);

    // Assert
    assert!((solved - r).abs() < 1e-5, "got {solved}, expected {r}");
}

#[test]
fn when_newton_solver_given_capsule_then_radius_preserves_volume() {
    // Arrange
    let r1 = 0.18;
    let r2 = 0.22;
    let v_total = sphere_volume_8d(r1) + sphere_volume_8d(r2);
    let arc_len = 0.5; // arbitrary segment length

    // Act
    let solved_r = solve_tube_radius(v_total, arc_len);

    // Assert
    let actual_v = tube_volume_8d(solved_r, arc_len);
    assert!(
        (actual_v - v_total).abs() < 1e-6,
        "tube volume {actual_v} must match target {v_total}, solved radius = {solved_r}"
    );
}

#[test]
fn when_polyline_has_two_points_then_arc_length_is_euclidean_distance() {
    // Arrange
    let a = CardSignature::new([0.0; 8]);
    let b = CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let len = polyline_arc_length(&[a, b]);

    // Assert
    assert!((len - 0.5).abs() < 1e-6);
}

#[test]
fn when_polyline_has_one_point_then_arc_length_is_zero() {
    let a = CardSignature::new([0.1; 8]);
    assert_eq!(polyline_arc_length(&[a]), 0.0);
}
```

Register in `crates/card_game/tests/suite/mod.rs` — add `mod card_reader_volume;`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo.exe test card_reader_volume -p card_game`
Expected: FAIL — module `volume` doesn't exist

- [ ] **Step 3: Create volume module with implementations**

Create `crates/card_game/src/card/reader/volume.rs`:

```rust
use crate::card::identity::signature::CardSignature;

use std::f32::consts::PI;

pub fn sphere_volume_8d(r: f32) -> f32 {
    PI.powi(4) / 24.0 * r.powi(8)
}

/// Volume of the 7D unit ball: V₇(1) = 16π³/105
fn ball_volume_7d_unit() -> f32 {
    16.0 * PI.powi(3) / 105.0
}

/// Volume of a tube of radius `r` around a curve of arc length `L` in 8D.
///
/// `V_tube = V₈(r) + L × V₇(r)` where `V₇(r) = V₇(1) × r⁷`.
pub fn tube_volume_8d(r: f32, arc_length: f32) -> f32 {
    sphere_volume_8d(r) + arc_length * ball_volume_7d_unit() * r.powi(7)
}

/// Derivative of tube volume w.r.t. radius:
/// dV/dr = 8 × π⁴/24 × r⁷ + 7 × L × V₇(1) × r⁶
fn tube_volume_derivative(r: f32, arc_length: f32) -> f32 {
    8.0 * PI.powi(4) / 24.0 * r.powi(7) + 7.0 * arc_length * ball_volume_7d_unit() * r.powi(6)
}

/// Solve for the tube radius that produces the target volume.
///
/// Uses Newton's method starting from the sphere-only radius.
/// Converges in 3-5 iterations for typical inputs.
pub fn solve_tube_radius(target_volume: f32, arc_length: f32) -> f32 {
    if target_volume <= 0.0 {
        return 0.0;
    }
    // Initial guess: radius of a sphere with this volume.
    // V₈(r) = π⁴/24 × r⁸  →  r = (V × 24/π⁴)^(1/8)
    let mut r = (target_volume * 24.0 / PI.powi(4)).powf(1.0 / 8.0);

    for _ in 0..10 {
        let f = tube_volume_8d(r, arc_length) - target_volume;
        let df = tube_volume_derivative(r, arc_length);
        if df.abs() < f32::EPSILON {
            break;
        }
        let delta = f / df;
        r -= delta;
        r = r.max(f32::EPSILON);
        if delta.abs() < 1e-8 {
            break;
        }
    }
    r
}

/// Total arc length of a polyline through the given control points.
///
/// For 0 or 1 points, returns 0.0.
/// For 2 points, returns the Euclidean distance between them.
/// For 3+ points, returns the total perimeter of the closed loop.
pub fn polyline_arc_length(points: &[CardSignature]) -> f32 {
    if points.len() <= 1 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 0..points.len() {
        let next = if i + 1 < points.len() {
            i + 1
        } else if points.len() > 2 {
            0 // close the loop for 3+ points
        } else {
            break;
        };
        total += points[i].distance_to(&points[next]);
    }
    total
}
```

Add to `crates/card_game/src/card/reader.rs` after `mod signature_space;`:

```rust
pub mod volume;
```

And add to the re-exports at the bottom if needed (the tests import directly from the module path).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo.exe test card_reader_volume -p card_game`
Expected: All 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/card_game/src/card/reader/volume.rs crates/card_game/src/card/reader.rs crates/card_game/tests/suite/card_reader_volume.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card): add 8D volume math with Newton solver for tube radius"
```

---

### Task 3: Generalize `SignatureSpace` and migrate all callsites

**Files:**
- Modify: `crates/card_game/src/card/reader/signature_space.rs`
- Modify: `crates/card_game/src/card/reader/insert.rs`
- Modify: `crates/card_game/src/card/screen_device.rs`
- Modify: `crates/card_game/tests/suite/card_screen_device.rs`
- Modify: `crates/card_game/tests/suite/card_jack_cable.rs`
- Modify: `crates/card_game/tests/suite/card_reader.rs`

This is a mechanical migration — the struct changes shape, all constructors update.

- [ ] **Step 1: Rewrite `signature_space.rs`**

Replace the entire contents of `crates/card_game/src/card/reader/signature_space.rs`:

```rust
use bevy_ecs::prelude::Component;

use crate::card::identity::signature::CardSignature;
use crate::card::reader::volume::{polyline_arc_length, solve_tube_radius, sphere_volume_8d};

/// Default radius for backward compatibility in tests.
pub const SIGNATURE_SPACE_RADIUS: f32 = 0.2;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct SignatureSpace {
    pub control_points: Vec<CardSignature>,
    pub radius: f32,
    pub volume: f32,
}

impl SignatureSpace {
    /// Create a single-point sphere signal (the common case for one card).
    pub fn from_single(center: CardSignature, radius: f32) -> Self {
        let volume = sphere_volume_8d(radius);
        Self {
            control_points: vec![center],
            radius,
            volume,
        }
    }

    /// Combine two signals by unioning control points and recomputing the radius.
    pub fn combine(a: &Self, b: &Self) -> Self {
        let mut points = Vec::with_capacity(a.control_points.len() + b.control_points.len());
        points.extend_from_slice(&a.control_points);
        points.extend_from_slice(&b.control_points);
        points.sort();
        points.dedup_by(|x, y| x.distance_to(y) < 1e-6);

        let volume = a.volume + b.volume;
        let arc_length = polyline_arc_length(&points);
        let radius = solve_tube_radius(volume, arc_length);

        Self {
            control_points: points,
            radius,
            volume,
        }
    }

    /// Check whether a point in signature space lies within this signal's volume.
    pub fn contains(&self, point: &CardSignature) -> bool {
        self.min_distance_to(point) <= self.radius
    }

    fn min_distance_to(&self, point: &CardSignature) -> f32 {
        match self.control_points.len() {
            0 => f32::INFINITY,
            1 => self.control_points[0].distance_to(point),
            _ => {
                let n = self.control_points.len();
                let mut best = f32::INFINITY;
                let segment_count = if n == 2 { 1 } else { n };
                for i in 0..segment_count {
                    let j = (i + 1) % n;
                    let d = point_to_segment_distance(
                        point,
                        &self.control_points[i],
                        &self.control_points[j],
                    );
                    best = best.min(d);
                }
                best
            }
        }
    }
}

fn point_to_segment_distance(p: &CardSignature, a: &CardSignature, b: &CardSignature) -> f32 {
    let pa = p.axes();
    let aa = a.axes();
    let ba = b.axes();

    let mut dot_ab_ab = 0.0_f32;
    let mut dot_ap_ab = 0.0_f32;
    for i in 0..8 {
        let ab_i = ba[i] - aa[i];
        let ap_i = pa[i] - aa[i];
        dot_ab_ab += ab_i * ab_i;
        dot_ap_ab += ap_i * ab_i;
    }

    if dot_ab_ab < f32::EPSILON {
        return p.distance_to(a);
    }

    let t = (dot_ap_ab / dot_ab_ab).clamp(0.0, 1.0);
    let mut dist_sq = 0.0_f32;
    for i in 0..8 {
        let closest_i = aa[i] + t * (ba[i] - aa[i]);
        let diff = pa[i] - closest_i;
        dist_sq += diff * diff;
    }
    dist_sq.sqrt()
}
```

- [ ] **Step 2: Update `insert.rs` to use `from_single`**

In `crates/card_game/src/card/reader/insert.rs`, change the import:

```rust
// Before:
use crate::card::reader::signature_space::{SIGNATURE_SPACE_RADIUS, SignatureSpace};

// After:
use crate::card::reader::signature_space::{SIGNATURE_SPACE_RADIUS, SignatureSpace};
```

(Import stays the same.) Change the construction at line 61-64:

```rust
// Before:
jack.data = Some(SignatureSpace {
    center: card.signature,
    radius: SIGNATURE_SPACE_RADIUS,
});

// After:
jack.data = Some(SignatureSpace::from_single(
    card.signature,
    SIGNATURE_SPACE_RADIUS,
));
```

- [ ] **Step 3: Update `screen_device.rs` — `display_axes` and render system**

In `crates/card_game/src/card/screen_device.rs`:

Change `display_axes` (line 84-88):

```rust
// Before:
pub fn display_axes(space: &SignatureSpace, display_index: usize) -> (f32, f32) {
    let x_element = Element::ALL[display_index * 2];
    let y_element = Element::ALL[display_index * 2 + 1];
    (space.center[x_element], space.center[y_element])
}

// After:
pub fn display_axes(space: &SignatureSpace, display_index: usize) -> (f32, f32) {
    let x_element = Element::ALL[display_index * 2];
    let y_element = Element::ALL[display_index * 2 + 1];
    (space.control_points[0][x_element], space.control_points[0][y_element])
}
```

Change `screen_render_system` (line 95-119) — for now, keep using the first control point (multi-point rendering comes in Task 6):

No changes needed — it calls `display_axes` which now reads `control_points[0]`.

Update the `#[cfg(test)]` block's `SignatureSpace` constructors. Change each occurrence of:

```rust
SignatureSpace {
    center: CardSignature::new([...]),
    radius: SIGNATURE_SPACE_RADIUS,
}
```

to:

```rust
SignatureSpace::from_single(
    CardSignature::new([...]),
    SIGNATURE_SPACE_RADIUS,
)
```

There are 3 occurrences in the test module: line 420-422, line 456 (indirect via `SIGNATURE_SPACE_RADIUS`).

For the `make_world` helper at line 380, update the parameter type — it already takes `Option<SignatureSpace>` so no signature change needed.

- [ ] **Step 4: Update test file `card_screen_device.rs`**

In `crates/card_game/tests/suite/card_screen_device.rs`, replace every `SignatureSpace { center: ..., radius: ... }` with `SignatureSpace::from_single(...)`.

There are 5 occurrences. Each follows this pattern:

```rust
// Before:
SignatureSpace {
    center: CardSignature::new([...]),
    radius: SIGNATURE_SPACE_RADIUS,
}

// After:
SignatureSpace::from_single(
    CardSignature::new([...]),
    SIGNATURE_SPACE_RADIUS,
)
```

Lines to update: 73-76, 96-99, 145-148, 211-214, 227-230.

Also update `display_axes` call — it still takes `&SignatureSpace` and returns `(f32, f32)`, so the test assertions remain correct.

- [ ] **Step 5: Update test file `card_jack_cable.rs`**

In `crates/card_game/tests/suite/card_jack_cable.rs`, update the `make_space` helper (line 28-32):

```rust
// Before:
fn make_space(center_values: [f32; 8]) -> SignatureSpace {
    SignatureSpace {
        center: CardSignature::new(center_values),
        radius: SIGNATURE_SPACE_RADIUS,
    }
}

// After:
fn make_space(center_values: [f32; 8]) -> SignatureSpace {
    SignatureSpace::from_single(
        CardSignature::new(center_values),
        SIGNATURE_SPACE_RADIUS,
    )
}
```

Also update line 150 if there's a direct construction there.

- [ ] **Step 6: Update test file `card_reader.rs`**

In `crates/card_game/tests/suite/card_reader.rs`, replace all 3 direct `SignatureSpace` constructions at lines 472-475, 873-876, 1124-1127:

```rust
// Before:
SignatureSpace {
    center: sig,
    radius: SIGNATURE_SPACE_RADIUS,
}

// After:
SignatureSpace::from_single(sig, SIGNATURE_SPACE_RADIUS)
```

- [ ] **Step 7: Run full test suite**

Run: `cargo.exe test -p card_game`
Expected: All existing tests PASS (the struct change is fully migrated)

- [ ] **Step 8: Commit**

```bash
git add crates/card_game/src/card/reader/signature_space.rs crates/card_game/src/card/reader/insert.rs crates/card_game/src/card/screen_device.rs crates/card_game/tests/suite/card_screen_device.rs crates/card_game/tests/suite/card_jack_cable.rs crates/card_game/tests/suite/card_reader.rs
git commit -m "refactor(card): generalize SignatureSpace to multi-point control_points + volume"
```

---

### Task 4: Per-card signature radius

**Files:**
- Modify: `crates/card_game/src/card/reader/signature_space.rs`
- Modify: `crates/card_game/src/card/reader/insert.rs`
- Modify: `crates/card_game/src/card/reader.rs` (re-export)
- Test: `crates/card_game/tests/suite/card_reader_volume.rs` (add tests)

- [ ] **Step 1: Write failing tests**

Add to `crates/card_game/tests/suite/card_reader_volume.rs`:

```rust
use card_game::card::reader::signature_radius;

#[test]
fn when_all_axes_zero_then_radius_is_015() {
    // Arrange
    let sig = CardSignature::new([0.0; 8]);

    // Act
    let r = signature_radius(&sig);

    // Assert
    assert!((r - 0.15).abs() < 1e-6);
}

#[test]
fn when_all_axes_max_intensity_then_radius_is_025() {
    // Arrange — all axes at ±1.0 (max intensity)
    let sig = CardSignature::new([1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);

    // Act
    let r = signature_radius(&sig);

    // Assert
    assert!((r - 0.25).abs() < 1e-6);
}

#[test]
fn when_signature_radius_called_twice_then_deterministic() {
    let sig = CardSignature::new([0.3, -0.7, 0.1, 0.4, -0.2, 0.5, -0.8, 0.6]);
    assert_eq!(signature_radius(&sig), signature_radius(&sig));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo.exe test when_all_axes_zero_then_radius -p card_game`
Expected: FAIL — `signature_radius` doesn't exist

- [ ] **Step 3: Implement `signature_radius`**

Add to `crates/card_game/src/card/reader/signature_space.rs`:

```rust
use crate::card::identity::signature::Element;

/// Compute the per-card signal sphere radius from signature intensity.
///
/// Maps mean absolute intensity [0.0, 1.0] → radius [0.15, 0.25].
pub fn signature_radius(sig: &CardSignature) -> f32 {
    let mean_intensity: f32 =
        Element::ALL.iter().map(|&e| sig.intensity(e)).sum::<f32>() / 8.0;
    0.15 + mean_intensity * 0.10
}
```

Add to `crates/card_game/src/card/reader.rs` re-exports:

```rust
pub use signature_space::{SIGNATURE_SPACE_RADIUS, SignatureSpace, signature_radius};
```

- [ ] **Step 4: Update `insert.rs` to use per-card radius**

In `crates/card_game/src/card/reader/insert.rs`:

```rust
// Before:
use crate::card::reader::signature_space::{SIGNATURE_SPACE_RADIUS, SignatureSpace};

// After:
use crate::card::reader::signature_space::{SignatureSpace, signature_radius};
```

```rust
// Before:
jack.data = Some(SignatureSpace::from_single(
    card.signature,
    SIGNATURE_SPACE_RADIUS,
));

// After:
jack.data = Some(SignatureSpace::from_single(
    card.signature,
    signature_radius(&card.signature),
));
```

- [ ] **Step 5: Run tests**

Run: `cargo.exe test -p card_game`
Expected: All tests PASS (existing tests use `SIGNATURE_SPACE_RADIUS` directly, unaffected)

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/reader/signature_space.rs crates/card_game/src/card/reader/insert.rs crates/card_game/src/card/reader.rs crates/card_game/tests/suite/card_reader_volume.rs
git commit -m "feat(card): derive per-card signal radius from signature intensity"
```

---

### Task 5: Combiner device — component, system, and tests

**Files:**
- Create: `crates/card_game/src/card/combiner_device.rs`
- Modify: `crates/card_game/src/card/mod.rs` (add `pub mod combiner_device;`)
- Create: `crates/card_game/tests/suite/card_combiner_device.rs`
- Modify: `crates/card_game/tests/suite/mod.rs` (add `mod card_combiner_device;`)

- [ ] **Step 1: Write failing tests**

Create `crates/card_game/tests/suite/card_combiner_device.rs`:

```rust
#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::combiner_device::{CombinerDevice, combiner_system};
use card_game::card::identity::signature::CardSignature;
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::volume::sphere_volume_8d;
use card_game::card::reader::SignatureSpace;

fn run_combiner(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(combiner_system);
    schedule.run(world);
}

fn make_signal(values: [f32; 8], radius: f32) -> SignatureSpace {
    SignatureSpace::from_single(CardSignature::new(values), radius)
}

fn spawn_combiner(world: &mut World) -> (Entity, Entity, Entity, Entity) {
    let input_a = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: None,
        })
        .id();
    let input_b = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: None,
        })
        .id();
    let output = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let device = world
        .spawn(CombinerDevice {
            input_a,
            input_b,
            output,
        })
        .id();
    (device, input_a, input_b, output)
}

#[test]
fn when_both_inputs_none_then_output_is_none() {
    // Arrange
    let mut world = World::new();
    let (_device, _a, _b, output) = spawn_combiner(&mut world);

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    assert!(jack.data.is_none());
}

#[test]
fn when_only_input_a_has_signal_then_output_passes_through() {
    // Arrange
    let mut world = World::new();
    let (_device, input_a, _input_b, output) = spawn_combiner(&mut world);
    let signal = make_signal([0.3, -0.2, 0.1, 0.4, 0.0, 0.0, 0.0, 0.0], 0.2);
    world.get_mut::<Jack<SignatureSpace>>(input_a).unwrap().data = Some(signal.clone());

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    assert_eq!(jack.data.as_ref(), Some(&signal));
}

#[test]
fn when_only_input_b_has_signal_then_output_passes_through() {
    // Arrange
    let mut world = World::new();
    let (_device, _input_a, input_b, output) = spawn_combiner(&mut world);
    let signal = make_signal([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.18);
    world.get_mut::<Jack<SignatureSpace>>(input_b).unwrap().data = Some(signal.clone());

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    assert_eq!(jack.data.as_ref(), Some(&signal));
}

#[test]
fn when_both_inputs_have_signals_then_output_combines_control_points() {
    // Arrange
    let mut world = World::new();
    let (_device, input_a, input_b, output) = spawn_combiner(&mut world);
    let sig_a = make_signal([0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_b = make_signal([0.5, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    world.get_mut::<Jack<SignatureSpace>>(input_a).unwrap().data = Some(sig_a.clone());
    world.get_mut::<Jack<SignatureSpace>>(input_b).unwrap().data = Some(sig_b.clone());

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    let combined = jack.data.as_ref().expect("output must have data");
    assert_eq!(combined.control_points.len(), 2, "must have 2 control points");
    // Points are sorted lexicographically — sig_a's point comes first
    assert_eq!(combined.control_points[0], sig_a.control_points[0]);
    assert_eq!(combined.control_points[1], sig_b.control_points[0]);
}

#[test]
fn when_both_inputs_have_signals_then_output_volume_is_sum() {
    // Arrange
    let mut world = World::new();
    let (_device, input_a, input_b, output) = spawn_combiner(&mut world);
    let r_a = 0.18;
    let r_b = 0.22;
    let sig_a = make_signal([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], r_a);
    let sig_b = make_signal([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], r_b);
    world.get_mut::<Jack<SignatureSpace>>(input_a).unwrap().data = Some(sig_a);
    world.get_mut::<Jack<SignatureSpace>>(input_b).unwrap().data = Some(sig_b);

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    let combined = jack.data.as_ref().expect("output must have data");
    let expected_vol = sphere_volume_8d(r_a) + sphere_volume_8d(r_b);
    assert!(
        (combined.volume - expected_vol).abs() < 1e-8,
        "combined volume {} must equal sum of inputs {}",
        combined.volume,
        expected_vol
    );
}

#[test]
fn when_combiner_chained_then_points_merged_and_sorted() {
    // Arrange — two combiners: C1 merges A+B, C2 takes C1's output + C
    let mut world = World::new();
    let (_dev1, in_a, in_b, out1) = spawn_combiner(&mut world);
    let (_dev2, in_c, in_d, out2) = spawn_combiner(&mut world);

    let sig_a = make_signal([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_b = make_signal([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_c = make_signal([0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);

    world.get_mut::<Jack<SignatureSpace>>(in_a).unwrap().data = Some(sig_a);
    world.get_mut::<Jack<SignatureSpace>>(in_b).unwrap().data = Some(sig_b);

    // First frame: C1 combines A+B
    run_combiner(&mut world);
    let c1_output = world
        .get::<Jack<SignatureSpace>>(out1)
        .unwrap()
        .data
        .clone();

    // Feed C1's output into C2's input, plus sig_c
    world.get_mut::<Jack<SignatureSpace>>(in_c).unwrap().data = c1_output;
    world.get_mut::<Jack<SignatureSpace>>(in_d).unwrap().data = Some(sig_c);

    // Act — second frame: C2 combines (A+B)+C
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(out2).unwrap();
    let combined = jack.data.as_ref().expect("output must have data");
    assert_eq!(combined.control_points.len(), 3, "must have 3 control points");
    // Sorted: 0.1, 0.3, 0.5
    assert_eq!(
        combined.control_points[0],
        CardSignature::new([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        combined.control_points[1],
        CardSignature::new([0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        combined.control_points[2],
        CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
}

#[test]
fn when_canonical_sort_then_same_points_different_order_produce_identical_output() {
    // Arrange
    let mut world = World::new();
    let (_dev1, in_a1, in_b1, out1) = spawn_combiner(&mut world);
    let (_dev2, in_a2, in_b2, out2) = spawn_combiner(&mut world);

    let sig_x = make_signal([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_y = make_signal([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);

    // C1: X on input_a, Y on input_b
    world.get_mut::<Jack<SignatureSpace>>(in_a1).unwrap().data = Some(sig_x.clone());
    world.get_mut::<Jack<SignatureSpace>>(in_b1).unwrap().data = Some(sig_y.clone());

    // C2: Y on input_a, X on input_b (swapped)
    world.get_mut::<Jack<SignatureSpace>>(in_a2).unwrap().data = Some(sig_y);
    world.get_mut::<Jack<SignatureSpace>>(in_b2).unwrap().data = Some(sig_x);

    // Act
    run_combiner(&mut world);

    // Assert
    let data1 = world
        .get::<Jack<SignatureSpace>>(out1)
        .unwrap()
        .data
        .as_ref()
        .unwrap()
        .clone();
    let data2 = world
        .get::<Jack<SignatureSpace>>(out2)
        .unwrap()
        .data
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(data1, data2, "identical card sets must produce identical output");
}
```

Register in `crates/card_game/tests/suite/mod.rs` — add `mod card_combiner_device;`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo.exe test card_combiner_device -p card_game`
Expected: FAIL — module `combiner_device` doesn't exist

- [ ] **Step 3: Create `combiner_device.rs`**

Create `crates/card_game/src/card/combiner_device.rs`:

```rust
use bevy_ecs::prelude::{Component, Entity, Query};

use crate::card::jack_cable::Jack;
use crate::card::reader::SignatureSpace;

#[derive(Component, Debug)]
pub struct CombinerDevice {
    pub input_a: Entity,
    pub input_b: Entity,
    pub output: Entity,
}

pub fn combiner_system(
    devices: Query<&CombinerDevice>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
) {
    let updates: Vec<(Entity, Option<SignatureSpace>)> = devices
        .iter()
        .filter_map(|device| {
            let data_a = jacks.get(device.input_a).ok()?.data.clone();
            let data_b = jacks.get(device.input_b).ok()?.data.clone();

            let combined = match (data_a, data_b) {
                (None, None) => None,
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (Some(a), Some(b)) => Some(SignatureSpace::combine(&a, &b)),
            };
            Some((device.output, combined))
        })
        .collect();

    for (output, data) in updates {
        if let Ok(mut jack) = jacks.get_mut(output) {
            jack.data = data;
        }
    }
}
```

Add to `crates/card_game/src/card/mod.rs`:

```rust
pub mod combiner_device;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo.exe test card_combiner_device -p card_game`
Expected: All 8 tests PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo.exe test -p card_game`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/combiner_device.rs crates/card_game/src/card/mod.rs crates/card_game/tests/suite/card_combiner_device.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card): add CombinerDevice with 2-in 1-out signal merging"
```

---

### Task 6: Combiner device spawn + visuals

**Files:**
- Modify: `crates/card_game/src/card/combiner_device.rs` (add spawn function + drag)

- [ ] **Step 1: Add spawn function and drag support**

Add to `crates/card_game/src/card/combiner_device.rs`:

```rust
use bevy_ecs::prelude::{
    Commands, Component, Entity, Query, Res, ResMut, Resource, Trigger, With, Without, World,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::prelude::{ChildOf, LocalSortOrder, SpawnChildExt, Visible};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, ClickedEntity};
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, on_socket_clicked};
use crate::card::reader::SignatureSpace;

const BODY_HALF_W: f32 = 40.0;
const BODY_HALF_H: f32 = 30.0;
const BODY_CORNER_RADIUS: f32 = 4.0;

const BODY_FILL: Color = Color {
    r: 0.12,
    g: 0.10,
    b: 0.16,
    a: 1.0,
};
const BODY_STROKE: Color = Color {
    r: 0.50,
    g: 0.35,
    b: 0.65,
    a: 1.0,
};
const SOCKET_COLOR: Color = Color {
    r: 0.4,
    g: 0.7,
    b: 0.9,
    a: 1.0,
};
const MERGE_LINE_COLOR: Color = Color {
    r: 0.35,
    g: 0.25,
    b: 0.55,
    a: 0.6,
};
const SOCKET_RADIUS: f32 = 8.0;
const SOCKET_SPACING: f32 = 20.0;
const COMBINER_LOCAL_SORT: i32 = -1;
const COMBINER_SOCKET_LOCAL_SORT: i32 = 1;
const COMBINER_DECOR_LOCAL_SORT: i32 = 0;

const COMBINER_HALF_EXTENTS: Vec2 = Vec2::new(BODY_HALF_W, BODY_HALF_H);
const INPUT_X: f32 = -(BODY_HALF_W + SOCKET_RADIUS + 4.0);
const OUTPUT_X: f32 = BODY_HALF_W + SOCKET_RADIUS + 4.0;

fn s_curve_points() -> Vec<Vec2> {
    let left_top = Vec2::new(INPUT_X + SOCKET_RADIUS, SOCKET_SPACING * 0.5);
    let left_bot = Vec2::new(INPUT_X + SOCKET_RADIUS, -SOCKET_SPACING * 0.5);
    let right = Vec2::new(OUTPUT_X - SOCKET_RADIUS, 0.0);
    let mid_x = (left_top.x + right.x) * 0.5;
    vec![
        left_top,
        Vec2::new(mid_x, SOCKET_SPACING * 0.25),
        Vec2::new(mid_x, 0.0),
        right,
        Vec2::new(mid_x, 0.0),
        Vec2::new(mid_x, -SOCKET_SPACING * 0.25),
        left_bot,
    ]
}

/// Spawns a combiner device at `position`.
///
/// Returns `(device_entity, input_a_jack, input_b_jack, output_jack)`.
pub fn spawn_combiner_device(world: &mut World, position: Vec2) -> (Entity, Entity, Entity, Entity) {
    let input_a = world
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
                position: position + Vec2::new(INPUT_X, SOCKET_SPACING * 0.5),
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
            LocalSortOrder(COMBINER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id();

    let input_b = world
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
                position: position + Vec2::new(INPUT_X, -SOCKET_SPACING * 0.5),
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
            LocalSortOrder(COMBINER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id();

    let output = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: SOCKET_RADIUS,
                color: SOCKET_COLOR,
                connected_cable: None,
            },
            Transform2D {
                position: position + Vec2::new(OUTPUT_X, 0.0),
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
            LocalSortOrder(COMBINER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id();

    let device = world
        .spawn((
            CombinerDevice {
                input_a,
                input_b,
                output,
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
            LocalSortOrder(COMBINER_LOCAL_SORT),
            Clickable(ClickHitShape::Aabb(COMBINER_HALF_EXTENTS)),
            CableCollider::from_aabb(COMBINER_HALF_EXTENTS),
        ))
        .id();

    world.entity_mut(device).observe(on_combiner_clicked);
    world.entity_mut(input_a).observe(on_socket_clicked);
    world.entity_mut(input_b).observe(on_socket_clicked);
    world.entity_mut(output).observe(on_socket_clicked);

    // S-curve decorative line
    world.spawn_child(
        device,
        (
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: s_curve_points(),
                },
                color: MERGE_LINE_COLOR,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(COMBINER_DECOR_LOCAL_SORT),
        ),
    );

    (device, input_a, input_b, output)
}

// ---- Drag support (same pattern as ScreenDevice) ----

#[derive(Resource, Debug, Default)]
pub struct CombinerDragState {
    pub dragging: Option<CombinerDragInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombinerDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
}

pub fn on_combiner_clicked(
    trigger: Trigger<ClickedEntity>,
    combiners: Query<&Transform2D, With<CombinerDevice>>,
    mut drag: ResMut<CombinerDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = combiners.get(entity) else {
        return;
    };
    drag.dragging = Some(CombinerDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}

pub fn combiner_drag_system(
    mouse: Res<MouseState>,
    drag: Res<CombinerDragState>,
    mut combiner_transforms: Query<&mut Transform2D, With<CombinerDevice>>,
    mut other_transforms: Query<&mut Transform2D, Without<CombinerDevice>>,
    combiners: Query<&CombinerDevice>,
) {
    let Some(info) = &drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = combiner_transforms.get_mut(info.entity) {
        transform.position = target;
    }
    if let Ok(device) = combiners.get(info.entity) {
        let jack_offsets = [
            (device.input_a, Vec2::new(INPUT_X, SOCKET_SPACING * 0.5)),
            (device.input_b, Vec2::new(INPUT_X, -SOCKET_SPACING * 0.5)),
            (device.output, Vec2::new(OUTPUT_X, 0.0)),
        ];
        for (jack_entity, offset) in jack_offsets {
            if let Ok(mut jack_t) = other_transforms.get_mut(jack_entity) {
                jack_t.position = target + offset;
            }
        }
    }
}

pub fn combiner_release_system(mouse: Res<MouseState>, mut drag: ResMut<CombinerDragState>) {
    if drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        drag.dragging = None;
    }
}
```

Remove the duplicate imports at the top (keep the expanded version that includes all needed types). The final file should have one set of imports covering both the system and spawn functions.

- [ ] **Step 2: Verify it compiles**

Run: `cargo.exe build -p card_game`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/card_game/src/card/combiner_device.rs
git commit -m "feat(card): add combiner device spawning, visuals, and drag support"
```

---

### Task 7: ScreenDevice multi-point rendering

**Files:**
- Modify: `crates/card_game/src/card/screen_device.rs`
- Modify: `crates/card_game/tests/suite/card_screen_device.rs`

- [ ] **Step 1: Write failing test**

Add to `crates/card_game/tests/suite/card_screen_device.rs`:

```rust
#[test]
fn when_signal_has_two_control_points_then_screen_draws_signal_shapes() {
    // Arrange
    let sig_a = CardSignature::new([0.3, 0.7, 0.1, 0.2, 0.4, 0.5, 0.6, 0.8]);
    let sig_b = CardSignature::new([-0.3, 0.2, 0.5, -0.1, 0.3, -0.4, 0.7, 0.1]);
    let combined = SignatureSpace::combine(
        &SignatureSpace::from_single(sig_a, 0.2),
        &SignatureSpace::from_single(sig_b, 0.2),
    );
    let (mut world, shape_calls) = make_screen_world(Some(combined));

    // Act
    run_screen_visuals(&mut world);

    // Assert — body fill + body stroke + socket + 4 panels + 4 signal shapes = 11
    let calls = shape_calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        11,
        "screen with 2-point signal must draw body, stroke, socket, 4 panels, and 4 signal shapes"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo.exe test when_signal_has_two_control_points -p card_game`
Expected: Likely panics — `display_axes` indexes `control_points[0]` which works, but the render might not produce the right output for multi-point.

Actually, the test may already pass since the current code renders a circle per panel regardless of point count. Let's adjust: this is about verifying the rendering works with multi-point data, not about count differences. The test verifies no crash/panic and correct draw call count.

- [ ] **Step 3: Update `display_axes` to `project_signal_points`**

In `crates/card_game/src/card/screen_device.rs`, add a new function and update `screen_render_system`:

```rust
/// Project all control points of a signal onto a 2D panel axis pair.
pub fn project_signal_points(space: &SignatureSpace, display_index: usize) -> Vec<Vec2> {
    let x_element = Element::ALL[display_index * 2];
    let y_element = Element::ALL[display_index * 2 + 1];
    space
        .control_points
        .iter()
        .map(|cp| Vec2::new(cp[x_element], cp[y_element]))
        .collect()
}
```

Keep `display_axes` for backward compat (it reads `control_points[0]`).

Rename `ScreenSignalDot` to `ScreenSignalShape`:

```rust
#[derive(Component)]
pub struct ScreenSignalShape {
    display_index: usize,
}
```

Update `screen_render_system` to render projected polylines:

```rust
pub fn screen_render_system(
    devices: Query<&ScreenDevice>,
    jacks: Query<&Jack<SignatureSpace>>,
    mut shapes: Query<(&ScreenSignalShape, &ChildOf, &mut Shape, &mut Visible)>,
) {
    for (signal_shape, parent, mut shape, mut visible) in &mut shapes {
        let Ok(device) = devices.get(parent.0) else {
            visible.0 = false;
            continue;
        };
        let Ok(jack) = jacks.get(device.signature_input) else {
            visible.0 = false;
            continue;
        };
        let Some(space) = jack.data.as_ref() else {
            visible.0 = false;
            continue;
        };

        let projected = project_signal_points(space, signal_shape.display_index);
        let visual_radius = space.radius * PANEL_HALF;

        if projected.len() == 1 {
            let center = projected[0] * PANEL_HALF;
            shape.variant = clipped_signal_circle(center, visual_radius);
        } else {
            let scaled: Vec<Vec2> = projected.iter().map(|p| *p * PANEL_HALF).collect();
            shape.variant = build_signal_polyline(&scaled, visual_radius);
        }
        shape.color = SIGNAL_COLOR;
        visible.0 = true;
    }
}
```

Add the polyline shape builder:

```rust
fn build_signal_polyline(points: &[Vec2], thickness: f32) -> ShapeVariant {
    // Build a polygon that traces along the points with visual thickness.
    // For each segment, create a ribbon (offset by ±thickness perpendicular).
    let mut polygon = Vec::new();
    let clamped_thickness = thickness.max(1.0);

    // Forward pass: offset left
    for i in 0..points.len() {
        let prev = if i == 0 { points[0] } else { points[i - 1] };
        let next = if i + 1 < points.len() {
            points[i + 1]
        } else {
            points[i]
        };
        let dir = (next - prev).normalize_or_zero();
        let normal = Vec2::new(-dir.y, dir.x);
        polygon.push(points[i] + normal * clamped_thickness);
    }

    // Reverse pass: offset right
    for i in (0..points.len()).rev() {
        let prev = if i == 0 { points[0] } else { points[i - 1] };
        let next = if i + 1 < points.len() {
            points[i + 1]
        } else {
            points[i]
        };
        let dir = (next - prev).normalize_or_zero();
        let normal = Vec2::new(-dir.y, dir.x);
        polygon.push(points[i] - normal * clamped_thickness);
    }

    let clipped = clip_polygon_to_rect(
        &polygon,
        Vec2::new(-PANEL_HALF, -PANEL_HALF),
        Vec2::new(PANEL_HALF, PANEL_HALF),
    );
    ShapeVariant::Polygon { points: clipped }
}
```

Update `spawn_screen_device` to use `ScreenSignalShape` instead of `ScreenSignalDot`:

```rust
// In the dot-spawning loop, change:
ScreenSignalDot { display_index }
// to:
ScreenSignalShape { display_index }
```

- [ ] **Step 4: Update `card_screen_device.rs` tests**

The `display_axes` function still works (backward compat). Update tests that referenced `ScreenSignalDot` if they import it. The external tests don't directly reference `ScreenSignalDot`, so only the inline `#[cfg(test)]` module in `screen_device.rs` needs updating (it constructs `SignatureSpace` which was already migrated in Task 3).

- [ ] **Step 5: Run tests**

Run: `cargo.exe test -p card_game`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/card/screen_device.rs crates/card_game/tests/suite/card_screen_device.rs
git commit -m "feat(card): adapt ScreenDevice to render multi-point signals as polylines"
```

---

### Task 8: Plugin + binary wiring

**Files:**
- Modify: `crates/card_game/src/plugin.rs`
- Modify: `crates/card_game_bin/src/main.rs`

- [ ] **Step 1: Register combiner systems in plugin**

In `crates/card_game/src/plugin.rs`, add imports:

```rust
use crate::card::combiner_device::{
    CombinerDragState, combiner_drag_system, combiner_release_system, combiner_system,
};
```

Add resource init in `fn build`:

```rust
world.insert_resource(CombinerDragState::default());
```

In `register_systems`, add `combiner_drag_system` to the drag chain (after `screen_drag_system`):

```rust
// In the Phase::Update chain, add combiner_drag_system after screen_drag_system:
screen_drag_system,
combiner_drag_system,
```

Add `combiner_release_system` after `screen_release_system`:

```rust
screen_release_system,
combiner_release_system,
```

Add `combiner_system` in the cable/signal chain, after `signature_space_propagation_system` and before `screen_render_system`:

```rust
signature_space_propagation_system,
combiner_system,
jack_socket_render_system,
screen_render_system,
```

- [ ] **Step 2: Spawn combiner device in the binary**

In `crates/card_game_bin/src/main.rs`, add import:

```rust
use card_game::card::combiner_device::spawn_combiner_device;
```

In `spawn_scene`, after the screen device spawn block, add:

```rust
// Spawn a combiner device — wire interactively.
let combiner_pos = Vec2::new(300.0, -150.0);
let (_combiner_entity, _comb_in_a, _comb_in_b, _comb_out) =
    spawn_combiner_device(world, combiner_pos);
```

- [ ] **Step 3: Build and verify**

Run: `cargo.exe build -p card_game_bin`
Expected: PASS

- [ ] **Step 4: Run full test suite**

Run: `cargo.exe test -p card_game`
Expected: All tests PASS

- [ ] **Step 5: Format and lint**

Run: `cargo.exe fmt --all && cargo.exe clippy -p card_game -p card_game_bin`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add crates/card_game/src/plugin.rs crates/card_game_bin/src/main.rs
git commit -m "feat(card): wire CombinerDevice into plugin schedule and spawn in scene"
```
