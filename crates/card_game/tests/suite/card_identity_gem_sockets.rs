#![allow(clippy::unwrap_used)]

use card_game::card::identity::gem_sockets::*;
use card_game::card::identity::signature::Aspect;
use engine_core::color::Color;
use engine_render::shape::{ShapeVariant, tessellate};
use glam::Vec2;

// --- aspect_color tests ---

#[test]
fn when_aspect_color_called_with_positive_aspect_then_returns_warm_hue() {
    // Arrange / Act / Assert — positive aspects should have r > b or g > b
    for aspect in [
        Aspect::Solid,
        Aspect::Heat,
        Aspect::Order,
        Aspect::Light,
        Aspect::Change,
        Aspect::Force,
        Aspect::Growth,
        Aspect::Expansion,
    ] {
        let color = aspect_color(aspect);
        assert!(
            color.r > color.b || color.g > color.b,
            "{aspect:?} should be warm (r={} g={} > b={})",
            color.r,
            color.g,
            color.b
        );
    }
}

#[test]
fn when_aspect_color_called_with_negative_aspect_then_returns_cool_hue() {
    // Arrange / Act / Assert — negative aspects should have b > r
    for aspect in [
        Aspect::Fragile,
        Aspect::Cold,
        Aspect::Chaos,
        Aspect::Dark,
        Aspect::Stasis,
        Aspect::Calm,
        Aspect::Decay,
        Aspect::Contraction,
    ] {
        let color = aspect_color(aspect);
        assert!(
            color.b > color.r,
            "{aspect:?} should be cool (b={} > r={})",
            color.b,
            color.r
        );
    }
}

// --- gem_border_positions tests ---

/// @doc: All 8 gem border positions must lie within card bounds plus `MAX_GEM_RADIUS` margin.
/// Out-of-bounds gems would render off-card, corrupting the visual frame and player experience.
#[test]
fn when_gem_positions_computed_then_all_within_card_border_band() {
    // Arrange
    let card_size = Vec2::new(60.0, 90.0);
    let half_w = card_size.x * 0.5;
    let half_h = card_size.y * 0.5;

    // Act
    let positions = gem_border_positions(card_size);

    // Assert — each gem must be within card bounds + margin
    let margin = MAX_GEM_RADIUS;
    for (i, pos) in positions.iter().enumerate() {
        assert!(
            pos.x.abs() <= half_w + margin && pos.y.abs() <= half_h + margin,
            "gem {i} at ({}, {}) is outside card bounds",
            pos.x,
            pos.y
        );
    }
}

// --- gem_desc_positions tests ---

/// @doc: Description-flanking gems split evenly: 4 on left, 4 on right (symmetric layout).
/// Asymmetric distribution would visually unbalance the card or collide with art.
#[test]
fn when_gem_desc_positions_called_then_four_left_four_right_of_center() {
    // Arrange
    let card_size = Vec2::new(60.0, 90.0);

    // Act
    let positions = gem_desc_positions(card_size);

    // Assert
    let left_count = positions.iter().filter(|p| p.x < 0.0).count();
    let right_count = positions.iter().filter(|p| p.x > 0.0).count();
    assert_eq!(left_count, 4, "should have 4 gems on the left");
    assert_eq!(right_count, 4, "should have 4 gems on the right");
}

#[test]
fn when_gem_desc_positions_called_then_all_within_card_bounds() {
    // Arrange
    let card_size = Vec2::new(60.0, 90.0);
    let half_w = card_size.x * 0.5;
    let half_h = card_size.y * 0.5;

    // Act
    let positions = gem_desc_positions(card_size);

    // Assert
    for (i, pos) in positions.iter().enumerate() {
        assert!(
            pos.x.abs() <= half_w && pos.y.abs() <= half_h,
            "gem {i} at ({}, {}) is outside card bounds",
            pos.x,
            pos.y
        );
    }
}

/// @doc: Left and right gem columns must be vertically aligned and horizontally mirrored.
/// Asymmetry would look unbalanced and suggest intentional layout bugs to players.
#[test]
fn when_gem_desc_positions_called_then_columns_are_symmetric() {
    // Arrange
    let card_size = Vec2::new(60.0, 90.0);

    // Act
    let positions = gem_desc_positions(card_size);
    let mut left: Vec<Vec2> = positions.iter().filter(|p| p.x < 0.0).copied().collect();
    let mut right: Vec<Vec2> = positions.iter().filter(|p| p.x > 0.0).copied().collect();
    left.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    right.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

    // Assert — each left/right pair should have same y and mirrored x
    for (l, r) in left.iter().zip(right.iter()) {
        assert!(
            (l.y - r.y).abs() < 1e-4,
            "y mismatch: left={}, right={}",
            l.y,
            r.y
        );
        assert!(
            (l.x + r.x).abs() < 1e-4,
            "x not mirrored: left={}, right={}",
            l.x,
            r.x
        );
    }
}

