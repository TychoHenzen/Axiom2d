use std::collections::BTreeMap;
use std::fmt::Write;

use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Metadata describing which card signature affinity this art represents.
pub struct ArtMetadata<'a> {
    pub element: &'a str,
    pub aspect: &'a str,
    pub signature_axes: [f32; 8],
}

/// Optional lossy export/preview optimizations applied after vectorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportOptimizationConfig {
    /// Decimal places to keep for geometry coordinates.
    /// The converter already emits at most 2 decimal places, so values above 2
    /// are treated as 2.
    #[serde(default = "default_coordinate_decimals")]
    pub coordinate_decimals: u8,
    /// Target shared palette size for compact manifest exports.
    /// `0` keeps per-shape RGB bytes.
    #[serde(default)]
    pub palette_size: usize,
}

const fn default_coordinate_decimals() -> u8 {
    2
}

impl Default for ExportOptimizationConfig {
    fn default() -> Self {
        Self {
            coordinate_decimals: default_coordinate_decimals(),
            palette_size: 0,
        }
    }
}

/// Errors from compact integer encoding.
#[derive(Debug, Error)]
pub enum CompactEncodingError {
    #[error("compact i16 encoding overflow for {field}: {value}")]
    I16Overflow { field: &'static str, value: f32 },
    #[error("shared palette was empty for indexed compact encoding")]
    EmptySharedPalette,
    #[error("shared palette exceeds u8 index capacity: {size}")]
    SharedPaletteTooLarge { size: usize },
}

/// Encoded compact shape payload.
#[derive(Debug)]
pub struct CompactShapeData {
    pub colors: Vec<u8>,
    pub data: Vec<i16>,
}

/// Encoded compact shape payload using shared palette indexes.
#[derive(Debug)]
pub struct IndexedCompactShapeData {
    pub color_indexes: Vec<u8>,
    pub data: Vec<i16>,
}

/// Apply post-vectorization export optimizations to a shape list.
pub fn optimize_shapes_for_export(
    shapes: &[Shape],
    config: &ExportOptimizationConfig,
) -> Vec<Shape> {
    if shapes.is_empty() {
        return Vec::new();
    }

    let mut optimized = shapes.to_vec();

    if clamp_coordinate_decimals(config.coordinate_decimals) < 2 {
        quantize_shapes_geometry(&mut optimized, config.coordinate_decimals);
    }

    optimized
}

/// Count distinct RGB colors used by a shape list.
pub fn unique_shape_color_count(shapes: &[Shape]) -> usize {
    let mut unique: Vec<Color> = Vec::new();
    for shape in shapes {
        if unique.iter().all(|color| !same_rgb(*color, shape.color)) {
            unique.push(shape.color);
        }
    }
    unique.len()
}

fn quantize_shapes_geometry(shapes: &mut [Shape], coordinate_decimals: u8) {
    let places = clamp_coordinate_decimals(coordinate_decimals);
    for shape in shapes {
        match &mut shape.variant {
            ShapeVariant::Circle { radius } => {
                *radius = round_to_places(*radius, places);
            }
            ShapeVariant::Polygon { points } => {
                for point in points {
                    quantize_vec2(point, places);
                }
            }
            ShapeVariant::Path { commands } => {
                for command in commands {
                    match command {
                        PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                            quantize_vec2(point, places);
                        }
                        PathCommand::QuadraticTo { control, to } => {
                            quantize_vec2(control, places);
                            quantize_vec2(to, places);
                        }
                        PathCommand::CubicTo {
                            control1,
                            control2,
                            to,
                        } => {
                            quantize_vec2(control1, places);
                            quantize_vec2(control2, places);
                            quantize_vec2(to, places);
                        }
                        PathCommand::Close | PathCommand::Reverse => {}
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct PaletteSample {
    color: Color,
    weight: f32,
}

/// Build one shared palette across multiple compact-export shape sets.
pub fn build_shared_palette(shape_sets: &[&[Shape]], palette_size: usize) -> Vec<Color> {
    if palette_size == 0 {
        return Vec::new();
    }

    let mut samples_by_color: BTreeMap<(u32, u32, u32), PaletteSample> = BTreeMap::new();
    for shapes in shape_sets {
        for shape in *shapes {
            if encodable_path_commands(shape).is_none() {
                continue;
            }
            let key = palette_key(shape.color);
            samples_by_color
                .entry(key)
                .and_modify(|sample| sample.weight += 1.0)
                .or_insert(PaletteSample {
                    color: shape.color,
                    weight: 1.0,
                });
        }
    }

    let samples: Vec<PaletteSample> = samples_by_color.into_values().collect();
    if samples.is_empty() {
        return Vec::new();
    }

    build_palette(&samples, palette_size.min(usize::from(u8::MAX) + 1))
}

fn build_palette(samples: &[PaletteSample], palette_size: usize) -> Vec<Color> {
    let target = palette_size.min(samples.len());
    let mut centroids = Vec::with_capacity(target);

    let first = samples
        .iter()
        .max_by(|a, b| {
            a.weight
                .partial_cmp(&b.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|sample| sample.color)
        .unwrap_or(Color::WHITE);
    centroids.push(first);

    while centroids.len() < target {
        let mut best_color = samples[0].color;
        let mut best_score = f32::NEG_INFINITY;
        for sample in samples {
            let score = sample.weight * nearest_distance_sq(sample.color, &centroids);
            if score > best_score {
                best_score = score;
                best_color = sample.color;
            }
        }
        centroids.push(best_color);
    }

    for _ in 0..8 {
        let mut sum_r = vec![0.0; target];
        let mut sum_g = vec![0.0; target];
        let mut sum_b = vec![0.0; target];
        let mut weight_sum = vec![0.0; target];

        for sample in samples {
            let idx = nearest_palette_index(sample.color, &centroids);
            sum_r[idx] += sample.color.r * sample.weight;
            sum_g[idx] += sample.color.g * sample.weight;
            sum_b[idx] += sample.color.b * sample.weight;
            weight_sum[idx] += sample.weight;
        }

        for idx in 0..target {
            if weight_sum[idx] > 0.0 {
                centroids[idx] = Color::new(
                    sum_r[idx] / weight_sum[idx],
                    sum_g[idx] / weight_sum[idx],
                    sum_b[idx] / weight_sum[idx],
                    1.0,
                );
            }
        }
    }

    centroids
}

fn nearest_palette_index(color: Color, palette: &[Color]) -> usize {
    let mut best_index = 0;
    let mut best_distance = f32::INFINITY;

    for (idx, candidate) in palette.iter().enumerate() {
        let distance = color_distance_sq_rgb(color, *candidate);
        if distance < best_distance {
            best_distance = distance;
            best_index = idx;
        }
    }

    best_index
}

fn nearest_distance_sq(color: Color, palette: &[Color]) -> f32 {
    palette
        .iter()
        .map(|candidate| color_distance_sq_rgb(color, *candidate))
        .fold(f32::INFINITY, f32::min)
}

fn color_distance_sq_rgb(a: Color, b: Color) -> f32 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    dr * dr + dg * dg + db * db
}

fn same_rgb(a: Color, b: Color) -> bool {
    a.r.to_bits() == b.r.to_bits()
        && a.g.to_bits() == b.g.to_bits()
        && a.b.to_bits() == b.b.to_bits()
}

fn palette_key(color: Color) -> (u32, u32, u32) {
    (color.r.to_bits(), color.g.to_bits(), color.b.to_bits())
}

fn quantize_vec2(point: &mut Vec2, places: u8) {
    point.x = round_to_places(point.x, places);
    point.y = round_to_places(point.y, places);
}

fn round_to_places(v: f32, places: u8) -> f32 {
    let scale = 10_f32.powi(i32::from(places));
    (v * scale).round() / scale
}

fn clamp_coordinate_decimals(places: u8) -> u8 {
    places.min(2)
}

/// Generate a complete `.rs` source file containing a `pub fn` that returns
/// `Vec<Shape>` — compact bezier path data suitable for runtime tessellation.
///
/// The output file includes `use` imports, signature metadata as doc comments,
/// and a function returning the shape vector.
pub fn shapes_to_art_file(shapes: &[Shape], metadata: &ArtMetadata<'_>, fn_name: &str) -> String {
    let name = if fn_name.is_empty() {
        "art_mesh"
    } else {
        fn_name
    };

    let mut out = String::new();

    // Imports
    out.push_str(
        "use engine_core::color::Color;\n\
         use engine_render::shape::{PathCommand, Shape, ShapeVariant};\n\
         use glam::Vec2;\n\n",
    );

    // Metadata doc comment
    let _ = writeln!(out, "/// Element: {}", metadata.element);
    let _ = writeln!(out, "/// Aspect: {}", metadata.aspect);
    let _ = write!(out, "/// Signature: [");
    for (i, &v) in metadata.signature_axes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}", fmt_f32(v));
    }
    out.push_str("]\n");

    // Function
    let _ = writeln!(out, "pub fn {name}() -> Vec<Shape> {{");

    if shapes.is_empty() {
        out.push_str("    vec![]\n");
    } else {
        out.push_str("    vec![\n");
        for (i, shape) in shapes.iter().enumerate() {
            if i > 0 {
                out.push_str(",\n");
            }
            write_shape(&mut out, shape);
        }
        out.push_str("\n    ]\n");
    }
    out.push_str("}\n");

    out
}

/// Generate a bare `vec![...]` literal containing shape data.
///
/// Unlike `shapes_to_art_file`, this produces no imports, metadata, or
/// function wrapper — just the vec expression. Useful for clipboard export.
pub fn shapes_to_vec_literal(shapes: &[Shape]) -> String {
    if shapes.is_empty() {
        return "vec![]".to_string();
    }

    let mut out = String::from("vec![\n");
    for (i, shape) in shapes.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        write_shape(&mut out, shape);
    }
    out.push_str("\n]");
    out
}

fn fmt_f32(v: f32) -> String {
    // Round to 2 decimal places to keep generated code compact.
    let rounded = (v * 100.0).round() / 100.0;
    let s = format!("{rounded}");
    if s.contains('.') {
        s
    } else {
        format!("{rounded}.0")
    }
}

fn write_shape(out: &mut String, shape: &Shape) {
    out.push_str("        Shape {\n");
    out.push_str("            variant: ");
    write_variant(out, &shape.variant);
    out.push_str(",\n            color: ");
    write_color(out, &shape.color);
    out.push_str(",\n        }");
}

fn write_variant(out: &mut String, variant: &ShapeVariant) {
    match variant {
        ShapeVariant::Circle { radius } => {
            let _ = write!(out, "ShapeVariant::Circle {{ radius: {radius:.1} }}");
        }
        ShapeVariant::Polygon { points } => {
            out.push_str("ShapeVariant::Polygon {\n                points: vec![\n");
            for pt in points {
                let _ = writeln!(
                    out,
                    "                    Vec2::new({}, {}),",
                    fmt_f32(pt.x),
                    fmt_f32(pt.y)
                );
            }
            out.push_str("                ],\n            }");
        }
        ShapeVariant::Path { commands } => {
            out.push_str("ShapeVariant::Path {\n                commands: vec![\n");
            for cmd in commands {
                out.push_str("                    ");
                write_command(out, cmd);
                out.push_str(",\n");
            }
            out.push_str("                ],\n            }");
        }
    }
}

fn write_command(out: &mut String, cmd: &PathCommand) {
    match cmd {
        PathCommand::MoveTo(v) => {
            let _ = write!(
                out,
                "PathCommand::MoveTo(Vec2::new({}, {}))",
                fmt_f32(v.x),
                fmt_f32(v.y)
            );
        }
        PathCommand::LineTo(v) => {
            let _ = write!(
                out,
                "PathCommand::LineTo(Vec2::new({}, {}))",
                fmt_f32(v.x),
                fmt_f32(v.y)
            );
        }
        PathCommand::QuadraticTo { control, to } => {
            let _ = write!(
                out,
                "PathCommand::QuadraticTo {{ control: Vec2::new({}, {}), to: Vec2::new({}, {}) }}",
                fmt_f32(control.x),
                fmt_f32(control.y),
                fmt_f32(to.x),
                fmt_f32(to.y)
            );
        }
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => {
            let _ = write!(
                out,
                "PathCommand::CubicTo {{ control1: Vec2::new({}, {}), control2: Vec2::new({}, {}), to: Vec2::new({}, {}) }}",
                fmt_f32(control1.x),
                fmt_f32(control1.y),
                fmt_f32(control2.x),
                fmt_f32(control2.y),
                fmt_f32(to.x),
                fmt_f32(to.y)
            );
        }
        PathCommand::Close => {
            out.push_str("PathCommand::Close");
        }
        PathCommand::Reverse => {
            out.push_str("PathCommand::Reverse");
        }
    }
}

fn write_color(out: &mut String, color: &Color) {
    let _ = write!(
        out,
        "Color::new({}, {}, {}, {})",
        fmt_f32(color.r),
        fmt_f32(color.g),
        fmt_f32(color.b),
        fmt_f32(color.a)
    );
}

// --- Compact encoding ---
//
// Exact delta codec optimized for the final binary layout.
//
// Colors are stored separately as RGB bytes (`&[u8]`).
//
// Tags in the geometry stream (`&[i16]`, scaled by 100):
// 1 = LineTo  → 2 values (dx, dy) from previous endpoint
// 2 = CubicTo → 6 values relative to previous endpoint
//               (dc1x, dc1y, dc2x, dc2y, dx, dy)
// 4 = shape   → 2 values absolute MoveTo (x, y)

const TAG_LINE_TO: i16 = 1;
const TAG_CUBIC_TO: i16 = 2;
const TAG_SHAPE: i16 = 4;
const GEOMETRY_SCALE: f32 = 100.0;

/// Generate a compact `.rs` file that stores colors as `u8` and geometry as
/// exact fixed-point `i16` deltas, then hydrates to `Vec<Shape>` at load time.
fn write_u8_array(out: &mut String, name: &str, values: &[u8]) {
    let _ = writeln!(out, "const {name}: &[u8] = &[");
    for (i, &v) in values.iter().enumerate() {
        if i % 16 == 0 {
            out.push_str("    ");
        }
        let _ = write!(out, "{v},");
        if i % 16 == 15 || i == values.len() - 1 {
            out.push('\n');
        } else {
            out.push(' ');
        }
    }
    out.push_str("];\n\n");
}

fn write_i16_array(out: &mut String, name: &str, values: &[i16]) {
    let _ = writeln!(out, "const {name}: &[i16] = &[");
    for (i, &v) in values.iter().enumerate() {
        if i % 12 == 0 {
            out.push_str("    ");
        }
        let _ = write!(out, "{v},");
        if i % 12 == 11 || i == values.len() - 1 {
            out.push('\n');
        } else {
            out.push(' ');
        }
    }
    out.push_str("];\n\n");
}

fn encodable_path_commands(shape: &Shape) -> Option<&[PathCommand]> {
    let ShapeVariant::Path { commands } = &shape.variant else {
        return None;
    };
    matches!(commands.first(), Some(PathCommand::MoveTo(_))).then_some(commands.as_slice())
}

fn encode_path_commands_to_compact_geometry(
    commands: &[PathCommand],
    data: &mut Vec<i16>,
) -> Result<(), CompactEncodingError> {
    let Some(PathCommand::MoveTo(start)) = commands.first() else {
        return Ok(());
    };

    data.push(TAG_SHAPE);
    push_i16_scaled(data, start.x, "shape start x")?;
    push_i16_scaled(data, start.y, "shape start y")?;

    let mut previous = *start;
    for cmd in &commands[1..] {
        match cmd {
            PathCommand::LineTo(to) => {
                data.push(TAG_LINE_TO);
                push_i16_scaled(data, to.x - previous.x, "line dx")?;
                push_i16_scaled(data, to.y - previous.y, "line dy")?;
                previous = *to;
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                data.push(TAG_CUBIC_TO);
                push_i16_scaled(data, control1.x - previous.x, "cubic control1 dx")?;
                push_i16_scaled(data, control1.y - previous.y, "cubic control1 dy")?;
                push_i16_scaled(data, control2.x - previous.x, "cubic control2 dx")?;
                push_i16_scaled(data, control2.y - previous.y, "cubic control2 dy")?;
                push_i16_scaled(data, to.x - previous.x, "cubic to dx")?;
                push_i16_scaled(data, to.y - previous.y, "cubic to dy")?;
                previous = *to;
            }
            PathCommand::Close | PathCommand::Reverse | PathCommand::MoveTo(_) => {}
            PathCommand::QuadraticTo { .. } => {}
        }
    }

    Ok(())
}

pub fn shapes_to_compact_art_file(
    shapes: &[Shape],
    metadata: &ArtMetadata<'_>,
    fn_name: &str,
) -> Result<String, CompactEncodingError> {
    let name = if fn_name.is_empty() {
        "art_mesh"
    } else {
        fn_name
    };

    let mut out = String::new();

    // Imports — use the shared hydrate module instead of inlining.
    out.push_str(
        "use engine_render::shape::Shape;\n\n\
         use super::hydrate::hydrate_shapes_compact;\n\n",
    );

    // Metadata doc comment
    let _ = writeln!(out, "/// Element: {}", metadata.element);
    let _ = writeln!(out, "/// Aspect: {}", metadata.aspect);
    let _ = write!(out, "/// Signature: [");
    for (i, &v) in metadata.signature_axes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}", fmt_f32(v));
    }
    out.push_str("]\n");

    // Encode shapes into compact integer data.
    let compact = encode_shapes_to_compact_data(shapes)?;

    write_u8_array(&mut out, "COLORS", &compact.colors);
    write_i16_array(&mut out, "DATA", &compact.data);

    // Hydration function
    let _ = writeln!(out, "pub fn {name}() -> Vec<Shape> {{");
    out.push_str("    hydrate_shapes_compact(COLORS, DATA)\n");
    out.push_str("}\n");

    Ok(out)
}

/// Generate a compact `.rs` file that stores one `u8` shared-palette index per
/// shape plus fixed-point `i16` geometry, with the shared palette defined in
/// the generated `hydrate.rs`.
pub fn shapes_to_compact_art_file_with_shared_palette(
    shapes: &[Shape],
    metadata: &ArtMetadata<'_>,
    fn_name: &str,
    shared_palette: &[Color],
) -> Result<String, CompactEncodingError> {
    let name = if fn_name.is_empty() {
        "art_mesh"
    } else {
        fn_name
    };

    let mut out = String::new();

    out.push_str(
        "use engine_render::shape::Shape;\n\n\
         use super::hydrate::hydrate_shapes_compact_indexed;\n\n",
    );

    let _ = writeln!(out, "/// Element: {}", metadata.element);
    let _ = writeln!(out, "/// Aspect: {}", metadata.aspect);
    let _ = write!(out, "/// Signature: [");
    for (i, &v) in metadata.signature_axes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}", fmt_f32(v));
    }
    out.push_str("]\n");

