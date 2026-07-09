// Bond formation, constraint, and breaking tests — converted from CLI self-tests.
// All tests use HeadlessCapture::try_new() and skip gracefully when no GPU is available.

use particle_poc::capture::{CaptureConfig, HeadlessCapture};
use particle_poc::{CAPSULE_RADIUS, GREEN_SPECIES, INVALID_BOND};

/// Use production `particle_radius` (0.002) so the spatial hash grid works correctly.
/// Grid `cell_size` = 1.6/256 = 0.00625 ≥ 2*r = 0.004, so neighbor search covers adjacent particles.
fn bond_test_config() -> CaptureConfig {
    CaptureConfig {
        width: 64,
        height: 64,
        particle_radius: 0.002,
        gravity: 0.0,
        sub_steps: 16,
        num_particles: 10,
        ..Default::default()
    }
}

/// Two adjacent green particles should form a mutual bond.
/// @doc: green particles within formation radius form mutual bond after simulation
#[test]
fn when_two_green_particles_adjacent_then_bond_forms() {
    let Some(mut capture) = HeadlessCapture::try_new(bond_test_config()) else {
        return;
    };

    // Place two green particles at contact distance (matching self-test setup).
    let r = 0.002;
    let spacing = 2.0 * r;
    capture.spawn_at(
        &[[-spacing * 0.5, 0.0], [spacing * 0.5, 0.0]],
        &[GREEN_SPECIES, GREEN_SPECIES],
    );

    // Run 5 frames (bond formation happens once per frame).
    capture.step_n(5);

    let bonds = capture.read_bonds();
    let has_bond = bonds.iter().any(|b| b.partner != INVALID_BOND);
    assert!(has_bond, "expected at least one bond to form");
}

/// A mutual bond at rest length should constrain particle distance.
/// @doc: bonded particles cannot move beyond rest distance under constraint
#[test]
fn when_bonded_particles_within_rest_length_then_distance_constrained() {
    let Some(mut capture) = HeadlessCapture::try_new(bond_test_config()) else {
        return;
    };

    let initial_dist = 0.06;
    let half = initial_dist * 0.5;
    capture.spawn_at(
        &[[-half, 0.0], [half, 0.0]],
        &[GREEN_SPECIES, GREEN_SPECIES],
    );
    capture.set_mutual_bond(0, 1, 0.03);

    capture.step_n(5);

    let positions = capture.read_positions(2);
    let dx = positions[0][0] - positions[1][0];
    let dy = positions[0][1] - positions[1][1];
    let current_dist = (dx * dx + dy * dy).sqrt();
    assert!(
        current_dist < initial_dist * 0.95,
        "distance not reduced: {initial_dist:.4} -> {current_dist:.4}"
    );
}

/// A bond stretched beyond 5x rest length should break.
/// @doc: bonds break when stretched beyond break threshold (5 * `rest_length`)
#[test]
fn when_bond_stretched_beyond_break_distance_then_cleared() {
    let Some(mut capture) = HeadlessCapture::try_new(bond_test_config()) else {
        return;
    };

    let half = 0.15;
    capture.spawn_at(
        &[[-half, 0.0], [half, 0.0]],
        &[GREEN_SPECIES, GREEN_SPECIES],
    );
    capture.set_mutual_bond(0, 1, 0.03);

    capture.step_n(5);

    let bonds = capture.read_bonds();
    let all_cleared = bonds.iter().all(|b| b.partner == INVALID_BOND);
    assert!(all_cleared, "all bonds should be cleared after breaking");
}

/// Particles on the conveyor belt surface should remain stable (no NaN, no tunneling).
/// @doc: conveyor belt simulation stays stable with no NaN or extreme velocity
#[test]
fn when_particles_on_belt_surface_then_stable() {
    let r = 0.002;
    let mut config = bond_test_config();
    config.gravity = -1.2;
    config.particle_radius = r;
    config.num_particles = 200;
    let Some(mut capture) = HeadlessCapture::try_new(config) else {
        return;
    };

    let count = 50u32;
    let spacing = 2.1 * r;
    let mut positions = Vec::new();
    let mut species = Vec::new();
    let belt_cx = 0.0f32;
    let belt_cy = -0.22f32;
    for i in 0..count {
        let col = i % 5;
        let row = i / 5;
        let x = belt_cx - 0.1 + col as f32 * spacing;
        let y = belt_cy + CAPSULE_RADIUS + r + row as f32 * spacing;
        positions.push([x, y]);
        species.push(i % 3);
    }
    capture.spawn_at(&positions, &species);

    capture.step_n(60);

    let pos = capture.read_positions(count);
    for (i, &[px, py]) in pos.iter().enumerate() {
        assert!(
            px.is_finite() && py.is_finite(),
            "particle {i}: NaN position ({px:.4}, {py:.4})"
        );
        assert!(
            (-1.5..=1.5).contains(&px),
            "particle {i}: x={px:.4} out of bounds"
        );
        assert!(
            (-1.5..=1.5).contains(&py),
            "particle {i}: y={py:.4} out of bounds"
        );
    }
}
