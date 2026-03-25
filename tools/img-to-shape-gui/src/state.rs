use engine_render::shape::Shape;
use img_to_shape::codegen::{ArtMetadata, shapes_to_art_file};
use img_to_shape::{ConvertConfig, ResizeMethod};

use crate::codegen::shapes_to_rust_code;

/// Loaded image data: raw RGBA bytes + dimensions.
pub struct LoadedImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

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

/// Top-level application state for the img-to-shape GUI.
pub struct AppState {
    pub image: Option<LoadedImage>,
    pub config: ConvertConfig,
    pub shapes: Vec<Shape>,
    /// The resized RGBA pixel buffer fed into segmentation (for preview display).
    pub resized_rgba: Vec<u8>,
    /// Dimensions of the coordinate space the shapes live in (may differ from
    /// the source image when resizing is active).
    pub shape_width: u32,
    pub shape_height: u32,
    /// Index into ELEMENTS for the selected element (0–7).
    pub element_index: usize,
    /// 0 = positive aspect, 1 = negative aspect.
    pub aspect_pole: usize,
    /// Signature axes for the art metadata.
    pub signature_axes: [f32; 8],
    /// Function name for the generated file.
    pub fn_name: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            image: None,
            config: ConvertConfig {
                color_threshold: 0.1,
                alpha_threshold: 128,
                rdp_epsilon: 0.5,
                bezier_error: 0.5,
                min_area: 4,
                max_dimension: 128,
                resize_method: ResizeMethod::Scale2x,
                use_bezier: true,
            },
            shapes: Vec::new(),
            resized_rgba: Vec::new(),
            shape_width: 0,
            shape_height: 0,
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
            fn_name: String::new(),
        }
    }

    /// Load new image data, clearing any previously computed shapes.
    pub fn load_image(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        self.image = Some(LoadedImage {
            rgba,
            width,
            height,
        });
        self.shapes.clear();
        self.resized_rgba.clear();
        self.shape_width = 0;
        self.shape_height = 0;
    }

    /// Run the conversion pipeline on the loaded image with current config.
    /// No-op if no image is loaded.
    pub fn run_conversion(&mut self) {
        let Some(img) = &self.image else { return };
        let result = img_to_shape::image_to_shapes(&img.rgba, img.width, img.height, &self.config);
        self.shapes = result.shapes;
        self.resized_rgba = result.rgba;
        self.shape_width = result.width;
        self.shape_height = result.height;
    }

    /// Update the conversion config. Does NOT auto-recompute shapes.
    pub fn set_config(&mut self, config: ConvertConfig) {
        self.config = config;
    }

    /// Generate Rust source code for the current shapes (legacy vec literal).
    pub fn generate_export_code(&self) -> String {
        shapes_to_rust_code(&self.shapes)
    }

    /// Generate a complete `.rs` file with compact `Vec<Shape>` data.
    pub fn generate_art_file(&self) -> String {
        let metadata = ArtMetadata {
            element: ELEMENTS[self.element_index],
            aspect: ASPECTS[self.element_index][self.aspect_pole],
            signature_axes: self.signature_axes,
        };
        shapes_to_art_file(&self.shapes, &metadata, &self.fn_name)
    }

    /// The auto-save filename derived from `fn_name` (e.g. "armor01" → "armor01.rs").
    pub fn art_filename(&self) -> String {
        let name = if self.fn_name.is_empty() {
            "art_mesh"
        } else {
            &self.fn_name
        };
        format!("{name}.rs")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_3x3_single_pixel_image() -> (Vec<u8>, u32, u32) {
        // 4x4 image with a 2x2 red block (4 pixels, meets default min_area=4)
        let mut rgba = vec![0u8; 4 * 4 * 4];
        for row in 1..3 {
            for col in 1..3 {
                let idx = (row * 4 + col) * 4;
                rgba[idx] = 255;
                rgba[idx + 3] = 255;
            }
        }
        (rgba, 4, 4)
    }

    // TC021
    #[test]
    fn when_app_starts_then_state_is_empty() {
        // Act
        let state = AppState::new();

        // Assert
        assert!(state.image.is_none());
        assert!(state.shapes.is_empty());
    }

    // TC022
    #[test]
    fn when_image_loaded_into_state_then_image_is_set_and_shapes_empty() {
        // Arrange
        let mut state = AppState::new();
        let (rgba, w, h) = make_3x3_single_pixel_image();

        // Act
        state.load_image(rgba, w, h);

        // Assert
        assert!(state.image.is_some());
        assert!(state.shapes.is_empty());
    }

    // TC023
    #[test]
    fn when_convert_triggered_with_image_then_shapes_are_populated() {
        // Arrange
        let mut state = AppState::new();
        let (rgba, w, h) = make_3x3_single_pixel_image();
        state.load_image(rgba, w, h);

        // Act
        state.run_conversion();

        // Assert
        assert!(
            !state.shapes.is_empty(),
            "shapes should be populated after conversion"
        );
    }

    // TC024
    #[test]
    fn when_convert_triggered_without_image_then_shapes_remain_empty() {
        // Arrange
        let mut state = AppState::new();

        // Act
        state.run_conversion();

        // Assert
        assert!(state.shapes.is_empty());
    }

    // TC025
    #[test]
    fn when_config_changed_after_conversion_then_shapes_are_stale() {
        // Arrange
        let mut state = AppState::new();
        let (rgba, w, h) = make_3x3_single_pixel_image();
        state.load_image(rgba, w, h);
        state.run_conversion();
        let shapes_before = state.shapes.clone();

        // Act
        state.set_config(ConvertConfig {
            color_threshold: 0.5,
            alpha_threshold: 64,
            rdp_epsilon: 2.0,
            bezier_error: 2.0,
            min_area: 4,
            max_dimension: 128,
            resize_method: ResizeMethod::Scale2x,
            use_bezier: true,
        });

        // Assert
        assert_eq!(
            state.shapes, shapes_before,
            "shapes should not auto-update on config change"
        );
    }

    // TC026
    #[test]
    fn when_shapes_present_then_export_generates_non_empty_rust_source() {
        // Arrange
        let mut state = AppState::new();
        let (rgba, w, h) = make_3x3_single_pixel_image();
        state.load_image(rgba, w, h);
        state.run_conversion();

        // Act
        let code = state.generate_export_code();

        // Assert
        assert!(!code.is_empty());
        assert!(code.contains("vec!"), "should contain vec! macro: {code}");
    }

    // TC027
    #[test]
    fn when_no_shapes_present_then_export_generates_empty_vec_literal() {
        // Arrange
        let state = AppState::new();

        // Act
        let code = state.generate_export_code();

        // Assert
        assert_eq!(code, "vec![]");
    }

    // TC028b
    #[test]
    fn when_shapes_present_then_mesh_export_generates_file_with_pub_fn() {
        // Arrange
        let mut state = AppState::new();
        let (rgba, w, h) = make_3x3_single_pixel_image();
        state.load_image(rgba, w, h);
        state.run_conversion();
        state.fn_name = "test_art".to_string();

        // Act
        let code = state.generate_art_file();

        // Assert
        assert!(code.contains("pub fn test_art"), "missing pub fn:\n{code}");
        assert!(code.contains("Vec<Shape>"), "missing return type:\n{code}");
        assert!(code.contains("Solidum"), "missing default element:\n{code}");
    }

    // TC028
    #[test]
    fn when_new_image_loaded_after_conversion_then_old_shapes_cleared() {
        // Arrange
        let mut state = AppState::new();
        let (rgba, w, h) = make_3x3_single_pixel_image();
        state.load_image(rgba.clone(), w, h);
        state.run_conversion();
        assert!(!state.shapes.is_empty());

        // Act — load transparent image
        let transparent = vec![0u8; 3 * 3 * 4];
        state.load_image(transparent, 3, 3);

        // Assert
        assert!(
            state.shapes.is_empty(),
            "old shapes should be cleared on new image load"
        );
    }
}