    let compact = encode_shapes_to_compact_data_with_shared_palette(shapes, shared_palette)?;

    write_u8_array(&mut out, "COLOR_INDEXES", &compact.color_indexes);
    write_i16_array(&mut out, "DATA", &compact.data);

    let _ = writeln!(out, "pub fn {name}() -> Vec<Shape> {{");
    out.push_str("    hydrate_shapes_compact_indexed(COLOR_INDEXES, DATA)\n");
    out.push_str("}\n");

    Ok(out)
}

/// Encode a slice of shapes into compact integer color/geometry arrays.
pub fn encode_shapes_to_compact_data(
    shapes: &[Shape],
) -> Result<CompactShapeData, CompactEncodingError> {
    let mut colors = Vec::new();
    let mut data = Vec::new();
    for shape in shapes {
        let Some(commands) = encodable_path_commands(shape) else {
            continue;
        };

        colors.push(color_channel_to_u8(shape.color.r));
        colors.push(color_channel_to_u8(shape.color.g));
        colors.push(color_channel_to_u8(shape.color.b));
        encode_path_commands_to_compact_geometry(commands, &mut data)?;
    }
    Ok(CompactShapeData { colors, data })
}

/// Encode a slice of shapes into shared-palette indexes plus compact geometry.
pub fn encode_shapes_to_compact_data_with_shared_palette(
    shapes: &[Shape],
    shared_palette: &[Color],
) -> Result<IndexedCompactShapeData, CompactEncodingError> {
    if shared_palette.len() > usize::from(u8::MAX) + 1 {
        return Err(CompactEncodingError::SharedPaletteTooLarge {
            size: shared_palette.len(),
        });
    }

    if shared_palette.is_empty()
        && shapes
            .iter()
            .any(|shape| encodable_path_commands(shape).is_some())
    {
        return Err(CompactEncodingError::EmptySharedPalette);
    }

    let mut color_indexes = Vec::new();
    let mut data = Vec::new();
    for shape in shapes {
        let Some(commands) = encodable_path_commands(shape) else {
            continue;
        };

        color_indexes.push(nearest_palette_index(shape.color, shared_palette) as u8);
        encode_path_commands_to_compact_geometry(commands, &mut data)?;
    }

    Ok(IndexedCompactShapeData {
        color_indexes,
        data,
    })
}

