#![allow(clippy::unwrap_used)]

use img_to_shape_gui::state::AppState;

/// @doc: New AppState has default values that are safe to query
#[test]
fn when_new_app_state_then_defaults_are_valid() {
    // Arrange / Act
    let state = AppState::new();

    // Assert
    assert_eq!(
        state.raw_unique_color_count(),
        0,
        "new state should have zero unique colors"
    );
    assert!(
        !state.art_filename().is_empty(),
        "new state should have a default art filename, got empty",
    );
}

/// @doc: After loading an image, raw unique color count is non-zero
#[test]
fn when_image_loaded_then_color_count_positive() {
    // Arrange
    let mut state = AppState::new();
    // 2×2 image with red and green pixels
    let rgba = vec![
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
    ];

    // Act
    state.load_image(rgba, 2, 2, None);
    state.run_conversion();

    // Assert
    assert!(
        state.raw_unique_color_count() > 0,
        "loaded image with 2 distinct pixel colors should have positive unique color count"
    );
}

/// @doc: generate_export_code produces non-empty string after image load
#[test]
fn when_image_loaded_then_generate_export_code_is_non_empty() {
    // Arrange
    let mut state = AppState::new();
    let rgba = vec![255u8; 16]; // 2×2 solid white
    state.load_image(rgba, 2, 2, None);
    state.run_conversion();

    // Act
    let code = state.generate_export_code();

    // Assert
    assert!(
        !code.is_empty(),
        "export code should be non-empty after converting a loaded image"
    );
}

/// @doc: generate_art_file produces non-empty result after image load
#[test]
fn when_image_loaded_then_generate_art_file_is_ok() {
    // Arrange
    let mut state = AppState::new();
    let rgba = vec![255u8; 16]; // 2×2 solid white
    state.load_image(rgba, 2, 2, None);
    state.run_conversion();

    // Act
    let result = state.generate_art_file();

    // Assert
    assert!(
        result.is_ok(),
        "generate_art_file should succeed after conversion: {:?}",
        result.err()
    );
    let art_code = result.unwrap();
    assert!(
        !art_code.is_empty(),
        "generated art file should be non-empty"
    );
}