/// @doc: All gems must fit vertically within the description strip bounds.
/// Gems outside this range would either overlap the card art above/below or look misplaced.
#[test]
fn when_gem_desc_positions_called_then_y_positions_span_desc_strip() {
    // Arrange
    let card_size = Vec2::new(60.0, 90.0);
    // Description strip: offset_y = 0.28, half_h_frac = 1.0 / 6.0
    let desc_half_h = card_size.y * (1.0 / 6.0);
    let desc_offset_y = card_size.y * 0.28;
    let desc_top = desc_offset_y - desc_half_h;
    let desc_bottom = desc_offset_y + desc_half_h;

    // Act
    let positions = gem_desc_positions(card_size);

    // Assert
    for (i, pos) in positions.iter().enumerate() {
        assert!(
            pos.y >= desc_top - 1e-4 && pos.y <= desc_bottom + 1e-4,
            "gem {i} y={} outside desc strip [{}, {}]",
            pos.y,
            desc_top,
            desc_bottom
        );
    }
}

/// @doc: The 4 gems per column must have distinct y-positions (spacing > 1.0 unit apart).
/// Overlapping gem positions would render on top of each other, hiding some visually.
#[test]
fn when_gem_desc_positions_called_then_four_distinct_y_values() {
    // Arrange
    let card_size = Vec2::new(60.0, 90.0);

    // Act
    let positions = gem_desc_positions(card_size);
    let mut left_ys: Vec<f32> = positions
        .iter()
        .filter(|p| p.x < 0.0)
        .map(|p| p.y)
        .collect();
    left_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Assert — all 4 y-values should be distinct
    for window in left_ys.windows(2) {
        assert!(
            (window[1] - window[0]).abs() > 1.0,
            "y-values too close: {} and {}",
            window[0],
            window[1]
        );
    }
}

// --- hexagon_vertices tests ---

/// @doc: Every vertex of a regular polygon lies exactly on its circumscribed circle, so
/// `vertex.length()` must equal `radius` for all 6 vertices.  If a vertex drifts off this
/// circle the polygon is no longer regular: gem facets become unequal in length, distorting
/// the gem silhouette and breaking the per-face normal computation in the shader.
#[test]
fn when_hexagon_vertices_generated_then_all_at_circumradius() {
    // Arrange
    let radius = 2.5_f32;

    // Act
    let vertices = hexagon_vertices(radius);

    // Assert
    for (i, v) in vertices.iter().enumerate() {
        assert!(
            (v.length() - radius).abs() < 1e-5,
            "vertex {i} distance from origin is {} (expected {})",
            v.length(),
            radius,
        );
    }
}

/// @doc: Gems are positioned by translating their mesh from the origin to a socket location.
/// A non-zero centroid means the mesh centre is displaced from its local origin, so every
/// translated gem would appear off-centre relative to its socket — visually misaligned and
/// inconsistent across different socket positions.
#[test]
fn when_hexagon_vertices_generated_then_centroid_is_at_origin() {
    // Arrange
    let radius = 3.0_f32;

    // Act
    let vertices = hexagon_vertices(radius);
    let centroid = vertices.iter().copied().fold(Vec2::ZERO, |acc, v| acc + v) / 6.0;

    // Assert
    assert!(
        centroid.length() < 1e-5,
        "centroid ({}, {}) is not at origin",
        centroid.x,
        centroid.y,
    );
}

/// @doc: Uniform 60-degree (PI/3) angular spacing is what makes the polygon regular.
/// Irregular spacing produces a skewed or stretched shape: some facets wider than others,
/// breaking visual symmetry and causing the gem to look like an unintentional blob rather
/// than a clean hexagon.
#[test]
fn when_hexagon_vertices_generated_then_angular_spacing_is_uniform() {
    // Arrange
    let radius = 2.0_f32;
    let expected_gap = std::f32::consts::PI / 3.0;

    // Act
    let vertices = hexagon_vertices(radius);
    let angles: [f32; 6] = core::array::from_fn(|i| vertices[i].y.atan2(vertices[i].x));

    // Assert — consecutive angular gaps must be ~PI/3 (with wrap-around on the last pair)
    for i in 0..6 {
        let a0 = angles[i];
        let a1 = angles[(i + 1) % 6];
        let mut gap = a1 - a0;
        // Normalise into (-PI, PI] to handle the wrap from ~PI back to ~-PI
        if gap <= -std::f32::consts::PI {
            gap += std::f32::consts::TAU;
        }
        assert!(
            (gap.abs() - expected_gap).abs() < 1e-5,
            "angular gap between vertex {i} and {} is {} (expected {})",
            (i + 1) % 6,
            gap,
            expected_gap,
        );
    }
}