/// Decode shapes from the exact compact integer format.
pub fn decode_shapes_from_compact_data(colors: &[u8], data: &[i16]) -> Vec<Shape> {
    decode_shapes_from_compact_impl(data, |shape_index| {
        let base = shape_index.checked_mul(3)?;
        Some(Color::from_u8(
            *colors.get(base)?,
            *colors.get(base + 1)?,
            *colors.get(base + 2)?,
            u8::MAX,
        ))
    })
}

/// Decode shapes from compact geometry plus shared palette indexes.
pub fn decode_shapes_from_compact_palette_data(
    shared_palette: &[Color],
    color_indexes: &[u8],
    data: &[i16],
) -> Vec<Shape> {
    decode_shapes_from_compact_impl(data, |shape_index| {
        let palette_index = usize::from(*color_indexes.get(shape_index)?);
        shared_palette.get(palette_index).copied()
    })
}

fn decode_shapes_from_compact_impl<F>(data: &[i16], mut color_for_shape: F) -> Vec<Shape>
where
    F: FnMut(usize) -> Option<Color>,
{
    let mut shapes = Vec::new();
    let mut i = 0;
    let mut shape_index = 0;

    while i < data.len() {
        let tag = data[i];
        i += 1;
        if tag != TAG_SHAPE || i + 1 >= data.len() {
            break;
        }

        let Some(color) = color_for_shape(shape_index) else {
            break;
        };
        shape_index += 1;

        let start = Vec2::new(
            f32::from(data[i]) / GEOMETRY_SCALE,
            f32::from(data[i + 1]) / GEOMETRY_SCALE,
        );
        i += 2;
        let mut previous = start;
        let mut commands = vec![PathCommand::MoveTo(start)];

        while i < data.len() && data[i] != TAG_SHAPE {
            let cmd_tag = data[i];
            i += 1;
            match cmd_tag {
                TAG_LINE_TO => {
                    if i + 1 >= data.len() {
                        break;
                    }
                    let to = Vec2::new(
                        previous.x + f32::from(data[i]) / GEOMETRY_SCALE,
                        previous.y + f32::from(data[i + 1]) / GEOMETRY_SCALE,
                    );
                    i += 2;
                    commands.push(PathCommand::LineTo(to));
                    previous = to;
                }
                TAG_CUBIC_TO => {
                    if i + 5 >= data.len() {
                        break;
                    }
                    let control1 = Vec2::new(
                        previous.x + f32::from(data[i]) / GEOMETRY_SCALE,
                        previous.y + f32::from(data[i + 1]) / GEOMETRY_SCALE,
                    );
                    let control2 = Vec2::new(
                        previous.x + f32::from(data[i + 2]) / GEOMETRY_SCALE,
                        previous.y + f32::from(data[i + 3]) / GEOMETRY_SCALE,
                    );
                    let to = Vec2::new(
                        previous.x + f32::from(data[i + 4]) / GEOMETRY_SCALE,
                        previous.y + f32::from(data[i + 5]) / GEOMETRY_SCALE,
                    );
                    i += 6;
                    commands.push(PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    });
                    previous = to;
                }
                _ => break,
            }
        }

        commands.push(PathCommand::Close);
        shapes.push(Shape {
            variant: ShapeVariant::Path { commands },
            color,
        });
    }

    shapes
}

