// Visual screen capture tests for particle_poc.
// All tests use HeadlessCapture::try_new() and skip gracefully when no GPU is available.

use particle_poc::capture::{CaptureConfig, HeadlessCapture};

fn test_config() -> CaptureConfig {
    CaptureConfig {
        width: 256,
        height: 256,
        particle_radius: 0.04,
        gravity: 0.0,
        sub_steps: 4,
        num_particles: 100,
        ..Default::default()
    }
}

/// Spawning a single red particle should produce visible red-tinted pixels
/// in the rendered output (R channel dominant over G and B).
/// @doc: single red particle renders with visible red channel dominance
#[test]
fn when_single_red_particle_then_pixels_contain_red_channel() {
    let Some(mut capture) = HeadlessCapture::try_new(test_config()) else {
        return;
    };

    // Spawn one red particle at the center.
    capture.spawn_at(&[[0.0, 0.0]], &[0]);

    // Render immediately (zero gravity, so it stays put).
    let pixels = capture.render_to_buffer();

    // Count pixels with R > G and R > B (red signature).
    let mut red_pixels = 0usize;
    let width = 256;
    for chunk in pixels.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        if r > g && r > b && r > 40 {
            red_pixels += 1;
        }
    }

    // At least some pixels must be red-tinted.
    assert!(
        red_pixels > 5,
        "expected at least 6 red-tinted pixels, got {red_pixels} out of {total}",
        total = width * width
    );
}

/// Spawning a red and blue particle should produce both color signatures.
/// @doc: red and blue particles each render with their respective color dominance
#[test]
fn when_red_and_blue_particles_then_both_colors_visible() {
    let Some(mut capture) = HeadlessCapture::try_new(test_config()) else {
        return;
    };

    capture.spawn_at(&[[-0.2, 0.0], [0.2, 0.0]], &[0, 1]);

    let pixels = capture.render_to_buffer();

    let mut red_pixels = 0usize;
    let mut blue_pixels = 0usize;
    for chunk in pixels.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        if r > g && r > b && r > 40 {
            red_pixels += 1;
        }
        if b > r + 10 && b > g && b > 40 {
            blue_pixels += 1;
        }
    }

    assert!(red_pixels > 3, "expected red-tinted pixels, got {red_pixels}");
    assert!(blue_pixels > 3, "expected blue-tinted pixels, got {blue_pixels}");
}

/// Two consecutive renders of the same state must produce identical pixel buffers.
/// @doc: identical simulation state produces deterministic render output
#[test]
fn when_particles_rendered_twice_then_buffers_identical() {
    let Some(mut capture) = HeadlessCapture::try_new(test_config()) else {
        return;
    };

    capture.spawn_grid(4);

    let pixels_a = capture.render_to_buffer();
    let pixels_b = capture.render_to_buffer();

    assert_eq!(pixels_a.len(), pixels_b.len());
    assert_eq!(pixels_a, pixels_b, "render must be deterministic");
}

/// Running N simulation steps should change particle positions.
/// @doc: particle positions change after simulation steps
#[test]
fn when_particles_simulated_then_positions_change_between_frames() {
    let mut config = test_config();
    config.gravity = -1.2;
    let Some(mut capture) = HeadlessCapture::try_new(config) else {
        return;
    };

    capture.spawn_at(&[[0.0, 0.5]], &[0]);

    let positions_before = capture.read_positions(1);
    capture.step_n(10);
    let positions_after = capture.read_positions(1);

    let dx = positions_after[0][0] - positions_before[0][0];
    let dy = positions_after[0][1] - positions_before[0][1];
    assert!(
        dy < 0.0,
        "particle should fall under gravity: dy={dy:.4}"
    );
    // Horizontal position should be stable (no horizontal forces).
    assert!(
        dx.abs() < 0.001,
        "particle should not drift horizontally: dx={dx:.4}"
    );
}

/// All spawned particles must stay within the wall bounds.
/// @doc: particles remain within simulation bounds after settling
#[test]
fn when_spawned_into_box_then_all_within_bounds() {
    let mut config = test_config();
    config.gravity = -1.2;
    config.particle_radius = 0.02;
    config.num_particles = 20;
    let Some(mut capture) = HeadlessCapture::try_new(config) else {
        return;
    };

    capture.spawn_grid(20);
    // Simulate long enough for particles to fall and settle.
    capture.step_n(120);

    let count = capture.particle_count();
    let positions = capture.read_positions(count);
    let r = 0.02;
    for (i, &[px, py]) in positions.iter().enumerate() {
        assert!(
            px.is_finite() && py.is_finite(),
            "particle {i}: NaN position ({px}, {py})"
        );
        assert!(
            px >= -0.8 - r && px <= 0.8 + r,
            "particle {i}: x={px:.4} out of bounds"
        );
        assert!(
            py >= -0.8 - r && py <= 0.8 + r,
            "particle {i}: y={py:.4} out of bounds"
        );
    }
}
