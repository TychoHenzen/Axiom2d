#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::OnceLock;

use image::{Rgba, RgbaImage};
use tiled_to_shapes::{
    codegen::generate_tileset_code,
    pipeline::{convert_tileset, default_convert_config},
};

/// Returns path to the test fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Cached fixture paths — written once, shared by all tests. Avoids races
/// between parallel tests that would otherwise read a PNG mid-truncation.
static FIXTURES: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();

/// Returns cached (`tsx_path`, `png_path`) fixture pair, creating files on first call.
fn ensure_fixtures() -> &'static (PathBuf, PathBuf) {
    FIXTURES.get_or_init(|| {
        let dir = fixtures_dir();
        std::fs::create_dir_all(&dir).expect("failed to create fixtures dir");
        let tsx_path = dir.join("test_tileset.tsx");
        let png_path = dir.join("test_tileset.png");
        write_test_png(&png_path);
        write_test_tsx(&tsx_path, "test_tileset.png");
        (tsx_path, png_path)
    })
}

/// Write the test PNG fixture (4×4 tiles of 16×16 pixels each = 64×64 image).
/// Called once per test run; safe to call multiple times (idempotent).
fn write_test_png(path: &std::path::Path) {
    let mut img = RgbaImage::new(64, 64);
    // Tile 0 (0,0) … tile 15 (3,3) — each a slightly different green shade
    for tile_id in 0u32..16u32 {
        let col = tile_id % 4;
        let row = tile_id / 4;
        let hue = (tile_id * 16) as u8;
        let color = Rgba([hue, 200u8.saturating_sub(hue / 2), 50, 255]);
        for py in row * 16..(row + 1) * 16 {
            for px in col * 16..(col + 1) * 16 {
                img.put_pixel(px, py, color);
            }
        }
    }
    img.save(path).expect("failed to save test PNG");
}

/// Write a minimal TSX fixture with one corner Wang set (16 tiles, IDs 0-15).
fn write_test_tsx(path: &std::path::Path, png_name: &str) {
    let mut wangtiles = String::new();
    // Map bitmask 0..15 → tile_id 0..15 (one-to-one for test)
    for bitmask in 0u8..16u8 {
        // Encode as wangid string from bitmask (NE=1, SE=2, SW=4, NW=8)
        let ne = u8::from(bitmask & 1 != 0);
        let se = u8::from(bitmask & 2 != 0);
        let sw = u8::from(bitmask & 4 != 0);
        let nw = u8::from(bitmask & 8 != 0);
        let _ = writeln!(
            wangtiles,
            "   <wangtile tileid=\"{bitmask}\" wangid=\"0,{ne},0,{se},0,{sw},0,{nw}\"/>"
        );
    }

    let xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="16" columns="4">
 <image source="{png_name}" width="64" height="64"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <properties>
    <property name="passability" type="string" value="passable"/>
    <property name="priority" type="int" value="0"/>
    <property name="hue_shift_max" type="float" value="5.0"/>
    <property name="brightness_shift_max" type="float" value="0.05"/>
   </properties>
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
{wangtiles}  </wangset>
 </wangsets>
</tileset>"##
    );

    std::fs::write(path, xml).expect("failed to write test TSX");
}

#[test]
fn integration_when_valid_tsx_then_produces_terrain_tileset() {
    // Arrange
    let (ref tsx_path, _) = *ensure_fixtures();
    let config = default_convert_config();

    // Act
    let result = convert_tileset(tsx_path, &config).expect("conversion should succeed");

    // Assert — at least 1 terrain type
    assert!(!result.tiles.is_empty(), "no terrain types in result");

    // Assert — grass terrain type present
    let grass = result
        .tiles
        .get("grass")
        .expect("grass terrain type missing");

    // Assert — 5 variants
    assert_eq!(grass.variants.len(), 5, "expected 5 variants");
}

#[test]
fn integration_when_valid_tsx_then_codegen_compiles_structurally() {
    // Arrange
    let (ref tsx_path, _) = *ensure_fixtures();
    let config = default_convert_config();

    let tileset = convert_tileset(tsx_path, &config).expect("conversion should succeed");

    // Act
    let code = generate_tileset_code(&tileset, "tileset", "test_tileset.tsx");

    // Assert — structural checks on the generated code
    assert!(code.contains("pub fn tileset()"), "missing pub fn");
    assert!(code.contains("TerrainTileSet"), "missing TerrainTileSet");
    assert!(code.contains("use terrain::prelude::*"), "missing import");
    assert!(code.contains("\"grass\""), "missing grass key");
    assert!(
        !code.contains("todo!()") && !code.contains("unimplemented!()"),
        "generated code contains stub markers"
    );
    assert!(!code.contains("FIXME"), "generated code contains FIXME");
    assert!(!code.contains("TODO"), "generated code contains TODO");
    assert!(!code.contains("panic"), "generated code contains panic");
    assert!(
        code.ends_with('\n'),
        "generated code should end with newline"
    );
}

