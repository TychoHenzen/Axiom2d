#![allow(clippy::unwrap_used)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_render::prelude::*;
use glam::Vec2;

fn bench_tessellate_circle(c: &mut Criterion) {
    let mut group = c.benchmark_group("tessellate_fill");

    group.bench_function("circle_r50", |b| {
        let variant = ShapeVariant::Circle { radius: 50.0 };
        b.iter(|| tessellate(black_box(&variant)).unwrap());
    });

    group.bench_function("circle_r200", |b| {
        let variant = ShapeVariant::Circle { radius: 200.0 };
        b.iter(|| tessellate(black_box(&variant)).unwrap());
    });

    group.finish();
}

fn bench_tessellate_polygon(c: &mut Criterion) {
    let mut group = c.benchmark_group("tessellate_fill");

    // Hexagon (6 vertices) — used for gem sockets on cards
    group.bench_function("hexagon_6v", |b| {
        let points: Vec<Vec2> = (0..6)
            .map(|i| {
                let angle = std::f32::consts::TAU * i as f32 / 6.0;
                Vec2::new(angle.cos() * 10.0, angle.sin() * 10.0)
            })
            .collect();
        let variant = ShapeVariant::Polygon { points };
        b.iter(|| tessellate(black_box(&variant)).unwrap());
    });

    // 32-vertex polygon — moderate complexity
    group.bench_function("polygon_32v", |b| {
        let points: Vec<Vec2> = (0..32)
            .map(|i| {
                let angle = std::f32::consts::TAU * i as f32 / 32.0;
                Vec2::new(angle.cos() * 50.0, angle.sin() * 50.0)
            })
            .collect();
        let variant = ShapeVariant::Polygon { points };
        b.iter(|| tessellate(black_box(&variant)).unwrap());
    });

    group.finish();
}

fn bench_tessellate_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("tessellate_fill");

    // Rounded rect with bezier corners — the card border shape
    group.bench_function("rounded_rect", |b| {
        let variant = rounded_rect_path(30.0, 45.0, 3.0);
        b.iter(|| tessellate(black_box(&variant)).unwrap());
    });

    // Complex path with many cubic beziers (simulating card art)
    group.bench_function("complex_path_20_cubics", |b| {
        let mut commands = Vec::new();
        commands.push(PathCommand::MoveTo(Vec2::ZERO));
        for i in 0..20 {
            let t = i as f32 * 0.5;
            commands.push(PathCommand::CubicTo {
                control1: Vec2::new(t + 1.0, t + 5.0),
                control2: Vec2::new(t + 3.0, t - 2.0),
                to: Vec2::new(t + 5.0, t + 1.0),
            });
        }
        commands.push(PathCommand::Close);
        let variant = ShapeVariant::Path { commands };
        b.iter(|| tessellate(black_box(&variant)).unwrap());
    });

    group.finish();
}

fn bench_tessellate_stroke(c: &mut Criterion) {
    let mut group = c.benchmark_group("tessellate_stroke");

    group.bench_function("circle_r50_w2", |b| {
        let variant = ShapeVariant::Circle { radius: 50.0 };
        b.iter(|| tessellate_stroke(black_box(&variant), black_box(2.0)).unwrap());
    });

    group.bench_function("rounded_rect_w2", |b| {
        let variant = rounded_rect_path(30.0, 45.0, 3.0);
        b.iter(|| tessellate_stroke(black_box(&variant), black_box(2.0)).unwrap());
    });

    group.bench_function("polygon_32v_w1", |b| {
        let points: Vec<Vec2> = (0..32)
            .map(|i| {
                let angle = std::f32::consts::TAU * i as f32 / 32.0;
                Vec2::new(angle.cos() * 50.0, angle.sin() * 50.0)
            })
            .collect();
        let variant = ShapeVariant::Polygon { points };
        b.iter(|| tessellate_stroke(black_box(&variant), black_box(1.0)).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tessellate_circle,
    bench_tessellate_polygon,
    bench_tessellate_path,
    bench_tessellate_stroke,
);
criterion_main!(benches);
