use eframe::egui;
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
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::new(),
            status: "No image loaded".to_string(),
            export_code: None,
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
        ui.separator();

        let has_image = self.state.image.is_some();
        if ui
            .add_enabled(has_image, egui::Button::new("Convert"))
            .clicked()
        {
            self.state.run_conversion();
            let count = self.state.shapes.len();
            self.status = format!("Converted: {count} shapes");
            self.export_code = None;
        }
        ui.separator();

        self.show_export(ctx, ui);
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

    fn show_preview(&self, ui: &mut egui::Ui) {
        if self.state.shapes.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Load an image and click Convert to preview shapes");
            });
            return;
        }

        let canvas_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::hover());

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
        let img_rect =
            egui::Rect::from_center_size(response.rect.center(), egui::vec2(img_w, img_h));

        // Draw image boundary outline.
        painter.rect_stroke(
            img_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
            egui::epaint::StrokeKind::Middle,
        );

        let offset = response.rect.min.to_vec2();
        // Clip shape rendering to the image boundary. Egui's PathShape fill
        // uses triangle-fan tessellation that produces artifacts (fill bleeding
        // outside vertex bounding box) for self-intersecting paths. Clipping
        // to the image rect prevents those artifacts from being visible.
        let mut painter = painter;
        painter.set_clip_rect(img_rect);
        for shape in &self.state.shapes {
            for mut egui_shape in shape_to_egui_shapes(shape, canvas_size) {
                egui_shape.translate(offset);
                painter.add(egui_shape);
            }
        }
        // Restore clip rect to full canvas for the tooltip and border.
        painter.set_clip_rect(response.rect);

        // Mouse coordinate display — shows engine-space position on hover.
        if let Some(pointer_pos) = response.hover_pos() {
            let rel = pointer_pos - response.rect.min;
            let engine_x = rel.x - canvas_size.x / 2.0;
            let engine_y = canvas_size.y / 2.0 - rel.y;
            let label = format!("({engine_x:.1}, {engine_y:.1})");
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
            self.show_preview(ui);
        });

        self.handle_dropped_files(ctx);
    }
}
