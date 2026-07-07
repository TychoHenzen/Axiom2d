#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use terrain::prelude::TerrainTileSet;
use tiled_to_shapes::codegen::generate_tileset_code;

/// @doc: Codegen produces valid Rust source even for an empty tileset
#[test]
fn when_empty_tileset_then_codegen_returns_valid_function() {
    // Arrange
    let tileset = TerrainTileSet {
        tiles: BTreeMap::new(),
        adjacency_rules: Vec::new(),
    };

    // Act
    let code = generate_tileset_code(&tileset, "tileset", "empty.tsx");

    // Assert
    assert!(
        code.contains("pub fn tileset()"),
        "generated code must contain pub fn tileset"
    );
    assert!(
        code.contains("TerrainTileSet"),
        "generated code must reference TerrainTileSet type"
    );
    assert!(
        !code.contains("todo!()") && !code.contains("unimplemented!()"),
        "generated code must not contain stub markers, got:\n{code}"
    );
}
