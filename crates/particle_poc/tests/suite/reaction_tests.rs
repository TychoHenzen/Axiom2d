// Reaction tests — particle species transmutation via contact.
// All tests use HeadlessCapture::try_new() and skip gracefully when no GPU is available.
//
// The reaction shader checks `dist < 2*radius` (strict). PBD with SOR ω=1.2
// over-corrects past contact distance for isolated pairs. Reactions need gravity
// compression from a tall column — sustained pressure creates transient overlaps.
// Without pressure, PBD equilibrium leaves particles at >2r apart and reactions
// never fire. This is an upstream simulation design issue, not a test framework bug.

use particle_poc::capture::{CaptureConfig, HeadlessCapture};

fn reaction_test_config() -> CaptureConfig {
    CaptureConfig {
        width: 64,
        height: 64,
        particle_radius: 0.002,
        gravity: -1.2,
        sub_steps: 16,
        num_particles: 100,
        ..Default::default()
    }
}

/// Red(0) + Blue(1) in contact should both become Green(2).
/// Tall column of mixed species under gravity provides sustained pressure.
/// @doc: red and blue particles react on contact to become green
#[test]
fn when_red_contacts_blue_then_both_become_green() {
    let Some(mut capture) = HeadlessCapture::try_new(reaction_test_config()) else {
        return;
    };

    // Tall, narrow column of 80 particles — alternating Red/Blue.
    // Gravity compresses the column, sustaining overlaps for reaction.
    let count = 80u32;
    let r = 0.002;
    let spacing = 1.8 * r; // tighter than contact distance for compression
    let cols = 4u32;
    let mut positions = Vec::new();
    let mut species = Vec::new();
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        positions.push([
            -spacing * (cols as f32 - 1.0) * 0.5 + col as f32 * spacing,
            0.7 - row as f32 * spacing,
        ]);
        species.push(i & 1); // alternating 0(Red), 1(Blue)
    }
    capture.spawn_at(&positions, &species);

    // Let column settle under gravity — reactions fire during the transient.
    capture.step_n(200);

    let final_species = capture.read_species(count);
    let green_count = final_species.iter().filter(|&&s| s == 2).count();
    assert!(
        green_count >= 2,
        "expected at least 2 particles to react to Green(2), got {green_count} out of {count}",
    );
}

/// Green(2) + Purple(4) in contact should turn Green into Yellow(3).
/// @doc: green particle contacting purple becomes yellow
#[test]
fn when_green_contacts_purple_then_green_becomes_yellow() {
    let Some(mut capture) = HeadlessCapture::try_new(reaction_test_config()) else {
        return;
    };

    // Tall, narrow column of Green(2) / Purple(4).
    let count = 80u32;
    let r = 0.002;
    let spacing = 1.8 * r;
    let cols = 4u32;
    let mut positions = Vec::new();
    let mut species = Vec::new();
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        positions.push([
            -spacing * (cols as f32 - 1.0) * 0.5 + col as f32 * spacing,
            0.7 - row as f32 * spacing,
        ]);
        species.push(if i & 1 == 0 { 2 } else { 4 }); // Green(2), Purple(4)
    }
    capture.spawn_at(&positions, &species);

    capture.step_n(200);

    let final_species = capture.read_species(count);
    let yellow_count = final_species.iter().filter(|&&s| s == 3).count();
    assert!(
        yellow_count >= 1,
        "expected at least 1 Green→Yellow(3) conversion, got {yellow_count} out of {count}",
    );
}
