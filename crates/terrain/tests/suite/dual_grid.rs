use terrain::dual_grid::{DualGrid, corner_bitmask};
use terrain::material::TerrainId;

#[test]
fn when_all_corners_same_then_bitmask_is_zero() {
    // Arrange
    let id = TerrainId(0);

    // Act
    let mask = corner_bitmask([id, id, id, id], id);

    // Assert — all corners match the "primary" type, no transitions
    assert_eq!(mask, 0);
}

#[test]
fn when_all_corners_differ_from_primary_then_bitmask_is_15() {
    // Arrange
    let primary = TerrainId(0);
    let other = TerrainId(1);

    // Act
    let mask = corner_bitmask([other, other, other, other], primary);

    // Assert
    assert_eq!(mask, 15);
}

#[test]
fn when_ne_corner_differs_then_bitmask_bit_0_set() {
    let a = TerrainId(0);
    let b = TerrainId(1);

    let mask = corner_bitmask([b, a, a, a], a);

    assert_eq!(mask, 1); // NE=1
}

#[test]
fn when_se_and_sw_corners_differ_then_bitmask_is_6() {
    let a = TerrainId(0);
    let b = TerrainId(1);

    let mask = corner_bitmask([a, b, b, a], a);

    assert_eq!(mask, 6); // SE=2 + SW=4
}

#[test]
fn when_grid_2x2_then_produces_3x3_visual_tiles() {
    // A 2x2 data grid produces a 3x3 visual grid (straddle pattern)
    let mut grid = DualGrid::new(2, 2, TerrainId(0));
    grid.set(1, 0, TerrainId(1));
    grid.set(1, 1, TerrainId(1));

    let tiles = grid.visual_tiles();

    assert_eq!(tiles.len(), 9); // (2+1) * (2+1)
}

#[test]
fn when_visual_tile_straddles_edge_then_corners_include_border_default() {
    let grid = DualGrid::new(2, 2, TerrainId(0));
    let tiles = grid.visual_tiles();

    // Top-left visual tile at (-0.5, -0.5) straddles outside the grid.
    // Out-of-bounds corners should use the border default (same as the nearest cell).
    let tl = &tiles[0];
    // All corners should be TerrainId(0) since the whole grid is uniform
    assert_eq!(tl.corners, [TerrainId(0); 4]);
}
