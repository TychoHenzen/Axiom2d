use std::fmt::Write;

use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};

/// Metadata describing which card signature affinity this art represents.
pub struct ArtMetadata<'a> {
    pub element: &'a str,
    pub aspect: &'a str,
    pub signature_axes: [f32; 8],
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
// Tags (encoded as f32 in the data array):
// 0.0 = MoveTo  → 2 floats (x, y)
// 1.0 = LineTo  → 2 floats (x, y)
// 2.0 = CubicTo → 6 floats (c1x, c1y, c2x, c2y, x, y)
// 3.0 = Close   → 0 floats
// 4.0 = shape   → 4 floats (r, g, b, a) — starts a new shape

const TAG_MOVE_TO: f32 = 0.0;
const TAG_LINE_TO: f32 = 1.0;
const TAG_CUBIC_TO: f32 = 2.0;
const TAG_CLOSE: f32 = 3.0;
const TAG_SHAPE: f32 = 4.0;

/// Generate a compact `.rs` file that stores shapes as a flat `&[f32]` array
/// with a hydration function that builds `Vec<Shape>` at load time.
pub fn shapes_to_compact_art_file(
    shapes: &[Shape],
    metadata: &ArtMetadata<'_>,
    fn_name: &str,
) -> String {
    let name = if fn_name.is_empty() {
        "art_mesh"
    } else {
        fn_name
    };

    let mut out = String::new();

    // Imports — use the shared hydrate module instead of inlining.
    out.push_str(
        "use engine_render::shape::Shape;\n\n\
         use super::hydrate::hydrate_shapes;\n\n",
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

    // Encode shapes into flat f32 data.
    let data = encode_shapes_to_floats(shapes);

    // Data array
    let _ = writeln!(out, "const DATA: &[f32] = &[");
    // Write 10 floats per line for compactness.
    for (i, &v) in data.iter().enumerate() {
        if i % 10 == 0 {
            out.push_str("    ");
        }
        let _ = write!(out, "{},", fmt_f32(v));
        if i % 10 == 9 || i == data.len() - 1 {
            out.push('\n');
        } else {
            out.push(' ');
        }
    }
    out.push_str("];\n\n");

    // Hydration function
    let _ = writeln!(out, "pub fn {name}() -> Vec<Shape> {{");
    out.push_str("    hydrate_shapes(DATA)\n");
    out.push_str("}\n");

    out
}

/// Encode a slice of shapes into a flat f32 array using the compact tag format.
pub fn encode_shapes_to_floats(shapes: &[Shape]) -> Vec<f32> {
    let mut data = Vec::new();
    for shape in shapes {
        data.push(TAG_SHAPE);
        push_f32_rounded(&mut data, shape.color.r);
        push_f32_rounded(&mut data, shape.color.g);
        push_f32_rounded(&mut data, shape.color.b);
        push_f32_rounded(&mut data, shape.color.a);

        if let ShapeVariant::Path { commands } = &shape.variant {
            for cmd in commands {
                match cmd {
                    PathCommand::MoveTo(v) => {
                        data.push(TAG_MOVE_TO);
                        push_f32_rounded(&mut data, v.x);
                        push_f32_rounded(&mut data, v.y);
                    }
                    PathCommand::LineTo(v) => {
                        data.push(TAG_LINE_TO);
                        push_f32_rounded(&mut data, v.x);
                        push_f32_rounded(&mut data, v.y);
                    }
                    PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    } => {
                        data.push(TAG_CUBIC_TO);
                        push_f32_rounded(&mut data, control1.x);
                        push_f32_rounded(&mut data, control1.y);
                        push_f32_rounded(&mut data, control2.x);
                        push_f32_rounded(&mut data, control2.y);
                        push_f32_rounded(&mut data, to.x);
                        push_f32_rounded(&mut data, to.y);
                    }
                    PathCommand::Close => {
                        data.push(TAG_CLOSE);
                    }
                    _ => {}
                }
            }
        }
    }
    data
}