fn push_i16_scaled(
    data: &mut Vec<i16>,
    v: f32,
    field: &'static str,
) -> Result<(), CompactEncodingError> {
    data.push(scale_to_i16(v, field)?);
    Ok(())
}

fn scale_to_i16(v: f32, field: &'static str) -> Result<i16, CompactEncodingError> {
    let rounded = round_to_places(v, 2);
    let scaled = (rounded * GEOMETRY_SCALE).round();
    if scaled < f32::from(i16::MIN) || scaled > f32::from(i16::MAX) {
        return Err(CompactEncodingError::I16Overflow {
            field,
            value: rounded,
        });
    }
    Ok(scaled as i16)
}

fn color_channel_to_u8(channel: f32) -> u8 {
    let scaled = (channel.clamp(0.0, 1.0) * f32::from(u8::MAX)).round();
    scaled as u8
}

fn palette_bytes_from_colors(colors: &[Color]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(colors.len() * 3);
    for color in colors {
        bytes.push(color_channel_to_u8(color.r));
        bytes.push(color_channel_to_u8(color.g));
        bytes.push(color_channel_to_u8(color.b));
    }
    bytes
}

/// Generate the `hydrate.rs` module source — a standalone file containing the
/// shared hydration functions. Placed alongside art files so there is no
/// cross-crate dependency on `img-to-shape`.
pub fn generate_hydrate_module() -> String {
    generate_hydrate_module_with_shared_palette(&[])
}

