#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_core::prelude::Seconds;
use engine_physics::collider::Collider;
use engine_physics::physics_backend::PhysicsBackend;
use engine_physics::rapier_backend::RapierBackend;
use engine_physics::rigid_body::RigidBody;
use glam::Vec2;

fn setup_backend(body_count: u32) -> RapierBackend {
    let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));
    let mut world = bevy_ecs::world::World::new();
    for i in 0..body_count {
        let entity = world.spawn_empty().id();
        let row = i / 50;
        let col = i % 50;
        let x = col as f32 * 1.5;
        let y = row as f32 * 1.5 + 10.0;
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(x, y));
        backend.add_collider(entity, &Collider::Circle(0.5));
    }
    backend
}

/// 5000 bodies stepping for 2000 frames — sustained broadphase + solver pressure.
fn bench_physics_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_physics");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("5000_bodies_2000_steps", |b| {
        b.iter_batched(
            || setup_backend(5000),
            |mut backend| {
                let dt = Seconds(1.0 / 60.0);
                for _ in 0..2000 {
                    backend.step(black_box(dt));
                }
            },
            criterion::BatchSize::PerIteration,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_physics_stress,);
criterion_main!(benches);
