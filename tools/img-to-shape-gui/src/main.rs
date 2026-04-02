use std::path::{Path, PathBuf};
use std::sync::mpsc;

use eframe::egui;
use img_to_shape::manifest::{
    ASPECTS, BatchBuildReport, BatchProgress, ELEMENTS, ShapeManifest, ShapeManifestEntry,
    batch_build_with_progress, load_manifest, save_manifest,
};
use img_to_shape::{ConvertProgress, ConvertResult, ResizeMethod};
use img_to_shape_gui::loader::load_image_from_bytes;
use img_to_shape_gui::preview::shape_to_egui_shapes;
use img_to_shape_gui::state::AppState;

/// Resolve the card art output directory relative to the executable's
/// location within the workspace (the exe lives in `target/debug/`).
/// Generated art goes into a `generated/` subfolder that is cleared
/// before each batch build.
fn art_output_dir() -> PathBuf {
    // Walk up from the exe dir to find the workspace root (has Cargo.toml).
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        while let Some(d) = dir {
            if d.join("Cargo.toml").exists() && d.join("crates").exists() {
                return d.join("crates/card_game/src/card/art/generated");
            }
            dir = d.parent().map(Path::to_path_buf);
        }
    }
    // Fallback: try current working directory as workspace root.
    PathBuf::from("crates/card_game/src/card/art/generated")
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "img-to-shape",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

struct App {
    state: AppState,
    status: String,
    export_code: Option<String>,
    /// Texture handle for the resized pixel buffer preview.
    pixel_texture: Option<egui::TextureHandle>,
    /// Zoom level (1.0 = fit-to-panel).
    zoom: f32,
    /// Pan offset in screen pixels.
    pan: egui::Vec2,
    /// Path to the currently loaded manifest file.
    manifest_path: String,
    /// In-memory manifest data.
    manifest: ShapeManifest,
    /// Currently selected manifest entry index.
    selected_entry: Option<usize>,
    /// Batch build result message.
    batch_status: Option<String>,
    /// Receiver for async batch build results.
    batch_rx: Option<mpsc::Receiver<BatchBuildReport>>,
    /// Progress tracking for batch build.
    batch_progress: Option<BatchProgress>,
    /// Receiver for async single-image conversion results.
    convert_rx: Option<mpsc::Receiver<ConvertResult>>,
    /// Progress tracking for single conversion.
    convert_progress: Option<ConvertProgress>,
    /// Whether the manifest has unsaved changes.
    dirty: bool,
    /// Whether we're showing the "unsaved changes" close confirmation dialog.
    close_requested: bool,
}

/// Resolve the default manifest path ("shape list.json" at workspace root).
fn default_manifest_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        while let Some(d) = dir {
            if d.join("Cargo.toml").exists() && d.join("crates").exists() {
                return d.join("shape list.json");
            }
            dir = d.parent().map(Path::to_path_buf);
        }
    }
    PathBuf::from("shape list.json")
}

impl App {
    fn new() -> Self {
        let manifest_file = default_manifest_path();
        let (manifest, manifest_path, status) = if let Ok(m) = load_manifest(&manifest_file) {
            let count = m.entries.len();
            (
                m,
                manifest_file.to_string_lossy().to_string(),
                format!("Loaded manifest: {count} entries"),
            )
        } else {
            (
                ShapeManifest::default(),
                String::new(),
                "No image loaded".to_string(),
            )
        };

        Self {
            state: AppState::new(),
            status,
            export_code: None,
            pixel_texture: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            manifest_path,
            manifest,
            selected_entry: None,
            batch_status: None,
            batch_rx: None,
            batch_progress: None,
            convert_rx: None,
            convert_progress: None,
            dirty: false,
            close_requested: false,
        }
    }

    fn load_file(&mut self, path: &Path) {
        match std::fs::read(path) {
            Ok(bytes) => match load_image_from_bytes(&bytes) {
                Ok((rgba, w, h)) => {
                    self.state.load_image(rgba, w, h, Some(path.to_path_buf()));
                    // Auto-set fn_name from filename stem.
                    self.state.fn_name = sanitize_fn_name(path);
                    self.status = format!("Loaded {w}x{h} image");
                    self.export_code = None;
                }
                Err(e) => self.status = format!("Decode error: {e}"),
            },
            Err(e) => self.status = format!("Read error: {e}"),
        }
    }

    /// Handle scroll-to-zoom and drag-to-pan, then return the image rect
    /// for content of the given native size, placed within the given canvas response.
    fn zoomed_image_rect(
        &mut self,
        response: &egui::Response,
        content_size: egui::Vec2,
        canvas_size: egui::Vec2,
    ) -> egui::Rect {
        // Zoom with scroll wheel (zoom toward pointer).
        let scroll = response.ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 && response.hovered() {
            let factor = (scroll * 0.005).exp();
            let old_zoom = self.zoom;
            self.zoom = (self.zoom * factor).clamp(0.1, 50.0);
            // Zoom toward pointer position.
            if let Some(pointer) = response.hover_pos() {
                let center = response.rect.center() + self.pan;
                let delta = pointer - center;
                self.pan += delta * (1.0 - self.zoom / old_zoom);
            }
        }
        // Pan with secondary (right) or middle mouse drag.
        if response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle)
        {
            self.pan += response.drag_delta();
        }
        // Double-click to reset zoom/pan.
        if response.double_clicked() {
            self.zoom = 1.0;
            self.pan = egui::Vec2::ZERO;
        }

