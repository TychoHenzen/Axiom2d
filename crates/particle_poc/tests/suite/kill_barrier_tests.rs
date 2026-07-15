use particle_poc::KILL_Y;
use particle_poc::capture::{CaptureConfig, HeadlessCapture};

/// @doc: Verify that particles falling below the kill barrier Y threshold are removed.
#[test]
fn when_particle_falls_below_kill_y_then_it_is_removed() {
    let Some(mut c) = HeadlessCapture::try_new(CaptureConfig {
        width: 256,
        height: 256,
        particle_radius: 0.01,
        gravity: -1.2,
        sub_steps: 16,
        num_particles: 0,
        // wall_min_y below KILL_Y so the particle can fall past the barrier
        // without being stopped by the wall clamp.
        wall_min_y: -0.9,
        wall_max_y: 0.9,
        ..Default::default()
    }) else {
        return;
    };

    c.disable_machines();

    // Spawn a particle near the top — gravity will pull it past KILL_Y (-0.8)
    // on its way to the bottom wall at -0.9.
    c.spawn_at(&[[-0.3, 0.5]], &[0]);

    assert_eq!(c.particle_count(), 1);

    // Step enough frames for the particle to fall well past KILL_Y.
    c.step_n(200);

    // The particle should have been killed when crossing y < KILL_Y.
    // If the barrier worked, particle_count would be 0.
    let count = c.particle_count();
    assert!(
        count < 1,
        "particle survived below kill barrier y={KILL_Y}, count={count}"
    );
}

/// @doc: Hollow SDF circle contains 100 particles while 100 outside fall through kill barrier.
/// Spawns 200 particles interleaved (inside/outside), simulates 5s, expects exactly 100 alive
/// all within the circle interior.
#[test]
fn hollow_circle_contains_inside_particles_while_outside_die() {
    let Some(mut c) = HeadlessCapture::try_new(CaptureConfig {
        width: 256,
        height: 256,
        particle_radius: 0.002,
        gravity: -1.2,
        sub_steps: 16,
        num_particles: 0,
        // Wall floor well below KILL_Y so particles can cross the kill barrier.
        wall_min_y: -0.9,
        wall_max_y: 0.9,
        ..Default::default()
    }) else {
        return;
    };

    c.disable_machines();

    // Paint a hollow circle (ring): wall from r=0.20 to r=0.40, free interior.
    // Thick wall prevents SDF tunneling; large interior gives particles room to settle.
    c.paint_sdf([0.0, 0.0], 0.40, false);
    c.paint_sdf([0.0, 0.0], 0.20, true);
    c.upload_sdf();

    // Spawn 200 particles interleaved: source A inside circle, source B outside.
    let mut positions = Vec::with_capacity(200);
    let species = vec![0u32; 200];
    for i in 0..200 {
        if i % 2 == 0 {
            // Inside the hollow circle.
            positions.push([0.0, 0.02]);
        } else {
            // Outside the circle, below it but above KILL_Y.
            positions.push([0.0, -0.55]);
        }
    }
    c.spawn_at(&positions, &species);
    assert_eq!(c.particle_count(), 200);

    // 5 seconds at 60fps.
    c.step_n(300);

    let alive = c.particle_count();
    // Only the 100 inside the circle should survive.
    assert_eq!(alive, 100, "expected 100 survivors, got {alive}");

    // Verify all survivors are inside the circle interior (dist < 0.23 from origin,
    // allowing small margin past the 0.20 inner boundary for SDF push tolerance).
    let positions = c.read_positions(alive);
    for (i, pos) in positions.iter().enumerate() {
        let dist = (pos[0] * pos[0] + pos[1] * pos[1]).sqrt();
        assert!(
            dist < 0.23,
            "survivor {i} at ({}, {}) dist={dist:.4} is outside circle interior",
            pos[0],
            pos[1],
        );
    }
}
