mod bezier_fit;
pub mod codegen;
mod contour;
mod segment;
mod simplify;
mod transform;

use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;

/// Compute the convex hull of a set of 2D points (Andrew's monotone chain).
///
/// Returns the hull vertices in counter-clockwise order. The output is always
/// a simple convex polygon, which eliminates self-intersection artifacts.
fn convex_hull(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut pts: Vec<(f32, f32)> = points.to_vec();
    pts.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then(a.1.partial_cmp(&b.1).unwrap())
    });
    pts.dedup();
    let n = pts.len();
    if n < 3 {
        return pts;
    }

    let cross = |o: (f32, f32), a: (f32, f32), b: (f32, f32)| -> f32 {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };

    // Lower hull
    let mut lower: Vec<(f32, f32)> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    // Upper hull
    let mut upper: Vec<(f32, f32)> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    // Remove last point of each half because it's repeated
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Evaluate a cubic bezier at parameter t.
fn sample_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// Check whether a point is outside the bounding box.
fn is_out_of_bounds(p: Vec2, x_lo: f32, x_hi: f32, y_lo: f32, y_hi: f32) -> bool {
    p.x < x_lo || p.x > x_hi || p.y < y_lo || p.y > y_hi
}

/// Replace any `CubicTo` whose curve leaves the image bounding box with `LineTo`.
///
/// First checks whether any control point is outside bounds (quick reject).
/// Then samples the curve densely; if any sample falls outside the bounds,
/// the curve is replaced with a straight line to its endpoint. This catches
/// self-intersecting curves, wild control points, and singularity artifacts.
fn cull_out_of_bounds_cubics(
    commands: Vec<PathCommand>,
    half_w: f32,
    half_h: f32,
) -> Vec<PathCommand> {
    let x_lo = -half_w;
    let x_hi = half_w;
    let y_lo = -half_h;
    let y_hi = half_h;

    let mut prev_end = Vec2::ZERO;
    commands
        .into_iter()
        .map(|cmd| {
            match &cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => prev_end = *v,
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } => {
                    let start = prev_end;
                    prev_end = *to;

                    // Quick reject: if any control point is outside bounds,
                    // use denser sampling since the curve might overshoot.
                    let any_control_oob = is_out_of_bounds(*control1, x_lo, x_hi, y_lo, y_hi)
                        || is_out_of_bounds(*control2, x_lo, x_hi, y_lo, y_hi);
                    let samples = if any_control_oob { 200 } else { 69 };

                    for i in 1..=samples {
                        let t = i as f32 / (samples + 1) as f32;
                        let p = sample_cubic(start, *control1, *control2, *to, t);
                        if is_out_of_bounds(p, x_lo, x_hi, y_lo, y_hi) {
                            return PathCommand::LineTo(*to);
                        }
                    }
                }
                _ => {}
            }
            cmd
        })
        .collect()
}

/// Clamp all coordinates in path commands to the image bounding box.
///
/// Safety net that catches any residual out-of-bounds coordinates after
/// bezier fitting and cubic culling. For `CubicTo`, clamps control points
/// to the bounds (which slightly changes curve shape but keeps it in-frame).
fn clamp_to_bounds(commands: Vec<PathCommand>, half_w: f32, half_h: f32) -> Vec<PathCommand> {
    let clamp =
        |v: Vec2| -> Vec2 { Vec2::new(v.x.clamp(-half_w, half_w), v.y.clamp(-half_h, half_h)) };
    commands
        .into_iter()
        .map(|cmd| match cmd {
            PathCommand::MoveTo(v) => PathCommand::MoveTo(clamp(v)),
            PathCommand::LineTo(v) => PathCommand::LineTo(clamp(v)),
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => PathCommand::CubicTo {
                control1: clamp(control1),
                control2: clamp(control2),
                to: clamp(to),
            },
            other => other,
        })
        .collect()
}

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
    /// Minimum pixel count for a region to produce a shape. Regions smaller
    /// than this are discarded. Use 0 to keep all regions.
    pub min_area: usize,
    /// Maximum size for the longest dimension. Images larger than this are
    /// downscaled (preserving aspect ratio) before processing. The output
    /// shapes use the original image dimensions for coordinate mapping.
    /// Use 0 to disable downscaling.
    pub max_dimension: u32,
}

