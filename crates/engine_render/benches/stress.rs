#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_render::font::bake_text_into_mesh;
use engine_render::prelude::*;
use glam::Vec2;

/// 3000 complex bezier paths (100 cubics each) — heavy lyon tessellation workload.
fn bench_tessellation_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_tessellation");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("3000_bezier_paths_100_cubics", |b| {
        let paths: Vec<ShapeVariant> = (0..3000)
            .map(|seed| {
                let s = seed as f32;
                let mut commands = Vec::with_capacity(102);
                commands.push(PathCommand::MoveTo(Vec2::new(s % 10.0, s % 7.0)));
                for j in 0..100 {
                    let t = j as f32 + s * 0.1;
                    commands.push(PathCommand::CubicTo {
                        control1: Vec2::new(t + 1.0, t + 5.0 + s * 0.01),
                        control2: Vec2::new(t + 3.0, t - 2.0 + s * 0.02),
                        to: Vec2::new(t + 5.0, t + 1.0),
                    });
                }
                commands.push(PathCommand::Close);
                ShapeVariant::Path { commands }
            })
            .collect();

        b.iter(|| {
            for path in &paths {
                let _ = tessellate(black_box(path)).unwrap();
            }
        });
    });

    group.finish();
}

/// Bulk text baking — 5000 paragraphs through the full glyph tessellation pipeline.
fn bench_font_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_font");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("bake_5000_paragraphs", |b| {
        let paragraphs: Vec<String> = (0..5000)
            .map(|i| {
                format!(
                    "Card #{i}: Deals {att} fire damage to target creature and {spl} splash damage \
                     to all adjacent creatures on the battlefield when played from hand",
                    att = 3 + i % 10,
                    spl = 1 + i % 5,
                )
            })
            .collect();

        b.iter(|| {
            for text in &paragraphs {
                let mut mesh = TessellatedColorMesh::new();
                bake_text_into_mesh(
                    black_box(&mut mesh),
                    black_box(text),
                    black_box(5.0),
                    black_box([1.0, 1.0, 1.0, 1.0]),
                    black_box(0.0),
                    black_box(0.0),
                );
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_tessellation_stress, bench_font_stress,);
criterion_main!(benches);
