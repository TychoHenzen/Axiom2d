use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::types::{
    ParsedTileset, ParsedWangSet, TiledToShapesError, WangTileMapping, to_snake_case,
};

/// Parse a Tiled TSX tileset file.
///
/// Only corner-type (`type="corner"`) Wang sets are processed.
/// Edge and mixed types are skipped with an eprintln warning.
pub fn parse_tsx(path: &Path) -> Result<ParsedTileset, TiledToShapesError> {
    let xml = std::fs::read_to_string(path)?;
    parse_tsx_str(&xml)
}

/// Parse TSX XML from a string. Exposed for testing.
pub fn parse_tsx_str(xml: &str) -> Result<ParsedTileset, TiledToShapesError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut tile_width: u32 = 16;
    let mut tile_height: u32 = 16;
    let mut columns: u32 = 0;
    let mut image_source = String::new();
    let mut wang_sets: Vec<ParsedWangSet> = Vec::new();

    // Parser state
    let mut in_wangsets = false;
    let mut current_wang: Option<WangSetBuilder> = None;
    let mut in_wang_properties = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let tag = std::str::from_utf8(e.name().0).unwrap_or("").to_owned();
                match tag.as_str() {
                    "tileset" => {
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.0).unwrap_or("").to_owned();
                            let val = attr.unescape_value().unwrap_or_default().into_owned();
                            match key.as_str() {
                                "tilewidth" => tile_width = val.parse().unwrap_or(16),
                                "tileheight" => tile_height = val.parse().unwrap_or(16),
                                "columns" => columns = val.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                    "image" => {
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.0).unwrap_or("").to_owned();
                            let val = attr.unescape_value().unwrap_or_default().into_owned();
                            if key == "source" {
                                image_source = val;
                            }
                        }
                    }
                    "wangsets" => {
                        in_wangsets = true;
                    }
                    "wangset" if in_wangsets => {
                        let mut name = String::new();
                        let mut class = String::new();
                        let mut wang_type = String::new();
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.0).unwrap_or("").to_owned();
                            let val = attr.unescape_value().unwrap_or_default().into_owned();
                            match key.as_str() {
                                "name" => name = val,
                                "class" => class = val,
                                "type" => wang_type = val,
                                _ => {}
                            }
                        }
                        if wang_type == "corner" {
                            let id = if class.is_empty() {
                                to_snake_case(&name)
                            } else {
                                to_snake_case(&class)
                            };
                            current_wang = Some(WangSetBuilder {
                                name,
                                id,
                                terrain_color: 1,
                                passability: "passable".to_owned(),
                                priority: 0,
                                hue_shift_max: 5.0,
                                brightness_shift_max: 0.05,
                                tiles: Vec::new(),
                            });
                        } else {
                            eprintln!(
                                "[tiled-to-shapes] Skipping Wang set of type '{wang_type}' (only 'corner' supported)"
                            );
                            current_wang = None;
                        }
                    }
                    "properties" if current_wang.is_some() => {
                        in_wang_properties = true;
                    }
                    "property" if in_wang_properties => {
                        if let Some(ref mut wang) = current_wang {
                            let mut prop_name = String::new();
                            let mut prop_val = String::new();
                            for attr in e.attributes().flatten() {
                                let key = std::str::from_utf8(attr.key.0).unwrap_or("").to_owned();
                                let val = attr.unescape_value().unwrap_or_default().into_owned();
                                match key.as_str() {
                                    "name" => prop_name = val,
                                    "value" => prop_val = val,
                                    _ => {}
                                }
                            }
                            match prop_name.as_str() {
                                "passability" => wang.passability = prop_val,
                                "priority" => wang.priority = prop_val.parse().unwrap_or(0),
                                "hue_shift_max" => {
                                    wang.hue_shift_max = prop_val.parse().unwrap_or(5.0);
                                }
                                "brightness_shift_max" => {
                                    wang.brightness_shift_max = prop_val.parse().unwrap_or(0.05);
                                }
                                "terraincolor" => {
                                    wang.terrain_color = prop_val.parse().unwrap_or(1);
                                }
                                _ => {}
                            }
                        }
                    }
                    "wangcolor" if current_wang.is_some() => {
                        // terrain_color is the index of this wangcolor element (1-based).
                        // CardCleaner reads a "terraincolor" property; here we just use
                        // the first wangcolor (index 1) as the terrain indicator unless
                        // overridden by a "terraincolor" property.
                    }
                    "wangtile" if current_wang.is_some() => {
                        if let Some(ref mut wang) = current_wang {
                            let mut tile_id: u32 = 0;
                            let mut wangid_str = String::new();
                            for attr in e.attributes().flatten() {
                                let key = std::str::from_utf8(attr.key.0).unwrap_or("").to_owned();
                                let val = attr.unescape_value().unwrap_or_default().into_owned();
                                match key.as_str() {
                                    "tileid" => tile_id = val.parse().unwrap_or(0),
                                    "wangid" => wangid_str = val,
                                    _ => {}
                                }
                            }
                            let bitmask = wangid_to_bitmask(&wangid_str, wang.terrain_color);
                            wang.tiles.push(WangTileMapping { tile_id, bitmask });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let tag = std::str::from_utf8(e.name().0).unwrap_or("").to_owned();
                match tag.as_str() {
                    "wangsets" => {
                        in_wangsets = false;
                    }
                    "wangset" => {
                        if let Some(builder) = current_wang.take() {
                            wang_sets.push(builder.build());
                        }
                    }
                    "properties" => {
                        in_wang_properties = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(TiledToShapesError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    if wang_sets.is_empty() {
        return Err(TiledToShapesError::NoCornerWangSets);
    }

    Ok(ParsedTileset {
        image_source,
        tile_width,
        tile_height,
        columns,
        wang_sets,
    })
}

/// Convert a wangid string to a corner16 bitmask.
///
/// wangid: 8 comma-separated values `"e0,c0,e1,c1,e2,c2,e3,c3"`.
/// Corner indices: 1=NE, 3=SE, 5=SW, 7=NW.
/// A corner value equal to `terrain_color` means "terrain present".
fn wangid_to_bitmask(wangid: &str, terrain_color: u8) -> u8 {
    let parts: Vec<u8> = wangid
        .split(',')
        .map(|s| s.trim().parse().unwrap_or(0))
        .collect();
    if parts.len() < 8 {
        return 0;
    }
    let mut bitmask: u8 = 0;
    if parts[1] == terrain_color {
        bitmask |= 1;
    } // NE
    if parts[3] == terrain_color {
        bitmask |= 2;
    } // SE
    if parts[5] == terrain_color {
        bitmask |= 4;
    } // SW
    if parts[7] == terrain_color {
        bitmask |= 8;
    } // NW
    bitmask
}

struct WangSetBuilder {
    name: String,
    id: String,
    terrain_color: u8,
    passability: String,
    priority: u8,
    hue_shift_max: f32,
    brightness_shift_max: f32,
    tiles: Vec<WangTileMapping>,
}

impl WangSetBuilder {
    fn build(self) -> ParsedWangSet {
        ParsedWangSet {
            name: self.name,
            id: self.id,
            terrain_color: self.terrain_color,
            passability: self.passability,
            priority: self.priority,
            hue_shift_max: self.hue_shift_max,
            brightness_shift_max: self.brightness_shift_max,
            tiles: self.tiles,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_TSX: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="256" columns="16">
 <image source="terrain.png" width="256" height="256"/>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <properties>
    <property name="passability" type="string" value="passable"/>
    <property name="priority" type="int" value="2"/>
   </properties>
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="1" wangid="0,1,0,0,0,0,0,0"/>
   <wangtile tileid="2" wangid="0,0,0,0,0,0,0,1"/>
  </wangset>
 </wangsets>
</tileset>"##;

    #[test]
    fn when_minimal_tsx_then_correct_dimensions() {
        let result = parse_tsx_str(MINIMAL_TSX).expect("parse should succeed");
        // Arrange / Act done in parse_tsx_str above
        // Assert
        assert_eq!(result.tile_width, 16);
        assert_eq!(result.tile_height, 16);
        assert_eq!(result.columns, 16);
        assert_eq!(result.image_source, "terrain.png");
    }

    #[test]
    fn when_wangtile_entries_then_correct_bitmasks() {
        let result = parse_tsx_str(MINIMAL_TSX).expect("parse should succeed");
        assert_eq!(result.wang_sets.len(), 1);
        let wang = &result.wang_sets[0];
        assert_eq!(wang.tiles.len(), 3);
        // tile 0: all corners = bitmask 15
        let t0 = wang
            .tiles
            .iter()
            .find(|t| t.tile_id == 0)
            .expect("tile 0 missing");
        assert_eq!(t0.bitmask, 15);
        // tile 1: only NE = bitmask 1
        let t1 = wang
            .tiles
            .iter()
            .find(|t| t.tile_id == 1)
            .expect("tile 1 missing");
        assert_eq!(t1.bitmask, 1);
        // tile 2: only NW = bitmask 8
        let t2 = wang
            .tiles
            .iter()
            .find(|t| t.tile_id == 2)
            .expect("tile 2 missing");
        assert_eq!(t2.bitmask, 8);
    }

    #[test]
    fn when_all_corners_wangid_then_bitmask_15() {
        // "0,1,0,1,0,1,0,1" => NE=1, SE=1, SW=1, NW=1 => all 4 corners => 1+2+4+8=15
        let bitmask = wangid_to_bitmask("0,1,0,1,0,1,0,1", 1);
        assert_eq!(bitmask, 15);
    }

    #[test]
    fn when_only_ne_wangid_then_bitmask_1() {
        // "0,1,0,0,0,0,0,0" => only NE=1 => bitmask 1
        let bitmask = wangid_to_bitmask("0,1,0,0,0,0,0,0", 1);
        assert_eq!(bitmask, 1);
    }

    #[test]
    fn when_custom_properties_then_extracted_correctly() {
        let result = parse_tsx_str(MINIMAL_TSX).expect("parse should succeed");
        let wang = &result.wang_sets[0];
        assert_eq!(wang.passability, "passable");
        assert_eq!(wang.priority, 2);
        assert_eq!(wang.id, "grass");
    }

    #[test]
    fn when_non_corner_wang_sets_then_skipped() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" name="test" tilewidth="16" tileheight="16" tilecount="16" columns="4">
 <image source="terrain.png" width="64" height="64"/>
 <wangsets>
  <wangset name="EdgeOnly" type="edge" tile="-1">
   <wangcolor name="EdgeOnly" color="#ff0000" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
  </wangset>
  <wangset name="Grass" type="corner" tile="-1" class="grass">
   <wangcolor name="Grass" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
  </wangset>
 </wangsets>
</tileset>"##;
        let result = parse_tsx_str(xml).expect("parse should succeed");
        // Only corner sets included
        assert_eq!(result.wang_sets.len(), 1);
        assert_eq!(result.wang_sets[0].id, "grass");
    }
}