/// Generate the `hydrate.rs` module source with an embedded shared palette for
/// indexed compact exports.
pub fn generate_hydrate_module_with_shared_palette(shared_palette: &[Color]) -> String {
    let mut out = String::from(
        "//! Shared hydration functions for compact generated shape data.\n\
         //!\n\
         //! Auto-generated by img-to-shape — do not edit by hand.\n\n\
         use engine_core::color::Color;\n\
         use engine_render::shape::{PathCommand, Shape, ShapeVariant};\n\
         use glam::Vec2;\n\n",
    );
    let palette_bytes = palette_bytes_from_colors(shared_palette);
    write_u8_array(&mut out, "SHARED_PALETTE", &palette_bytes);
    out.push_str(HYDRATE_MODULE_FN);
    out
}

/// Input describing a single art entry for repository codegen.
pub struct RepositoryEntry<'a> {
    /// Module/function name (e.g. "armor1").
    pub fn_name: &'a str,
    /// Index into `ELEMENTS` (0–7).
    pub element_index: usize,
    /// 0 = positive aspect, 1 = negative aspect.
    pub aspect_pole: usize,
    /// Signature axes for the `CardSignature`.
    pub signature_axes: [f32; 8],
}

/// Element enum variant names, indexed by `element_index`.
const ELEMENT_VARIANTS: [&str; 8] = [
    "Solidum",
    "Febris",
    "Ordinem",
    "Lumines",
    "Varias",
    "Inertiae",
    "Subsidium",
    "Spatium",
];

