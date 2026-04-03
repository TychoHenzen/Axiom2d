use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use engine_render::shape::Shape;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::codegen::{
    ArtMetadata, CompactEncodingError, ExportOptimizationConfig, RepositoryEntry,
    build_shared_palette, generate_art_mod, generate_hydrate_module_with_shared_palette,
    generate_repository_module, optimize_shapes_for_export, shapes_to_art_file,
    shapes_to_compact_art_file, shapes_to_compact_art_file_with_shared_palette,
};
use crate::{ConvertConfig, ConvertProgress};

/// Element names for the card identity system.
pub const ELEMENTS: [&str; 8] = [
    "Solidum",
    "Febris",
    "Ordinem",
    "Lumines",
    "Varias",
    "Inertiae",
    "Subsidium",
    "Spatium",
];

/// Aspect poles per element (positive, negative).
pub const ASPECTS: [[&str; 2]; 8] = [
    ["Solid", "Fragile"],
    ["Heat", "Cold"],
    ["Order", "Chaos"],
    ["Light", "Dark"],
    ["Change", "Stasis"],
    ["Force", "Calm"],
    ["Growth", "Decay"],
    ["Expansion", "Contraction"],
];

/// A single shape entry in the manifest, storing all settings needed to
/// reproduce a codegen run from an image file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeManifestEntry {
    /// Path to the source image (relative to manifest file location).
    pub image_path: String,
    /// Output `.rs` file path (relative to manifest file location).
    pub output_path: String,
    /// Function name for the generated Rust function.
    pub fn_name: String,
    /// Conversion pipeline settings.
    pub config: ConvertConfig,
    /// Index into `ELEMENTS` (0–7).
    pub element_index: usize,
    /// 0 = positive aspect, 1 = negative aspect.
    pub aspect_pole: usize,
    /// Signature axes for art metadata.
    pub signature_axes: [f32; 8],
    /// Use compact integer-array encoding instead of verbose vec literal.
    pub compact_encoding: bool,
    /// Optional lossy preview/export optimizations applied after vectorization.
    #[serde(default)]
    pub export_optimizations: ExportOptimizationConfig,
    /// Optional human-readable description of the image / art piece.
    #[serde(default)]
    pub description: String,
    /// Category for grouping entries in the manifest panel.
    #[serde(default)]
    pub category: String,
}

/// A collection of shape entries that can be batch-processed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShapeManifest {
    /// Shared palette size for compact batch builds. `0` keeps per-shape RGB bytes.
    #[serde(default)]
    pub shared_palette_size: usize,
    pub entries: Vec<ShapeManifestEntry>,
}

