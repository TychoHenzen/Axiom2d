use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::codegen::{
    ArtMetadata, RepositoryEntry, generate_art_mod, generate_hydrate_module,
    generate_repository_module, shapes_to_art_file, shapes_to_compact_art_file,
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
    /// Use compact float-array encoding instead of verbose vec literal.
    pub compact_encoding: bool,
    /// Optional human-readable description of the image / art piece.
    #[serde(default)]
    pub description: String,
}

/// A collection of shape entries that can be batch-processed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShapeManifest {
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
    let manifest = serde_json::from_str(&contents)?;
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
pub fn batch_build_with_progress(
    manifest: &ShapeManifest,
    base_dir: &Path,
    progress: Option<&BatchProgress>,
) -> BatchBuildReport {
    let results: Vec<EntryBuildResult> = manifest
        .entries
        .iter()
        .map(|entry| {
            let result = build_entry(entry, base_dir, progress);
            if let Some(p) = progress {
                p.completed.fetch_add(1, Ordering::Relaxed);
            }
            result
        })
        .collect();

    // Generate support files next to the art outputs.
    let successful_entries: Vec<(&ShapeManifestEntry, &EntryBuildResult)> = manifest
        .entries
        .iter()
        .zip(results.iter())
        .filter(|(_, r)| r.success)
        .collect();

    if !successful_entries.is_empty() {
        let successful_names: Vec<&str> = successful_entries
            .iter()
            .map(|(_, r)| r.fn_name.as_str())
            .collect();

        let repo_entries: Vec<RepositoryEntry<'_>> = successful_entries
            .iter()
            .map(|(manifest_entry, result)| RepositoryEntry {
                fn_name: result.fn_name.as_str(),
                element_index: manifest_entry.element_index,
                aspect_pole: manifest_entry.aspect_pole,
                signature_axes: manifest_entry.signature_axes,
            })
            .collect();

        // Determine the output directory from the first successful entry's output_path.
        if let Some(first_output) = successful_entries
            .first()
            .map(|(e, _)| base_dir.join(&e.output_path))
            && let Some(art_dir) = first_output.parent()
        {
            let _ = std::fs::create_dir_all(art_dir);
            let _ = std::fs::write(art_dir.join("hydrate.rs"), generate_hydrate_module());
            let _ = std::fs::write(
                art_dir.join("repository.rs"),
                generate_repository_module(&repo_entries),
            );
            let _ = std::fs::write(art_dir.join("mod.rs"), generate_art_mod(&successful_names));
        }
    }

    BatchBuildReport { results }
}

fn build_entry(
    entry: &ShapeManifestEntry,
    base_dir: &Path,
    progress: Option<&BatchProgress>,
) -> EntryBuildResult {
    let image_path = base_dir.join(&entry.image_path);
    let output_path = base_dir.join(&entry.output_path);

    let result = (|| -> Result<(), ManifestError> {
        // Load and decode image.
        let img = image::open(&image_path)?;
        let rgba_image = img.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let rgba = rgba_image.into_raw();

        // Run the conversion pipeline with per-entry progress.
        let entry_progress = progress.map(|p| &p.entry_progress);
        let convert_result = crate::image_to_shapes_with_progress(
            &rgba,
            width,
            height,
            &entry.config,
            entry_progress,
        );

        // Build metadata.
        let element = ELEMENTS
            .get(entry.element_index)
            .copied()
            .unwrap_or("Solidum");
        let aspect = ASPECTS
            .get(entry.element_index)
            .and_then(|a| a.get(entry.aspect_pole).copied())
            .unwrap_or("Solid");
        let metadata = ArtMetadata {
            element,
            aspect,
            signature_axes: entry.signature_axes,
        };

        // Generate code.
        let code = if entry.compact_encoding {
            shapes_to_compact_art_file(&convert_result.shapes, &metadata, &entry.fn_name)
        } else {
            shapes_to_art_file(&convert_result.shapes, &metadata, &entry.fn_name)
        };

        // Write output file.
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, code)?;

        Ok(())
    })();

    match result {
        Ok(()) => EntryBuildResult {
            fn_name: entry.fn_name.clone(),
            output_path: entry.output_path.clone(),
            success: true,
            error: None,
        },
        Err(e) => EntryBuildResult {
            fn_name: entry.fn_name.clone(),
            output_path: entry.output_path.clone(),
            success: false,
            error: Some(e.to_string()),
        },
    }
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
            description: String::new(),
        }
    }

    #[test]
    fn when_manifest_roundtripped_then_data_preserved() {
        // Arrange
        let manifest = ShapeManifest {
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
            entries: vec![sample_entry()],
        };

        // Act
        let json = serde_json::to_string_pretty(&manifest).unwrap();

        // Assert
        assert!(json.contains("\"fn_name\": \"test_art\""));
        assert!(json.contains("\"image_path\": \"test.png\""));
        assert!(json.contains("\"compact_encoding\": true"));
    }

    #[test]
    fn when_batch_build_with_missing_image_then_entry_reports_failure() {
        // Arrange
        let manifest = ShapeManifest {
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
            entries: vec![ShapeManifestEntry {
                image_path: "red.png".to_string(),
                output_path: "art/red_art.rs".to_string(),
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
                description: String::new(),
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