        let base_scale = fit_scale(content_size, canvas_size);
        let effective_scale = base_scale * self.zoom;
        let scaled = content_size * effective_scale;
        egui::Rect::from_center_size(response.rect.center() + self.pan, scaled)
    }

    fn upload_pixel_texture(&mut self, ctx: &egui::Context) {
        let w = self.state.shape_width as usize;
        let h = self.state.shape_height as usize;
        if w == 0 || h == 0 || self.state.resized_rgba.len() != w * h * 4 {
            self.pixel_texture = None;
            return;
        }
        let image = egui::ColorImage::from_rgba_unmultiplied([w, h], &self.state.resized_rgba);
        let options = egui::TextureOptions::NEAREST;
        match &mut self.pixel_texture {
            Some(handle) => handle.set(image, options),
            None => {
                self.pixel_texture = Some(ctx.load_texture("pixel_preview", image, options));
            }
        }
    }

    /// Upload the raw (unconverted) source image as the pixel preview texture.
    fn upload_raw_texture(&mut self, ctx: &egui::Context) {
        let Some(img) = &self.state.image else { return };
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [img.width as usize, img.height as usize],
            &img.rgba,
        );
        self.pixel_texture =
            Some(ctx.load_texture("pixel_preview", color_image, egui::TextureOptions::NEAREST));
    }

    fn show_controls(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("img-to-shape");
        ui.separator();

        if ui.button("Load Image...").clicked()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                .pick_file()
        {
            self.load_file(&path);
        }
        // Update status from conversion progress before displaying.
        if let Some(p) = &self.convert_progress {
            let pct = p.percent();
            let stage = p.stage();
            self.status = format!("{pct}% — {stage}");
        }
        ui.label(&self.status);
        // Progress bar right under status — always visible.
        if let Some(p) = &self.convert_progress {
            ui.add(egui::ProgressBar::new(f32::from(p.percent()) / 100.0));
        }
        ui.separator();

        self.show_parameters(ui);

        let has_image = self.state.image.is_some();
        let is_converting = self.convert_rx.is_some();
        if ui
            .add_enabled(has_image && !is_converting, egui::Button::new("Convert"))
            .clicked()
            && let Some(img) = &self.state.image
        {
            let rgba = img.rgba.clone();
            let width = img.width;
            let height = img.height;
            let config = self.state.config.clone();
            let progress = ConvertProgress::new();
            self.convert_progress = Some(progress.clone());
            let (tx, rx) = mpsc::channel();
            self.convert_rx = Some(rx);
            self.status = "Converting...".to_string();
            self.export_code = None;
            std::thread::spawn(move || {
                let result = img_to_shape::image_to_shapes_with_progress(
                    &rgba,
                    width,
                    height,
                    &config,
                    Some(&progress),
                );
                let _ = tx.send(result);
            });
        }
        // Poll for completed conversion.
        if let Some(rx) = &self.convert_rx
            && let Ok(result) = rx.try_recv()
        {
            self.state.background = result.background;
            self.state.shapes = result.shapes;
            self.state.resized_rgba = result.rgba;
            self.state.shape_width = result.width;
            self.state.shape_height = result.height;
            self.state.estimate = Some(result.estimate);
            let count = self.state.shapes.len();
            self.status = format!("Converted: {count} shapes");
            self.upload_pixel_texture(ctx);
            self.convert_rx = None;
            self.convert_progress = None;
        }
        ui.separator();

        self.show_export(ctx, ui);
    }

    #[allow(clippy::too_many_lines)]
    fn show_parameters(&mut self, ui: &mut egui::Ui) {
        ui.heading("Parameters");
        ui.add(
            egui::Slider::new(&mut self.state.config.color_threshold, 0.0..=1.0)
                .text("Color threshold"),
        );
        let mut alpha = f32::from(self.state.config.alpha_threshold);
        if ui
            .add(egui::Slider::new(&mut alpha, 0.0..=255.0).text("Alpha threshold"))
            .changed()
        {
            self.state.config.alpha_threshold = alpha as u8;
        }
        ui.add(
            egui::Slider::new(&mut self.state.config.rdp_epsilon, 0.0..=10.0).text("RDP epsilon"),
        );
        ui.add(
            egui::Slider::new(&mut self.state.config.bezier_error, 0.0..=10.0).text("Bezier error"),
        );
        let mut max_dim = self.state.config.max_dimension as f32;
        if ui
            .add(egui::Slider::new(&mut max_dim, 0.0..=512.0).text("Max dimension"))
            .changed()
        {
            self.state.config.max_dimension = max_dim as u32;
        }
        egui::ComboBox::from_label("Resize method")
            .selected_text(match self.state.config.resize_method {
                ResizeMethod::Nearest => "Nearest",
                ResizeMethod::Scale2x => "Scale2x",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.state.config.resize_method,
                    ResizeMethod::Nearest,
                    "Nearest",
                );
                ui.selectable_value(
                    &mut self.state.config.resize_method,
                    ResizeMethod::Scale2x,
                    "Scale2x",
                );
            });
        ui.checkbox(&mut self.state.config.use_bezier, "Bezier curves");
        ui.separator();

        ui.heading("Output Size");
        let mut merge_below = self.state.config.merge_below as f32;
        if ui
            .add(egui::Slider::new(&mut merge_below, 0.0..=200.0).text("Merge below (px)"))
            .changed()
        {
            self.state.config.merge_below = merge_below as usize;
        }
        let mut max_shapes = self.state.config.max_shapes as f32;
        if ui
            .add(egui::Slider::new(&mut max_shapes, 0.0..=500.0).text("Max shapes (0=unlimited)"))
            .changed()
        {
            self.state.config.max_shapes = max_shapes as usize;
        }
        ui.checkbox(&mut self.state.compact_encoding, "Compact encoding");

        if let Some(est) = &self.state.estimate {
            ui.group(|ui| {
                ui.label(format!("Shapes: {}", est.shape_count));
                ui.label(format!(
                    "Commands: {} ({} LineTo, {} CubicTo)",
                    est.command_count, est.line_to_count, est.cubic_to_count
                ));
                ui.label(format!("Est. LoC: ~{}", est.estimated_loc));
                ui.label(format!("Est. floats: ~{}", est.estimated_floats));
            });
        }
        ui.separator();
    }

    fn show_metadata(&mut self, ui: &mut egui::Ui) {
        ui.heading("Art Metadata");
        egui::ComboBox::from_label("Element")
            .selected_text(ELEMENTS[self.state.element_index])
            .show_ui(ui, |ui| {
                for (i, name) in ELEMENTS.iter().enumerate() {
                    ui.selectable_value(&mut self.state.element_index, i, *name);
                }
            });

        let aspects = ASPECTS[self.state.element_index];
        egui::ComboBox::from_label("Aspect")
            .selected_text(aspects[self.state.aspect_pole])
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state.aspect_pole, 0, aspects[0]);
                ui.selectable_value(&mut self.state.aspect_pole, 1, aspects[1]);
            });

        ui.horizontal(|ui| {
            ui.label("Function name:");
            ui.text_edit_singleline(&mut self.state.fn_name);
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.text_edit_singleline(&mut self.state.description);
        });

        ui.collapsing("Signature Axes", |ui| {
            for (i, name) in ELEMENTS.iter().enumerate() {
                ui.add(
                    egui::Slider::new(&mut self.state.signature_axes[i], -1.0..=1.0).text(*name),
                );
            }
        });
    }

    fn show_export(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let has_shapes = !self.state.shapes.is_empty();

        self.show_metadata(ui);
        ui.separator();

        // Auto-save: writes directly to crates/card_game/src/card/art/{fn_name}.rs
        let art_dir = art_output_dir();
        let auto_save_label = format!("Save to card/art/{}", self.state.art_filename());
        if ui
            .add_enabled(has_shapes, egui::Button::new(&auto_save_label))
            .clicked()
        {
            let code = self.state.generate_art_file();
            let path = art_dir.join(self.state.art_filename());
            match std::fs::write(&path, &code) {
                Ok(()) => {
                    self.status = format!("Saved to {}", path.display());
                    self.export_code = Some(code);
                }
                Err(e) => self.status = format!("Save error: {e}"),
            }
        }

        if ui
            .add_enabled(has_shapes, egui::Button::new("Save to File..."))
            .clicked()
        {
            let code = self.state.generate_art_file();
            let dialog = rfd::FileDialog::new()
                .add_filter("Rust", &["rs"])
                .set_file_name(self.state.art_filename());
            let dialog = if art_dir.exists() {
                dialog.set_directory(&art_dir)
            } else {
                dialog
            };
            if let Some(path) = dialog.save_file() {
                match std::fs::write(&path, &code) {
                    Ok(()) => self.status = format!("Saved to {}", path.display()),
                    Err(e) => self.status = format!("Save error: {e}"),
                }
            }
            self.export_code = Some(code);
        }

        if let Some(code) = &self.export_code {
            ui.separator();
            if ui.button("Copy to Clipboard").clicked() {
                ctx.copy_text(code.clone());
            }
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.monospace(code);
                });
        }
    }

    fn show_pixel_preview(&mut self, ui: &mut egui::Ui) {
        if let Some(tex) = &self.pixel_texture {
            let tex_size = tex.size_vec2();
            let tex_id = tex.id();
            let canvas_size = ui.available_size();
            let sense = egui::Sense::click_and_drag().union(egui::Sense::hover());
            let (response, painter) = ui.allocate_painter(canvas_size, sense);
            painter.rect_filled(response.rect, 0.0, egui::Color32::from_gray(32));

            let img_rect = self.zoomed_image_rect(&response, tex_size, canvas_size);

            painter.image(
                tex_id,
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            painter.rect_stroke(
                img_rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
                egui::epaint::StrokeKind::Middle,
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No pixel data");
            });
        }
    }

    fn show_shape_preview(&mut self, ui: &mut egui::Ui) {
        if self.state.shapes.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Load an image and click Convert to preview shapes");
            });
            return;
        }

        let canvas_size = ui.available_size();
        let sense = egui::Sense::click_and_drag().union(egui::Sense::hover());
        let (response, painter) = ui.allocate_painter(canvas_size, sense);

        painter.rect_filled(response.rect, 0.0, egui::Color32::from_gray(32));

        // Use the shape coordinate space dimensions (accounts for downscaling).
        let img_w = if self.state.shape_width > 0 {
            self.state.shape_width as f32
        } else {
            self.state
                .image
                .as_ref()
                .map_or(256.0, |img| img.width as f32)
        };
        let img_h = if self.state.shape_height > 0 {
            self.state.shape_height as f32
        } else {
            self.state
                .image
                .as_ref()
                .map_or(256.0, |img| img.height as f32)
        };

        let content_size = egui::vec2(img_w, img_h);
        let img_rect = self.zoomed_image_rect(&response, content_size, canvas_size);
        let effective_scale = img_rect.width() / img_w;

        // Draw image boundary outline.
        painter.rect_stroke(
            img_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
            egui::epaint::StrokeKind::Middle,
        );

        // Shape rendering: scale engine coords to fit the zoomed rect.
        let shape_canvas = egui::vec2(img_w, img_h);
        let mut painter = painter;
        painter.set_clip_rect(img_rect);
        for shape in self.state.background.iter().chain(&self.state.shapes) {
            for mut egui_shape in shape_to_egui_shapes(shape, shape_canvas) {
                egui_shape.translate(-egui::vec2(shape_canvas.x / 2.0, shape_canvas.y / 2.0));
                scale_egui_shape(&mut egui_shape, effective_scale);
                egui_shape.translate(img_rect.center().to_vec2());
                painter.add(egui_shape);
            }
        }
        painter.set_clip_rect(response.rect);

        // Mouse coordinate display — shows engine-space position on hover.
        if let Some(pointer_pos) = response.hover_pos() {
            let rel = pointer_pos - img_rect.center();
            let engine_x = rel.x / effective_scale;
            let engine_y = -(rel.y / effective_scale);
            let label = format!("({engine_x:.1}, {engine_y:.1})  {:.0}%", self.zoom * 100.0);
            painter.text(
                pointer_pos + egui::vec2(12.0, -12.0),
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::monospace(12.0),
                egui::Color32::WHITE,
            );
        }
    }

    /// Create a manifest entry from the current app state.
    fn state_to_entry(&self, image_path: &str) -> ShapeManifestEntry {
        let art_dir = art_output_dir();
        let output = art_dir
            .join(self.state.art_filename())
            .to_string_lossy()
            .to_string();
        ShapeManifestEntry {
            image_path: image_path.to_string(),
            output_path: output,
            fn_name: self.state.fn_name.clone(),
            config: self.state.config.clone(),
            element_index: self.state.element_index,
            aspect_pole: self.state.aspect_pole,
            signature_axes: self.state.signature_axes,
            compact_encoding: self.state.compact_encoding,
            description: self.state.description.clone(),
            category: String::new(),
        }
    }

    /// Load a manifest entry's settings into the app state (does NOT load the image).
    fn entry_to_state(&mut self, entry: &ShapeManifestEntry) {
        self.state.config = entry.config.clone();
        self.state.element_index = entry.element_index;
        self.state.aspect_pole = entry.aspect_pole;
        self.state.signature_axes = entry.signature_axes;
        entry.fn_name.clone_into(&mut self.state.fn_name);
        self.state.compact_encoding = entry.compact_encoding;
        entry.description.clone_into(&mut self.state.description);
    }

    /// Save the manifest to disk if a path is set. Clears the dirty flag on
    /// success, sets it on failure (or when no path is configured).
    fn auto_save(&mut self, context: &str) {
        if self.manifest_path.is_empty() {
            self.dirty = true;
            return;
        }
        match save_manifest(&self.manifest, Path::new(&self.manifest_path)) {
            Ok(()) => {
                self.dirty = false;
                self.status = format!("Auto-saved ({context})");
            }
            Err(e) => {
                self.dirty = true;
                self.status = format!("Auto-save error: {e}");
            }
        }
    }

    /// Build the image path string for a new manifest entry, relative to
    /// the manifest file's directory when possible.
    fn resolve_image_path_for_entry(&self) -> String {
        let source = self
            .state
            .image
            .as_ref()
            .and_then(|img| img.source_path.as_ref());
        let Some(src) = source else {
            return "(no image)".to_string();
        };
        // Try to make it relative to the manifest's directory.
        if !self.manifest_path.is_empty() {
            let base = Path::new(&self.manifest_path)
                .parent()
                .unwrap_or(Path::new("."));
            if let Ok(rel) = src.strip_prefix(base) {
                return rel.to_string_lossy().to_string();
            }
        }
        src.to_string_lossy().to_string()
    }

    #[allow(clippy::too_many_lines)]
    fn show_manifest_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Shape Manifest");

        // Manifest file controls — all horizontal.
        ui.horizontal(|ui| {
            if ui.button("Load...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
            {
                self.manifest_path = path.to_string_lossy().to_string();
                match load_manifest(&path) {
                    Ok(m) => {
                        self.manifest = m;
                        self.selected_entry = None;
                        self.dirty = false;
                        self.status =
                            format!("Loaded manifest: {} entries", self.manifest.entries.len());
                    }
                    Err(e) => self.status = format!("Manifest load error: {e}"),
                }
            }
            if ui.button("New").clicked() {
                self.manifest = ShapeManifest::default();
                self.selected_entry = None;
                self.manifest_path.clear();
                self.dirty = false;
                self.status = "New empty manifest".to_string();
            }
            let can_save = !self.manifest_path.is_empty();
            if ui
                .add_enabled(can_save, egui::Button::new("Save"))
                .clicked()
            {
                let path = Path::new(&self.manifest_path);
                match save_manifest(&self.manifest, path) {
                    Ok(()) => {
                        self.dirty = false;
                        self.status = format!(
                            "Saved manifest ({} entries) to {}",
                            self.manifest.entries.len(),
                            self.manifest_path
                        );
                    }
                    Err(e) => self.status = format!("Manifest save error: {e}"),
                }
            }
            if ui.button("Save As...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .save_file()
            {
                self.manifest_path = path.to_string_lossy().to_string();
                match save_manifest(&self.manifest, &path) {
                    Ok(()) => {
                        self.dirty = false;
                        self.status = format!(
                            "Saved manifest ({} entries) to {}",
                            self.manifest.entries.len(),
                            self.manifest_path
                        );
                    }
                    Err(e) => self.status = format!("Manifest save error: {e}"),
                }
            }
        });

        // Show current manifest path (read-only label).
        if !self.manifest_path.is_empty() {
            ui.label(
                egui::RichText::new(&self.manifest_path)
                    .small()
                    .color(egui::Color32::GRAY),
            );
        }

        ui.separator();

        // Entry list — categorized scroll view with drag-to-reorder.
        let entry_count = self.manifest.entries.len();
        // Snapshot owned data so the scroll closure doesn't borrow self.
        let entry_labels: Vec<String> = self
            .manifest
            .entries
            .iter()
            .map(|e| {
                if e.fn_name.is_empty() {
                    "(unnamed)".to_string()
                } else {
                    e.fn_name.clone()
                }
            })
            .collect();
        let entry_cats: Vec<String> = self
            .manifest
            .entries
            .iter()
            .map(|e| e.category.clone())
            .collect();
        let groups = build_category_groups(&self.manifest.entries);
        let has_categories = groups.len() > 1 || groups.first().is_some_and(|(c, _)| !c.is_empty());
        let cur_selection = self.selected_entry;
        let mut new_selection = cur_selection;
        let mut pending_move: Option<(usize, usize)> = None;
        let mut pending_cat: Option<(usize, String)> = None;
        let mut pending_cat_reorder: Option<(String, String)> = None;

        ui.label(format!("Entries ({entry_count}):"));
        if entry_count > 0 {
            egui::ScrollArea::vertical()
                .id_salt("manifest_entries")
                .max_height(200.0)
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    if has_categories {
                        for (cat_name, indices) in &groups {
                            let display = if cat_name.is_empty() {
                                "Uncategorized"
                            } else {
                                cat_name.as_str()
                            };
                            let resp = egui::CollapsingHeader::new(format!(
                                "{display} ({})",
                                indices.len()
                            ))
                            .id_salt(format!("cat_{cat_name}"))
                            .default_open(true)
                            .show(ui, |ui| {
                                for &idx in indices {
                                    let (clicked, dropped) = entry_row_ui(
                                        ui,
                                        idx,
                                        &entry_labels[idx],
                                        cur_selection == Some(idx),
                                    );
                                    if clicked {
                                        new_selection = Some(idx);
                                    }
                                    if let Some(source) = dropped {
                                        if entry_cats[source] == entry_cats[idx] {
                                            // Same category — reorder within it.
                                            pending_move = Some((source, idx));
                                        } else {
                                            // Cross-category — assign to this category.
                                            pending_cat = Some((source, cat_name.clone()));
                                        }
                                    }
                                }
                            });

                            // Category header: drag source (reorder) + drop target.
                            // Use pointer_latest_pos — pointer_hover_pos returns None
                            // while the pointer is captured by another widget's drag.
                            let header_rect = resp.header_response.rect;
                            let over_header = ui
                                .ctx()
                                .input(|i| i.pointer.latest_pos())
                                .is_some_and(|p| header_rect.contains(p));
                            let released = ui.ctx().input(|i| i.pointer.any_released());

                            // Manual drag detection — avoids ui.interact() which
                            // would steal clicks from the CollapsingHeader.
                            let cat_drag_id = egui::Id::new(("cat_drag", cat_name.as_str()));
                            let drag_origin: Option<egui::Pos2> =
                                ui.ctx().data(|d| d.get_temp(cat_drag_id));
                            let ptr_pressed = ui.ctx().input(|i| i.pointer.primary_pressed());
                            let ptr_down = ui.ctx().input(|i| i.pointer.primary_down());

                            if ptr_pressed && over_header {
                                if let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos()) {
                                    ui.ctx().data_mut(|d| d.insert_temp(cat_drag_id, pos));
                                }
                            } else if !ptr_down {
                                ui.ctx().data_mut(|d| {
                                    d.remove_temp::<egui::Pos2>(cat_drag_id);
                                });
                            } else if let Some(origin) = drag_origin
                                && let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos())
                                && (pos - origin).length() > 5.0
                            {
                                egui::DragAndDrop::set_payload(ui.ctx(), cat_name.clone());
                            }

                            if over_header {
                                let has_cat_dnd =
                                    egui::DragAndDrop::has_payload_of_type::<String>(ui.ctx());
                                let has_entry_dnd =
                                    egui::DragAndDrop::has_payload_of_type::<usize>(ui.ctx());

                                // Visual feedback.
                                if has_cat_dnd {
                                    let stroke =
                                        egui::Stroke::new(3.0, ui.visuals().selection.stroke.color);
                                    ui.painter().hline(
                                        header_rect.x_range(),
                                        header_rect.top(),
                                        stroke,
                                    );
                                } else if has_entry_dnd {
                                    ui.painter().rect_stroke(
                                        header_rect,
                                        2.0,
                                        egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
                                        egui::epaint::StrokeKind::Middle,
                                    );
                                }

                                // Drop — type-check before consuming payload.
                                if released {
                                    if has_cat_dnd {
                                        if let Some(src) =
                                            egui::DragAndDrop::take_payload::<String>(ui.ctx())
                                            && *src != *cat_name
                                        {
                                            pending_cat_reorder =
                                                Some(((*src).clone(), cat_name.clone()));
                                        }
                                    } else if has_entry_dnd
                                        && let Some(source) =
                                            egui::DragAndDrop::take_payload::<usize>(ui.ctx())
                                    {
                                        pending_cat = Some((*source, cat_name.clone()));
                                    }
                                }
                            }
                        }
                    } else {
                        // Flat list (no categories).
                        for idx in 0..entry_labels.len() {
                            let (clicked, dropped) = entry_row_ui(
                                ui,
                                idx,
                                &entry_labels[idx],
                                cur_selection == Some(idx),
                            );
                            if clicked {
                                new_selection = Some(idx);
                            }
                            if let Some(source) = dropped {
                                pending_move = Some((source, idx));
                            }
                        }
                    }
                });
        } else {
            ui.label("No entries. Add one below.");
        }

        // Apply pending reorder.
        if let Some((from, to)) = pending_move
            && from != to
        {
            let entry = self.manifest.entries.remove(from);
            let insert_at = if to > from { to - 1 } else { to };
            self.manifest.entries.insert(insert_at, entry);
            // Adjust selection to follow the moved entry.
            if self.selected_entry == Some(from) {
                self.selected_entry = Some(insert_at);
                new_selection = Some(insert_at);
            } else if let Some(sel) = self.selected_entry {
                let new_sel = if from < sel && sel <= insert_at {
                    sel - 1
                } else if insert_at <= sel && sel < from {
                    sel + 1
                } else {
                    sel
                };
                self.selected_entry = Some(new_sel);
                new_selection = Some(new_sel);
            }
            self.auto_save("reordered entry");
        }

        // Apply pending category change.
        if let Some((idx, cat)) = pending_cat
            && self.manifest.entries[idx].category != cat
        {
            self.manifest.entries[idx].category = cat;
            self.auto_save("changed category");
        }

        // Apply pending category reorder.
        if let Some((source_cat, target_cat)) = pending_cat_reorder {
            let source_indices: Vec<usize> = self
                .manifest
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.category == source_cat)
                .map(|(i, _)| i)
                .collect();
            let mut extracted: Vec<ShapeManifestEntry> = source_indices
                .iter()
                .rev()
                .map(|&i| self.manifest.entries.remove(i))
                .collect();
            extracted.reverse();
            let insert_at = self
                .manifest
                .entries
                .iter()
                .position(|e| e.category == target_cat)
                .unwrap_or(self.manifest.entries.len());
            for (i, entry) in extracted.into_iter().enumerate() {
                self.manifest.entries.insert(insert_at + i, entry);
            }
            self.selected_entry = None;
            self.auto_save("reordered categories");
        }

        // Handle entry selection change — load raw image for preview.
        if new_selection != cur_selection {
            self.selected_entry = new_selection;
            self.pixel_texture = None;
            self.export_code = None;
            if let Some(idx) = new_selection {
                let entry = self.manifest.entries[idx].clone();
                let base = Path::new(&self.manifest_path)
                    .parent()
                    .unwrap_or(Path::new("."));
                let img_path = base.join(&entry.image_path);
                if img_path.exists() {
                    self.load_file(&img_path);
                    self.upload_raw_texture(ui.ctx());
                }
                self.entry_to_state(&entry);
            }
        }

        // Entry action buttons — packed horizontal.
        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                // Auto-dedup the fn_name before creating the entry.
                let unique_name = dedup_fn_name(&self.state.fn_name, &self.manifest);
                self.state.fn_name = unique_name;
                let img_path = self.resolve_image_path_for_entry();
                let entry = self.state_to_entry(&img_path);
                self.manifest.entries.push(entry);
                self.selected_entry = Some(self.manifest.entries.len() - 1);
                self.auto_save("added entry");
            }
            if ui.button("Load Multiple...").clicked()
                && let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                    .pick_files()
            {
                let count = paths.len();
                for path in &paths {
                    // Load the image to get dimensions and source path.
                    let Ok(bytes) = std::fs::read(path) else {
                        continue;
                    };
                    let Ok((rgba, w, h)) = load_image_from_bytes(&bytes) else {
                        continue;
                    };
                    // Temporarily load into state so state_to_entry/resolve work.
                    self.state.load_image(rgba, w, h, Some(path.clone()));
                    self.state.fn_name = dedup_fn_name(&sanitize_fn_name(path), &self.manifest);
                    let img_path = self.resolve_image_path_for_entry();
                    let entry = self.state_to_entry(&img_path);
                    self.manifest.entries.push(entry);
                }
                if count > 0 {
                    self.selected_entry = Some(self.manifest.entries.len() - 1);
                    self.auto_save(&format!("added {count} entries"));
                    // Load the last image for preview.
                    if let Some(last) = paths.last() {
                        self.load_file(last);
                        self.upload_raw_texture(ui.ctx());
                    }
                }
            }
            if let Some(idx) = self.selected_entry {
                if ui.button("Update").clicked() {
                    let category = self.manifest.entries[idx].category.clone();
                    let img_path = self.resolve_image_path_for_entry();
                    let mut entry = self.state_to_entry(&img_path);
                    entry.category = category;
                    self.manifest.entries[idx] = entry;
                    self.auto_save(&format!("updated entry [{idx}]"));
                }
                if ui.button("Remove").clicked() {
                    self.manifest.entries.remove(idx);
                    self.selected_entry = if self.manifest.entries.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.manifest.entries.len() - 1))
                    };
                    self.auto_save("removed entry");
                }
            }
        });

        // Selected entry detail editor.
        if let Some(idx) = self.selected_entry
            && idx < self.manifest.entries.len()
        {
            let mut field_changed = false;
            ui.horizontal(|ui| {
                ui.label("Image:");
                if ui
                    .text_edit_singleline(&mut self.manifest.entries[idx].image_path)
                    .changed()
                {
                    field_changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Output:");
                if ui
                    .text_edit_singleline(&mut self.manifest.entries[idx].output_path)
                    .changed()
                {
                    field_changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Desc:");
                if ui
                    .text_edit_singleline(&mut self.manifest.entries[idx].description)
                    .changed()
                {
                    field_changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Category:");
                if ui
                    .text_edit_singleline(&mut self.manifest.entries[idx].category)
                    .changed()
                {
                    field_changed = true;
                }
            });
            if field_changed {
                self.auto_save("edited field");
            }
        }

        ui.separator();

        // Batch build (async — runs on a background thread).
        let is_building = self.batch_rx.is_some();
        let can_batch = !self.manifest.entries.is_empty() && !is_building;
        if ui
            .add_enabled(can_batch, egui::Button::new("Batch Build All"))
            .clicked()
        {
            let manifest = self.manifest.clone();
            let base = Path::new(&self.manifest_path)
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf();
            let progress = BatchProgress::new(manifest.entries.len());
            self.batch_progress = Some(progress.clone());
            let (tx, rx) = mpsc::channel();
            self.batch_rx = Some(rx);
            self.batch_status = Some("Building...".to_string());
            self.status = "Batch build started...".to_string();
            std::thread::spawn(move || {
                let report = batch_build_with_progress(&manifest, &base, Some(&progress));
                let _ = tx.send(report);
            });
        }

        // Show batch progress.
        if let Some(bp) = &self.batch_progress {
            let done = bp.completed_count();
            let total = bp.total_count();
            let ep = &bp.entry_progress;
            let pct = ep.percent();
            let stage = ep.stage();
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!("[{done}/{total}] {pct}% — {stage}"));
            });
            let overall = if total > 0 {
                (done as f32 + f32::from(pct) / 100.0) / total as f32
            } else {
                0.0
            };
            ui.add(egui::ProgressBar::new(overall));
        }

        // Poll for completed batch build.
        if let Some(rx) = &self.batch_rx
            && let Ok(report) = rx.try_recv()
        {
            let msg = format!(
                "Batch: {} succeeded, {} failed",
                report.succeeded(),
                report.failed()
            );
            let details: Vec<String> = report
                .results
                .iter()
                .filter(|r| !r.success)
                .map(|r| {
                    format!(
                        "  FAIL {}: {}",
                        r.fn_name,
                        r.error.as_deref().unwrap_or("unknown")
                    )
                })
                .collect();
            if details.is_empty() {
                self.status.clone_from(&msg);
                self.batch_status = Some(msg);
            } else {
                self.status.clone_from(&msg);
                self.batch_status = Some(format!("{msg}\n{}", details.join("\n")));
            }
            self.batch_rx = None;
            self.batch_progress = None;
        }

        if let Some(batch_msg) = &self.batch_status {
            ui.group(|ui| {
                ui.label(batch_msg);
            });
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if let Some(file) = i.raw.dropped_files.first()
                && let Some(path) = &file.path
            {
                let path = path.clone();
                self.load_file(&path);
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints while async work is in progress.
        if self.convert_rx.is_some() || self.batch_rx.is_some() {
            ctx.request_repaint();
        }

        // Intercept close request when there are unsaved changes.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_requested = true;
        }

        // "Unsaved changes" confirmation dialog.
        if self.close_requested {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("The manifest has unsaved changes.");
                    ui.horizontal(|ui| {
                        if ui.button("Save & Close").clicked() {
                            if !self.manifest_path.is_empty() {
                                let _ =
                                    save_manifest(&self.manifest, Path::new(&self.manifest_path));
                            }
                            self.dirty = false;
                            self.close_requested = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Discard & Close").clicked() {
                            self.dirty = false;
                            self.close_requested = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.close_requested = false;
                        }
                    });
                });
        }

        egui::SidePanel::left("controls")
            .min_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_manifest_panel(ui);
                    ui.separator();
                    self.show_controls(ctx, ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let has_pixels = self.pixel_texture.is_some();
            let has_shapes = !self.state.shapes.is_empty();

            if has_pixels && has_shapes {
                // Split view: pixels left, shapes right.
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Pixels");
                        self.show_pixel_preview(ui);
                    });
                    cols[1].vertical(|ui| {
                        ui.label("Shapes");
                        self.show_shape_preview(ui);
                    });
                });
            } else if has_shapes {
                self.show_shape_preview(ui);
            } else if has_pixels {
                self.show_pixel_preview(ui);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Load an image and click Convert to preview shapes");
                });
            }
        });

        self.handle_dropped_files(ctx);
    }
}