fn push_f32_rounded(data: &mut Vec<f32>, v: f32) {
    data.push((v * 100.0).round() / 100.0);
}

/// Generate the `hydrate.rs` module source — a standalone file containing the
/// shared `hydrate_shapes` function. Placed alongside art files so there is no
/// cross-crate dependency on `img-to-shape`.
pub fn generate_hydrate_module() -> String {
    format!(
        "//! Shared hydration function for compact f32-encoded shape data.\n\
         //!\n\
         //! Auto-generated by img-to-shape — do not edit by hand.\n\n\
         {HYDRATE_MODULE_FN}"
    )
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

/// Element enum variant names, indexed by element_index.
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

/// Aspect enum variant names, indexed by [element_index][aspect_pole].
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
            h ^= b as u64;
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

/// Generate the `mod.rs` for the art directory, declaring all art modules plus
/// the hydrate and repository modules.
pub fn generate_art_mod(fn_names: &[&str]) -> String {
    let mut out = String::new();

    out.push_str(
        "//! Card art modules.\n\
         //!\n\
         //! @generated by img-to-shape — DO NOT EDIT.\n\n\
         pub mod hydrate;\n\
         pub mod repository;\n\n",
    );

    for name in fn_names {
        let _ = writeln!(out, "pub mod {name};");
    }

    // Re-export tessellate helper and repository.
    out.push_str(
        "\nuse engine_render::prelude::tessellate;\n\
         use engine_render::shape::{Shape, TessellatedColorMesh};\n\n\
         pub use repository::{ArtEntry, ShapeRepository};\n\n\
         /// Tessellate a slice of `Shape`s into a single `TessellatedColorMesh`.\n\
         ///\n\
         /// Each shape is tessellated via lyon and its color applied to every vertex.\n\
         /// Shapes that fail tessellation or produce empty geometry are skipped.\n\
         pub fn tessellate_art_shapes(shapes: &[Shape]) -> TessellatedColorMesh {\n\
         \x20   let mut mesh = TessellatedColorMesh::new();\n\
         \x20   for shape in shapes {\n\
         \x20       if let Ok(tess) = tessellate(&shape.variant) {\n\
         \x20           if tess.vertices.is_empty() {\n\
         \x20               continue;\n\
         \x20           }\n\
         \x20           let color = [shape.color.r, shape.color.g, shape.color.b, shape.color.a];\n\
         \x20           let flipped: Vec<[f32; 2]> = tess.vertices.iter().map(|&[x, y]| [x, -y]).collect();\n\
         \x20           mesh.push_vertices(&flipped, &tess.indices, color);\n\
         \x20       }\n\
         \x20   }\n\
         \x20   mesh\n\
         }\n",
    );

    out
}

/// The hydration module source code — standalone file with imports.
const HYDRATE_MODULE_FN: &str = "\
use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;

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
";

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn default_metadata() -> ArtMetadata<'static> {
        ArtMetadata {
            element: "Solidum",
            aspect: "Solid",
            signature_axes: [0.0; 8],
        }
    }

    fn triangle_shape(color: Color) -> Shape {
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                    PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                    PathCommand::LineTo(Vec2::new(5.0, 10.0)),
                    PathCommand::Close,
                ],
            },
            color,
        }
    }

    #[test]
    fn when_shapes_empty_then_file_contains_empty_vec() {
        // Arrange
        let shapes: Vec<Shape> = vec![];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(code.contains("vec![]"), "expected empty vec:\n{code}");
    }

    #[test]
    fn when_single_shape_then_file_starts_with_use_imports() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(
            code.contains("use engine_core::color::Color;"),
            "missing Color import:\n{code}"
        );
        assert!(
            code.contains("use engine_render::shape::{PathCommand, Shape, ShapeVariant};"),
            "missing shape imports:\n{code}"
        );
        assert!(
            code.contains("use glam::Vec2;"),
            "missing Vec2 import:\n{code}"
        );
    }

    #[test]
    fn when_single_shape_then_file_contains_pub_fn_returning_vec_shape() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(
            code.contains("pub fn test_art() -> Vec<Shape>"),
            "missing fn sig:\n{code}"
        );
    }

    #[test]
    fn when_single_path_shape_then_file_contains_path_commands() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(code.contains("MoveTo"), "missing MoveTo:\n{code}");
        assert!(code.contains("LineTo"), "missing LineTo:\n{code}");
        assert!(code.contains("Close"), "missing Close:\n{code}");
    }

    #[test]
    fn when_red_shape_then_file_contains_color_components() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(code.contains("Color::new(1"), "missing red color:\n{code}");
    }

    #[test]
    fn when_two_shapes_then_file_contains_both() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        let count = code.matches("Shape {").count();
        assert_eq!(count, 2, "expected 2 shapes:\n{code}");
    }

    #[test]
    fn when_metadata_has_element_then_file_contains_it() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = ArtMetadata {
            element: "Febris",
            aspect: "Heat",
            signature_axes: [0.5, -0.3, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0],
        };

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(code.contains("Febris"), "missing element:\n{code}");
        assert!(code.contains("Heat"), "missing aspect:\n{code}");
        assert!(code.contains("0.5"), "missing axis value:\n{code}");
    }

    #[test]
    fn when_fn_name_provided_then_file_uses_it() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "solidum_flame");

        // Assert
        assert!(
            code.contains("pub fn solidum_flame"),
            "missing fn name:\n{code}"
        );
    }

    #[test]
    fn when_fn_name_empty_then_file_uses_fallback() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "");

        // Assert
        assert!(
            code.contains("pub fn art_mesh"),
            "missing fallback name:\n{code}"
        );
    }

    #[test]
    fn when_single_shape_then_file_has_balanced_braces_and_brackets() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        let open_brackets = code.matches('[').count();
        let close_brackets = code.matches(']').count();
        assert_eq!(open_braces, close_braces, "unbalanced braces");
        assert_eq!(open_brackets, close_brackets, "unbalanced brackets");
    }

    #[test]
    fn when_cubic_command_then_file_contains_control_points() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::ZERO),
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.0, 2.0),
                        control2: Vec2::new(3.0, 4.0),
                        to: Vec2::new(5.0, 0.0),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::WHITE,
        }];
        let meta = default_metadata();

        // Act
        let code = shapes_to_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(code.contains("CubicTo"), "missing CubicTo:\n{code}");
        assert!(code.contains("control1"), "missing control1:\n{code}");
        assert!(code.contains("control2"), "missing control2:\n{code}");
    }

    // --- Compact encoding tests ---

    #[test]
    fn when_compact_encoding_then_output_contains_data_array_and_hydrate_import() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];
        let meta = default_metadata();

        // Act
        let code = shapes_to_compact_art_file(&shapes, &meta, "test_art");

        // Assert
        assert!(
            code.contains("const DATA: &[f32]"),
            "missing DATA array:\n{code}"
        );
        assert!(
            code.contains("use super::hydrate::hydrate_shapes"),
            "missing hydrate import:\n{code}"
        );
        assert!(
            code.contains("hydrate_shapes(DATA)"),
            "missing hydrate call:\n{code}"
        );
        assert!(code.contains("pub fn test_art"), "missing pub fn:\n{code}");
    }

    #[test]
    fn when_compact_encoding_then_data_contains_shape_tag() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];

        // Act
        let data = encode_shapes_to_floats(&shapes);

        // Assert — first float should be TAG_SHAPE (4.0)
        assert!(
            (data[0] - 4.0).abs() < f32::EPSILON,
            "first tag should be 4.0 (shape)"
        );
        // Then 4 color floats (r=1.0, g=0.0, b=0.0, a=1.0)
        assert!((data[1] - 1.0).abs() < f32::EPSILON, "r should be 1.0");
        assert!((data[2]).abs() < f32::EPSILON, "g should be 0.0");
        assert!((data[3]).abs() < f32::EPSILON, "b should be 0.0");
        assert!((data[4] - 1.0).abs() < f32::EPSILON, "a should be 1.0");
    }

    #[test]
    fn when_compact_encoding_roundtripped_then_shape_count_matches() {
        // Arrange — encode then paste the data into a quick decode check
        let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];

        // Act
        let data = encode_shapes_to_floats(&shapes);

        // Assert — count shape tags (4.0 values at positions where a tag is expected)
        let shape_tags = data
            .iter()
            .filter(|&&v| (v - 4.0).abs() < f32::EPSILON)
            .count();
        // There should be at least 2 shape tags (could be more if a float happens to be 4.0,
        // but with our test colors that won't happen)
        assert_eq!(shape_tags, 2, "expected 2 shape tags in data");
    }

    #[test]
    fn when_compact_encoding_with_many_commands_then_fewer_lines_than_verbose() {
        // Arrange — shape with 200 LineTo commands (simulates real art output)
        let commands: Vec<PathCommand> = std::iter::once(PathCommand::MoveTo(Vec2::ZERO))
            .chain((1..200).map(|i| PathCommand::LineTo(Vec2::new(i as f32, (i * 2) as f32))))
            .chain(std::iter::once(PathCommand::Close))
            .collect();
        let shapes = vec![Shape {
            variant: ShapeVariant::Path { commands },
            color: Color::RED,
        }];
        let meta = default_metadata();

        // Act
        let verbose = shapes_to_art_file(&shapes, &meta, "test");
        let compact = shapes_to_compact_art_file(&shapes, &meta, "test");

        // Assert — compact should have fewer lines once data volume exceeds hydrate fn overhead
        let verbose_lines = verbose.lines().count();
        let compact_lines = compact.lines().count();
        assert!(
            compact_lines < verbose_lines,
            "compact ({compact_lines} lines) should be fewer than verbose ({verbose_lines} lines)"
        );
    }

    // --- Generated module tests ---

    #[test]
    fn when_hydrate_module_generated_then_contains_pub_fn_and_imports() {
        // Act
        let code = generate_hydrate_module();

        // Assert
        assert!(
            code.contains("pub fn hydrate_shapes"),
            "missing pub fn:\n{code}"
        );
        assert!(
            code.contains("use engine_core::color::Color"),
            "missing Color import:\n{code}"
        );
        assert!(
            code.contains("use engine_render::shape"),
            "missing shape import:\n{code}"
        );
    }

    #[test]
    fn when_repository_generated_then_contains_hydrate_all_and_get() {
        // Arrange
        let entries = vec![
            RepositoryEntry {
                fn_name: "armor1",
                element_index: 0,
                aspect_pole: 0,
                signature_axes: [0.0; 8],
            },
            RepositoryEntry {
                fn_name: "sword2",
                element_index: 1,
                aspect_pole: 0,
                signature_axes: [0.0; 8],
            },
        ];

        // Act
        let code = generate_repository_module(&entries);

        // Assert
        assert!(
            code.contains("pub fn hydrate_all"),
            "missing hydrate_all:\n{code}"
        );
        assert!(code.contains("pub fn get"), "missing get:\n{code}");
        assert!(
            code.contains("armor1::armor1()"),
            "missing armor1 call:\n{code}"
        );
        assert!(
            code.contains("sword2::sword2()"),
            "missing sword2 call:\n{code}"
        );
        assert!(
            code.contains("use super::armor1"),
            "missing armor1 import:\n{code}"
        );
    }

    #[test]
    fn when_repository_generated_then_contains_art_entry_constructor() {
        // Arrange
        let entries = vec![RepositoryEntry {
            fn_name: "armor1",
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        }];

        // Act
        let code = generate_repository_module(&entries);

        // Assert
        assert!(
            code.contains("ArtEntry::new("),
            "missing ArtEntry constructor:\n{code}"
        );
        assert!(
            code.contains("Element::Solidum"),
            "missing Element::Solidum:\n{code}"
        );
        assert!(
            code.contains("Aspect::Solid"),
            "missing Aspect::Solid:\n{code}"
        );
        assert!(
            code.contains("CardSignature::new("),
            "missing CardSignature::new:\n{code}"
        );
    }

    #[test]
    fn when_repository_generated_for_two_different_elements_then_both_variants_present() {
        // Arrange
        let entries = vec![
            RepositoryEntry {
                fn_name: "armor1",
                element_index: 0,
                aspect_pole: 0,
                signature_axes: [0.0; 8],
            },
            RepositoryEntry {
                fn_name: "fire_card",
                element_index: 1,
                aspect_pole: 0,
                signature_axes: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            },
        ];

        // Act
        let code = generate_repository_module(&entries);

        // Assert
        assert!(
            code.contains("Element::Solidum"),
            "missing Solidum:\n{code}"
        );
        assert!(code.contains("Element::Febris"), "missing Febris:\n{code}");
        assert!(code.contains("Aspect::Solid"), "missing Solid:\n{code}");
        assert!(code.contains("Aspect::Heat"), "missing Heat:\n{code}");
    }

    #[test]
    fn when_repository_generated_then_contains_all_query_method_signatures() {
        // Arrange
        let entries = vec![RepositoryEntry {
            fn_name: "armor1",
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        }];

        // Act
        let code = generate_repository_module(&entries);

        // Assert
        assert!(
            code.contains("pub fn get_entry"),
            "missing get_entry:\n{code}"
        );
        assert!(
            code.contains("pub fn by_element"),
            "missing by_element:\n{code}"
        );
        assert!(
            code.contains("pub fn by_aspect"),
            "missing by_aspect:\n{code}"
        );
        assert!(
            code.contains("pub fn closest_to"),
            "missing closest_to:\n{code}"
        );
    }

    #[test]
    fn when_art_mod_generated_then_declares_all_modules() {
        // Arrange
        let names = vec!["armor1", "sword2"];

        // Act
        let code = generate_art_mod(&names);

        // Assert
        assert!(
            code.contains("pub mod hydrate;"),
            "missing hydrate mod:\n{code}"
        );
        assert!(
            code.contains("pub mod repository;"),
            "missing repository mod:\n{code}"
        );
        assert!(
            code.contains("pub mod armor1;"),
            "missing armor1 mod:\n{code}"
        );
        assert!(
            code.contains("pub mod sword2;"),
            "missing sword2 mod:\n{code}"
        );
        assert!(
            code.contains("pub use repository::{ArtEntry, ShapeRepository}"),
            "missing ArtEntry/ShapeRepository re-export:\n{code}"
        );
    }

    // --- Vec literal tests ---

    #[test]
    fn when_vec_literal_empty_shapes_then_returns_empty_vec() {
        // Arrange
        let shapes: Vec<Shape> = vec![];

        // Act
        let code = shapes_to_vec_literal(&shapes);

        // Assert
        assert_eq!(code, "vec![]");
    }

    #[test]
    fn when_vec_literal_single_shape_then_contains_shape_struct() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED)];

        // Act
        let code = shapes_to_vec_literal(&shapes);

        // Assert
        assert!(code.starts_with("vec!["), "should start with vec![: {code}");
        assert!(code.ends_with(']'), "should end with ]: {code}");
        assert!(code.contains("Shape {"), "missing Shape:\n{code}");
        assert!(code.contains("MoveTo"), "missing MoveTo:\n{code}");
        assert!(code.contains("Close"), "missing Close:\n{code}");
    }

    #[test]
    fn when_vec_literal_multiple_shapes_then_contains_all() {
        // Arrange
        let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];

        // Act
        let code = shapes_to_vec_literal(&shapes);

        // Assert
        let count = code.matches("Shape {").count();
        assert_eq!(count, 2, "expected 2 shapes:\n{code}");
    }
}
