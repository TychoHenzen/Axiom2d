use eframe::egui;
use img_to_shape::ResizeMethod;
use img_to_shape_gui::loader::load_image_from_bytes;
use img_to_shape_gui::preview::shape_to_egui_shapes;
use img_to_shape_gui::state::{ASPECTS, AppState, ELEMENTS};
use std::path::{Path, PathBuf};

/// Resolve the card art output directory relative to the executable's
/// location within the workspace (the exe lives in `target/debug/`).
fn art_output_dir() -> PathBuf {
    // Walk up from the exe dir to find the workspace root (has Cargo.toml).
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        while let Some(d) = dir {
            if d.join("Cargo.toml").exists() && d.join("crates").exists() {
                return d.join("crates/card_game/src/card/art");
            }
            dir = d.parent().map(Path::to_path_buf);
        }
    }
    // Fallback: try current working directory as workspace root.
    PathBuf::from("crates/card_game/src/card/art")
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
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::new(),
            status: "No image loaded".to_string(),
            export_code: None,
            pixel_texture: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
        }
    }

    fn load_file(&mut self, path: &Path) {
        match std::fs::read(path) {
            Ok(bytes) => match load_image_from_bytes(&bytes) {
                Ok((rgba, w, h)) => {
                    self.state.load_image(rgba, w, h);
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
        ui.label(&self.status);
        ui.separator();

        self.show_parameters(ui);

        let has_image = self.state.image.is_some();
        if ui
            .add_enabled(has_image, egui::Button::new("Convert"))
            .clicked()
        {
            self.state.run_conversion();
            let count = self.state.shapes.len();
            self.status = format!("Converted: {count} shapes");
            self.export_code = None;
            self.upload_pixel_texture(ctx);
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
        egui::SidePanel::left("controls")
            .min_width(250.0)
            .show(ctx, |ui| {
                self.show_controls(ctx, ui);
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
