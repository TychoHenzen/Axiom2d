#![allow(clippy::unwrap_used)]

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use terrain::material::TerrainId;
use terrain::wfc::{ConstraintTable, Grid, collapse};

fn two_type_constraints() -> ConstraintTable {
    let a = TerrainId(0);
    let b = TerrainId(1);
    let mut table = ConstraintTable::new(vec![a, b]);
    table.allow(a, b);
    table.allow(b, a);
    table.allow(a, a);
    table.allow(b, b);
    table
}

#[test]
fn when_collapsing_small_grid_then_all_cells_filled() {
    // Arrange
    let mut grid = Grid::new(4, 4);
    let constraints = two_type_constraints();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Act
    let result = collapse(&mut grid, &constraints, &mut rng);

    // Assert
    assert!(result.is_ok());
    for y in 0..4 {
        for x in 0..4 {
            assert!(grid.get(x, y).is_some(), "cell ({x},{y}) was not collapsed");
        }
    }
}

#[test]
fn when_cell_pinned_then_pinned_value_preserved() {
    // Arrange
    let mut grid = Grid::new(4, 4);
    let pinned = TerrainId(1);
    grid.set(2, 2, Some(pinned));

    let constraints = two_type_constraints();
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    // Act
    let result = collapse(&mut grid, &constraints, &mut rng);

    // Assert
    assert!(result.is_ok());
    assert_eq!(grid.get(2, 2), Some(pinned));
}

#[test]
fn when_same_seed_then_deterministic_output() {
    // Arrange
    let constraints = two_type_constraints();

    let mut grid1 = Grid::new(6, 6);
    let mut rng1 = ChaCha8Rng::seed_from_u64(123);
    collapse(&mut grid1, &constraints, &mut rng1).unwrap();

    let mut grid2 = Grid::new(6, 6);
    let mut rng2 = ChaCha8Rng::seed_from_u64(123);
    collapse(&mut grid2, &constraints, &mut rng2).unwrap();

    // Assert
    for y in 0..6 {
        for x in 0..6 {
            assert_eq!(grid1.get(x, y), grid2.get(x, y), "mismatch at ({x},{y})");
        }
    }
}

#[test]
fn when_impossible_constraints_then_returns_error() {
    // Arrange
    let a = TerrainId(0);
    let b = TerrainId(1);
    let mut table = ConstraintTable::new(vec![a, b]);
    table.allow(a, a);
    table.allow(b, b);
    // no cross-type adjacency allowed

    let mut grid = Grid::new(2, 2);
    grid.set(0, 0, Some(a));
    grid.set(1, 1, Some(b));
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    // Act
    let result = collapse(&mut grid, &table, &mut rng);

    // Assert
    assert!(result.is_err());
}
