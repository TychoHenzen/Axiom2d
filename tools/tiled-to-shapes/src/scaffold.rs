use std::fmt::Write as _;
use std::path::Path;

use crate::types::{ParsedTileset, TiledToShapesError};

/// Required default properties to scaffold when missing.
static REQUIRED_PROPERTIES: &[(&str, &str, &str)] = &[
    ("passability", "string", "passable"),
    ("priority", "int", "0"),
    ("hue_shift_max", "float", "5.0"),
    ("brightness_shift_max", "float", "0.05"),
];

/// Add missing default properties to corner-type Wang sets in the TSX file.
///
/// Returns `true` if the file was modified. Only modifies the file on disk
/// when `--scaffold` is explicitly requested.
pub fn scaffold_tsx(path: &Path, _tileset: &ParsedTileset) -> Result<bool, TiledToShapesError> {
    let original = std::fs::read_to_string(path)?;
    let modified = scaffold_tsx_str(&original);
    if modified == original {
        return Ok(false);
    }
    std::fs::write(path, &modified)?;
    Ok(true)
}

/// Scaffold a TSX XML string in memory (no file I/O). Exposed for testing.
///
/// Strategy: string-based injection rather than full re-serialization, to
/// preserve formatting. For each `<wangset type="corner">` element, scan its
/// `<properties>` block and add missing properties before `</properties>`.
/// If no `<properties>` block exists, inject one after the `<wangset ...>` tag.
pub fn scaffold_tsx_str(xml: &str) -> String {
    let mut result = xml.to_owned();

    // Find all corner wangset regions and scaffold each.
    let mut search_from = 0;
    loop {
        // Find next corner wangset opening tag
        let Some(wangset_start) = find_corner_wangset(&result, search_from) else {
            break;
        };

        // Find the end of this wangset element
        let wangset_end = find_wangset_end(&result, wangset_start);

        // Extract the wangset region
        let region = &result[wangset_start..wangset_end];

        // Determine which properties are already present
        let missing: Vec<(&str, &str, &str)> = REQUIRED_PROPERTIES
            .iter()
            .copied()
            .filter(|(name, _, _)| !has_property(region, name))
            .collect();

        if missing.is_empty() {
            search_from = wangset_end;
            continue;
        }

        // Build property lines to inject
        let indent = "    "; // 4 spaces default indent for properties
        let mut prop_lines = String::new();
        for (name, r#type, value) in &missing {
            let _ = writeln!(
                prop_lines,
                "{indent} <property name=\"{name}\" type=\"{type}\" value=\"{value}\"/>"
            );
        }

        // Inject into existing <properties> block or create one
        let inject_pos = if let Some(props_close) = find_properties_close(region) {
            // Insert before </properties>
            wangset_start + props_close
        } else if let Some(open_end) = find_tag_close(&result[wangset_start..]) {
            // No <properties> block — inject one after the opening <wangset ...> tag
            let tag_end = wangset_start + open_end;
            let new_block = format!("\n{indent}<properties>\n{prop_lines}{indent}</properties>");
            result.insert_str(tag_end, &new_block);
            search_from = tag_end + new_block.len();
            continue;
        } else {
            search_from = wangset_end;
            continue;
        };

        result.insert_str(inject_pos, &prop_lines);
        search_from = inject_pos + prop_lines.len();
    }

    result
}

/// Find the byte offset of the next `<wangset` element with `type="corner"`.
fn find_corner_wangset(xml: &str, from: usize) -> Option<usize> {
    let search = &xml[from..];
    let mut pos = 0;
    while let Some(offset) = search[pos..].find("<wangset") {
        let abs = from + pos + offset;
        // Extract this tag's content up to the closing `>`
        let tag_end = xml[abs..].find('>').map_or(xml.len(), |e| abs + e + 1);
        let tag = &xml[abs..tag_end];
        if tag.contains("type=\"corner\"") {
            return Some(abs);
        }
        pos += offset + 1;
    }
    None
}

/// Find the byte offset just past `</wangset>` that closes the wangset at `from`.
fn find_wangset_end(xml: &str, from: usize) -> usize {
    xml[from..]
        .find("</wangset>")
        .map_or(xml.len(), |e| from + e + "</wangset>".len())
}

/// Check if a `<properties>` block contains a property with the given name.
fn has_property(xml_region: &str, name: &str) -> bool {
    xml_region.contains(&format!("name=\"{name}\""))
}

/// Find the byte offset of `</properties>` within `region`.
fn find_properties_close(region: &str) -> Option<usize> {
    region.find("</properties>")
}

/// Find the offset past the first `>` in the string (end of the opening tag).
fn find_tag_close(s: &str) -> Option<usize> {
    s.find('>').map(|i| i + 1)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const TSX_NO_PROPS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="16" columns="4">
 <image source="terrain.png" width="64" height="64"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
  </wangset>
 </wangsets>
</tileset>"##;

    const TSX_WITH_SOME_PROPS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="16" columns="4">
 <image source="terrain.png" width="64" height="64"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <properties>
    <property name="passability" type="string" value="solid"/>
   </properties>
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
  </wangset>
 </wangsets>
</tileset>"##;

    const TSX_ALL_PROPS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="16" columns="4">
 <image source="terrain.png" width="64" height="64"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <properties>
    <property name="passability" type="string" value="solid"/>
    <property name="priority" type="int" value="1"/>
    <property name="hue_shift_max" type="float" value="3.0"/>
    <property name="brightness_shift_max" type="float" value="0.1"/>
   </properties>
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
  </wangset>
 </wangsets>
</tileset>"##;

    #[test]
    fn when_no_properties_then_adds_missing() {
        // Arrange
        let xml = TSX_NO_PROPS;
        // Act
        let result = scaffold_tsx_str(xml);
        // Assert — all 4 required properties should now be present
        assert!(
            result.contains("name=\"passability\""),
            "passability missing"
        );
        assert!(result.contains("name=\"priority\""), "priority missing");
        assert!(
            result.contains("name=\"hue_shift_max\""),
            "hue_shift_max missing"
        );
        assert!(
            result.contains("name=\"brightness_shift_max\""),
            "brightness_shift_max missing"
        );
    }

    #[test]
    fn when_existing_property_then_not_overwritten() {
        // Arrange — passability already set to "solid"
        let xml = TSX_WITH_SOME_PROPS;
        // Act
        let result = scaffold_tsx_str(xml);
        // Assert — "solid" is preserved, not replaced with "passable"
        assert!(
            result.contains("value=\"solid\""),
            "existing value was overwritten"
        );
        // Other missing properties are added
        assert!(result.contains("name=\"priority\""), "priority missing");
    }

    #[test]
    fn when_all_properties_present_then_returns_false_equivalent() {
        // Arrange — all properties present
        let xml = TSX_ALL_PROPS;
        // Act
        let result = scaffold_tsx_str(xml);
        // Assert — no change
        assert_eq!(result, xml);
    }
}
