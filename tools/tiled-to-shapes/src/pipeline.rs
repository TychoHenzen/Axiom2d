use std::collections::BTreeMap;
use std::path::Path;

use img_to_shape::{ConvertConfig, ResizeMethod, image_to_shapes};
use terrain::prelude::{
    AnnotatedShape, EdgeId, GameplayTag, ShapePurpose, TerrainTileDefinition, TerrainTileSet,
    TilePattern, TileVariant, TintRange, bitmask_to_variant,
};

use crate::extract::extract_tile;
use crate::normalize::normalize_shapes;
use crate::tsx_parser::parse_tsx;
use crate::types::{ParsedWangSet, TiledToShapesError};

/// Default conversion config for pixel-art tiles.
pub fn default_convert_config() -> ConvertConfig {
    ConvertConfig {
        color_threshold: 0.1,
        alpha_threshold: 128,
        rdp_epsilon: 1.5,
        bezier_error: 1.5,
        min_area: 4,
        max_dimension: 64,
        resize_method: ResizeMethod::Scale2x,
        use_bezier: true,
        merge_below: 5,
        max_shapes: 0,
    }
}

/// Convert a TSX tileset file into a `TerrainTileSet` with baked shapes.
pub fn convert_tileset(
    tsx_path: &Path,
    config: &ConvertConfig,
) -> Result<TerrainTileSet, TiledToShapesError> {
    let tileset = parse_tsx(tsx_path)?;

    // Resolve tilesheet path relative to the TSX file
    let tsx_dir = tsx_path.parent().unwrap_or(Path::new("."));
    let sheet_path = tsx_dir.join(&tileset.image_source);
    if !sheet_path.exists() {
        return Err(TiledToShapesError::TilesheetNotFound(
            sheet_path.display().to_string(),
        ));
    }

    let img = image::open(&sheet_path)
        .map_err(TiledToShapesError::ImageError)?
        .to_rgba8();

    let mut tiles = BTreeMap::new();

    for wang_set in &tileset.wang_sets {
        let definition = convert_wang_set(
            wang_set,
            &img,
            tileset.tile_width,
            tileset.tile_height,
            tileset.columns,
            config,
        )?;
        tiles.insert(wang_set.id.clone(), definition);
    }

    Ok(TerrainTileSet {
        tiles,
        adjacency_rules: Vec::new(),
    })
}

/// Canonical bitmask for each `TilePattern` (the first/lowest tile in each group).
const CANONICAL_BITMASKS: &[(TilePattern, u8)] = &[
    (TilePattern::Solid, 15),
    (TilePattern::OuterCorner, 1),
    (TilePattern::Edge, 3),
    (TilePattern::Diagonal, 5),
    (TilePattern::InnerCorner, 7),
];

