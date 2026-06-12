use thiserror::Error;

/// All errors this tool can produce.
#[derive(Debug, Error)]
pub enum TiledToShapesError {
    /// TSX file not found or not readable.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Invalid XML in TSX file.
    #[error("XML error: {0}")]
    XmlError(#[from] quick_xml::Error),
    /// Tilesheet image not found at the path referenced by the TSX.
    #[error("tilesheet not found: {0}")]
    TilesheetNotFound(String),
    /// Wang set tile ID is out of bounds for the tile grid.
    #[error("tile id {tile_id} out of bounds (max {max_id})")]
    TileIdOutOfBounds { tile_id: u32, max_id: u32 },
    /// No corner-type Wang sets found in the TSX.
    #[error("no corner-type Wang sets found in TSX")]
    NoCornerWangSets,
    /// img-to-shape conversion failed for a tile.
    #[error("conversion failed for tile {tile_id}: {reason}")]
    ConversionFailed { tile_id: u32, reason: String },
    /// Image decoding error.
    #[error("image error: {0}")]
    ImageError(#[from] image::ImageError),
}

/// Result of parsing a TSX file.
#[derive(Clone, Debug)]
pub struct ParsedTileset {
    /// Relative path to the tilesheet PNG.
    pub image_source: String,
    /// Pixel width of each tile.
    pub tile_width: u32,
    /// Pixel height of each tile.
    pub tile_height: u32,
    /// Number of tile columns in the tilesheet.
    pub columns: u32,
    /// All corner-type Wang sets found in the TSX.
    pub wang_sets: Vec<ParsedWangSet>,
}

/// One corner-type Wang set parsed from the TSX.
#[derive(Clone, Debug)]
pub struct ParsedWangSet {
    /// Display name (e.g. "Grass").
    pub name: String,
    /// `snake_case` identifier for Rust codegen (falls back to name if class absent).
    pub id: String,
    /// Wang color index that means "this terrain is present" (default 1).
    pub terrain_color: u8,
    /// Passability string: "passable", "solid", or "difficult".
    pub passability: String,
    /// Render priority (higher = rendered on top).
    pub priority: u8,
    /// Hue shift range in degrees.
    pub hue_shift_max: f32,
    /// Brightness multiplier range.
    pub brightness_shift_max: f32,
    /// Tile-to-bitmask mappings for tiles in this Wang set.
    pub tiles: Vec<WangTileMapping>,
}

/// Maps a tile ID to its corner16 bitmask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WangTileMapping {
    /// Local 0-based tile ID within the tilesheet grid.
    pub tile_id: u32,
    /// Corner16 bitmask: NE=1, SE=2, SW=4, NW=8.
    pub bitmask: u8,
}

/// Convert a name string to `snake_case` for use as a Rust identifier.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        } else {
            // Replace non-alphanumeric with underscore, collapse runs
            if !result.ends_with('_') && !result.is_empty() {
                result.push('_');
            }
        }
    }
    // Trim trailing underscore
    result.trim_end_matches('_').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_ascii_name_then_snake_case() {
        assert_eq!(to_snake_case("GrassLand"), "grassland");
        assert_eq!(to_snake_case("Deep Water"), "deep_water");
        assert_eq!(to_snake_case("stone-path"), "stone_path");
        assert_eq!(to_snake_case("Sand"), "sand");
    }
}
