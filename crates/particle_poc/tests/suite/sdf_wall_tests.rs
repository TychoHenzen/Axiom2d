use particle_poc::capture::{CaptureConfig, HeadlessCapture};
use particle_poc::compute_distance_field;

#[test]
fn distance_field_smoke_test() {
    let res = 32u32;
    let mut grid = vec![1.0f32; (res * res) as usize];
    // Paint a horizontal floor: row 10-12 all wall.
    for y in 10..=12 {
        for x in 0..res {
            grid[(y * res + x) as usize] = -1.0;
        }
    }
    compute_distance_field(&mut grid, res);
    // Cell inside wall (row 11, far from boundary): should be negative
    let inside = grid[(11 * res + 16) as usize];
    assert!(inside < 0.0, "inside wall should be negative, got {inside}");
    // Cell just above wall (row 13): should be small positive
    let above = grid[(13 * res + 16) as usize];
    assert!(
        above > 0.0 && above < 3.0,
        "just above wall should be small positive, got {above}"
    );
    // Cell far above wall (row 25): should be large positive
    let far = grid[(25 * res + 16) as usize];
    assert!(
        far > 5.0,
        "far above wall should be large positive, got {far}"
    );
    // Gradient at row 14 should point upward (increasing y = away from wall)
    let below_14 = grid[(13 * res + 16) as usize];
    let above_14 = grid[(15 * res + 16) as usize];
    assert!(
        above_14 > below_14,
        "gradient should point up: above={above_14} below={below_14}"
    );
    eprintln!("EDT values: inside={inside:.2} above={above:.2} far={far:.2}");
}

fn bowl_config() -> CaptureConfig {
    CaptureConfig {
        width: 256,
        height: 256,
        particle_radius: 0.01,
        gravity: -0.3,
        sub_steps: 16,
        num_particles: 0,
        ..Default::default()
    }
}

fn make_sdf_capture() -> Option<HeadlessCapture> {
    let mut c = HeadlessCapture::try_new(bowl_config())?;
    c.disable_machines();
    Some(c)
}

fn paint_floor_with_rims(c: &mut HeadlessCapture, floor_y: f32, rim_top: f32) {
    let br: f32 = 0.10;
    let s: f32 = 0.008;
    let paint_y = floor_y - br;
    let left_x: f32 = -0.55;
    let right_x: f32 = -0.05;
    // Floor slab.
    let mut y: f32 = paint_y - br * 0.3;
    while y <= paint_y + br * 0.3 {
        let mut x: f32 = left_x - 0.02;
        while x <= right_x + 0.02 {
            c.paint_sdf([x, y], br, false);
            x += s;
        }
        y += s;
    }
    // Left rim.
    let mut x: f32 = left_x - 0.02;
    while x <= left_x + 0.04 {
        let mut y2: f32 = paint_y - br * 0.3;
        while y2 <= rim_top {
            c.paint_sdf([x, y2], br * 0.8, false);
            y2 += s;
        }
        x += s;
    }
    // Right rim.
    x = right_x - 0.04;
    while x <= right_x + 0.02 {
        let mut y2: f32 = paint_y - br * 0.3;
        while y2 <= rim_top {
            c.paint_sdf([x, y2], br * 0.8, false);
            y2 += s;
        }
        x += s;
    }
    c.upload_sdf();
}

#[test]
fn when_sdf_painted_then_gpu_readback_matches() {
    let Some(mut c) = make_sdf_capture() else {
        return;
    };
    c.paint_sdf([-0.30, -0.70], 0.10, false);
    c.upload_sdf();
    assert!(c.read_sdf_at([-0.30, -0.70]) < -0.5);
}

#[test]
fn when_single_particle_dropped_on_sdf_floor_then_stops() {
    let Some(mut c) = make_sdf_capture() else {
        return;
    };
    paint_floor_with_rims(&mut c, -0.50, -0.20);

    // Verify SDF values along the vertical column at x=-0.30
    for y_i in [
        -0.35f32, -0.40, -0.45, -0.47, -0.48, -0.50, -0.55, -0.60, -0.70,
    ] {
        let v = c.read_sdf_at([-0.30, y_i]);
        eprintln!("SDF at y={y_i:.2}: {v:.4}");
    }

    c.spawn_at(&[[-0.30, -0.40]], &[0]);
    // Run frames 0-29 without granular logging.
    c.step_n(30);
    let p30 = c.read_positions(1);
    let v30 = c.read_velocities(1);
    eprintln!("f 30: y={:.6} vy={:.6}", p30[0][1], v30[0][1]);
    // Run frames 30-37 with per-frame logging to find the ejection moment.
    for frame in 31..=40 {
        c.step();
        let p = c.read_positions(1);
        let v = c.read_velocities(1);
        eprintln!("f{frame:>3}: y={:.6} vy={:.6}", p[0][1], v[0][1]);
    }
    let p = c.read_positions(1);
    eprintln!("final y={:.4}", p[0][1]);
    assert!(p[0][1] > -0.52 && p[0][1] < -0.38, "y={:.4}", p[0][1]);
}

#[test]
fn when_10_particles_dropped_on_sdf_floor_then_all_stop() {
    let Some(mut c) = make_sdf_capture() else {
        return;
    };
    paint_floor_with_rims(&mut c, -0.50, -0.20);
    let n = 10u32;
    let mut pos = Vec::new();
    let mut sp = Vec::new();
    for i in 0..n {
        pos.push([
            -0.30 + 0.02 * (i as f32).cos(),
            -0.38 + 0.015 * (i as f32).sin(),
        ]);
        sp.push(0);
    }
    c.spawn_at(&pos, &sp);
    c.step_n(120);
    let p = c.read_positions(n);
    let through = p.iter().filter(|pt| pt[1] < -0.55).count();
    assert_eq!(through, 0, "{through}/{n} through floor");
}

#[test]
fn when_30_particles_dropped_on_wide_sdf_floor_then_none_phase_through() {
    let Some(mut c) = make_sdf_capture() else {
        return;
    };
    let br: f32 = 0.06;
    let s: f32 = 0.005;
    // Full-width floor so horizontally ejected particles can't fall off the edge.
    let mut y: f32 = -0.62;
    while y <= -0.58 {
        let mut x: f32 = -0.95;
        while x <= 0.95 {
            c.paint_sdf([x, y], br, false);
            x += s;
        }
        y += s;
    }
    c.upload_sdf();

    let n = 30u32;
    let mut pos = Vec::with_capacity(n as usize);
    let mut sp = Vec::with_capacity(n as usize);
    for i in 0..n {
        let a = (i as f32 / n as f32) * std::f32::consts::TAU;
        pos.push([-0.30 + 0.04 * a.cos(), -0.48 + 0.02 * a.sin()]);
        sp.push(1);
    }
    c.spawn_at(&pos, &sp);
    c.step_n(300);
    let p = c.read_positions(n);
    assert_eq!(p.len(), n as usize);
    // Wall zone extends ~brush above painted area = y∈[-0.64, -0.52].
    // Particles should rest above y=-0.55.
    let through = p.iter().filter(|pt| pt[1] < -0.55).count();
    assert_eq!(through, 0, "{through}/{n} through floor");
    let pb = c.read_positions(n);
    c.step_n(1);
    let pa = c.read_positions(n);
    let md = pa
        .iter()
        .zip(pb.iter())
        .map(|(a, b)| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt())
        .fold(0.0_f32, f32::max);
    assert!(md < 0.05, "not settled: {md:.6}");
}
