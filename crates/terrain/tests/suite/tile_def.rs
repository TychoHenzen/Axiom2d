use engine_render::prelude::PathCommand;
use glam::Vec2;
use terrain::prelude::*;

// ===========================================================================
// Step 2: bitmask_to_variant
// ===========================================================================

#[test]
fn bitmask_to_variant_all_16_produce_valid_pattern_and_rotation() {
    let valid_rotations = [0, 90, 180, 270];
    for mask in 0..16u8 {
        let (pattern, rotation) = bitmask_to_variant(mask);
        assert!(
            valid_rotations.contains(&rotation),
            "bitmask {mask}: unexpected rotation {rotation}"
        );
        match pattern {
            TilePattern::Solid
            | TilePattern::OuterCorner
            | TilePattern::Edge
            | TilePattern::Diagonal
            | TilePattern::InnerCorner => {}
        }
    }
}

#[test]
fn bitmask_to_variant_0_and_15_return_solid_0() {
    assert_eq!(bitmask_to_variant(0), (TilePattern::Solid, 0));
    assert_eq!(bitmask_to_variant(15), (TilePattern::Solid, 0));
}

#[test]
fn bitmask_to_variant_outer_corner_rotations() {
    assert_eq!(bitmask_to_variant(1), (TilePattern::OuterCorner, 0));
    assert_eq!(bitmask_to_variant(2), (TilePattern::OuterCorner, 90));
    assert_eq!(bitmask_to_variant(4), (TilePattern::OuterCorner, 180));
    assert_eq!(bitmask_to_variant(8), (TilePattern::OuterCorner, 270));
}

#[test]
fn bitmask_to_variant_edge_rotations() {
    assert_eq!(bitmask_to_variant(3), (TilePattern::Edge, 0));
    assert_eq!(bitmask_to_variant(6), (TilePattern::Edge, 90));
    assert_eq!(bitmask_to_variant(12), (TilePattern::Edge, 180));
    assert_eq!(bitmask_to_variant(9), (TilePattern::Edge, 270));
}

#[test]
fn bitmask_to_variant_diagonal_rotations() {
    assert_eq!(bitmask_to_variant(5), (TilePattern::Diagonal, 0));
    assert_eq!(bitmask_to_variant(10), (TilePattern::Diagonal, 90));
}

#[test]
fn bitmask_to_variant_inner_corner_rotations() {
    assert_eq!(bitmask_to_variant(7), (TilePattern::InnerCorner, 0));
    assert_eq!(bitmask_to_variant(14), (TilePattern::InnerCorner, 90));
    assert_eq!(bitmask_to_variant(13), (TilePattern::InnerCorner, 180));
    assert_eq!(bitmask_to_variant(11), (TilePattern::InnerCorner, 270));
}

// ===========================================================================
// Step 3: transform_path
// ===========================================================================

const IDENTITY: [Vec2; 4] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(0.0, 1.0),
];

fn approx_eq(a: Vec2, b: Vec2) -> bool {
    (a - b).length() < 1e-5
}

#[test]
fn transform_path_identity_corners_preserves_input() {
    let input = vec![
        PathCommand::MoveTo(Vec2::new(0.25, 0.75)),
        PathCommand::LineTo(Vec2::new(0.5, 0.5)),
        PathCommand::Close,
    ];
    let output = transform_path(&input, IDENTITY);
    assert_eq!(input.len(), output.len());
    match (&input[0], &output[0]) {
        (PathCommand::MoveTo(a), PathCommand::MoveTo(b)) => assert!(approx_eq(*a, *b)),
        _ => panic!("expected MoveTo"),
    }
    match (&input[1], &output[1]) {
        (PathCommand::LineTo(a), PathCommand::LineTo(b)) => assert!(approx_eq(*a, *b)),
        _ => panic!("expected LineTo"),
    }
}

#[test]
fn transform_path_scale_corners_scales_coordinates() {
    let corners = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 100.0),
    ];
    let input = vec![PathCommand::MoveTo(Vec2::new(0.5, 0.5))];
    let output = transform_path(&input, corners);
    match &output[0] {
        PathCommand::MoveTo(p) => assert!(approx_eq(*p, Vec2::new(50.0, 50.0))),
        _ => panic!("expected MoveTo"),
    }
}

#[test]
fn transform_path_translation_corners_translates() {
    let corners = [
        Vec2::new(10.0, 20.0),
        Vec2::new(11.0, 20.0),
        Vec2::new(11.0, 21.0),
        Vec2::new(10.0, 21.0),
    ];
    let input = vec![PathCommand::MoveTo(Vec2::ZERO)];
    let output = transform_path(&input, corners);
    match &output[0] {
        PathCommand::MoveTo(p) => assert!(approx_eq(*p, Vec2::new(10.0, 20.0))),
        _ => panic!("expected MoveTo"),
    }
}

