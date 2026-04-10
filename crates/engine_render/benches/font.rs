#![allow(clippy::unwrap_used)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine_render::font::{
    GlyphCache, bake_text_into_mesh, balanced_wrap_text, measure_text, wrap_text,
};
use engine_render::shape::TessellatedColorMesh;

fn bench_measure_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_measure");

    group.bench_function("short_5_chars", |b| {
        b.iter(|| measure_text(black_box("Hello"), black_box(14.0)));
    });

    group.bench_function("medium_20_chars", |b| {
        b.iter(|| measure_text(black_box("A quick brown fox ju"), black_box(14.0)));
    });

    group.bench_function("long_50_chars", |b| {
        let text = "The quick brown fox jumps over the lazy dog abcdef";
        b.iter(|| measure_text(black_box(text), black_box(14.0)));
    });

    group.finish();
}

fn bench_wrap_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_wrap");

    group.bench_function("short_no_wrap", |b| {
        b.iter(|| wrap_text(black_box("Fire"), black_box(8.0), black_box(200.0)));
    });

    // Typical card description length
    group.bench_function("card_desc_15_words", |b| {
        let text =
            "Deals 5 fire damage to target creature and 2 splash damage to adjacent creatures";
        b.iter(|| wrap_text(black_box(text), black_box(4.5), black_box(50.0)));
    });

    group.finish();
}

fn bench_balanced_wrap(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_balanced_wrap");

    group.bench_function("card_name_2_words", |b| {
        b.iter(|| balanced_wrap_text(black_box("Fire Drake"), black_box(7.0), black_box(50.0)));
    });

    group.bench_function("card_name_4_words", |b| {
        b.iter(|| {
            balanced_wrap_text(
                black_box("Ancient Fire Drake King"),
                black_box(7.0),
                black_box(50.0),
            )
        });
    });

    group.finish();
}

fn bench_bake_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_bake");

    group.bench_function("short_name", |b| {
        b.iter(|| {
            let mut mesh = TessellatedColorMesh::new();
            bake_text_into_mesh(
                black_box(&mut mesh),
                black_box("Fire Drake"),
                black_box(7.0),
                black_box([1.0, 1.0, 1.0, 1.0]),
                black_box(0.0),
                black_box(0.0),
            );
            mesh
        });
    });

    group.bench_function("long_description", |b| {
        b.iter(|| {
            let mut mesh = TessellatedColorMesh::new();
            bake_text_into_mesh(
                black_box(&mut mesh),
                black_box("Deals 5 fire damage to target"),
                black_box(4.5),
                black_box([1.0, 1.0, 1.0, 1.0]),
                black_box(0.0),
                black_box(0.0),
            );
            mesh
        });
    });

    group.finish();
}

fn bench_glyph_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_glyph_cache");

    group.bench_function("cold_cache_10_glyphs", |b| {
        let face = ttf_parser::Face::parse(engine_render::font::FONT_BYTES, 0).expect("valid font");
        b.iter(|| {
            let mut cache = GlyphCache::new();
            for ch in "ABCDEFGHIJ".chars() {
                if let Some(glyph_id) = face.glyph_index(ch) {
                    cache.get_or_tessellate(&face, glyph_id, 14.0);
                }
            }
        });
    });

    group.bench_function("warm_cache_10_glyphs", |b| {
        let face = ttf_parser::Face::parse(engine_render::font::FONT_BYTES, 0).expect("valid font");
        let mut cache = GlyphCache::new();
        // Pre-warm
        for ch in "ABCDEFGHIJ".chars() {
            if let Some(glyph_id) = face.glyph_index(ch) {
                cache.get_or_tessellate(&face, glyph_id, 14.0);
            }
        }
        b.iter(|| {
            for ch in "ABCDEFGHIJ".chars() {
                if let Some(glyph_id) = face.glyph_index(ch) {
                    cache.get_or_tessellate(&face, glyph_id, 14.0);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_measure_text,
    bench_wrap_text,
    bench_balanced_wrap,
    bench_bake_text,
    bench_glyph_cache,
);
criterion_main!(benches);
