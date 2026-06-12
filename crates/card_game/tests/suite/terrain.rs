use card_game::terrain;

#[test]
fn when_terrain_tileset_called_then_returns_terrain_tileset() {
    // Arrange / Act
    let ts = terrain::tileset();

    // Assert — API is callable and returns a TerrainTileSet.
    // tiles may be empty until TSX assets are added to assets/terrain/.
    let _ = ts.tiles;
    let _ = ts.adjacency_rules;
}
