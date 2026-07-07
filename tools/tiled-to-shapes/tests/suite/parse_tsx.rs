#![allow(clippy::unwrap_used)]

use tiled_to_shapes::tsx_parser::parse_tsx_str;

/// @doc: Minimal valid TSX XML string parses successfully
#[test]
fn when_minimal_tsx_with_one_wangset_then_parses() {
    // Arrange
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="1" columns="1">
 <image source="sheet.png" width="16" height="16"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,0,0,0,0,0,0,0"/>
  </wangset>
 </wangsets>
</tileset>"##;

    // Act
    let result = parse_tsx_str(xml);

    // Assert
    assert!(
        result.is_ok(),
        "minimal valid TSX should parse: {:?}",
        result.err()
    );
    let tileset = result.unwrap();
    assert_eq!(tileset.image_source, "sheet.png", "image_source mismatch");
    assert_eq!(tileset.tile_width, 16, "tile_width mismatch");
    assert_eq!(tileset.tile_height, 16, "tile_height mismatch");
    assert_eq!(tileset.wang_sets.len(), 1, "expected 1 wangset");
}

/// @doc: No corner-type wangset returns NoCornerWangSets error
#[test]
fn when_tsx_with_no_wangsets_then_returns_no_corner_error() {
    // Arrange
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="empty" tilewidth="16" tileheight="16" tilecount="1" columns="1">
 <image source="sheet.png" width="16" height="16"/>
</tileset>"##;

    // Act
    let result = parse_tsx_str(xml);

    // Assert — parse_tsx_str performs validation and rejects empty wangsets
    assert!(result.is_err(), "no-wangset TSX should produce error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Wang"),
        "expected wangset-related error, got: {err}"
    );
}

/// @doc: Malformed XML input returns XmlError
#[test]
fn when_malformed_xml_then_returns_error() {
    // Arrange — unclosed tag is genuinely invalid XML
    let xml = r##"<tileset><wangset><open></tileset>"##;

    // Act
    let result = parse_tsx_str(xml);

    // Assert
    assert!(result.is_err(), "malformed XML should produce parse error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("XML error"),
        "expected XML error for malformed input, got: {err}"
    );
}
