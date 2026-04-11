#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_core::spring::spring_step;

/// 500 cards × 3 axes × 6000 frames (~100s of game time at 60fps).
/// 9,000,000 spring evaluations per iteration.
fn bench_spring_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_spring");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("500_cards_6000_frames", |b| {
        let dt = 1.0 / 60.0_f32;
        let stiffness = 300.0_f32;
        let damping = 15.0_f32;

        b.iter(|| {
            for card in 0..500 {
                let offset = card as f32 * 5.0;
                let mut x_pos = offset;
                let mut x_vel = 0.0_f32;
                let mut y_pos = -50.0_f32;
                let mut y_vel = 0.0_f32;
                let mut r_pos = 0.1 * offset;
                let mut r_vel = 0.0_f32;
                let x_target = offset + 5.0;
                let y_target = -45.0_f32;

                for _ in 0..6000 {
                    let (nx, nv) = spring_step(
                        black_box(x_pos),
                        black_box(x_target),
                        black_box(x_vel),
                        black_box(dt),
                        black_box(stiffness),
                        black_box(damping),
                    );
                    x_pos = nx;
                    x_vel = nv;

                    let (ny, nyv) = spring_step(
                        black_box(y_pos),
                        black_box(y_target),
                        black_box(y_vel),
                        black_box(dt),
                        black_box(stiffness),
                        black_box(damping),
                    );
                    y_pos = ny;
                    y_vel = nyv;

                    let (nr, nrv) = spring_step(
                        black_box(r_pos),
                        black_box(0.0),
                        black_box(r_vel),
                        black_box(dt),
                        black_box(stiffness),
                        black_box(damping),
                    );
                    r_pos = nr;
                    r_vel = nrv;
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_spring_stress,);
criterion_main!(benches);