/// @doc: A hexagon gives gems a distinct crystalline silhouette at card scale without
/// overlapping neighbors. The vertex count is a hard contract: rendering code indexes
/// exactly 6 facets to compute per-face normals, so deviating from 6 corrupts every
/// gem drawn on screen.
#[test]
fn when_hexagon_vertices_called_with_minimum_radius_then_produces_six_points() {
    // Arrange
    let radius = MIN_GEM_RADIUS;

    // Act
    let vertices = hexagon_vertices(radius);

    // Assert
    assert_eq!(
        vertices.len(),
        6,
        "hexagon_vertices must return 6 points regardless of radius (got {} at radius {})",
        vertices.len(),
        radius,
    );
}

// --- hexagon_uvs tests ---

/// @doc: The gem shader uses UV coordinates to identify which facet a fragment belongs to.
/// UVs outside [0,1] would cause the shader to miscalculate facet indices, producing wrong
/// normals and misplaced specular highlights.
#[test]
fn when_computing_hexagon_uvs_then_all_in_zero_to_one_range() {
    // Arrange
    let vertices = hexagon_vertices(2.0);

    // Act
    let uvs = hexagon_uvs(&vertices);

    // Assert
    for (i, uv) in uvs.iter().enumerate() {
        assert!(
            uv[0] >= -1e-5 && uv[0] <= 1.0 + 1e-5,
            "uv[{i}].u = {} is outside [0, 1]",
            uv[0],
        );
        assert!(
            uv[1] >= -1e-5 && uv[1] <= 1.0 + 1e-5,
            "uv[{i}].v = {} is outside [0, 1]",
            uv[1],
        );
    }
}

/// @doc: The UV normalization must use the full [0,1] span; a compressed range would make
/// the shader's facet boundary detection fail because it assumes edge UVs sit at 0.0 and 1.0.
#[test]
fn when_computing_hexagon_uvs_then_full_range_used_on_both_axes() {
    // Arrange
    let vertices = hexagon_vertices(2.0);

    // Act
    let uvs = hexagon_uvs(&vertices);
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
    let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);

    // Assert
    assert!(
        min_u.abs() < 1e-5,
        "min U = {min_u} (expected 0.0); normalization does not reach the lower bound"
    );
    assert!(
        (max_u - 1.0).abs() < 1e-5,
        "max U = {max_u} (expected 1.0); normalization does not reach the upper bound"
    );
    assert!(
        min_v.abs() < 1e-5,
        "min V = {min_v} (expected 0.0); normalization does not reach the lower bound"
    );
    assert!(
        (max_v - 1.0).abs() < 1e-5,
        "max V = {max_v} (expected 1.0); normalization does not reach the upper bound"
    );
}

// --- hexagon tessellation tests ---

/// @doc: The hexagon vertices produced by `hexagon_vertices` must form a valid polygon
/// for lyon's fill tessellator. If the vertices are colinear, self-intersecting, or have
/// fewer than 3 unique points, tessellation silently produces an empty mesh and the gem
/// becomes invisible — with no runtime error to catch the problem.
#[test]
fn when_tessellating_hexagon_as_polygon_then_mesh_is_nonempty_with_valid_indices() {
    // Arrange
    let vertices = hexagon_vertices(2.0);
    let points: Vec<_> = vertices.to_vec();

    // Act
    let mesh = tessellate(&ShapeVariant::Polygon { points }).unwrap();

    // Assert
    assert!(
        !mesh.vertices.is_empty(),
        "tessellated mesh has no vertices"
    );
    assert!(
        mesh.indices.len().is_multiple_of(3),
        "index count {} is not a multiple of 3",
        mesh.indices.len()
    );
    let vertex_count = mesh.vertices.len() as u32;
    for (i, &idx) in mesh.indices.iter().enumerate() {
        assert!(
            idx < vertex_count,
            "index [{i}] = {idx} is out of bounds (vertex count = {vertex_count})"
        );
    }
}

/// @doc: A convex polygon with N vertices triangulates into exactly N-2 triangles.
/// The gem shader assigns one surface normal per triangle (facet), so the facet count
/// must be stable at 4 for a hexagon. Extra degenerate triangles from the tessellator
/// would create invisible facets that steal specular highlights from real ones.
#[test]
fn when_tessellating_hexagon_then_produces_four_triangles() {
    // Arrange
    let vertices = hexagon_vertices(2.0);
    let points: Vec<_> = vertices.to_vec();

    // Act
    let mesh = tessellate(&ShapeVariant::Polygon { points }).unwrap();

    // Assert — N-2 triangles for N=6 → 4 triangles → 12 indices
    assert_eq!(
        mesh.indices.len(),
        12,
        "hexagon should produce 4 triangles (12 indices), got {} indices",
        mesh.indices.len()
    );
}

