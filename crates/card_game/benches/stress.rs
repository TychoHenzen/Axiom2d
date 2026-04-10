#![allow(clippy::unwrap_used)]

use std::time::Duration;

use card_game::card::component::CardLabel;
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::bake::{bake_back_face, bake_front_face};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec2;

/// Bake 400 unique cards (front + back) — full text layout + tessellation pipeline.
fn bench_bake_deck_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_card_bake");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    let card_size = Vec2::new(60.0, 90.0);

    let cards: Vec<(CardSignature, CardLabel)> = (0..400)
        .map(|i| {
            let sig = CardSignature::new([
                (i as f32 * 0.037) % 1.0,
                (i as f32 * 0.071) % 1.0,
                (i as f32 * 0.113) % 1.0,
                (i as f32 * 0.157) % 1.0,
                (i as f32 * 0.199) % 1.0,
                (i as f32 * 0.241) % 1.0,
                (i as f32 * 0.283) % 1.0,
                (i as f32 * 0.331) % 1.0,
            ]);
            let label = CardLabel {
                name: format!("Ancient Dragon Warrior #{i}"),
                description: format!(
                    "Deals {att} fire damage to target creature and {spl} splash damage \
                     to all adjacent creatures on the battlefield when played from hand",
                    att = 3 + i % 10,
                    spl = 1 + i % 5,
                ),
            };
            (sig, label)
        })
        .collect();

    group.bench_function("400_cards_front_and_back", |b| {
        b.iter(|| {
            for (sig, label) in &cards {
                bake_front_face(black_box(sig), black_box(card_size), black_box(label), None);
                bake_back_face(black_box(card_size));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_bake_deck_stress,);
criterion_main!(benches);