/// Aspect enum variant names, indexed by [`element_index`][aspect_pole].
const ASPECT_VARIANTS: [[&str; 2]; 8] = [
    ["Solid", "Fragile"],
    ["Heat", "Cold"],
    ["Order", "Chaos"],
    ["Light", "Dark"],
    ["Change", "Stasis"],
    ["Force", "Calm"],
    ["Growth", "Decay"],
    ["Expansion", "Contraction"],
];

/// Generate a `repository.rs` module that caches hydrated shapes with identity
/// metadata and provides lookup by name, element, aspect, and signature distance.
pub fn generate_repository_module(entries: &[RepositoryEntry<'_>]) -> String {
    let mut out = String::new();

    out.push_str(
        "//! Shape repository — caches hydrated art shapes for fast access by name.\n\
         //!\n\
         //! @generated by img-to-shape — DO NOT EDIT.\n\n\
         use std::collections::BTreeMap;\n\n\
         use bevy_ecs::prelude::Resource;\n\
         use engine_render::shape::Shape;\n\n\
         use crate::card::identity::signature::{Aspect, CardSignature, Element};\n\n",
    );

    // Import each art module.
    for entry in entries {
        let _ = writeln!(out, "use super::{};", entry.fn_name);
    }

    // ArtEntry struct
    out.push_str(
        "\n/// A resolved art entry binding a shape list to its card identity metadata.\n\
         pub struct ArtEntry {\n\
         \x20   shapes: Vec<Shape>,\n\
         \x20   element: Element,\n\
         \x20   aspect: Aspect,\n\
         \x20   signature: CardSignature,\n\
         }\n\n\
         impl ArtEntry {\n\
         \x20   /// Creates a new `ArtEntry` from a shape list and its card identity metadata.\n\
         \x20   pub fn new(shapes: Vec<Shape>, element: Element, aspect: Aspect, signature: CardSignature) -> Self {\n\
         \x20       Self { shapes, element, aspect, signature }\n\
         \x20   }\n\n\
         \x20   /// Returns the element this art entry is associated with.\n\
         \x20   pub fn element(&self) -> Element {\n\
         \x20       self.element\n\
         \x20   }\n\n\
         \x20   /// Returns the aspect this art entry is associated with.\n\
         \x20   pub fn aspect(&self) -> Aspect {\n\
         \x20       self.aspect\n\
         \x20   }\n\n\
         \x20   /// Returns the card signature this art entry is associated with.\n\
         \x20   pub fn signature(&self) -> CardSignature {\n\
         \x20       self.signature\n\
         \x20   }\n\n\
         \x20   /// Returns the shapes for this art entry.\n\
         \x20   pub fn shapes(&self) -> &[Shape] {\n\
         \x20       &self.shapes\n\
         \x20   }\n\
         }\n\n",
    );

    // ShapeRepository struct
    out.push_str(
        "/// Cached shape repository. Call `hydrate_all` once during startup\n\
         /// (e.g. splash screen) to populate, then `get` to retrieve cloned shapes.\n\
         #[derive(Resource)]\n\
         pub struct ShapeRepository {\n\
         \x20   cache: BTreeMap<&'static str, ArtEntry>,\n\
         }\n\n\
         impl Default for ShapeRepository {\n\
         \x20   fn default() -> Self {\n\
         \x20       Self::new()\n\
         \x20   }\n\
         }\n\n\
         impl ShapeRepository {\n\
         \x20   /// Creates a new empty repository.\n\
         \x20   pub fn new() -> Self {\n\
         \x20       Self {\n\
         \x20           cache: BTreeMap::new(),\n\
         \x20       }\n\
         \x20   }\n\n\
         \x20   /// Hydrate all registered art shapes and store them in the cache.\n\
         \x20   pub fn hydrate_all(&mut self) {\n",
    );

    // Emit insert calls with ArtEntry
    for entry in entries {
        let name = entry.fn_name;
        let element = ELEMENT_VARIANTS[entry.element_index];
        let aspect = ASPECT_VARIANTS[entry.element_index][entry.aspect_pole];
        #[allow(clippy::float_cmp)]
        let axes = if entry.signature_axes == [0.0; 8] {
            seed_signature_from_name(name)
        } else {
            entry.signature_axes
        };
        let sig_str = format_signature_axes(&axes);
        let _ = writeln!(
            out,
            "        self.cache.insert(\"{name}\", ArtEntry::new(\n\
             \x20           {name}::{name}(),\n\
             \x20           Element::{element},\n\
             \x20           Aspect::{aspect},\n\
             \x20           CardSignature::new([{sig_str}]),\n\
             \x20       ));"
        );
    }

    // Methods
    out.push_str(
        "    }\n\n\
         \x20   /// Get a clone of the cached shapes for the given name.\n\
         \x20   /// Returns `None` if the name is not in the cache (call `hydrate_all` first).\n\
         \x20   pub fn get(&self, name: &str) -> Option<Vec<Shape>> {\n\
         \x20       self.cache.get(name).map(|entry| entry.shapes.clone())\n\
         \x20   }\n\n\
         \x20   /// Get a reference to the full art entry for the given name.\n\
         \x20   /// Returns `None` if the name is not in the cache (call `hydrate_all` first).\n\
         \x20   pub fn get_entry(&self, name: &str) -> Option<&ArtEntry> {\n\
         \x20       self.cache.get(name)\n\
         \x20   }\n\n\
         \x20   /// Insert an art entry into the cache.\n\
         \x20   pub fn insert(&mut self, name: &'static str, entry: ArtEntry) {\n\
         \x20       self.cache.insert(name, entry);\n\
         \x20   }\n\n\
         \x20   /// Returns all entries matching the given element.\n\
         \x20   pub fn by_element(&self, element: Element) -> Vec<(&str, &ArtEntry)> {\n\
         \x20       self.cache\n\
         \x20           .iter()\n\
         \x20           .filter(|(_, entry)| entry.element == element)\n\
         \x20           .map(|(&name, entry)| (name, entry))\n\
         \x20           .collect()\n\
         \x20   }\n\n\
         \x20   /// Returns all entries matching the given aspect.\n\
         \x20   pub fn by_aspect(&self, aspect: Aspect) -> Vec<(&str, &ArtEntry)> {\n\
         \x20       self.cache\n\
         \x20           .iter()\n\
         \x20           .filter(|(_, entry)| entry.aspect == aspect)\n\
         \x20           .map(|(&name, entry)| (name, entry))\n\
         \x20           .collect()\n\
         \x20   }\n\n\
         \x20   /// Returns the `n` entries closest to the given signature, sorted by distance ascending.\n\
         \x20   pub fn closest_to(&self, query: &CardSignature, n: usize) -> Vec<(&str, &ArtEntry)> {\n\
         \x20       let mut entries: Vec<(&str, &ArtEntry, f32)> = self\n\
         \x20           .cache\n\
         \x20           .iter()\n\
         \x20           .map(|(&name, entry)| (name, entry, entry.signature.distance_to(query)))\n\
         \x20           .collect();\n\
         \x20       entries.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));\n\
         \x20       entries.into_iter().take(n).map(|(name, entry, _)| (name, entry)).collect()\n\
         \x20   }\n\n\
         \x20   /// Returns the number of cached entries.\n\
         \x20   pub fn len(&self) -> usize {\n\
         \x20       self.cache.len()\n\
         \x20   }\n\n\
         \x20   /// Returns true if the cache is empty.\n\
         \x20   pub fn is_empty(&self) -> bool {\n\
         \x20       self.cache.is_empty()\n\
         \x20   }\n\n\
         \x20   /// Returns an iterator over all cached (name, entry) pairs.\n\
         \x20   pub fn iter(&self) -> impl Iterator<Item = (&&'static str, &ArtEntry)> {\n\
         \x20       self.cache.iter()\n\
         \x20   }\n\
         }\n",
    );

    out
}

/// Deterministically generate a signature from an art name when no axes are specified.
/// Uses a simple hash-based approach seeded from the name bytes so that re-running
/// codegen produces identical output.
fn seed_signature_from_name(name: &str) -> [f32; 8] {
    let mut axes = [0.0_f32; 8];
    // FNV-1a-inspired hash per axis, seeded differently for each
    for (i, axis) in axes.iter_mut().enumerate() {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ (i as u64 * 0x100_0000_01b3);
        for &b in name.as_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        // Map to [-1, 1]
        *axis = (h as f32 / u64::MAX as f32) * 2.0 - 1.0;
    }
    axes
}

fn format_signature_axes(axes: &[f32; 8]) -> String {
    axes.iter()
        .map(|&v| fmt_f32(v))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate the `mod.rs` for the `generated/` subdirectory, declaring only
/// the individual art shape modules (no support modules or utility functions).
pub fn generate_art_mod(fn_names: &[&str]) -> String {
    let mut out = String::new();

    out.push_str(
        "//! Generated art shape modules.\n\
         //!\n\
         //! @generated by img-to-shape — DO NOT EDIT.\n\n\
         // Re-export hydrate from the parent so `super::hydrate` works\n\
         // inside each art module.\n\
         pub use super::hydrate;\n\n",
    );

    for name in fn_names {
        let _ = writeln!(out, "pub mod {name};");
    }

    out
}

/// The hydration module source code — standalone file body after imports.
const HYDRATE_MODULE_FN: &str = "\
pub fn hydrate_shapes(data: &[f32]) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let tag = data[i] as u8;
        i += 1;
        if tag == 4 {
            let color = Color::new(data[i], data[i + 1], data[i + 2], data[i + 3]);
            i += 4;
            let mut commands = Vec::new();
            while i < data.len() && data[i] as u8 != 4 {
                let cmd_tag = data[i] as u8;
                i += 1;
                match cmd_tag {
                    0 => {
                        commands.push(PathCommand::MoveTo(Vec2::new(data[i], data[i + 1])));
                        i += 2;
                    }
                    1 => {
                        commands.push(PathCommand::LineTo(Vec2::new(data[i], data[i + 1])));
                        i += 2;
                    }
                    2 => {
                        commands.push(PathCommand::CubicTo {
                            control1: Vec2::new(data[i], data[i + 1]),
                            control2: Vec2::new(data[i + 2], data[i + 3]),
                            to: Vec2::new(data[i + 4], data[i + 5]),
                        });
                        i += 6;
                    }
                    3 => {
                        commands.push(PathCommand::Close);
                    }
                    _ => break,
                }
            }
            shapes.push(Shape {
                variant: ShapeVariant::Path { commands },
                color,
            });
        }
    }
    shapes
}

