#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_core::spring::spring_step;

fn bench_spring_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("spring");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    group.bench_function("single_step", |b| {
        b.iter(|| {
            spring_step(
                black_box(0.0),
                black_box(100.0),
                black_box(0.0),
                black_box(1.0 / 60.0),
                black_box(300.0),
                black_box(15.0),
            )
        });
    });

    // Simulate hand layout: 10 cards × 3 axes (x, y, rotation)
    group.bench_function("batch_10_cards_3_axes", |b| {
        let dt = 1.0 / 60.0_f32;
        let stiffness = 300.0_f32;
        let damping = 15.0_f32;
        b.iter(|| {
            for i in 0..10 {
                let offset = i as f32 * 20.0;
                // x axis
                spring_step(
                    black_box(offset),
                    black_box(offset + 5.0),
                    black_box(1.0),
                    black_box(dt),
                    black_box(stiffness),
                    black_box(damping),
                );
                // y axis
                spring_step(
                    black_box(-50.0),
                    black_box(-45.0),
                    black_box(0.5),
                    black_box(dt),
                    black_box(stiffness),
                    black_box(damping),
                );
                // rotation
                spring_step(
                    black_box(0.1 * offset),
                    black_box(0.0),
                    black_box(0.2),
                    black_box(dt),
                    black_box(stiffness),
                    black_box(damping),
                );
            }
        });
    });

    // 50 cards stress test
    group.bench_function("batch_50_cards_3_axes", |b| {
        let dt = 1.0 / 60.0_f32;
        let stiffness = 300.0_f32;
        let damping = 15.0_f32;
        b.iter(|| {
            for i in 0..50 {
                let offset = i as f32 * 5.0;
                spring_step(
                    black_box(offset),
                    black_box(offset + 5.0),
                    black_box(1.0),
                    black_box(dt),
                    black_box(stiffness),
                    black_box(damping),
                );
                spring_step(
                    black_box(-50.0),
                    black_box(-45.0),
                    black_box(0.5),
                    black_box(dt),
                    black_box(stiffness),
                    black_box(damping),
                );
                spring_step(
                    black_box(0.1 * offset),
                    black_box(0.0),
                    black_box(0.2),
                    black_box(dt),
                    black_box(stiffness),
                    black_box(damping),
                );
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_spring_step,);
criterion_main!(benches);
