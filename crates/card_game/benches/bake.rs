#![allow(clippy::unwrap_used)]

use std::time::Duration;

use card_game::card::component::CardLabel;
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::bake::{bake_back_face, bake_front_face};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec2;

fn bench_bake_front_face(c: &mut Criterion) {
    let mut group = c.benchmark_group("card_bake");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    let card_size = Vec2::new(60.0, 90.0);

    group.bench_function("front_face_short_name", |b| {
        let sig = CardSignature::default();
        let label = CardLabel {
            name: "Fire Drake".to_owned(),
            description: "Deals 5 damage".to_owned(),
        };
        b.iter(|| {
            bake_front_face(
                black_box(&sig),
                black_box(card_size),
                black_box(&label),
                None,
            )
        });
    });

    group.bench_function("front_face_long_text", |b| {
        let sig = CardSignature::new([0.8, 0.2, 0.5, 0.9, 0.1, 0.7, 0.3, 0.6]);
        let label = CardLabel {
            name: "Ancient Fire Drake King".to_owned(),
            description: "Deals 5 fire damage to target creature and 2 splash damage to adjacent creatures on the battlefield".to_owned(),
        };
        b.iter(|| {
            bake_front_face(
                black_box(&sig),
                black_box(card_size),
                black_box(&label),
                None,
            )
        });
    });

    group.finish();
}

fn bench_bake_back_face(c: &mut Criterion) {
    let mut group = c.benchmark_group("card_bake");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    let card_size = Vec2::new(60.0, 90.0);

    group.bench_function("back_face", |b| {
        b.iter(|| bake_back_face(black_box(card_size)));
    });

    group.finish();
}

criterion_group!(benches, bench_bake_front_face, bench_bake_back_face,);
criterion_main!(benches);
