#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_core::spring::spring_step;

/// 3000 cards × 3 axes × 36000 frames (10 minutes of game time at 60fps).
/// 324,000,000 spring evaluations per iteration.
fn bench_spring_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_spring");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("3000_cards_36000_frames", |b| {
        let dt = 1.0 / 60.0_f32;
        let stiffness = 300.0_f32;
        let damping = 15.0_f32;

        b.iter(|| {
            for card in 0..3000 {
                let offset = card as f32 * 5.0;
                let mut x_pos = offset;
                let mut x_vel = 0.0_f32;
                let mut y_pos = -50.0_f32;
                let mut y_vel = 0.0_f32;
                let mut r_pos = 0.1 * offset;
                let mut r_vel = 0.0_f32;
                let x_target = offset + 5.0;
                let y_target = -45.0_f32;

                for _ in 0..36000 {
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