#[test]
fn transform_path_cubic_to_transforms_all_three_coordinates() {
    let corners = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 100.0),
    ];
    let input = vec![PathCommand::CubicTo {
        control1: Vec2::new(0.25, 0.25),
        control2: Vec2::new(0.75, 0.75),
        to: Vec2::new(1.0, 1.0),
    }];
    let output = transform_path(&input, corners);
    match &output[0] {
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => {
            assert!(approx_eq(*control1, Vec2::new(25.0, 25.0)));
            assert!(approx_eq(*control2, Vec2::new(75.0, 75.0)));
            assert!(approx_eq(*to, Vec2::new(100.0, 100.0)));
        }
        _ => panic!("expected CubicTo"),
    }
}

// ===========================================================================
// Step 4: compute_tint
// ===========================================================================

#[test]
fn compute_tint_zero_range_returns_identity() {
    let range = TintRange {
        hue_shift_max: 0.0,
        brightness_shift_max: 0.0,
    };
    assert_eq!(compute_tint(42, &range), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(compute_tint(0, &range), [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn compute_tint_different_seeds_produce_different_tints() {
    let range = TintRange {
        hue_shift_max: 10.0,
        brightness_shift_max: 0.1,
    };
    let a = compute_tint(1, &range);
    let b = compute_tint(2, &range);
    assert_ne!(a, b);
}

#[test]
fn compute_tint_same_seed_is_deterministic() {
    let range = TintRange {
        hue_shift_max: 10.0,
        brightness_shift_max: 0.1,
    };
    assert_eq!(compute_tint(42, &range), compute_tint(42, &range));
}

// ===========================================================================
// Step 5: quad_grid
// ===========================================================================

#[test]
fn quad_grid_quad_count_returns_width_minus_1_height_minus_1() {
    let grid = QuadGrid::new(4, 3, TerrainId(0));
    assert_eq!(grid.quad_count(), (3, 2));
}

#[test]
fn quad_grid_quad_corners_returns_correct_terrain_ids() {
    let mut grid = QuadGrid::new(3, 3, TerrainId(0));
    grid.set_vertex(1, 0, TerrainId(1));
    grid.set_vertex(1, 1, TerrainId(2));
    grid.set_vertex(0, 1, TerrainId(3));

    let corners = grid.quad_corners(0, 0).unwrap();
    assert_eq!(corners[0], TerrainId(0)); // SW = (0,0)
    assert_eq!(corners[1], TerrainId(1)); // SE = (1,0)
    assert_eq!(corners[2], TerrainId(2)); // NE = (1,1)
    assert_eq!(corners[3], TerrainId(3)); // NW = (0,1)
}

#[test]
fn quad_grid_corner_positions_correctly_spaced() {
    let grid = QuadGrid::new(3, 3, TerrainId(0));
    let positions = grid.quad_corner_positions(1, 0, 32.0).unwrap();
    assert!(approx_eq(positions[0], Vec2::new(32.0, 0.0)));
    assert!(approx_eq(positions[1], Vec2::new(64.0, 0.0)));
    assert!(approx_eq(positions[2], Vec2::new(64.0, 32.0)));
    assert!(approx_eq(positions[3], Vec2::new(32.0, 32.0)));
}

// ===========================================================================
// Step 6: example_tileset
// ===========================================================================

#[test]
fn example_tileset_has_two_terrain_types() {
    let set = terrain::tile_def::example::example_tileset();
    assert_eq!(set.tiles.len(), 2);
    assert!(set.tiles.contains_key("grass"));
    assert!(set.tiles.contains_key("stone"));
}

#[test]
fn example_tileset_each_type_has_five_variants() {
    let set = terrain::tile_def::example::example_tileset();
    for (name, def) in &set.tiles {
        assert_eq!(
            def.variants.len(),
            5,
            "terrain type '{name}' should have 5 variants"
        );
    }
}

#[test]
fn example_tileset_each_variant_has_nonempty_shapes() {
    let set = terrain::tile_def::example::example_tileset();
    for (name, def) in &set.tiles {
        for (i, variant) in def.variants.iter().enumerate() {
            assert!(
                !variant.shapes.is_empty(),
                "terrain '{name}' variant {i} has no shapes"
            );
            for shape in &variant.shapes {
                assert!(
                    !shape.path.is_empty(),
                    "terrain '{name}' variant {i} has shape with empty path"
                );
            }
        }
    }
}

#[test]
fn example_tileset_all_coordinates_in_unit_range() {
    let set = terrain::tile_def::example::example_tileset();
    for (name, def) in &set.tiles {
        for (i, variant) in def.variants.iter().enumerate() {
            for shape in &variant.shapes {
                for cmd in &shape.path {
                    let coords = match *cmd {
                        PathCommand::MoveTo(p) | PathCommand::LineTo(p) => vec![p],
                        PathCommand::QuadraticTo { control, to } => vec![control, to],
                        PathCommand::CubicTo {
                            control1,
                            control2,
                            to,
                        } => vec![control1, control2, to],
                        PathCommand::Close | PathCommand::Reverse => vec![],
                    };
                    for p in coords {
                        assert!(
                            p.x >= 0.0 && p.x <= 1.0 && p.y >= 0.0 && p.y <= 1.0,
                            "terrain '{name}' variant {i}: coord ({}, {}) outside [0,1]²",
                            p.x,
                            p.y
                        );
                    }
                }
            }
        }
    }
}