fn convert_wang_set(
    wang_set: &ParsedWangSet,
    img: &image::RgbaImage,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
    config: &ConvertConfig,
) -> Result<TerrainTileDefinition, TiledToShapesError> {
    // Build a bitmask → tile_id map (first occurrence wins)
    let mut bitmask_to_tile: BTreeMap<u8, u32> = BTreeMap::new();
    for mapping in &wang_set.tiles {
        bitmask_to_tile
            .entry(mapping.bitmask)
            .or_insert(mapping.tile_id);
    }

    // For each TilePattern, pick canonical tile
    let mut variants: Vec<TileVariant> = Vec::with_capacity(5);

    for (pattern, canonical_bitmask) in CANONICAL_BITMASKS {
        // Collect all bitmasks in this pattern's group
        let group_bitmasks: Vec<u8> = (0u8..=15u8)
            .filter(|&b| bitmask_to_variant(b).0 == *pattern)
            .collect();

        // Pick canonical bitmask first, then any in the group
        let tile_id_opt = bitmask_to_tile.get(canonical_bitmask).copied().or_else(|| {
            group_bitmasks
                .iter()
                .find_map(|b| bitmask_to_tile.get(b).copied())
        });

        let shapes = if let Some(tile_id) = tile_id_opt {
            convert_tile_to_shapes(
                tile_id,
                img,
                tile_width,
                tile_height,
                columns,
                wang_set,
                config,
            )?
        } else {
            eprintln!(
                "[tiled-to-shapes] Warning: no tile for pattern {:?} in wang set '{}'",
                pattern, wang_set.id
            );
            Vec::new()
        };

        variants.push(TileVariant {
            pattern: *pattern,
            shapes,
            edge_ids: [EdgeId::NONE; 4],
        });
    }

    let gameplay_tags = passability_to_tags(&wang_set.passability);

    // Apply gameplay_tags to all shapes
    let variants: [TileVariant; 5] = variants
        .into_iter()
        .map(|mut v| {
            for shape in &mut v.shapes {
                shape.gameplay_tags.clone_from(&gameplay_tags);
            }
            v
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("exactly 5 variants");

    Ok(TerrainTileDefinition {
        name: wang_set.name.clone(),
        variants,
        priority: wang_set.priority,
        tint_range: TintRange {
            hue_shift_max: wang_set.hue_shift_max,
            brightness_shift_max: wang_set.brightness_shift_max,
        },
    })
}

fn convert_tile_to_shapes(
    tile_id: u32,
    img: &image::RgbaImage,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
    wang_set: &ParsedWangSet,
    config: &ConvertConfig,
) -> Result<Vec<AnnotatedShape>, TiledToShapesError> {
    let raw_pixels = extract_tile(img, tile_id, tile_width, tile_height, columns)?;

    let result = image_to_shapes(&raw_pixels, tile_width, tile_height, config);

    // Post-upscale dimensions
    let post_w = result.width as f32;
    let post_h = result.height as f32;

    let normalized = normalize_shapes(&result.shapes, post_w, post_h);

    let annotated: Vec<AnnotatedShape> = result
        .shapes
        .iter()
        .zip(normalized.iter())
        .map(|(shape, path)| {
            AnnotatedShape {
                path: path.clone(),
                color: [shape.color.r, shape.color.g, shape.color.b, shape.color.a],
                terrain_tag: wang_set.id.clone(),
                purpose: ShapePurpose::Fill,
                gameplay_tags: Vec::new(), // filled in by caller
            }
        })
        .collect();

    Ok(annotated)
}

/// Convert all TSX files to merged generated Rust code.
///
/// Intended for use in `build.rs`. Processes each TSX, merges all terrain
/// types into one `TerrainTileSet`, and returns generated Rust source as a
/// `String`. Conversion errors per-file are returned as `(path, error)` pairs
/// alongside the (possibly partial) generated code so callers can emit
/// `cargo:warning` lines without aborting the build.
///
/// Returns `(generated_code, warnings)`.
pub fn generate_build_code(
    tsx_paths: &[&Path],
    config: &ConvertConfig,
) -> (String, Vec<(String, String)>) {
    use std::collections::BTreeMap;

    use crate::codegen::generate_tileset_code;

    let mut merged_tiles = BTreeMap::new();
    let mut warnings = Vec::new();

    for &tsx in tsx_paths {
        match convert_tileset(tsx, config) {
            Ok(ts) => merged_tiles.extend(ts.tiles),
            Err(e) => {
                warnings.push((tsx.display().to_string(), e.to_string()));
            }
        }
    }

    let merged = TerrainTileSet {
        tiles: merged_tiles,
        adjacency_rules: Vec::new(),
    };
    let code = generate_tileset_code(&merged, "tileset", "assets/terrain/");
    (code, warnings)
}

/// Convert passability string to gameplay tags.
pub fn passability_to_tags(passability: &str) -> Vec<GameplayTag> {
    match passability {
        "solid" => vec![GameplayTag::Solid],
        "difficult" => vec![GameplayTag::DifficultTerrain],
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ParsedTileset, ParsedWangSet, WangTileMapping};
    use image::{Rgba, RgbaImage};

    fn solid_color_image(w: u32, h: u32, color: [u8; 4]) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for px in img.pixels_mut() {
            *px = Rgba(color);
        }
        img
    }

    /// Build a `ParsedWangSet` with a complete 16-tile mapping.
    fn full_wang_set() -> ParsedWangSet {
        // For a complete set, each of the 16 bitmasks 0-15 maps to a tile.
        // We use a single-color tilesheet, so all tiles look the same.
        let tiles: Vec<WangTileMapping> = (0u32..16u32)
            .map(|i| WangTileMapping {
                tile_id: i,
                bitmask: i as u8,
            })
            .collect();
        ParsedWangSet {
            name: "Grass".to_owned(),
            id: "grass".to_owned(),
            terrain_color: 1,
            passability: "passable".to_owned(),
            priority: 0,
            hue_shift_max: 5.0,
            brightness_shift_max: 0.05,
            tiles,
        }
    }

    /// Create a simple TSX string + tilesheet for end-to-end tests.
    fn make_test_tileset_and_image() -> (ParsedTileset, RgbaImage) {
        // 16 tiles of 16×16 pixels each, laid out in 4 columns × 4 rows
        let img = solid_color_image(64, 64, [0, 200, 50, 255]); // green
        let tileset = ParsedTileset {
            image_source: "test.png".to_owned(),
            tile_width: 16,
            tile_height: 16,
            columns: 4,
            wang_sets: vec![full_wang_set()],
        };
        (tileset, img)
    }

    #[test]
    fn when_full_wang_set_then_five_variants_populated() {
        // Arrange
        let (tileset, img) = make_test_tileset_and_image();
        let wang_set = &tileset.wang_sets[0];
        let config = default_convert_config();

        // Act
        let result = convert_wang_set(
            wang_set,
            &img,
            tileset.tile_width,
            tileset.tile_height,
            tileset.columns,
            &config,
        )
        .expect("convert should succeed");

        // Assert — all 5 variants exist and have the correct patterns
        assert_eq!(result.variants.len(), 5);
        let patterns: Vec<TilePattern> = result.variants.iter().map(|v| v.pattern).collect();
        assert!(patterns.contains(&TilePattern::Solid));
        assert!(patterns.contains(&TilePattern::OuterCorner));
        assert!(patterns.contains(&TilePattern::Edge));
        assert!(patterns.contains(&TilePattern::Diagonal));
        assert!(patterns.contains(&TilePattern::InnerCorner));
    }

    #[test]
    fn when_passability_solid_then_gameplay_tag_solid() {
        // Arrange
        let tags = passability_to_tags("solid");
        // Assert
        assert_eq!(tags, vec![GameplayTag::Solid]);
    }

    #[test]
    fn when_passability_difficult_then_gameplay_tag_difficult_terrain() {
        let tags = passability_to_tags("difficult");
        assert_eq!(tags, vec![GameplayTag::DifficultTerrain]);
    }

    #[test]
    fn when_passability_passable_then_no_tags() {
        let tags = passability_to_tags("passable");
        assert!(tags.is_empty());
    }

    #[test]
    fn when_convert_tileset_with_solid_terrain_then_correct_tag_count() {
        // Arrange — build a wang set with passability=solid + 5 canonical tiles
        let tiles: Vec<WangTileMapping> = vec![
            WangTileMapping {
                tile_id: 0,
                bitmask: 15,
            }, // Solid
            WangTileMapping {
                tile_id: 1,
                bitmask: 1,
            }, // OuterCorner
            WangTileMapping {
                tile_id: 2,
                bitmask: 3,
            }, // Edge
            WangTileMapping {
                tile_id: 3,
                bitmask: 5,
            }, // Diagonal
            WangTileMapping {
                tile_id: 4,
                bitmask: 7,
            }, // InnerCorner
        ];
        let mut wang = full_wang_set();
        wang.passability = "solid".to_owned();
        wang.tiles = tiles;

        let img = solid_color_image(16 * 5, 16, [100, 100, 100, 255]);
        let config = default_convert_config();

        // Act
        let result =
            convert_wang_set(&wang, &img, 16, 16, 5, &config).expect("convert should succeed");

        // Assert — every non-empty shape in every variant has Solid tag
        for variant in &result.variants {
            for shape in &variant.shapes {
                assert!(
                    shape.gameplay_tags.contains(&GameplayTag::Solid),
                    "shape missing Solid tag in variant {:?}",
                    variant.pattern
                );
            }
        }
    }

    #[test]
    fn when_generate_build_code_with_empty_paths_then_empty_tileset_code() {
        // Arrange
        let config = default_convert_config();

        // Act
        let (code, warnings) = generate_build_code(&[], &config);

        // Assert — no warnings (nothing to fail), code has tileset fn and empty BTreeMap
        assert!(warnings.is_empty());
        assert!(code.contains("pub fn tileset()"), "missing pub fn tileset");
        assert!(
            code.contains("BTreeMap::from"),
            "missing BTreeMap constructor"
        );
    }
}
