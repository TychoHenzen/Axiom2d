use terrain::dual_grid::{DualGrid, corner_bitmask};
use terrain::material::TerrainId;

#[test]
fn when_all_corners_same_then_bitmask_is_zero() {
    // Arrange
    let id = TerrainId(0);

    // Act
    let mask = corner_bitmask([id, id, id, id], id);

    // Assert — all corners match the "primary" type, no transitions
    assert_eq!(mask, 0, "bitmask should be 0 when all corners match primary");
}

#[test]
fn when_all_corners_differ_from_primary_then_bitmask_is_15() {
    // Arrange
    let primary = TerrainId(0);
    let other = TerrainId(1);

    // Act
    let mask = corner_bitmask([other, other, other, other], primary);

    // Assert
    assert_eq!(mask, 15, "bitmask should be 15 when all corners differ from primary");
}

#[test]
fn when_ne_corner_differs_then_bitmask_bit_0_set() {
    let a = TerrainId(0);
    let b = TerrainId(1);

    let mask = corner_bitmask([b, a, a, a], a);

    assert_eq!(mask, 1, "NE corner differing should set bitmask bit 0 (value 1)"); // NE=1
}

#[test]
fn when_se_and_sw_corners_differ_then_bitmask_is_6() {
    let a = TerrainId(0);
    let b = TerrainId(1);

    let mask = corner_bitmask([a, b, b, a], a);

    assert_eq!(mask, 6, "SE and SW corners differing should set bits 1 and 2 (value 6)"); // SE=2 + SW=4
}

#[test]
fn when_grid_2x2_then_produces_3x3_visual_tiles() {
    // A 2x2 data grid produces a 3x3 visual grid (straddle pattern)
    let mut grid = DualGrid::new(2, 2, TerrainId(0));
    grid.set(1, 0, TerrainId(1));
    grid.set(1, 1, TerrainId(1));

    let tiles = grid.visual_tiles();

    assert_eq!(tiles.len(), 9, "2x2 DualGrid should produce 9 visual tiles"); // (2+1) * (2+1)
}

#[test]
fn when_visual_tile_straddles_edge_then_corners_include_border_default() {
    let grid = DualGrid::new(2, 2, TerrainId(0));
    let tiles = grid.visual_tiles();

    // Top-left visual tile at (-0.5, -0.5) straddles outside the grid.
    // Out-of-bounds corners should use the border default (same as the nearest cell).
    let tl = &tiles[0];
    // All corners should be TerrainId(0) since the whole grid is uniform
    assert_eq!(tl.corners, [TerrainId(0); 4], "uniform grid border tile should have all corners equal to default");
}

proptest::proptest! {
    #[test]
    fn when_any_corners_and_primary_then_bitmask_is_4_bit(
        c0 in proptest::num::u8::ANY,
        c1 in proptest::num::u8::ANY,
        c2 in proptest::num::u8::ANY,
        c3 in proptest::num::u8::ANY,
        primary in proptest::num::u8::ANY,
    ) {
        let corners = [TerrainId(c0), TerrainId(c1), TerrainId(c2), TerrainId(c3)];
        let mask = corner_bitmask(corners, TerrainId(primary));
        assert!(mask <= 15, "bitmask {mask} exceeds 4-bit range");
    }

    #[test]
    fn when_any_grid_dimensions_then_visual_tile_count_is_w_plus_1_times_h_plus_1(
        w in 1_usize..=16,
        h in 1_usize..=16,
    ) {
        let grid = DualGrid::new(w, h, TerrainId(0));
        let tiles = grid.visual_tiles();
        assert_eq!(tiles.len(), (w + 1) * (h + 1), "visual tile count for {w}x{h} grid should be {}", (w + 1) * (h + 1));
    }
}