/// Downscale RGBA pixel data by nearest-neighbor sampling.
fn downscale_rgba(rgba: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx * src_w / dst_w).min(src_w - 1);
            let sy = (dy * src_h / dst_h).min(src_h - 1);
            let si = (sy * src_w + sx) as usize * 4;
            let di = (dy * dst_w + dx) as usize * 4;
            out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    out
}

/// Convert an RGBA pixel buffer into a vector of engine `Shape`s.
///
/// Each flood-fill color region in the image becomes one `Shape` with a
/// `ShapeVariant::Path` contour and the region's average color.
/// When `max_dimension` is set, the image is downscaled first and the output
/// shapes use the downscaled dimensions as their coordinate space.
///
/// Returns `(shapes, output_width, output_height)` so callers know the
/// coordinate space the shapes live in.
pub fn image_to_shapes(
    rgba: &[u8],
    width: u32,
    height: u32,
    config: &ConvertConfig,
) -> (Vec<Shape>, u32, u32) {
    if rgba.is_empty() || width == 0 || height == 0 {
        return (Vec::new(), width, height);
    }

    let max_dim = config.max_dimension;
    let (work_rgba, work_w, work_h);
    if max_dim > 0 && (width > max_dim || height > max_dim) {
        let scale = max_dim as f32 / width.max(height) as f32;
        work_w = (width as f32 * scale).round() as u32;
        work_h = (height as f32 * scale).round() as u32;
        work_rgba = downscale_rgba(rgba, width, height, work_w, work_h);
    } else {
        work_w = width;
        work_h = height;
        work_rgba = rgba.to_vec();
    }

    let regions = segment::segment(
        &work_rgba,
        work_w,
        work_h,
        config.color_threshold,
        config.alpha_threshold,
    );
    let w = work_w as f32;
    let h = work_h as f32;

    let mut shapes: Vec<(usize, Shape)> = Vec::new();
    for region in &regions {
        let area = region.mask.iter().filter(|&&b| b).count();
        if config.min_area > 0 && area < config.min_area {
            continue;
        }
        let contours = contour::trace_contours(&region.mask, work_w, work_h);
        for contour_pts in &contours {
            let float_pts: Vec<(f32, f32)> = contour_pts
                .iter()
                .map(|&(x, y)| (x as f32, y as f32))
                .collect();

            // Convex hull: guarantees the polygon is simple (non-self-
            // intersecting) so fill tessellation works correctly.
            // For tiny regions (< 3 unique points), keep original points.
            let hull = convex_hull(&float_pts);
            let base_pts = if hull.len() >= 3 { &hull } else { &float_pts };

            let simplified = simplify::rdp_simplify(base_pts, config.rdp_epsilon);
            let engine_pts: Vec<(f32, f32)> = simplified
                .iter()
                .map(|&(x, y)| {
                    let v = transform::pixel_to_engine(x, y, w, h);
                    (v.x, v.y)
                })
                .collect();

            let raw_commands = bezier_fit::fit_bezier_path(&engine_pts, config.bezier_error);
            if !raw_commands.is_empty() {
                let half_w = w / 2.0;
                let half_h = h / 2.0;
                let culled = cull_out_of_bounds_cubics(raw_commands, half_w, half_h);
                let commands = clamp_to_bounds(culled, half_w, half_h);
                shapes.push((
                    area,
                    Shape {
                        variant: ShapeVariant::Path { commands },
                        color: region.color,
                    },
                ));
            }
        }
    }

    // Sort largest first so big shapes act as background, small details on top.
    shapes.sort_by(|a, b| b.0.cmp(&a.0));
    (
        shapes.into_iter().map(|(_, shape)| shape).collect(),
        work_w,
        work_h,
    )
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
            min_area: 0,
            max_dimension: 0,
        }
    }

    #[test]
    fn when_empty_rgba_then_returns_empty() {
        // Arrange / Act
        let (shapes, _, _) = image_to_shapes(&[], 0, 0, &default_config());

        // Assert
        assert!(shapes.is_empty());
    }

    #[test]
    fn when_fully_transparent_image_then_returns_empty() {
        // Arrange — 4x4 all alpha=0
        let rgba = vec![0u8; 4 * 4 * 4];

        // Act
        let (shapes, _, _) = image_to_shapes(&rgba, 4, 4, &default_config());

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
        let (shapes, _, _) = image_to_shapes(&rgba, 3, 3, &default_config());

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
        let (shapes, _, _) = image_to_shapes(&rgba, 3, 3, &default_config());

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
        let (shapes, _, _) = image_to_shapes(&rgba, 9, 3, &default_config());

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
        let (shapes, _, _) = image_to_shapes(&rgba, 10, 10, &default_config());

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
        let (shapes, _, _) = image_to_shapes(&rgba, 10, 10, &default_config());

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
            min_area: 0,
            max_dimension: 0,
        };

        // Act
        let (shapes, _, _) = image_to_shapes(&rgba, 20, 20, &config);

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

    #[test]
    fn when_gradient_segmented_with_min_area_then_tiny_regions_discarded() {
        // Arrange — 6x1 gradient from red to orange, tight threshold fragments it
        let mut rgba = vec![0u8; 6 * 1 * 4];
        for col in 0..6 {
            let idx = col * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = (col as u8) * 25; // G: 0, 25, 50, 75, 100, 125
            rgba[idx + 2] = 0;
            rgba[idx + 3] = 255;
        }
        let config = ConvertConfig {
            color_threshold: 0.05,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 4,
            max_dimension: 0,
        };

        // Act
        let (shapes, _, _) = image_to_shapes(&rgba, 6, 1, &config);

        // Assert — all surviving shapes must come from regions with >= 4 pixels
        // With threshold=0.05 on a 6-pixel gradient, no single flood-fill
        // region can reach 4 pixels, so everything should be filtered out.
        assert!(
            shapes.is_empty(),
            "expected no shapes (all regions < min_area=4), got {}",
            shapes.len()
        );
    }

    #[test]
    fn when_min_area_zero_then_all_regions_kept() {
        // Arrange — same gradient, but min_area=0 keeps everything
        let mut rgba = vec![0u8; 6 * 1 * 4];
        for col in 0..6 {
            let idx = col * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = (col as u8) * 25;
            rgba[idx + 2] = 0;
            rgba[idx + 3] = 255;
        }
        let config = ConvertConfig {
            color_threshold: 0.05,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
        };

        // Act
        let (shapes, _, _) = image_to_shapes(&rgba, 6, 1, &config);

        // Assert — at least one shape exists (no filtering)
        assert!(!shapes.is_empty(), "min_area=0 should keep all regions");
    }

    #[test]
    fn when_large_region_present_then_not_discarded_by_min_area() {
        // Arrange — 4x4 all-red opaque (16 pixels, well above min_area=4)
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 4,
            max_dimension: 0,
        };

        // Act
        let (shapes, _, _) = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(
            shapes.len(),
            1,
            "large region should survive min_area filter"
        );
    }

    #[test]
    fn when_large_triangle_then_output_has_few_commands() {
        // Arrange — 40x20 filled triangle (tip at top-center, base at bottom)
        let w = 40u32;
        let h = 20u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            // Triangle: at each row, the opaque span widens from center
            let half_width = (row as f32 / (h - 1) as f32 * (w as f32 / 2.0)) as u32;
            let cx = w / 2;
            let left = cx.saturating_sub(half_width);
            let right = (cx + half_width).min(w - 1);
            for col in left..=right {
                let idx = ((row * w + col) * 4) as usize;
                rgba[idx] = 255;
                rgba[idx + 1] = 128;
                rgba[idx + 3] = 255;
            }
        }
        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
        };

        // Act
        let (shapes, _, _) = image_to_shapes(&rgba, w, h, &config);

        // Assert — a triangle should produce a single shape with relatively few commands
        assert_eq!(shapes.len(), 1);
        let cmd_count = match &shapes[0].variant {
            ShapeVariant::Path { commands } => commands.len(),
            _ => panic!("expected Path variant"),
        };
        assert!(
            cmd_count < 20,
            "large triangle should produce fewer than 20 commands (got {cmd_count}); \
             staircase pre-smoothing should collapse diagonal pixel steps"
        );
    }
}
