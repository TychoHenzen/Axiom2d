#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_core::prelude::Seconds;
use engine_physics::collider::Collider;
use engine_physics::physics_backend::PhysicsBackend;
use engine_physics::rapier_backend::RapierBackend;
use engine_physics::rigid_body::RigidBody;
use glam::Vec2;

fn make_entity_id(index: u32) -> bevy_ecs::entity::Entity {
    let mut world = bevy_ecs::world::World::new();
    let mut entities = Vec::new();
    for _ in 0..=index {
        entities.push(world.spawn_empty().id());
    }
    entities[index as usize]
}

fn setup_backend(body_count: u32) -> RapierBackend {
    let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));
    let mut world = bevy_ecs::world::World::new();
    for i in 0..body_count {
        let entity = world.spawn_empty().id();
        // Spread bodies in a grid to create realistic collision scenarios
        let row = i / 10;
        let col = i % 10;
        let x = col as f32 * 2.0;
        let y = row as f32 * 2.0 + 10.0;
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(x, y));
        backend.add_collider(entity, &Collider::Circle(0.5));
    }
    backend
}

fn bench_physics_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("rapier_step");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    for count in [10, 50, 100, 200] {
        group.bench_function(format!("{count}_bodies"), |b| {
            let mut backend = setup_backend(count);
            b.iter(|| {
                backend.step(black_box(Seconds(1.0 / 60.0)));
            });
        });
    }

    group.finish();
}

fn bench_physics_add_body(c: &mut Criterion) {
    let mut group = c.benchmark_group("rapier_add_body");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    group.bench_function("add_single_body", |b| {
        let entity = make_entity_id(0);
        b.iter(|| {
            let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));
            backend.add_body(
                black_box(entity),
                black_box(&RigidBody::Dynamic),
                black_box(Vec2::ZERO),
            );
            backend.add_collider(black_box(entity), black_box(&Collider::Circle(0.5)));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_physics_step, bench_physics_add_body,);
criterion_main!(benches);
