#![allow(clippy::unwrap_used)]

use tiled_to_shapes::scaffold::scaffold_tsx_str;

/// @doc: Scaffold adds required properties to an empty corner wangset
#[test]
fn when_wangset_lacks_properties_then_scaffold_adds_them() {
    // Arrange — minimal corner wangset with no properties block
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="1" columns="1">
 <image source="sheet.png" width="16" height="16"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
  </wangset>
 </wangsets>
</tileset>"##;

    // Act
    let result = scaffold_tsx_str(xml);

    // Assert — should have injected a properties block
    assert!(
        result.contains("passability"),
        "scaffold should add passability property"
    );
    assert!(
        result.contains("priority"),
        "scaffold should add priority property"
    );
    assert!(
        result.contains("hue_shift_max"),
        "scaffold should add hue_shift_max property"
    );
    assert!(
        result.contains("brightness_shift_max"),
        "scaffold should add brightness_shift_max property"
    );
}

/// @doc: Scaffold is idempotent — already-complete TSX is unchanged
#[test]
fn when_all_properties_already_present_then_scaffold_unchanged() {
    // Arrange — wangset already has all required properties
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="1" columns="1">
 <image source="sheet.png" width="16" height="16"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <properties>
    <property name="passability" type="string" value="passable"/>
    <property name="priority" type="int" value="0"/>
    <property name="hue_shift_max" type="float" value="5.0"/>
    <property name="brightness_shift_max" type="float" value="0.05"/>
   </properties>
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
  </wangset>
 </wangsets>
</tileset>"##;

    // Act
    let result = scaffold_tsx_str(xml);

    // Assert — unchanged
    assert_eq!(
        result, xml,
        "already-complete TSX should be unchanged by scaffold"
    );
}

/// @doc: No corner wangsets → no changes
#[test]
fn when_no_corner_wangsets_then_scaffold_noop() {
    // Arrange
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="empty" tilewidth="16" tileheight="16" tilecount="1" columns="1">
 <image source="sheet.png" width="16" height="16"/>
</tileset>"#;

    // Act
    let result = scaffold_tsx_str(xml);

    // Assert
    assert_eq!(result, xml, "TSX with no wangsets should be unchanged");
}