pub fn hydrate_shapes_compact(colors: &[u8], data: &[i16]) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let mut i = 0;
    let mut color_index = 0;
    while i < data.len() {
        let tag = data[i];
        i += 1;
        if tag != 4 || i + 1 >= data.len() || color_index + 2 >= colors.len() {
            break;
        }

        let color = Color::from_u8(
            colors[color_index],
            colors[color_index + 1],
            colors[color_index + 2],
            u8::MAX,
        );
        color_index += 3;
        let start = Vec2::new(f32::from(data[i]) / 100.0, f32::from(data[i + 1]) / 100.0);
        i += 2;

        let mut previous = start;
        let mut commands = vec![PathCommand::MoveTo(start)];

        while i < data.len() && data[i] != 4 {
            let cmd_tag = data[i];
            i += 1;
            match cmd_tag {
                1 => {
                    if i + 1 >= data.len() {
                        break;
                    }
                    let to = Vec2::new(
                        previous.x + f32::from(data[i]) / 100.0,
                        previous.y + f32::from(data[i + 1]) / 100.0,
                    );
                    i += 2;
                    commands.push(PathCommand::LineTo(to));
                    previous = to;
                }
                2 => {
                    if i + 5 >= data.len() {
                        break;
                    }
                    let control1 = Vec2::new(
                        previous.x + f32::from(data[i]) / 100.0,
                        previous.y + f32::from(data[i + 1]) / 100.0,
                    );
                    let control2 = Vec2::new(
                        previous.x + f32::from(data[i + 2]) / 100.0,
                        previous.y + f32::from(data[i + 3]) / 100.0,
                    );
                    let to = Vec2::new(
                        previous.x + f32::from(data[i + 4]) / 100.0,
                        previous.y + f32::from(data[i + 5]) / 100.0,
                    );
                    i += 6;
                    commands.push(PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    });
                    previous = to;
                }
                _ => break,
            }
        }

        commands.push(PathCommand::Close);
        shapes.push(Shape {
            variant: ShapeVariant::Path { commands },
            color,
        });
    }
    shapes
}

