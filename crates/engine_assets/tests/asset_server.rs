#![allow(clippy::unwrap_used)]

use engine_assets::asset_server::{AssetError, AssetServer};
use engine_assets::handle::Handle;

#[test]
fn when_adding_asset_then_returns_handle_with_id_zero() {
    // Arrange
    let mut server: AssetServer<String> = AssetServer::default();

    // Act
    let handle = server.add("hello".to_string());

    // Assert
    assert_eq!(handle.id, 0);
}

#[test]
fn when_adding_second_asset_then_returns_different_handle() {
    // Arrange
    let mut server: AssetServer<String> = AssetServer::default();

    // Act
    let first = server.add("hello".to_string());
    let second = server.add("world".to_string());

    // Assert
    assert_ne!(first, second);
    assert_eq!(second.id, 1);
}

#[test]
fn when_getting_by_handle_then_returns_stored_value() {
    // Arrange
    let mut server = AssetServer::default();
    let handle = server.add("hello".to_string());

    // Act
    let value = server.get(handle);

    // Assert
    assert_eq!(value, Some(&"hello".to_string()));
}

#[test]
fn when_getting_unknown_handle_then_returns_none() {
    // Arrange
    let server: AssetServer<String> = AssetServer::default();
    let unknown = Handle::<String>::new(99);

    // Act
    let value = server.get(unknown);

    // Assert
    assert_eq!(value, None);
}

#[test]
fn when_getting_mut_then_mutation_is_visible_on_next_get() {
    // Arrange
    let mut server = AssetServer::default();
    let handle = server.add("hello".to_string());

    // Act
    if let Some(v) = server.get_mut(handle) {
        *v = "world".to_string();
    }

    // Assert
    assert_eq!(server.get(handle), Some(&"world".to_string()));
}

#[test]
fn when_asset_added_then_ref_count_is_one() {
    // Arrange
    let mut server = AssetServer::default();

    // Act
    let handle = server.add("hello".to_string());

    // Assert
    assert_eq!(server.ref_count(handle), Some(1));
}

#[test]
fn when_clone_handle_called_then_ref_count_increments() {
    // Arrange
    let mut server = AssetServer::default();
    let handle = server.add("hello".to_string());

    // Act
    server.clone_handle(handle);

    // Assert
    assert_eq!(server.ref_count(handle), Some(2));
}

/// @doc: Reference-counted removal — decrementing a shared handle must not
/// evict the asset while other holders exist. Without this, removing one
/// shader reference could delete the GPU resource while another system
/// still expects it, causing a missing-asset panic on next draw call.
#[test]
fn when_remove_with_ref_count_above_one_then_decrements_without_evicting() {
    // Arrange
    let mut server = AssetServer::default();
    let handle = server.add("hello".to_string());
    server.clone_handle(handle);

    // Act
    let removed = server.remove(handle);

    // Assert
    assert!(removed);
    assert_eq!(server.ref_count(handle), Some(1));
    assert_eq!(server.get(handle), Some(&"hello".to_string()));
}

/// @doc: Final remove evicts the asset from the server — the handle becomes
/// invalid. This prevents memory leaks from orphaned assets that no system
/// references anymore.
#[test]
fn when_remove_with_ref_count_one_then_evicts_asset() {
    // Arrange
    let mut server = AssetServer::default();
    let handle = server.add("hello".to_string());

    // Act
    let removed = server.remove(handle);

    // Assert
    assert!(removed);
    assert_eq!(server.get(handle), None);
    assert_eq!(server.ref_count(handle), None);
}

#[test]
fn when_remove_unknown_handle_then_returns_false() {
    // Arrange
    let mut server: AssetServer<String> = AssetServer::default();
    let unknown = Handle::<String>::new(42);

    // Act
    let removed = server.remove(unknown);

    // Assert
    assert!(!removed);
}

proptest::proptest! {
    #[test]
    fn when_cloned_n_times_then_ref_count_lifecycle_is_correct(
        clone_count in 1usize..=5,
    ) {
        // Arrange
        let mut server: AssetServer<String> = AssetServer::default();
        let handle = server.add("test".to_string());
        for _ in 0..clone_count {
            server.clone_handle(handle);
        }
        let expected_initial = 1 + clone_count;
        assert_eq!(server.ref_count(handle), Some(expected_initial));

        // Act — remove (clone_count) times, asset should still exist
        for k in 0..clone_count {
            assert!(server.remove(handle));
            assert_eq!(
                server.ref_count(handle),
                Some(expected_initial - 1 - k),
                "after {} removes",
                k + 1
            );
        }

        // Act — final remove evicts
        assert!(server.remove(handle));
        assert_eq!(server.ref_count(handle), None);
        assert_eq!(server.get(handle), None);

        // Act — extra remove returns false
        assert!(!server.remove(handle));
    }
}

#[test]
fn when_loading_nonexistent_file_then_returns_io_error() {
    // Arrange
    let mut server: AssetServer<String> = AssetServer::default();

    // Act
    let result = server.load("/no/such/file.ron");

    // Assert
    assert!(matches!(result, Err(AssetError::Io(_))));
}

#[test]
fn when_loading_invalid_ron_then_returns_parse_error() {
    // Arrange
    let dir = std::env::temp_dir().join("axiom2d_test_tc012");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bad.ron");
    std::fs::write(&path, "this is not valid RON {{{").unwrap();
    let mut server: AssetServer<Vec<i32>> = AssetServer::default();

    // Act
    let result = server.load(path.to_str().unwrap());

    // Assert
    assert!(matches!(result, Err(AssetError::Parse(_))));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn when_loading_valid_ron_file_then_returns_handle_to_deserialized_value() {
    // Arrange
    let dir = std::env::temp_dir().join("axiom2d_test_tc013");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("data.ron");
    std::fs::write(&path, "[1, 2, 3]").unwrap();
    let mut server: AssetServer<Vec<i32>> = AssetServer::default();

    // Act
    let handle = server.load(path.to_str().unwrap()).unwrap();

    // Assert
    assert_eq!(server.get(handle), Some(&vec![1, 2, 3]));
    let _ = std::fs::remove_dir_all(&dir);
}
