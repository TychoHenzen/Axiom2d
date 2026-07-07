#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use img_to_shape::manifest::{load_manifest, save_manifest, ShapeManifest};

/// @doc: load_manifest returns empty manifest for non-existent path
#[test]
fn when_path_does_not_exist_then_load_returns_empty_manifest() {
    // Arrange
    let path = PathBuf::from("/nonexistent/img_to_shape_test/manifest.json");

    // Act
    let result = load_manifest(&path);

    // Assert
    assert!(
        result.is_ok(),
        "loading non-existent manifest should succeed with empty"
    );
    let manifest = result.unwrap();
    assert!(
        manifest.entries.is_empty(),
        "empty manifest should have no entries"
    );
}

/// @doc: save_manifest writes valid JSON to disk
#[test]
fn when_save_then_load_roundtrips() {
    // Arrange
    let manifest = ShapeManifest::default();
    let dir = std::env::temp_dir().join("img_to_shape_test_manifest_ext");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test.json");

    // Act
    save_manifest(&manifest, &path).expect("save should succeed");
    let loaded = load_manifest(&path).expect("load should succeed");

    // Assert
    assert!(loaded.entries.is_empty(), "roundtripped empty manifest should have no entries");

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}
