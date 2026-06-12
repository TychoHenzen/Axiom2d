use std::collections::BTreeMap;

use engine_render::prelude::PathCommand;
use glam::Vec2;

use super::{
    AdjacencyRule, AnnotatedShape, EdgeId, ShapePurpose, TerrainTileDefinition, TerrainTileSet,
    TilePattern, TileVariant, TintRange,
};

/// Constructs a minimal example tileset with 2 terrain types ("grass" and "stone"),
/// each with 5 variants containing simple hand-authored shapes.
#[must_use]
pub fn example_tileset() -> TerrainTileSet {
    TerrainTileSet {
        tiles: BTreeMap::from([
            ("grass".to_owned(), grass_definition()),
            ("stone".to_owned(), stone_definition()),
        ]),
        adjacency_rules: vec![AdjacencyRule {
            from: "grass".to_owned(),
            to: "stone".to_owned(),
            allowed: true,
            weight: 1.0,
        }],
    }
}

fn grass_definition() -> TerrainTileDefinition {
    let green = [0.2, 0.6, 0.15, 1.0];
    TerrainTileDefinition {
        name: "grass".to_owned(),
        variants: [
            make_solid(green, "grass"),
            make_outer_corner(green, "grass"),
            make_edge(green, "grass"),
            make_diagonal(green, "grass"),
            make_inner_corner(green, "grass"),
        ],
        priority: 0,
        tint_range: TintRange {
            hue_shift_max: 5.0,
            brightness_shift_max: 0.05,
        },
    }
}

fn stone_definition() -> TerrainTileDefinition {
    let grey = [0.5, 0.48, 0.45, 1.0];
    TerrainTileDefinition {
        name: "stone".to_owned(),
        variants: [
            make_solid(grey, "stone"),
            make_outer_corner(grey, "stone"),
            make_edge(grey, "stone"),
            make_diagonal(grey, "stone"),
            make_inner_corner(grey, "stone"),
        ],
        priority: 1,
        tint_range: TintRange {
            hue_shift_max: 2.0,
            brightness_shift_max: 0.03,
        },
    }
}

// -- Shape builders ---------------------------------------------------------
// All shapes use normalized [0,1]² tile space.

/// Solid fill: full-tile rectangle.
fn make_solid(color: [f32; 4], tag: &str) -> TileVariant {
    TileVariant {
        pattern: TilePattern::Solid,
        shapes: vec![rect_shape(Vec2::ZERO, Vec2::ONE, color, tag)],
        edge_ids: [EdgeId::NONE; 4],
    }
}

/// Outer corner: triangle in bottom-left corner (SW).
fn make_outer_corner(color: [f32; 4], tag: &str) -> TileVariant {
    TileVariant {
        pattern: TilePattern::OuterCorner,
        shapes: vec![triangle_shape(
            Vec2::ZERO,
            Vec2::new(0.5, 0.0),
            Vec2::new(0.0, 0.5),
            color,
            tag,
        )],
        edge_ids: [EdgeId::NONE, EdgeId::NONE, EdgeId(1), EdgeId(1)],
    }
}

/// Edge: bottom half rectangle.
fn make_edge(color: [f32; 4], tag: &str) -> TileVariant {
    TileVariant {
        pattern: TilePattern::Edge,
        shapes: vec![rect_shape(Vec2::ZERO, Vec2::new(1.0, 0.5), color, tag)],
        edge_ids: [EdgeId::NONE, EdgeId(2), EdgeId(2), EdgeId::NONE],
    }
}

/// Diagonal: two triangles in opposite corners (SW and NE).
fn make_diagonal(color: [f32; 4], tag: &str) -> TileVariant {
    TileVariant {
        pattern: TilePattern::Diagonal,
        shapes: vec![
            triangle_shape(
                Vec2::ZERO,
                Vec2::new(0.5, 0.0),
                Vec2::new(0.0, 0.5),
                color,
                tag,
            ),
            triangle_shape(
                Vec2::ONE,
                Vec2::new(0.5, 1.0),
                Vec2::new(1.0, 0.5),
                color,
                tag,
            ),
        ],
        edge_ids: [EdgeId(3), EdgeId(3), EdgeId(3), EdgeId(3)],
    }
}

/// Inner corner: L-shaped region (full tile minus top-right triangle).
fn make_inner_corner(color: [f32; 4], tag: &str) -> TileVariant {
    TileVariant {
        pattern: TilePattern::InnerCorner,
        shapes: vec![AnnotatedShape {
            path: vec![
                PathCommand::MoveTo(Vec2::ZERO),
                PathCommand::LineTo(Vec2::new(1.0, 0.0)),
                PathCommand::LineTo(Vec2::new(1.0, 0.5)),
                PathCommand::LineTo(Vec2::new(0.5, 1.0)),
                PathCommand::LineTo(Vec2::new(0.0, 1.0)),
                PathCommand::Close,
            ],
            color,
            terrain_tag: tag.to_owned(),
            purpose: ShapePurpose::Fill,
            gameplay_tags: Vec::new(),
        }],
        edge_ids: [EdgeId(4), EdgeId(4), EdgeId(4), EdgeId(4)],
    }
}

fn rect_shape(min: Vec2, max: Vec2, color: [f32; 4], tag: &str) -> AnnotatedShape {
    AnnotatedShape {
        path: vec![
            PathCommand::MoveTo(min),
            PathCommand::LineTo(Vec2::new(max.x, min.y)),
            PathCommand::LineTo(max),
            PathCommand::LineTo(Vec2::new(min.x, max.y)),
            PathCommand::Close,
        ],
        color,
        terrain_tag: tag.to_owned(),
        purpose: ShapePurpose::Fill,
        gameplay_tags: Vec::new(),
    }
}

fn triangle_shape(a: Vec2, b: Vec2, c: Vec2, color: [f32; 4], tag: &str) -> AnnotatedShape {
    AnnotatedShape {
        path: vec![
            PathCommand::MoveTo(a),
            PathCommand::LineTo(b),
            PathCommand::LineTo(c),
            PathCommand::Close,
        ],
        color,
        terrain_tag: tag.to_owned(),
        purpose: ShapePurpose::Fill,
        gameplay_tags: Vec::new(),
    }
}
