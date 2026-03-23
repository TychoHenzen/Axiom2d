mod bezier_fit;
mod contour;
mod segment;
mod simplify;
mod transform;

use engine_render::shape::{Shape, ShapeVariant};

/// Configuration for the image-to-shapes conversion pipeline.
pub struct ConvertConfig {
    /// Maximum Euclidean distance in normalized RGB space (0.0–1.0) for two
    /// adjacent pixels to be considered the "same color" during flood-fill.
    pub color_threshold: f32,
    /// Minimum alpha (0–255) for a pixel to be considered non-transparent.
    pub alpha_threshold: u8,
    /// RDP simplification epsilon — larger values produce simpler shapes.
    pub rdp_epsilon: f32,
    /// Maximum error for bezier curve fitting — larger values produce fewer,
    /// less precise curves.
    pub bezier_error: f32,
}

/// Convert an RGBA pixel buffer into a vector of engine `Shape`s.
///
/// Each flood-fill color region in the image becomes one `Shape` with a
/// `ShapeVariant::Path` contour and the region's average color.
pub fn image_to_shapes(rgba: &[u8], width: u32, height: u32, config: &ConvertConfig) -> Vec<Shape> {
    if rgba.is_empty() || width == 0 || height == 0 {
        return Vec::new();
    }

    let regions = segment::segment(
        rgba,
        width,
        height,
        config.color_threshold,
        config.alpha_threshold,
    );
    let w = width as f32;
    let h = height as f32;

    let mut shapes = Vec::new();
    for region in &regions {
        let contours = contour::trace_contours(&region.mask, width, height);
        for contour_pts in &contours {
            let float_pts: Vec<(f32, f32)> = contour_pts
                .iter()
                .map(|&(x, y)| (x as f32, y as f32))
                .collect();

            let simplified = simplify::rdp_simplify(&float_pts, config.rdp_epsilon);

            let engine_pts: Vec<(f32, f32)> = simplified
                .iter()
                .map(|&(x, y)| {
                    let v = transform::pixel_to_engine(x, y, w, h);
                    (v.x, v.y)
                })
                .collect();

            let commands = bezier_fit::fit_bezier_path(&engine_pts, config.bezier_error);
            if !commands.is_empty() {
                shapes.push(Shape {
                    variant: ShapeVariant::Path { commands },
                    color: region.color,
                });
            }
        }
    }

    shapes
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use engine_render::shape::PathCommand;
    use glam::Vec2;

    fn default_config() -> ConvertConfig {
        ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
        }
    }

    #[test]
    fn when_empty_rgba_then_returns_empty() {
        // Arrange / Act
        let shapes = image_to_shapes(&[], 0, 0, &default_config());

        // Assert
        assert!(shapes.is_empty());
    }

    #[test]
    fn when_fully_transparent_image_then_returns_empty() {
        // Arrange — 4x4 all alpha=0
        let rgba = vec![0u8; 4 * 4 * 4];

        // Act
        let shapes = image_to_shapes(&rgba, 4, 4, &default_config());

        // Assert
        assert!(shapes.is_empty());
    }

    #[test]
    fn when_single_opaque_pixel_then_returns_one_path() {
        // Arrange — 3x3, only center pixel opaque red
        let mut rgba = vec![0u8; 3 * 3 * 4];
        let center_byte = (3 + 1) * 4; // pixel (1,1)
        rgba[center_byte] = 255;
        rgba[center_byte + 1] = 0;
        rgba[center_byte + 2] = 0;
        rgba[center_byte + 3] = 255;

        // Act
        let shapes = image_to_shapes(&rgba, 3, 3, &default_config());

        // Assert
        assert_eq!(shapes.len(), 1);
        assert!(matches!(shapes[0].variant, ShapeVariant::Path { .. }));
    }

    #[test]
    fn when_single_opaque_pixel_then_path_starts_with_moveto_ends_with_close() {
        // Arrange — 3x3, only center pixel opaque
        let mut rgba = vec![0u8; 3 * 3 * 4];
        let center_byte = (3 + 1) * 4; // pixel (1,1)
        rgba[center_byte] = 255;
        rgba[center_byte + 3] = 255;

        // Act
        let shapes = image_to_shapes(&rgba, 3, 3, &default_config());

        // Assert
        let commands = match &shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path variant"),
        };
        assert!(matches!(commands[0], PathCommand::MoveTo(_)));
        assert!(matches!(*commands.last().unwrap(), PathCommand::Close));
    }

    #[test]
    fn when_two_disconnected_regions_then_returns_two_shapes() {
        // Arrange — 9x3, two 2x2 red blocks separated by transparent gap
        let mut rgba = vec![0u8; 9 * 3 * 4];
        // Block 1: columns 0-1, rows 0-1
        for row in 0..2 {
            for col in 0..2 {
                let idx = (row * 9 + col) * 4;
                rgba[idx] = 255;
                rgba[idx + 3] = 255;
            }
        }
        // Block 2: columns 6-7, rows 0-1
        for row in 0..2 {
            for col in 6..8 {
                let idx = (row * 9 + col) * 4;
                rgba[idx] = 255;
                rgba[idx + 3] = 255;
            }
        }

        // Act
        let shapes = image_to_shapes(&rgba, 9, 3, &default_config());

        // Assert
        assert_eq!(shapes.len(), 2);
        assert!(
            shapes
                .iter()
                .all(|s| matches!(s.variant, ShapeVariant::Path { .. }))
        );
    }

    #[test]
    fn when_fully_opaque_square_then_path_coordinates_centered_at_origin() {
        // Arrange — 10x10 all opaque white
        let rgba = vec![255u8; 10 * 10 * 4];

        // Act
        let shapes = image_to_shapes(&rgba, 10, 10, &default_config());

        // Assert
        assert_eq!(shapes.len(), 1);
        let commands = match &shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path variant"),
        };
        // Extract all Vec2 coordinates from commands
        let mut has_neg_x = false;
        let mut has_pos_x = false;
        let mut has_neg_y = false;
        let mut has_pos_y = false;
        for cmd in commands {
            let pts: Vec<Vec2> = match cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => vec![*v],
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } => vec![*control1, *control2, *to],
                _ => vec![],
            };
            for v in pts {
                if v.x < 0.0 {
                    has_neg_x = true;
                }
                if v.x > 0.0 {
                    has_pos_x = true;
                }
                if v.y < 0.0 {
                    has_neg_y = true;
                }
                if v.y > 0.0 {
                    has_pos_y = true;
                }
            }
        }
        assert!(
            has_neg_x && has_pos_x,
            "path should span both sides of x-axis"
        );
        assert!(
            has_neg_y && has_pos_y,
            "path should span both sides of y-axis"
        );
    }

    #[test]
    fn when_fully_opaque_square_then_top_edge_has_positive_y() {
        // Arrange — 10x10 all opaque white
        let rgba = vec![255u8; 10 * 10 * 4];

        // Act
        let shapes = image_to_shapes(&rgba, 10, 10, &default_config());

        // Assert — maximum Y should be positive (top of image = positive y)
        let max_y = match &shapes[0].variant {
            ShapeVariant::Path { commands } => commands
                .iter()
                .filter_map(|cmd| match cmd {
                    PathCommand::MoveTo(v) | PathCommand::LineTo(v) => Some(v.y),
                    _ => None,
                })
                .fold(f32::NEG_INFINITY, f32::max),
            _ => panic!("expected Path variant"),
        };
        assert!(
            max_y > 0.0,
            "top edge should map to positive y, got {max_y}"
        );
    }

    #[test]
    fn when_circle_image_with_large_epsilon_then_fewer_commands_than_boundary_pixels() {
        // Arrange — 20x20 image with a solid circle (radius ~8)
        let mut rgba = vec![0u8; 20 * 20 * 4];
        let cx = 10.0_f32;
        let cy = 10.0_f32;
        let r = 8.0_f32;
        for row in 0..20 {
            for col in 0..20 {
                let dx = col as f32 + 0.5 - cx;
                let dy = row as f32 + 0.5 - cy;
                if dx * dx + dy * dy <= r * r {
                    let idx = (row * 20 + col) * 4;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 128;
                    rgba[idx + 3] = 255;
                }
            }
        }
        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 2.0,
            bezier_error: 1.0,
        };

        // Act
        let shapes = image_to_shapes(&rgba, 20, 20, &config);

        // Assert — the circle boundary has ~50 pixels, commands should be fewer
        assert_eq!(shapes.len(), 1);
        let cmd_count = match &shapes[0].variant {
            ShapeVariant::Path { commands } => commands.len(),
            _ => panic!("expected Path variant"),
        };
        assert!(
            cmd_count < 50,
            "expected fewer than 50 commands for simplified circle, got {cmd_count}"
        );
    }
}