// --- gem_specular_intensity tests ---

/// @doc: Specular intensity ramps continuously from a dim floor at zero intensity
/// to full brilliance at maximum intensity. Higher intensity must always produce
/// brighter specular — any plateau or inversion would break the visual feedback
/// that communicates element strength to the player.
#[test]
fn when_specular_intensity_increases_then_output_strictly_increases() {
    // Arrange
    let samples = [0.0, 0.25, 0.5, 0.75, 1.0];

    // Act
    let values: Vec<f32> = samples.iter().map(|&i| gem_specular_intensity(i)).collect();

    // Assert — strictly increasing
    for w in values.windows(2) {
        assert!(w[1] > w[0], "specular must increase: {} -> {}", w[0], w[1]);
    }
    assert!(values[0] > 0.0, "zero-intensity specular must be positive");
    assert!(
        (values[4] - 1.0).abs() < 1e-5,
        "full-intensity specular must be 1.0"
    );
}

// --- gem_color tests ---

/// @doc: At zero intensity, gem color must be neutral gray regardless of aspect.
/// This ensures the visual hierarchy: faded gems look dormant/empty, and the
/// positive-to-negative color transition is smooth through the neutral midpoint.
#[test]
fn when_gem_color_at_zero_intensity_then_neutral_gray() {
    // Arrange / Act
    let color = gem_color(Aspect::Heat, 0.0);

    // Assert
    // Note: NEUTRAL_GRAY is private in the source, so we reconstruct it
    let neutral_gray = Color {
        r: 0.45,
        g: 0.45,
        b: 0.45,
        a: 1.0,
    };
    assert!(
        (color.r - neutral_gray.r).abs() < 1e-5
            && (color.g - neutral_gray.g).abs() < 1e-5
            && (color.b - neutral_gray.b).abs() < 1e-5,
        "zero intensity should be neutral gray, got ({}, {}, {})",
        color.r,
        color.g,
        color.b,
    );
}

/// @doc: At full intensity, gem color must match the aspect's base color.
/// This is the ceiling of the visual range — max-intensity gems must be
/// fully saturated so the aspect identity is unmistakable.
#[test]
fn when_gem_color_at_full_intensity_then_matches_aspect_color() {
    // Arrange
    let aspect = Aspect::Heat;
    let expected = aspect_color(aspect);

    // Act
    let color = gem_color(aspect, 1.0);

    // Assert
    assert!(
        (color.r - expected.r).abs() < 1e-5
            && (color.g - expected.g).abs() < 1e-5
            && (color.b - expected.b).abs() < 1e-5,
        "full intensity should match aspect color",
    );
}

/// @doc: At mid intensity, gem color must be between neutral gray and aspect
/// color on all channels — verifying the lerp is actually blending.
#[test]
fn when_gem_color_at_mid_intensity_then_between_gray_and_aspect() {
    // Arrange
    let aspect = Aspect::Cold;
    let full = aspect_color(aspect);
    let neutral_gray = Color {
        r: 0.45,
        g: 0.45,
        b: 0.45,
        a: 1.0,
    };

    // Act
    let color = gem_color(aspect, 0.5);

    // Assert — each channel must be between gray and full
    for (ch, gray, full_ch) in [
        (color.r, neutral_gray.r, full.r),
        (color.g, neutral_gray.g, full.g),
        (color.b, neutral_gray.b, full.b),
    ] {
        let lo = gray.min(full_ch);
        let hi = gray.max(full_ch);
        assert!(
            ch >= lo - 1e-5 && ch <= hi + 1e-5,
            "channel {ch} not between {lo} and {hi}",
        );
    }
}

// --- gem_radius tests ---

#[test]
fn when_gem_radius_at_zero_intensity_then_equals_minimum() {
    // Arrange / Act
    let radius = gem_radius(0.0);

    // Assert
    assert_eq!(radius, MIN_GEM_RADIUS);
}

#[test]
fn when_gem_radius_at_full_intensity_then_equals_maximum() {
    // Arrange / Act
    let radius = gem_radius(1.0);

    // Assert
    assert_eq!(radius, MAX_GEM_RADIUS);
}

/// @doc: Gem radius scales linearly from `MIN_GEM_RADIUS` to `MAX_GEM_RADIUS` with intensity [0.0, 1.0].
/// Non-linearity would distort the visual feedback of element intensity on the card.
#[test]
fn when_gem_radius_at_mid_intensity_then_between_min_and_max() {
    // Arrange / Act
    let radius = gem_radius(0.5);

    // Assert
    assert!(
        radius > MIN_GEM_RADIUS && radius < MAX_GEM_RADIUS,
        "radius {radius} should be between {MIN_GEM_RADIUS} and {MAX_GEM_RADIUS}"
    );
}