#[test]
fn integration_fixture_files_exist() {
    // Arrange + Act
    let (tsx_path, png_path) = ensure_fixtures();

    // Assert
    assert!(
        tsx_path.exists(),
        "TSX fixture missing at {}",
        tsx_path.display()
    );
    assert!(
        png_path.exists(),
        "PNG fixture missing at {}",
        png_path.display()
    );
}

// ---------------------------------------------------------------------------
// Error-recovery tests
// ---------------------------------------------------------------------------

/// @doc: Non-existent TSX file returns an IO error, not a panic
#[test]
fn integration_when_tsx_missing_then_returns_io_error() {
    // Arrange
    let config = default_convert_config();
    let nonexistent = fixtures_dir().join("does_not_exist.tsx");

    // Act
    let result = convert_tileset(&nonexistent, &config);

    // Assert
    assert!(
        result.is_err(),
        "conversion should fail for non-existent TSX file"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("IO error"),
        "expected IO error for missing file, got: {err}"
    );
}

/// @doc: TSX referencing non-existent PNG returns `TilesheetNotFound`
#[test]
fn integration_when_png_missing_then_returns_tilesheet_not_found() {
    // Arrange
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).expect("failed to create fixtures dir");
    let tsx_path = dir.join("orphan_tileset.tsx");
    write_test_tsx(&tsx_path, "missing.png");

    let config = default_convert_config();

    // Act
    let result = convert_tileset(&tsx_path, &config);

    // Assert
    assert!(
        result.is_err(),
        "conversion should fail when tilesheet PNG is missing"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("tilesheet not found"),
        "expected TilesheetNotFound error, got: {err}"
    );
}

/// @doc: Garbage XML input returns an XML parse error, not a panic
#[test]
fn integration_when_malformed_xml_then_returns_xml_error() {
    // Arrange
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).expect("failed to create fixtures dir");
    let tsx_path = dir.join("malformed.tsx");
    // Unclosed element is genuinely invalid XML that quick-xml cannot parse
    std::fs::write(&tsx_path, "<tileset><open></tileset>").expect("failed to write malformed TSX");

    let config = default_convert_config();

    // Act
    let result = convert_tileset(&tsx_path, &config);

    // Assert
    assert!(
        result.is_err(),
        "conversion should fail for malformed XML input"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("XML error"),
        "expected XML error for malformed file, got: {err}"
    );
}

/// @doc: TSX with no wangsets returns `NoCornerWangSets` error
#[test]
fn integration_when_no_wangsets_then_returns_no_corner_wang_sets() {
    // Arrange
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).expect("failed to create fixtures dir");
    let png_path = dir.join("nowang.png");
    write_test_png(&png_path);

    let tsx_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="nowang" tilewidth="16" tileheight="16" tilecount="1" columns="1">
 <image source="nowang.png" width="64" height="64"/>
</tileset>"#
        .to_string();
    let tsx_path = dir.join("nowang.tsx");
    std::fs::write(&tsx_path, tsx_xml).expect("failed to write TSX");

    let config = default_convert_config();

    // Act
    let result = convert_tileset(&tsx_path, &config);

    // Assert
    assert!(
        result.is_err(),
        "conversion should fail when no wangsets are present"
    );
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("wang"),
        "expected wangset-related error, got: {err_str}"
    );
}

/// @doc: Converted tileset has all 5 variant patterns present
#[test]
fn integration_when_valid_tsx_then_all_tile_patterns_present() {
    // Arrange
    let (ref tsx_path, _) = *ensure_fixtures();
    let config = default_convert_config();

    // Act
    let result = convert_tileset(tsx_path, &config).expect("conversion should succeed");

    // Assert — verify all 5 patterns are represented in the grass variants
    let grass = result
        .tiles
        .get("grass")
        .expect("grass terrain type missing");
    let patterns: Vec<_> = grass.variants.iter().map(|v| v.pattern).collect();
    use terrain::prelude::TilePattern;
    assert!(
        patterns.contains(&TilePattern::Solid),
        "missing Solid pattern; patterns: {patterns:?}"
    );
    assert!(
        patterns.contains(&TilePattern::OuterCorner),
        "missing OuterCorner pattern"
    );
    assert!(
        patterns.contains(&TilePattern::Edge),
        "missing Edge pattern"
    );
    assert!(
        patterns.contains(&TilePattern::Diagonal),
        "missing Diagonal pattern"
    );
    assert!(
        patterns.contains(&TilePattern::InnerCorner),
        "missing InnerCorner pattern"
    );
}