/// Group manifest entry indices by category, preserving first-appearance order.
fn build_category_groups(entries: &[ShapeManifestEntry]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        if let Some(group) = groups.iter_mut().find(|(name, _)| *name == entry.category) {
            group.1.push(i);
        } else {
            groups.push((entry.category.clone(), vec![i]));
        }
    }
    groups
}

/// Draw a single draggable entry row. Returns `(clicked, dropped_source_index)`.
fn entry_row_ui(
    ui: &mut egui::Ui,
    idx: usize,
    label: &str,
    selected: bool,
) -> (bool, Option<usize>) {
    let row_height = ui.spacing().interact_size.y;
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    let dragging_this = response.dragged();

    // Background.
    if selected && !dragging_this {
        ui.painter()
            .rect_filled(rect, 2.0, ui.visuals().selection.bg_fill);
    } else if response.hovered() && !dragging_this {
        ui.painter()
            .rect_filled(rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
    }

    // Text.
    let text_color = if selected {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().text_color()
    };
    let text_color = if dragging_this {
        let c = text_color;
        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), c.a() / 3)
    } else {
        text_color
    };
    ui.painter().text(
        egui::pos2(rect.left() + 4.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::TextStyle::Body.resolve(ui.style()),
        text_color,
    );

    // Drag payload.
    response.dnd_set_drag_payload(idx);

    // Drop indicator — horizontal line above this row.
    if response.dnd_hover_payload::<usize>().is_some() && !dragging_this {
        let stroke = egui::Stroke::new(2.0, ui.visuals().selection.stroke.color);
        ui.painter().hline(rect.x_range(), rect.top(), stroke);
    }

    // Drop detection.
    let dropped = response.dnd_release_payload::<usize>().map(|arc| *arc);

    (response.clicked() && !dragging_this, dropped)
}

