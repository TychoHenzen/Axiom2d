#![allow(clippy::unwrap_used)]

use tiled_to_shapes::pipeline::{default_convert_config, passability_to_tags};
use terrain::prelude::GameplayTag;

/// @doc: Default config produces sensible values
#[test]
fn when_default_config_then_all_fields_sensible() {
    // Arrange / Act
    let config = default_convert_config();

    // Assert
    assert!(
        config.color_threshold > 0.0 && config.color_threshold < 1.0,
        "color_threshold should be in (0,1), got {}",
        config.color_threshold
    );
    assert!(
        config.alpha_threshold > 0,
        "alpha_threshold should be > 0, got {}",
        config.alpha_threshold
    );
    assert!(
        config.rdp_epsilon > 0.0,
        "rdp_epsilon should be positive, got {}",
        config.rdp_epsilon
    );
    assert!(
        config.max_dimension > 0,
        "max_dimension should be positive, got {}",
        config.max_dimension
    );
}

/// @doc: passability_to_tags maps known strings to correct GameplayTags
#[test]
fn when_passage_passable_then_returns_empty_tags() {
    // Arrange / Act
    let tags = passability_to_tags("passable");

    // Assert
    assert!(
        tags.is_empty(),
        "passable should yield no gameplay tags, got {tags:?}"
    );
}

/// @doc: passability_to_tags maps "solid" to [Solid]
#[test]
fn when_passage_solid_then_returns_solid_tag() {
    // Arrange / Act
    let tags = passability_to_tags("solid");

    // Assert
    assert_eq!(
        tags,
        vec![GameplayTag::Solid],
        "solid should yield [Solid]"
    );
}

/// @doc: passability_to_tags maps "difficult" to [DifficultTerrain]
#[test]
fn when_passage_difficult_then_returns_difficult_terrain_tag() {
    // Arrange / Act
    let tags = passability_to_tags("difficult");

    // Assert
    assert_eq!(
        tags,
        vec![GameplayTag::DifficultTerrain],
        "difficult should yield [DifficultTerrain]"
    );
}

/// @doc: Unknown passability string returns empty tags
#[test]
fn when_passage_unknown_then_returns_empty_tags() {
    // Arrange / Act
    let tags = passability_to_tags("something_weird");

    // Assert
    assert!(
        tags.is_empty(),
        "unknown passability should yield no tags, got {tags:?}"
    );
}