/// Errors from manifest operations.
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Image decode error: {0}")]
    ImageDecode(#[from] image::ImageError),
    #[error("Compact encoding error: {0}")]
    CompactEncoding(#[from] CompactEncodingError),
}

/// Shared progress handle for batch builds. Poll from the UI thread.
#[derive(Clone)]
pub struct BatchProgress {
    /// Number of entries completed so far.
    pub completed: Arc<AtomicUsize>,
    /// Total number of entries.
    pub total: Arc<AtomicUsize>,
    /// Per-entry conversion progress (reused across entries).
    pub entry_progress: ConvertProgress,
}

impl BatchProgress {
    pub fn new(total: usize) -> Self {
        Self {
            completed: Arc::new(AtomicUsize::new(0)),
            total: Arc::new(AtomicUsize::new(total)),
            entry_progress: ConvertProgress::new(),
        }
    }

    /// How many entries have been completed.
    pub fn completed_count(&self) -> usize {
        self.completed.load(Ordering::Relaxed)
    }

    /// Total number of entries.
    pub fn total_count(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }
}

/// Result of building a single manifest entry.
#[derive(Debug)]
pub struct EntryBuildResult {
    pub fn_name: String,
    pub output_path: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Summary of a batch build run.
#[derive(Debug)]
pub struct BatchBuildReport {
    pub results: Vec<EntryBuildResult>,
}

impl BatchBuildReport {
    pub fn succeeded(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    pub fn failed(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }
}

/// Load a manifest from a JSON file. Returns an empty manifest if the file
/// does not exist.
pub fn load_manifest(path: &Path) -> Result<ShapeManifest, ManifestError> {
    if !path.exists() {
        return Ok(ShapeManifest::default());
    }
    let contents = std::fs::read_to_string(path)?;
    let mut manifest: ShapeManifest = serde_json::from_str(&contents)?;
    if manifest.shared_palette_size == 0 {
        manifest.shared_palette_size = manifest
            .entries
            .iter()
            .map(|entry| entry.export_optimizations.palette_size)
            .max()
            .unwrap_or(0);
    }
    for entry in &mut manifest.entries {
        entry.export_optimizations.palette_size = 0;
    }
    Ok(manifest)
}

/// Save a manifest to a JSON file with pretty-printing. Creates parent
/// directories if needed.
pub fn save_manifest(manifest: &ShapeManifest, path: &Path) -> Result<(), ManifestError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(manifest)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Process all entries in the manifest, generating `.rs` files from source
/// images. Paths are resolved relative to `base_dir`.
///
/// After building all entries, generates support files in the output directory
/// of the first entry: `hydrate.rs` (shared hydration function),
/// `repository.rs` (cached shape lookup), and `mod.rs` (module declarations).
///
/// Per-entry errors are collected into the report — processing does not
/// stop on the first failure.
pub fn batch_build(manifest: &ShapeManifest, base_dir: &Path) -> BatchBuildReport {
    batch_build_with_progress(manifest, base_dir, None)
}

/// Same as `batch_build` but updates a shared progress handle.
///
/// Art files are always written into a `generated/` subfolder next to the
/// entries' output directory.  That folder is cleared before each build so
/// stale files from removed manifest entries don't linger.  Hand-written
/// files in the parent (e.g. `art/mod.rs`) are never touched.
pub fn batch_build_with_progress(
    manifest: &ShapeManifest,
    base_dir: &Path,
    progress: Option<&BatchProgress>,
) -> BatchBuildReport {
    // Clear the generated/ output directory before building.
    if let Some(first_entry) = manifest.entries.first() {
        let generated_dir = resolve_generated_dir(base_dir, &first_entry.output_path);
        if generated_dir.exists() {
            let _ = std::fs::remove_dir_all(&generated_dir);
        }
        let _ = std::fs::create_dir_all(&generated_dir);
    }

    let mut results: Vec<Option<EntryBuildResult>> =
        (0..manifest.entries.len()).map(|_| None).collect();
    let mut prepared_entries = Vec::new();

    for (index, entry) in manifest.entries.iter().enumerate() {
        match prepare_entry(index, entry, base_dir, progress) {
            Ok(prepared) => prepared_entries.push(prepared),
            Err(error) => {
                results[index] = Some(EntryBuildResult {
                    fn_name: entry.fn_name.clone(),
                    output_path: entry.output_path.clone(),
                    success: false,
                    error: Some(error.to_string()),
                });
            }
        }

        if let Some(p) = progress {
            p.completed.fetch_add(1, Ordering::Relaxed);
        }
    }

    let palette_shape_sets: Vec<&[Shape]> = prepared_entries
        .iter()
        .filter(|prepared| prepared.compact_encoding && manifest.shared_palette_size > 0)
        .map(|prepared| prepared.export_shapes.as_slice())
        .collect();
    let shared_palette = build_shared_palette(&palette_shape_sets, manifest.shared_palette_size);

    let mut successful_support_entries = Vec::new();
    for prepared in &prepared_entries {
        let metadata = prepared.metadata();
        let result = (|| -> Result<(), ManifestError> {
            let code = if prepared.compact_encoding {
                if manifest.shared_palette_size > 0 {
                    shapes_to_compact_art_file_with_shared_palette(
                        &prepared.export_shapes,
                        &metadata,
                        &prepared.fn_name,
                        &shared_palette,
                    )?
                } else {
                    shapes_to_compact_art_file(
                        &prepared.export_shapes,
                        &metadata,
                        &prepared.fn_name,
                    )?
                }
            } else {
                shapes_to_art_file(&prepared.export_shapes, &metadata, &prepared.fn_name)
            };

            if let Some(parent) = prepared.generated_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&prepared.generated_path, code)?;
            Ok(())
        })();

        results[prepared.index] = Some(match result {
            Ok(()) => {
                successful_support_entries.push(SuccessfulSupportEntry {
                    fn_name: prepared.fn_name.clone(),
                    output_path: prepared.output_path.clone(),
                    element_index: prepared.element_index,
                    aspect_pole: prepared.aspect_pole,
                    signature_axes: prepared.signature_axes,
                });
                EntryBuildResult {
                    fn_name: prepared.fn_name.clone(),
                    output_path: prepared.output_path.clone(),
                    success: true,
                    error: None,
                }
            }
            Err(error) => EntryBuildResult {
                fn_name: prepared.fn_name.clone(),
                output_path: prepared.output_path.clone(),
                success: false,
                error: Some(error.to_string()),
            },
        });
    }

    let results: Vec<EntryBuildResult> = results
        .into_iter()
        .map(|result| result.expect("every manifest entry should produce a result"))
        .collect();

    // Generate support files next to the art outputs.
    if !successful_support_entries.is_empty() {
        let successful_names: Vec<&str> = successful_support_entries
            .iter()
            .map(|entry| entry.fn_name.as_str())
            .collect();

        let repo_entries: Vec<RepositoryEntry<'_>> = successful_support_entries
            .iter()
            .map(|entry| RepositoryEntry {
                fn_name: entry.fn_name.as_str(),
                element_index: entry.element_index,
                aspect_pole: entry.aspect_pole,
                signature_axes: entry.signature_axes,
            })
            .collect();

        // generated/ dir holds individual art .rs files + its own mod.rs.
        // Support files (hydrate, repository) go one level up in art/.
        if let Some(first_entry) = successful_support_entries.first() {
            let generated_dir = resolve_generated_dir(base_dir, &first_entry.output_path);
            let _ = std::fs::create_dir_all(&generated_dir);
            // mod.rs for the generated/ directory — art module declarations only.
            let _ = std::fs::write(
                generated_dir.join("mod.rs"),
                generate_art_mod(&successful_names),
            );
            // Support files go to the parent (art/) directory.
            if let Some(art_dir) = generated_dir.parent() {
                let _ = std::fs::write(
                    art_dir.join("hydrate.rs"),
                    generate_hydrate_module_with_shared_palette(&shared_palette),
                );
                let _ = std::fs::write(
                    art_dir.join("repository.rs"),
                    generate_repository_module(&repo_entries),
                );
            }
        }
    }

    BatchBuildReport { results }
}

/// Given an entry's `output_path`, return the `generated/` directory it
/// should live in.  If the path already contains a `generated/` level,
/// return that directory; otherwise insert one.
fn resolve_generated_dir(base_dir: &Path, output_path: &str) -> std::path::PathBuf {
    let full = base_dir.join(output_path);
    let parent = full.parent().unwrap_or(base_dir);
    if parent.file_name().is_some_and(|n| n == "generated") {
        parent.to_path_buf()
    } else {
        parent.join("generated")
    }
}

/// Resolve a single entry's output file path, ensuring it lands inside a
/// `generated/` subfolder even if the manifest entry omits it.
fn resolve_generated_path(base_dir: &Path, output_path: &str) -> std::path::PathBuf {
    let full = base_dir.join(output_path);
    let parent = full.parent().unwrap_or(base_dir);
    let file_name = full.file_name().unwrap_or_default();
    if parent.file_name().is_some_and(|n| n == "generated") {
        full
    } else {
        parent.join("generated").join(file_name)
    }
}

struct PreparedEntryBuild {
    index: usize,
    output_path: String,
    generated_path: std::path::PathBuf,
    fn_name: String,
    element: &'static str,
    aspect: &'static str,
    element_index: usize,
    aspect_pole: usize,
    signature_axes: [f32; 8],
    compact_encoding: bool,
    export_shapes: Vec<Shape>,
}

impl PreparedEntryBuild {
    fn metadata(&self) -> ArtMetadata<'_> {
        ArtMetadata {
            element: self.element,
            aspect: self.aspect,
            signature_axes: self.signature_axes,
        }
    }
}

struct SuccessfulSupportEntry {
    fn_name: String,
    output_path: String,
    element_index: usize,
    aspect_pole: usize,
    signature_axes: [f32; 8],
}

fn prepare_entry(
    index: usize,
    entry: &ShapeManifestEntry,
    base_dir: &Path,
    progress: Option<&BatchProgress>,
) -> Result<PreparedEntryBuild, ManifestError> {
    let image_path = base_dir.join(&entry.image_path);
    let generated_path = resolve_generated_path(base_dir, &entry.output_path);

    let img = image::open(&image_path)?;
    let rgba_image = img.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let rgba = rgba_image.into_raw();

    let entry_progress = progress.map(|p| &p.entry_progress);
    let convert_result =
        crate::image_to_shapes_with_progress(&rgba, width, height, &entry.config, entry_progress);

    let element = ELEMENTS
        .get(entry.element_index)
        .copied()
        .unwrap_or("Solidum");
    let aspect = ASPECTS
        .get(entry.element_index)
        .and_then(|a| a.get(entry.aspect_pole).copied())
        .unwrap_or("Solid");
    let export_shapes =
        optimize_shapes_for_export(&convert_result.shapes, &entry.export_optimizations);

    Ok(PreparedEntryBuild {
        index,
        output_path: entry.output_path.clone(),
        generated_path,
        fn_name: entry.fn_name.clone(),
        element,
        aspect,
        element_index: entry.element_index,
        aspect_pole: entry.aspect_pole,
        signature_axes: entry.signature_axes,
        compact_encoding: entry.compact_encoding,
        export_shapes,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ResizeMethod;
    use std::path::PathBuf;

    fn sample_entry() -> ShapeManifestEntry {
        ShapeManifestEntry {
            image_path: "test.png".to_string(),
            output_path: "output/test_art.rs".to_string(),
            fn_name: "test_art".to_string(),
            config: ConvertConfig {
                color_threshold: 0.1,
                alpha_threshold: 128,
                rdp_epsilon: 1.5,
                bezier_error: 1.5,
                min_area: 4,
                max_dimension: 128,
                resize_method: ResizeMethod::Scale2x,
                use_bezier: true,
                merge_below: 5,
                max_shapes: 0,
            },
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
            compact_encoding: true,
            export_optimizations: ExportOptimizationConfig::default(),
            description: String::new(),
            category: String::new(),
        }
    }

    #[test]
    fn when_manifest_roundtripped_then_data_preserved() {
        // Arrange
        let manifest = ShapeManifest {
            shared_palette_size: 0,
            entries: vec![sample_entry()],
        };
        let dir = std::env::temp_dir().join("img_to_shape_test_manifest");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_manifest.json");

        // Act
        save_manifest(&manifest, &path).unwrap();
        let loaded = load_manifest(&path).unwrap();

        // Assert
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].fn_name, "test_art");
        assert_eq!(loaded.entries[0].image_path, "test.png");
        assert!((loaded.entries[0].config.color_threshold - 0.1).abs() < f32::EPSILON);
        assert_eq!(loaded.shared_palette_size, 0);
        assert_eq!(
            loaded.entries[0].export_optimizations,
            ExportOptimizationConfig::default()
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_manifest_file_missing_then_load_returns_empty() {
        // Arrange
        let path = PathBuf::from("/nonexistent/path/manifest.json");

        // Act
        let manifest = load_manifest(&path).unwrap();

        // Assert
        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn when_manifest_serialized_then_json_is_valid() {
        // Arrange
        let manifest = ShapeManifest {
            shared_palette_size: 0,
            entries: vec![sample_entry()],
        };

        // Act
        let json = serde_json::to_string_pretty(&manifest).unwrap();

        // Assert
        assert!(json.contains("\"fn_name\": \"test_art\""));
        assert!(json.contains("\"image_path\": \"test.png\""));
        assert!(json.contains("\"compact_encoding\": true"));
        assert!(json.contains("\"shared_palette_size\": 0"));
        assert!(json.contains("\"coordinate_decimals\": 2"));
    }

    #[test]
    fn when_batch_build_with_missing_image_then_entry_reports_failure() {
        // Arrange
        let manifest = ShapeManifest {
            shared_palette_size: 0,
            entries: vec![sample_entry()],
        };
        let base_dir = PathBuf::from("/nonexistent/base");

        // Act
        let report = batch_build(&manifest, &base_dir);

        // Assert
        assert_eq!(report.results.len(), 1);
        assert!(!report.results[0].success);
        assert!(report.results[0].error.is_some());
        assert_eq!(report.failed(), 1);
        assert_eq!(report.succeeded(), 0);
    }

    #[test]
    fn when_batch_build_succeeds_then_repository_contains_art_entry_inserts() {
        // Arrange — create a minimal 2x2 PNG so the pipeline has something to process
        let dir = std::env::temp_dir().join("img_to_shape_test_art_entry");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let img = image::RgbaImage::from_fn(2, 2, |_, _| image::Rgba([255, 0, 0, 255]));
        let img_path = dir.join("red.png");
        img.save(&img_path).unwrap();

        let manifest = ShapeManifest {
            shared_palette_size: 0,
            entries: vec![ShapeManifestEntry {
                image_path: "red.png".to_string(),
                output_path: "art/generated/red_art.rs".to_string(),
                fn_name: "red_art".to_string(),
                config: ConvertConfig {
                    color_threshold: 0.1,
                    alpha_threshold: 128,
                    rdp_epsilon: 1.0,
                    bezier_error: 1.0,
                    min_area: 0,
                    max_dimension: 4,
                    resize_method: ResizeMethod::Scale2x,
                    use_bezier: false,
                    merge_below: 0,
                    max_shapes: 0,
                },
                element_index: 1, // Febris
                aspect_pole: 0,   // Heat
                signature_axes: [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                compact_encoding: true,
                export_optimizations: ExportOptimizationConfig::default(),
                description: String::new(),
                category: String::new(),
            }],
        };

        // Act
        let report = batch_build(&manifest, &dir);

        // Assert
        assert_eq!(report.succeeded(), 1, "batch_build should succeed");
        let repo_content = std::fs::read_to_string(dir.join("art/repository.rs"))
            .expect("repository.rs should exist");
        assert!(
            repo_content.contains("ArtEntry::new("),
            "missing ArtEntry constructor:\n{repo_content}"
        );
        assert!(
            repo_content.contains("Element::Febris"),
            "missing Element::Febris:\n{repo_content}"
        );
        assert!(
            repo_content.contains("Aspect::Heat"),
            "missing Aspect::Heat:\n{repo_content}"
        );
        assert!(
            repo_content.contains("CardSignature::new("),
            "missing CardSignature:\n{repo_content}"
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_batch_build_uses_shared_palette_then_generated_art_uses_color_indexes() {
        // Arrange
        let dir = std::env::temp_dir().join("img_to_shape_test_shared_palette");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let img = image::RgbaImage::from_fn(2, 2, |_, _| image::Rgba([255, 0, 0, 255]));
        let img_path = dir.join("red.png");
        img.save(&img_path).unwrap();

        let manifest = ShapeManifest {
            shared_palette_size: 1,
            entries: vec![ShapeManifestEntry {
                image_path: "red.png".to_string(),
                output_path: "art/generated/red_art.rs".to_string(),
                fn_name: "red_art".to_string(),
                config: ConvertConfig {
                    color_threshold: 0.1,
                    alpha_threshold: 128,
                    rdp_epsilon: 1.0,
                    bezier_error: 1.0,
                    min_area: 0,
                    max_dimension: 4,
                    resize_method: ResizeMethod::Scale2x,
                    use_bezier: false,
                    merge_below: 0,
                    max_shapes: 0,
                },
                element_index: 0,
                aspect_pole: 0,
                signature_axes: [0.0; 8],
                compact_encoding: true,
                export_optimizations: ExportOptimizationConfig {
                    coordinate_decimals: 2,
                    palette_size: 0,
                },
                description: String::new(),
                category: String::new(),
            }],
        };

        // Act
        let report = batch_build(&manifest, &dir);

        // Assert
        assert_eq!(report.succeeded(), 1, "batch_build should succeed");
        let art_content = std::fs::read_to_string(dir.join("art/generated/red_art.rs"))
            .expect("art file should exist");
        assert!(
            art_content.contains("const COLOR_INDEXES: &[u8]"),
            "missing shared palette indexes:\n{art_content}"
        );
        assert!(
            art_content.contains("hydrate_shapes_compact_indexed(COLOR_INDEXES, DATA)"),
            "missing shared palette hydrate call:\n{art_content}"
        );
        assert!(
            !art_content.contains("const COLORS: &[u8]"),
            "per-file RGB bytes should not be emitted:\n{art_content}"
        );

        let hydrate_content =
            std::fs::read_to_string(dir.join("art/hydrate.rs")).expect("hydrate.rs should exist");
        assert!(
            hydrate_content.contains("const SHARED_PALETTE: &[u8]"),
            "missing shared palette const:\n{hydrate_content}"
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_loading_legacy_entry_palette_size_then_manifest_migrates_it_to_shared_palette() {
        // Arrange
        let dir = std::env::temp_dir().join("img_to_shape_test_manifest_migration");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("manifest.json");
        let json = r#"{
  "entries": [
    {
      "image_path": "test.png",
      "output_path": "output/test_art.rs",
      "fn_name": "test_art",
      "config": {
        "color_threshold": 0.1,
        "alpha_threshold": 128,
        "rdp_epsilon": 1.5,
        "bezier_error": 1.5,
        "min_area": 4,
        "max_dimension": 128,
        "resize_method": "Scale2x",
        "use_bezier": true,
        "merge_below": 5,
        "max_shapes": 0
      },
      "element_index": 0,
      "aspect_pole": 0,
      "signature_axes": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
      "compact_encoding": true,
      "export_optimizations": {
        "coordinate_decimals": 2,
        "palette_size": 256
      },
      "description": "",
      "category": ""
    }
  ]
}"#;
        std::fs::write(&path, json).unwrap();

        // Act
        let manifest = load_manifest(&path).unwrap();

        // Assert
        assert_eq!(manifest.shared_palette_size, 256);
        assert_eq!(manifest.entries[0].export_optimizations.palette_size, 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_empty_manifest_batch_built_then_report_is_empty() {
        // Arrange
        let manifest = ShapeManifest::default();
        let base_dir = PathBuf::from(".");

        // Act
        let report = batch_build(&manifest, &base_dir);

        // Assert
        assert!(report.results.is_empty());
        assert_eq!(report.succeeded(), 0);
        assert_eq!(report.failed(), 0);
    }
}
