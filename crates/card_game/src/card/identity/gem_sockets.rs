// EVOLVE-BLOCK-START
use std::f32::consts::TAU;

use engine_core::color::Color;
use glam::Vec2;

use crate::card::identity::signature::Aspect;

/// Maps each of the 16 Aspect variants to a distinct color.
/// Positive aspects get warm hues (r > b or g > b), negative aspects get cool hues (b > r).
pub fn aspect_color(aspect: Aspect) -> Color {
    match aspect {
        // Positive (warm) — reds, oranges, yellows, greens
        Aspect::Solid => Color::new(0.85, 0.55, 0.20, 1.0), // amber
        Aspect::Heat => Color::new(0.95, 0.25, 0.10, 1.0),  // red-orange
        Aspect::Order => Color::new(0.90, 0.80, 0.10, 1.0), // gold
        Aspect::Light => Color::new(0.98, 0.95, 0.40, 1.0), // bright yellow
        Aspect::Change => Color::new(0.70, 0.85, 0.10, 1.0), // yellow-green
        Aspect::Force => Color::new(0.90, 0.40, 0.05, 1.0), // deep orange
        Aspect::Growth => Color::new(0.20, 0.80, 0.20, 1.0), // green
        Aspect::Expansion => Color::new(0.60, 0.90, 0.30, 1.0), // lime

        // Negative (cool) — blues, purples, teals, indigos
        Aspect::Fragile => Color::new(0.30, 0.50, 0.85, 1.0), // periwinkle
        Aspect::Cold => Color::new(0.10, 0.70, 0.95, 1.0),    // ice blue
        Aspect::Chaos => Color::new(0.55, 0.10, 0.80, 1.0),   // violet
        Aspect::Dark => Color::new(0.15, 0.05, 0.40, 1.0),    // deep indigo
        Aspect::Stasis => Color::new(0.20, 0.60, 0.80, 1.0),  // steel blue
        Aspect::Calm => Color::new(0.10, 0.75, 0.70, 1.0),    // teal
        Aspect::Decay => Color::new(0.35, 0.20, 0.60, 1.0),   // muted purple
        Aspect::Contraction => Color::new(0.05, 0.20, 0.70, 1.0), // navy
    }
}

/// Returns the 8 gem socket positions around the card border perimeter.
/// Positions are relative to card center (child-space offsets).
/// Layout: 2 on each of the 4 edges, evenly spaced at 1/3 and 2/3 along each edge.
pub fn gem_border_positions(card_size: Vec2) -> [Vec2; 8] {
    let hw = card_size.x * 0.5;
    let hh = card_size.y * 0.5;
    let third_w = card_size.x / 3.0;
    let third_h = card_size.y / 3.0;

    [
        // Top edge: left-of-center, right-of-center
        Vec2::new(-third_w * 0.5, hh),
        Vec2::new(third_w * 0.5, hh),
        // Bottom edge
        Vec2::new(-third_w * 0.5, -hh),
        Vec2::new(third_w * 0.5, -hh),
        // Left edge: upper-third, lower-third
        Vec2::new(-hw, third_h * 0.5),
        Vec2::new(-hw, -third_h * 0.5),
        // Right edge
        Vec2::new(hw, third_h * 0.5),
        Vec2::new(hw, -third_h * 0.5),
    ]
}

/// Returns the 8 gem socket positions flanking the description strip.
/// Layout: 4 gems in a vertical column on each side, in the border margin
/// between the description strip outer edge and the card edge.
/// Positions are relative to card center (child-space offsets).
pub fn gem_desc_positions(card_size: Vec2) -> [Vec2; 8] {
    use crate::card::rendering::face_layout::FRONT_FACE_REGIONS;

    let card_half_w = card_size.x * 0.5;
    let (desc_half_w, desc_half_h, desc_offset_y) =
        FRONT_FACE_REGIONS[3].resolve(card_size.x, card_size.y);

    // Gem columns sit in the margin between desc strip edge and card edge,
    // centered so they don't exceed card bounds.
    let outer_limit = card_half_w - MAX_GEM_RADIUS;
    let col_x = (desc_half_w + outer_limit) * 0.5;

    let step = (desc_half_h * 2.0) / (GEM_DESC_PER_COL as f32 + 1.0);

    let mut positions = [Vec2::ZERO; 8];
    for i in 0..GEM_DESC_PER_COL {
        let y = desc_offset_y - desc_half_h + step * (i as f32 + 1.0);
        positions[i] = Vec2::new(-col_x, y);
        positions[i + GEM_DESC_PER_COL] = Vec2::new(col_x, y);
    }
    positions
}

pub const MIN_GEM_RADIUS: f32 = 1.0;
pub const MAX_GEM_RADIUS: f32 = 3.5;
/// Number of gem sockets per column flanking the description strip (4 left + 4 right = 8 total).
pub const GEM_DESC_PER_COL: usize = 4;

/// Maps element intensity (0.0..=1.0) to gem circle radius.
pub fn gem_radius(intensity: f32) -> f32 {
    MIN_GEM_RADIUS + intensity * (MAX_GEM_RADIUS - MIN_GEM_RADIUS)
}

/// Returns the 6 vertices of a regular hexagon centered at the origin.
/// Vertices are ordered counter-clockwise starting at angle 0 (positive x-axis).
pub fn hexagon_vertices(radius: f32) -> [Vec2; 6] {
    core::array::from_fn(|i| {
        let angle = TAU * i as f32 / 6.0;
        Vec2::new(radius * angle.cos(), radius * angle.sin())
    })
}

/// Maps hexagon vertices to normalized [0, 1]² UV coordinates via AABB normalization.
/// The shader uses these UVs to determine per-facet surface normals.
pub fn hexagon_uvs(vertices: &[Vec2; 6]) -> [[f32; 2]; 6] {
    let min_x = vertices.iter().map(|v| v.x).fold(f32::INFINITY, f32::min);
    let max_x = vertices
        .iter()
        .map(|v| v.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = vertices.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
    let max_y = vertices
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;
    core::array::from_fn(|i| {
        [
            (vertices[i].x - min_x) / range_x,
            (vertices[i].y - min_y) / range_y,
        ]
    })
}

/// Neutral gray used for zero-intensity gems (faded/dormant).
const NEUTRAL_GRAY: Color = Color {
    r: 0.45,
    g: 0.45,
    b: 0.45,
    a: 1.0,
};

/// Maps an aspect and intensity to a gradually-blended gem color.
/// At intensity 0.0 the gem is neutral gray; at 1.0 it is the full aspect color.
/// Because positive and negative aspects both converge to the same gray at
/// low intensities, the hue transition across the sign boundary is smooth.
pub fn gem_color(aspect: Aspect, intensity: f32) -> Color {
    let full = aspect_color(aspect);
    let t = intensity.clamp(0.0, 1.0);
    Color::new(
        NEUTRAL_GRAY.r + (full.r - NEUTRAL_GRAY.r) * t,
        NEUTRAL_GRAY.g + (full.g - NEUTRAL_GRAY.g) * t,
        NEUTRAL_GRAY.b + (full.b - NEUTRAL_GRAY.b) * t,
        1.0,
    )
}

/// Maps element intensity (0.0..=1.0) to a continuous specular multiplier.
/// Ramps linearly from a dim frosted look (0.15) to full brilliance (1.0).
pub fn gem_specular_intensity(intensity: f32) -> f32 {
    let t = intensity.clamp(0.0, 1.0);
    0.15 + t * 0.85
}
// EVOLVE-BLOCK-END