/// Derive a valid Rust identifier from a file path's stem.
/// E.g. `"My Cool Armor-2.png"` → `"my_cool_armor_2"`.
fn sanitize_fn_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let mut name = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
        } else if !name.ends_with('_') {
            name.push('_');
        }
    }
    let name = name.trim_matches('_').to_string();
    // Rust identifiers can't start with a digit — prefix if needed.
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("img_{name}")
    } else {
        name
    }
}

/// If `base_name` already exists in `manifest`, append/increment a trailing
/// number until unique. E.g. `"armor"` → `"armor2"` → `"armor3"`.
fn dedup_fn_name(base_name: &str, manifest: &ShapeManifest) -> String {
    let existing: std::collections::HashSet<&str> = manifest
        .entries
        .iter()
        .map(|e| e.fn_name.as_str())
        .collect();
    if !existing.contains(base_name) {
        return base_name.to_string();
    }
    // Strip any trailing digits to get the root.
    let root = base_name.trim_end_matches(|c: char| c.is_ascii_digit());
    let mut n = 2u32;
    loop {
        let candidate = format!("{root}{n}");
        if !existing.contains(candidate.as_str()) {
            return candidate;
        }
        n += 1;
    }
}

/// Compute a uniform scale factor to fit `content` within `available`, preserving aspect ratio.
fn fit_scale(content: egui::Vec2, available: egui::Vec2) -> f32 {
    let sx = available.x / content.x;
    let sy = available.y / content.y;
    sx.min(sy).max(0.001)
}

/// Scale all positions in an egui shape by a uniform factor around the origin.
fn scale_egui_shape(shape: &mut egui::Shape, scale: f32) {
    match shape {
        egui::Shape::Path(ps) => {
            for p in &mut ps.points {
                p.x *= scale;
                p.y *= scale;
            }
        }
        egui::Shape::Circle(cs) => {
            cs.center.x *= scale;
            cs.center.y *= scale;
            cs.radius *= scale;
        }
        egui::Shape::Mesh(mesh) => {
            let mesh = std::sync::Arc::make_mut(mesh);
            for v in &mut mesh.vertices {
                v.pos.x *= scale;
                v.pos.y *= scale;
            }
        }
        egui::Shape::Vec(shapes) => {
            for s in shapes {
                scale_egui_shape(s, scale);
            }
        }
        _ => {}
    }
}
