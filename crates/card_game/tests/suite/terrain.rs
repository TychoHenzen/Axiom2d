use card_game::terrain;

/// The build script generates an empty tileset when no TSX assets exist.
/// These tests verify the empty-codegen path works and the API surface is sound.
/// Once TSX assets are added to assets/terrain/, add tests for non-empty tiles.

#[test]
fn when_tileset_called_then_returns_valid_empty_tileset() {
    // Arrange / Act
    let ts = terrain::tileset();

    // Assert — empty TSX assets path produces empty but valid structure
    assert!(
        ts.tiles.is_empty(),
        "expected empty tiles map when no TSX assets present, got {} entries",
        ts.tiles.len()
    );
    assert!(
        ts.adjacency_rules.is_empty(),
        "expected empty adjacency_rules when no TSX assets present, got {} rules",
        ts.adjacency_rules.len()
    );
}

#[test]
fn when_tileset_called_twice_then_returns_same_empty_structure() {
    // Arrange / Act
    let ts1 = terrain::tileset();
    let ts2 = terrain::tileset();

    // Assert — calls are consistent (empty = always empty without TSX assets)
    assert_eq!(
        ts1.tiles.len(),
        ts2.tiles.len(),
        "tiles count should be consistent across calls"
    );
    assert_eq!(
        ts1.adjacency_rules.len(),
        ts2.adjacency_rules.len(),
        "adjacency_rules count should be consistent across calls"
    );
}

#[test]
fn when_tileset_accessed_then_tiles_is_btree_map() {
    // Arrange / Act
    let ts = terrain::tileset();

    // Assert — verify type-instability safety: iter, keys, values work
    assert!(ts.tiles.keys().next().is_none(), "empty tileset should have no keys");
    assert!(ts.tiles.values().next().is_none(), "empty tileset should have no values");
    assert_eq!(ts.tiles.iter().count(), 0, "empty tileset iteration count should be zero");
}