pub fn hydrate_shapes_compact_indexed(color_indexes: &[u8], data: &[i16]) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let mut i = 0;
    let mut shape_index = 0;
    while i < data.len() {
        let tag = data[i];
        i += 1;
        if tag != 4 || i + 1 >= data.len() || shape_index >= color_indexes.len() {
            break;
        }

        let palette_base = usize::from(color_indexes[shape_index]) * 3;
        shape_index += 1;
        if palette_base + 2 >= SHARED_PALETTE.len() {
            break;
        }

        let color = Color::from_u8(
            SHARED_PALETTE[palette_base],
            SHARED_PALETTE[palette_base + 1],
            SHARED_PALETTE[palette_base + 2],
            u8::MAX,
        );
        let start = Vec2::new(f32::from(data[i]) / 100.0, f32::from(data[i + 1]) / 100.0);
        i += 2;

        let mut previous = start;
        let mut commands = vec![PathCommand::MoveTo(start)];

        while i < data.len() && data[i] != 4 {
            let cmd_tag = data[i];
            i += 1;
            match cmd_tag {
                1 => {
                    if i + 1 >= data.len() {
                        break;
                    }
                    let to = Vec2::new(
                        previous.x + f32::from(data[i]) / 100.0,
                        previous.y + f32::from(data[i + 1]) / 100.0,
                    );
                    i += 2;
                    commands.push(PathCommand::LineTo(to));
                    previous = to;
                }
                2 => {
                    if i + 5 >= data.len() {
                        break;
                    }
                    let control1 = Vec2::new(
                        previous.x + f32::from(data[i]) / 100.0,
                        previous.y + f32::from(data[i + 1]) / 100.0,
                    );
                    let control2 = Vec2::new(
                        previous.x + f32::from(data[i + 2]) / 100.0,
                        previous.y + f32::from(data[i + 3]) / 100.0,
                    );
                    let to = Vec2::new(
                        previous.x + f32::from(data[i + 4]) / 100.0,
                        previous.y + f32::from(data[i + 5]) / 100.0,
                    );
                    i += 6;
                    commands.push(PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    });
                    previous = to;
                }
                _ => break,
            }
        }

        commands.push(PathCommand::Close);
        shapes.push(Shape {
            variant: ShapeVariant::Path { commands },
            color,
        });
    }
    shapes
}
";
