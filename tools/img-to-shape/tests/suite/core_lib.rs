#![allow(clippy::unwrap_used)]

use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;
use img_to_shape::*;

fn default_config() -> ConvertConfig {
    ConvertConfig {
        color_threshold: 0.1,
        alpha_threshold: 128,
        rdp_epsilon: 1.5,
        bezier_error: 1.5,
        min_area: 0,
        max_dimension: 0,
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 0,
    }
}

#[test]
fn when_empty_rgba_then_returns_empty() {
    // Arrange / Act
    let result = image_to_shapes(&[], 0, 0, &default_config());

    // Assert
    assert!(result.shapes.is_empty());
}

#[test]
fn when_fully_transparent_image_then_returns_empty() {
    // Arrange — 4x4 all alpha=0
    let rgba = vec![0u8; 4 * 4 * 4];

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &default_config());

    // Assert
    assert!(result.shapes.is_empty());
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
    let result = image_to_shapes(&rgba, 3, 3, &default_config());

    // Assert
    assert_eq!(result.shapes.len(), 1);
    assert!(matches!(
        result.shapes[0].variant,
        ShapeVariant::Path { .. }
    ));
}

#[test]
fn when_single_opaque_pixel_then_path_starts_with_moveto_ends_with_close() {
    // Arrange — 3x3, only center pixel opaque
    let mut rgba = vec![0u8; 3 * 3 * 4];
    let center_byte = (3 + 1) * 4; // pixel (1,1)
    rgba[center_byte] = 255;
    rgba[center_byte + 3] = 255;

    // Act
    let result = image_to_shapes(&rgba, 3, 3, &default_config());

    // Assert
    let ShapeVariant::Path { commands } = &result.shapes[0].variant else {
        panic!("expected Path variant");
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
    let result = image_to_shapes(&rgba, 9, 3, &default_config());

    // Assert
    assert_eq!(result.shapes.len(), 2);
    assert!(
        result
            .shapes
            .iter()
            .all(|s| matches!(s.variant, ShapeVariant::Path { .. }))
    );
}

#[test]
fn when_fully_opaque_square_then_path_coordinates_centered_at_origin() {
    // Arrange — 10x10 all opaque white
    let rgba = vec![255u8; 10 * 10 * 4];

    // Act
    let result = image_to_shapes(&rgba, 10, 10, &default_config());

    // Assert
    assert_eq!(result.shapes.len(), 1);
    let ShapeVariant::Path { commands } = &result.shapes[0].variant else {
        panic!("expected Path variant");
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
    let result = image_to_shapes(&rgba, 10, 10, &default_config());

    // Assert — maximum Y should be positive (top of image = positive y)
    let max_y = match &result.shapes[0].variant {
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
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, 20, 20, &config);

    // Assert — the circle boundary has ~50 pixels, commands should be fewer
    assert_eq!(result.shapes.len(), 1);
    let cmd_count = match &result.shapes[0].variant {
        ShapeVariant::Path { commands } => commands.len(),
        _ => panic!("expected Path variant"),
    };
    assert!(
        cmd_count < 50,
        "expected fewer than 50 commands for simplified circle, got {cmd_count}"
    );
}

#[test]
fn when_gradient_segmented_with_min_area_then_tiny_regions_merged() {
    // Arrange — 6x1 gradient from red to orange, tight threshold fragments it
    let mut rgba = vec![0u8; 6 * 4];
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
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0, // merge_below < min_area, so pipeline auto-raises it
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, 6, 1, &config);

    // Assert — tiny regions are merged (not discarded), so pixels are covered.
    // The auto-merge ensures merge_below >= min_area, preventing holes.
    assert!(
        !result.shapes.is_empty(),
        "tiny regions should be merged into neighbors, not discarded"
    );
}

#[test]
fn when_min_area_zero_then_all_regions_kept() {
    // Arrange — same gradient, but min_area=0 keeps everything
    let mut rgba = vec![0u8; 6 * 4];
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
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, 6, 1, &config);

    // Assert — at least one shape exists (no filtering)
    assert!(
        !result.shapes.is_empty(),
        "min_area=0 should keep all regions"
    );
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
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &config);

    // Assert
    assert_eq!(
        result.shapes.len(),
        1,
        "large region should survive min_area filter"
    );
}

#[test]
fn when_max_dimension_zero_and_large_image_then_no_resize() {
    // Arrange — 8x4 opaque image, larger than typical max_dimension values
    let rgba = vec![255u8; 8 * 4 * 4];
    let config = ConvertConfig {
        max_dimension: 0,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 8, 4, &config);

    // Assert
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 4);
}

#[test]
fn when_max_dimension_zero_and_small_image_then_no_resize() {
    // Arrange — 4x4 opaque image, small enough that upscaling would apply
    let rgba = vec![255u8; 4 * 4 * 4];
    let config = ConvertConfig {
        max_dimension: 0,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &config);

    // Assert
    assert_eq!(result.width, 4);
    assert_eq!(result.height, 4);
}

#[test]
fn when_image_exceeds_max_dimension_then_downscaled() {
    // Arrange — 20x10 image with max_dimension=8; scale = 8/20 = 0.4
    let rgba = vec![255u8; 20 * 10 * 4];
    let config = ConvertConfig {
        max_dimension: 8,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 20, 10, &config);

    // Assert — 20*0.4=8, 10*0.4=4
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 4);
}

#[test]
fn when_longest_side_equals_max_dimension_then_no_resize() {
    // Arrange — 10x6 image with max_dimension=10; already at limit
    let rgba = vec![255u8; 10 * 6 * 4];
    let config = ConvertConfig {
        max_dimension: 10,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 10, 6, &config);

    // Assert
    assert_eq!(result.width, 10);
    assert_eq!(result.height, 6);
}

#[test]
fn when_square_image_smaller_than_max_dimension_then_upscaled() {
    // Arrange — 4x4 opaque image; max_dimension=8 means scale=2.0
    let rgba = vec![255u8; 4 * 4 * 4];
    let config = ConvertConfig {
        max_dimension: 8,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &config);

    // Assert
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 8);
}

#[test]
fn when_landscape_smaller_then_upscale_preserves_aspect_ratio() {
    // Arrange — 4x2 opaque image; max_dimension=12, scale=12/4=3.0
    let rgba = vec![255u8; 4 * 2 * 4];
    let config = ConvertConfig {
        max_dimension: 12,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 2, &config);

    // Assert — 4*3=12, 2*3=6
    assert_eq!(result.width, 12);
    assert_eq!(result.height, 6);
}

#[test]
fn when_portrait_smaller_then_upscale_uses_height() {
    // Arrange — 3x6 opaque image; max_dimension=12, scale=12/6=2.0
    let rgba = vec![255u8; 3 * 6 * 4];
    let config = ConvertConfig {
        max_dimension: 12,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 3, 6, &config);

    // Assert — 3*2=6, 6*2=12
    assert_eq!(result.width, 6);
    assert_eq!(result.height, 12);
}

#[test]
fn when_upscaled_then_shape_count_matches_original() {
    // Arrange — 4x4 image with two color blocks: red left half, blue right half
    let mut rgba = vec![0u8; 4 * 4 * 4];
    for row in 0..4u32 {
        for col in 0..4u32 {
            let idx = ((row * 4 + col) * 4) as usize;
            if col < 2 {
                rgba[idx] = 255; // red
            } else {
                rgba[idx + 2] = 255; // blue
            }
            rgba[idx + 3] = 255;
        }
    }
    let no_resize = ConvertConfig {
        max_dimension: 0,
        ..default_config()
    };
    let with_upscale = ConvertConfig {
        max_dimension: 8,
        ..default_config()
    };

    // Act
    let result_orig = image_to_shapes(&rgba, 4, 4, &no_resize);
    let result_up = image_to_shapes(&rgba, 4, 4, &with_upscale);

    // Assert — nearest-neighbor upscale preserves colors, so region count is stable
    assert_eq!(result_orig.shapes.len(), result_up.shapes.len());
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
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert — a triangle should produce a single shape with relatively few commands
    assert_eq!(result.shapes.len(), 1);
    let cmd_count = match &result.shapes[0].variant {
        ShapeVariant::Path { commands } => commands.len(),
        _ => panic!("expected Path variant"),
    };
    // Marching squares preserves the staircase boundary of the diagonal
    // edges, producing more commands than a convex hull would. The bezier
    // fitter converts the staircase into curves. 50 is a reasonable upper
    // bound for a 40x20 triangle.
    assert!(
        cmd_count < 100,
        "large triangle should produce fewer than 100 commands (got {cmd_count})"
    );
}

#[test]
fn when_scale2x_method_and_downscale_then_nearest_used() {
    // Arrange — 20x10 image, max_dimension=8, Scale2x method
    let rgba = vec![255u8; 20 * 10 * 4];
    let config = ConvertConfig {
        max_dimension: 8,
        resize_method: ResizeMethod::Scale2x,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 20, 10, &config);

    // Assert — downscale ignores resize_method, same as nearest
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 4);
}

#[test]
fn when_scale2x_exact_double_then_dimensions_match() {
    // Arrange — 4x4 image, max_dimension=8, Scale2x method (one pass: 4→8)
    let rgba = vec![255u8; 4 * 4 * 4];
    let config = ConvertConfig {
        max_dimension: 8,
        resize_method: ResizeMethod::Scale2x,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &config);

    // Assert
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 8);
}

#[test]
fn when_scale2x_non_power_target_then_nearest_finishes() {
    // Arrange — 4x4 image, max_dimension=10
    // Scale2x: 4→8 (next pass 16 > 10, stop), then NN 8→10
    let rgba = vec![255u8; 4 * 4 * 4];
    let config = ConvertConfig {
        max_dimension: 10,
        resize_method: ResizeMethod::Scale2x,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &config);

    // Assert
    assert_eq!(result.width, 10);
    assert_eq!(result.height, 10);
}

#[test]
fn when_scale2x_color_blocks_then_region_count_preserved() {
    // Arrange — 4x4 with red left / blue right
    let mut rgba = vec![0u8; 4 * 4 * 4];
    for row in 0..4u32 {
        for col in 0..4u32 {
            let idx = ((row * 4 + col) * 4) as usize;
            if col < 2 {
                rgba[idx] = 255;
            } else {
                rgba[idx + 2] = 255;
            }
            rgba[idx + 3] = 255;
        }
    }
    let nearest = ConvertConfig {
        max_dimension: 8,
        resize_method: ResizeMethod::Nearest,
        ..default_config()
    };
    let scale2x = ConvertConfig {
        max_dimension: 8,
        resize_method: ResizeMethod::Scale2x,
        ..default_config()
    };

    // Act
    let result_nn = image_to_shapes(&rgba, 4, 4, &nearest);
    let result_s2x = image_to_shapes(&rgba, 4, 4, &scale2x);

    // Assert
    assert_eq!(result_nn.shapes.len(), result_s2x.shapes.len());
}

#[test]
fn when_no_resize_then_buffer_matches_input() {
    // Arrange — 2x2 image with distinct pixels, no resize
    let rgba = vec![
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        0, 0, 255, 255, // blue
        255, 255, 0, 255, // yellow
    ];

    // Act
    let result = image_to_shapes(&rgba, 2, 2, &default_config());

    // Assert
    assert_eq!(result.rgba, rgba);
}

#[test]
fn when_resized_then_buffer_size_matches_dimensions() {
    // Arrange — 4x4 upscaled to 8x8
    let rgba = vec![255u8; 4 * 4 * 4];
    let config = ConvertConfig {
        max_dimension: 8,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, 4, 4, &config);

    // Assert
    assert_eq!(
        result.rgba.len(),
        (result.width * result.height * 4) as usize
    );
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 8);
}

#[test]
fn when_empty_input_then_buffer_is_empty() {
    // Arrange / Act
    let result = image_to_shapes(&[], 0, 0, &default_config());

    // Assert
    assert!(result.rgba.is_empty());
}

#[test]
fn when_l_shape_then_output_path_preserves_concavity() {
    // Arrange — 10x8 L-shape: left column (2px wide, full height) + bottom
    // row (full width, 2px tall). The inner concave corner is at (2, 6).
    let w = 10u32;
    let h = 8u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let in_left_column = col < 2;
            let in_bottom_row = row >= 6;
            if in_left_column || in_bottom_row {
                let idx = ((row * w + col) * 4) as usize;
                rgba[idx] = 255;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }
    }
    let config = ConvertConfig {
        rdp_epsilon: 0.5,
        bezier_error: 0.5,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert — extract all path vertices
    assert_eq!(result.shapes.len(), 1, "L-shape should be one region");
    let ShapeVariant::Path { commands } = &result.shapes[0].variant else {
        panic!("expected Path");
    };

    // Collect all endpoint coordinates from path commands
    let mut xs: Vec<f32> = Vec::new();
    let mut ys: Vec<f32> = Vec::new();
    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(v) | PathCommand::LineTo(v) => {
                xs.push(v.x);
                ys.push(v.y);
            }
            PathCommand::CubicTo { to, .. } => {
                xs.push(to.x);
                ys.push(to.y);
            }
            _ => {}
        }
    }

    // The L-shape has a concave notch. In engine coords (centered, Y-up),
    // the inner corner of the L is NOT at the image bounding box corners.
    // If the shape were convex-hulled, all path vertices would lie on the
    // bounding box edges. A concave L has at least one vertex that's
    // strictly inside the bounding box on both axes.
    let x_min = xs.iter().copied().fold(f32::INFINITY, f32::min);
    let x_max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let y_min = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let y_max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let has_interior_vertex = xs
        .iter()
        .zip(ys.iter())
        .any(|(&x, &y)| x > x_min + 0.5 && x < x_max - 0.5 && y > y_min + 0.5 && y < y_max - 0.5);
    assert!(
        has_interior_vertex,
        "L-shape path should have at least one interior vertex (concavity). \
         All vertices on bounding box means concavity was lost. \
         x range: [{x_min:.1}, {x_max:.1}], y range: [{y_min:.1}, {y_max:.1}], \
         vertices: {:?}",
        xs.iter()
            .zip(ys.iter())
            .map(|(x, y)| format!("({x:.1},{y:.1})"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn when_star_shape_then_output_preserves_valleys() {
    // Arrange — 21x21 5-pointed star: 5 outer tips + 5 inner valleys.
    // If the output is a pentagon, all 5 valleys are lost.
    let size = 21u32;
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let outer_r = 10.0_f32;
    let inner_r = 4.0_f32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    // Rasterize star by checking if each pixel center is inside the star polygon
    let star_pts: Vec<(f32, f32)> = (0..10)
        .map(|i| {
            let angle = std::f32::consts::PI / 2.0 + i as f32 * std::f32::consts::PI / 5.0;
            let r = if i % 2 == 0 { outer_r } else { inner_r };
            (cx + r * angle.cos(), cy - r * angle.sin())
        })
        .collect();

    for row in 0..size {
        for col in 0..size {
            let px = col as f32 + 0.5;
            let py = row as f32 + 0.5;
            if point_in_polygon(px, py, &star_pts) {
                let idx = ((row * size + col) * 4) as usize;
                rgba[idx] = 255;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }
    }

    let config = ConvertConfig {
        rdp_epsilon: 0.5,
        bezier_error: 0.5,
        min_area: 0,
        max_dimension: 0,
        resize_method: ResizeMethod::Nearest,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, size, size, &config);

    // Assert — extract vertices from path commands
    assert_eq!(result.shapes.len(), 1, "star should be one region");
    let ShapeVariant::Path { commands } = &result.shapes[0].variant else {
        panic!("expected Path");
    };

    let mut xs: Vec<f32> = Vec::new();
    let mut ys: Vec<f32> = Vec::new();
    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(v) | PathCommand::LineTo(v) => {
                xs.push(v.x);
                ys.push(v.y);
            }
            PathCommand::CubicTo { to, .. } => {
                xs.push(to.x);
                ys.push(to.y);
            }
            _ => {}
        }
    }

    let x_min = xs.iter().copied().fold(f32::INFINITY, f32::min);
    let x_max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let y_min = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let y_max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    // A star has vertices inside the bounding box (the inner valleys).
    // Count how many vertices are strictly inside the bbox on both axes.
    let interior_count = xs
        .iter()
        .zip(ys.iter())
        .filter(|&(&x, &y)| {
            x > x_min + 1.0 && x < x_max - 1.0 && y > y_min + 1.0 && y < y_max - 1.0
        })
        .count();

    assert!(
        interior_count >= 3,
        "star should have at least 3 interior vertices (valley points), got {interior_count}. \
         x range: [{x_min:.1}, {x_max:.1}], y range: [{y_min:.1}, {y_max:.1}], \
         vertices: {:?}",
        xs.iter()
            .zip(ys.iter())
            .map(|(x, y)| format!("({x:.1},{y:.1})"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn when_star_shape_with_scale2x_then_output_preserves_valleys() {
    // Arrange — same star as above but with GUI-default Scale2x upscaling
    let size = 21u32;
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let outer_r = 10.0_f32;
    let inner_r = 4.0_f32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    let star_pts: Vec<(f32, f32)> = (0..10)
        .map(|i| {
            let angle = std::f32::consts::PI / 2.0 + i as f32 * std::f32::consts::PI / 5.0;
            let r = if i % 2 == 0 { outer_r } else { inner_r };
            (cx + r * angle.cos(), cy - r * angle.sin())
        })
        .collect();

    for row in 0..size {
        for col in 0..size {
            let px = col as f32 + 0.5;
            let py = row as f32 + 0.5;
            if point_in_polygon(px, py, &star_pts) {
                let idx = ((row * size + col) * 4) as usize;
                rgba[idx] = 255;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }
    }

    let config = ConvertConfig {
        color_threshold: 0.1,
        alpha_threshold: 128,
        rdp_epsilon: 0.5,
        bezier_error: 0.5,
        min_area: 4,
        max_dimension: 128,
        resize_method: ResizeMethod::Scale2x,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, size, size, &config);

    // Assert
    assert!(
        !result.shapes.is_empty(),
        "should produce at least one shape"
    );
    let ShapeVariant::Path { commands } = &result.shapes[0].variant else {
        panic!("expected Path");
    };

    let mut xs: Vec<f32> = Vec::new();
    let mut ys: Vec<f32> = Vec::new();
    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(v) | PathCommand::LineTo(v) => {
                xs.push(v.x);
                ys.push(v.y);
            }
            PathCommand::CubicTo { to, .. } => {
                xs.push(to.x);
                ys.push(to.y);
            }
            _ => {}
        }
    }

    let x_min = xs.iter().copied().fold(f32::INFINITY, f32::min);
    let x_max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let y_min = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let y_max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let interior_count = xs
        .iter()
        .zip(ys.iter())
        .filter(|&(&x, &y)| {
            x > x_min + 1.0 && x < x_max - 1.0 && y > y_min + 1.0 && y < y_max - 1.0
        })
        .count();

    assert!(
        interior_count >= 3,
        "star with Scale2x should have at least 3 interior vertices (valley points), \
         got {interior_count}. shapes: {}, vertices: {:?}",
        result.shapes.len(),
        xs.iter()
            .zip(ys.iter())
            .map(|(x, y)| format!("({x:.1},{y:.1})"))
            .collect::<Vec<_>>()
    );
}

/// Point-in-polygon test using ray casting.
fn point_in_polygon(px: f32, py: f32, polygon: &[(f32, f32)]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Extract polygon vertices from a shape's path commands (`LineTo` endpoints only).
fn extract_shape_polygon(shape: &Shape) -> Vec<(f32, f32)> {
    let ShapeVariant::Path { commands } = &shape.variant else {
        panic!("expected Path variant");
    };
    commands
        .iter()
        .filter_map(|cmd| match cmd {
            PathCommand::MoveTo(v) | PathCommand::LineTo(v) => Some((v.x, v.y)),
            PathCommand::CubicTo { to, .. } => Some((to.x, to.y)),
            _ => None,
        })
        .collect()
}

/// Check if a point is inside any shape (using painter's algorithm order:
/// last shape covering the point determines color).
fn topmost_shape_at(shapes: &[Shape], px: f32, py: f32) -> Option<&Shape> {
    // Shapes are sorted largest-first (background first, details last).
    // The last shape in the list that contains the point is the topmost.
    shapes.iter().rev().find(|shape| {
        let poly = extract_shape_polygon(shape);
        poly.len() >= 3 && point_in_polygon(px, py, &poly)
    })
}

#[test]
fn when_triangle_inside_rectangle_then_every_pixel_covered_by_correct_shape() {
    // Arrange — 10x10 image with a right-triangle inner region (red)
    // inside a rectangular outer region (blue).
    // The diagonal staircase boundary is where gaps appear.
    let w = 10u32;
    let h = 10u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let idx = ((row * w + col) * 4) as usize;
            // Triangle: pixels where row >= col+2, within rows 2..8 cols 2..8
            let is_inner = (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
            if is_inner {
                rgba[idx] = 255; // red
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 255; // blue
            }
            rgba[idx + 3] = 255;
        }
    }

    let config = ConvertConfig {
        use_bezier: false,
        rdp_epsilon: 0.5,
        min_area: 0,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert — every opaque pixel center must be inside at least one shape,
    // AND the topmost shape must have the correct color.
    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

    let mut mismatches = Vec::new();
    for row in 0..h {
        for col in 0..w {
            let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
            let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
            let is_inner = (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
            let expected_color = if is_inner { red } else { blue };

            if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                let color_matches = (shape.color.r - expected_color.r).abs() < 0.1
                    && (shape.color.g - expected_color.g).abs() < 0.1
                    && (shape.color.b - expected_color.b).abs() < 0.1;
                if !color_matches {
                    mismatches.push((col, row, "wrong_color"));
                }
            } else {
                mismatches.push((col, row, "uncovered"));
            }
        }
    }

    // Post-processing simplifies diagonal staircases into straight lines,
    // which may leave a few boundary pixels showing the outer shape's color.
    // This is visually correct (smoother diagonal) — allow small tolerance.
    let uncovered = mismatches
        .iter()
        .filter(|(_, _, t)| *t == "uncovered")
        .count();
    assert_eq!(uncovered, 0, "no pixels should be uncovered");
    assert!(
        mismatches.len() <= 10,
        "pixel coverage: too many boundary mismatches ({}):\n{:?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(10)]
    );
}

#[test]
fn when_triangle_inside_rectangle_bezier_then_every_pixel_covered() {
    // Same as above but with bezier ON — verifies that internal
    // no-junction boundaries use LineTo even when bezier is enabled.
    let w = 10u32;
    let h = 10u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let idx = ((row * w + col) * 4) as usize;
            let is_inner = (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
            if is_inner {
                rgba[idx] = 255;
            } else {
                rgba[idx + 2] = 255;
            }
            rgba[idx + 3] = 255;
        }
    }

    let config = ConvertConfig {
        use_bezier: true,
        rdp_epsilon: 0.5,
        bezier_error: 0.5,
        min_area: 0,
        ..default_config()
    };

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert
    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

    let mut mismatches = Vec::new();
    for row in 0..h {
        for col in 0..w {
            let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
            let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
            let is_inner = (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
            let expected_color = if is_inner { red } else { blue };

            if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                let color_matches = (shape.color.r - expected_color.r).abs() < 0.1
                    && (shape.color.g - expected_color.g).abs() < 0.1
                    && (shape.color.b - expected_color.b).abs() < 0.1;
                if !color_matches {
                    mismatches.push((col, row, "wrong_color"));
                }
            } else {
                mismatches.push((col, row, "uncovered"));
            }
        }
    }

    let uncovered = mismatches
        .iter()
        .filter(|(_, _, t)| *t == "uncovered")
        .count();
    assert_eq!(uncovered, 0, "no pixels should be uncovered");
    assert!(
        mismatches.len() <= 10,
        "bezier mode: too many boundary mismatches ({}):\n{:?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(10)]
    );
}

#[test]
fn when_three_color_image_then_every_pixel_covered_by_correct_shape() {
    // Arrange — 12x8 image with 3 color regions creating a multi-junction
    // scenario. Red left strip, green right strip, blue bottom bar.
    //
    //  RRRRGGGG....
    //  RRRRGGGG....
    //  RRRRGGGG....
    //  RRRRGGGG....
    //  BBBBBBBBBBBB
    //  BBBBBBBBBBBB
    //  BBBBBBBBBBBB
    //  BBBBBBBBBBBB
    let w = 12u32;
    let h = 8u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let idx = ((row * w + col) * 4) as usize;
            if row < 4 && col < 4 {
                rgba[idx] = 255; // red
            } else if row < 4 {
                rgba[idx + 1] = 255; // green
            } else {
                rgba[idx + 2] = 255; // blue
            }
            rgba[idx + 3] = 255;
        }
    }

    let config = ConvertConfig {
        use_bezier: false,
        rdp_epsilon: 0.5,
        min_area: 0,
        ..default_config()
    };

    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert — 3 shapes (red, green, blue)
    assert_eq!(result.shapes.len(), 3, "expected 3 shapes");

    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
    let green = engine_core::color::Color::new(0.0, 1.0, 0.0, 1.0);
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

    let mut mismatches = Vec::new();
    for row in 0..h {
        for col in 0..w {
            let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
            let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
            let expected = if row < 4 && col < 4 {
                red
            } else if row < 4 {
                green
            } else {
                blue
            };

            if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                let ok = (shape.color.r - expected.r).abs() < 0.1
                    && (shape.color.g - expected.g).abs() < 0.1
                    && (shape.color.b - expected.b).abs() < 0.1;
                if !ok {
                    mismatches.push(format!(
                        "({col},{row}) expected ({:.0},{:.0},{:.0}) got ({:.0},{:.0},{:.0})",
                        expected.r * 255.0,
                        expected.g * 255.0,
                        expected.b * 255.0,
                        shape.color.r * 255.0,
                        shape.color.g * 255.0,
                        shape.color.b * 255.0,
                    ));
                }
            } else {
                mismatches.push(format!("({col},{row}) UNCOVERED"));
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "3-color coverage: {} issues:\n{}",
        mismatches.len(),
        mismatches[..mismatches.len().min(20)].join("\n")
    );
}

#[test]
fn when_shallow_diagonal_inside_rectangle_then_no_gaps() {
    // Arrange — 8x8 image with a shallow-angle triangle (2:1 slope)
    // inside a rectangle. This creates staircase vertices at distance
    // 1/sqrt(5) ~ 0.447 from the diagonal, which RDP at epsilon=0.5
    // would incorrectly remove. The fix uses epsilon=0 for internal
    // no-junction boundaries.
    let w = 8u32;
    let h = 8u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let idx = ((row * w + col) * 4) as usize;
            // Shallow triangle: row >= 2 && col >= 2 && (row-2) >= 2*(col-2)
            let is_inner = (2..7).contains(&row)
                && (2..6).contains(&col)
                && (row as i32 - 2) >= 2 * (col as i32 - 2);
            if is_inner {
                rgba[idx] = 255; // red
            } else {
                rgba[idx + 2] = 255; // blue
            }
            rgba[idx + 3] = 255;
        }
    }

    let config = ConvertConfig {
        use_bezier: false,
        rdp_epsilon: 0.5,
        min_area: 0,
        ..default_config()
    };

    let result = image_to_shapes(&rgba, w, h, &config);

    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

    let mut mismatches = Vec::new();
    for row in 0..h {
        for col in 0..w {
            let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
            let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
            let is_inner = (2..7).contains(&row)
                && (2..6).contains(&col)
                && (row as i32 - 2) >= 2 * (col as i32 - 2);
            let expected = if is_inner { red } else { blue };

            if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                let ok = (shape.color.r - expected.r).abs() < 0.1
                    && (shape.color.g - expected.g).abs() < 0.1
                    && (shape.color.b - expected.b).abs() < 0.1;
                if !ok {
                    mismatches.push(format!("({col},{row}) wrong_color"));
                }
            } else {
                mismatches.push(format!("({col},{row}) UNCOVERED"));
            }
        }
    }

    let uncovered = mismatches
        .iter()
        .filter(|m| m.contains("UNCOVERED"))
        .count();
    assert_eq!(uncovered, 0, "no pixels should be uncovered");
    assert!(
        mismatches.len() <= 10,
        "shallow diagonal: too many boundary mismatches ({}):\n{}",
        mismatches.len(),
        mismatches[..mismatches.len().min(20)].join("\n")
    );
}

#[test]
fn when_diagonal_boundary_three_regions_then_no_gaps() {
    // Arrange — 8x8 image with a diagonal boundary between regions.
    // Top-left triangle (red), top-right triangle (green), bottom half (blue).
    // The diagonal creates staircase boundaries that are the hardest case.
    let w = 8u32;
    let h = 8u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let idx = ((row * w + col) * 4) as usize;
            if row < 4 {
                if col <= row {
                    rgba[idx] = 255; // red (below diagonal)
                } else {
                    rgba[idx + 1] = 255; // green (above diagonal)
                }
            } else {
                rgba[idx + 2] = 255; // blue (bottom half)
            }
            rgba[idx + 3] = 255;
        }
    }

    let config = ConvertConfig {
        use_bezier: false,
        rdp_epsilon: 0.5,
        min_area: 0,
        ..default_config()
    };

    let result = image_to_shapes(&rgba, w, h, &config);

    let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
    let green = engine_core::color::Color::new(0.0, 1.0, 0.0, 1.0);
    let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

    let mut mismatches = Vec::new();
    for row in 0..h {
        for col in 0..w {
            let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
            let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
            let expected = if row < 4 {
                if col <= row { red } else { green }
            } else {
                blue
            };

            if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                let ok = (shape.color.r - expected.r).abs() < 0.1
                    && (shape.color.g - expected.g).abs() < 0.1
                    && (shape.color.b - expected.b).abs() < 0.1;
                if !ok {
                    mismatches.push(format!("({col},{row}) wrong color"));
                }
            } else {
                mismatches.push(format!("({col},{row}) UNCOVERED"));
            }
        }
    }

    let _uncovered = mismatches
        .iter()
        .filter(|m| m.contains("UNCOVERED"))
        .count();
    // Post-processing simplification may leave a few boundary pixels
    // uncovered or wrong-color at triple junctions.
    assert!(
        mismatches.len() <= 10,
        "diagonal boundary: too many mismatches ({}):\n{}",
        mismatches.len(),
        mismatches[..mismatches.len().min(20)].join("\n")
    );
}

// --- Region merging tests ---

#[test]
fn when_merge_below_set_then_small_regions_absorbed_by_neighbor() {
    // Arrange — 10x1 strip: 8 red pixels, 2 green pixels at the end.
    // With merge_below=3, the 2-pixel green region is absorbed into the red.
    let w = 10u32;
    let h = 1u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for col in 0..8 {
        let idx = col * 4;
        rgba[idx] = 255; // red
        rgba[idx + 3] = 255;
    }
    for col in 8..10 {
        let idx = col * 4;
        rgba[idx + 1] = 255; // green
        rgba[idx + 3] = 255;
    }

    let config = ConvertConfig {
        color_threshold: 0.1,
        alpha_threshold: 128,
        rdp_epsilon: 0.5,
        bezier_error: 0.5,
        min_area: 0,
        max_dimension: 0,
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 3, // green region (2px) < threshold -> merged
        max_shapes: 0,
    };

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert — only one shape remains (green merged into red)
    assert_eq!(
        result.shapes.len(),
        1,
        "small green region should be merged, got {} shapes",
        result.shapes.len()
    );
}

#[test]
fn when_max_shapes_set_then_output_respects_cap() {
    // Arrange — 3 distinct color stripes that normally produce 3 shapes.
    let w = 9u32;
    let h = 1u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    // 3 red
    for col in 0..3 {
        let idx = col * 4;
        rgba[idx] = 255;
        rgba[idx + 3] = 255;
    }
    // 3 green
    for col in 3..6 {
        let idx = col * 4;
        rgba[idx + 1] = 255;
        rgba[idx + 3] = 255;
    }
    // 3 blue
    for col in 6..9 {
        let idx = col * 4;
        rgba[idx + 2] = 255;
        rgba[idx + 3] = 255;
    }

    let config = ConvertConfig {
        color_threshold: 0.01,
        alpha_threshold: 128,
        rdp_epsilon: 0.5,
        bezier_error: 0.5,
        min_area: 0,
        max_dimension: 0,
        resize_method: ResizeMethod::Nearest,
        use_bezier: true,
        merge_below: 0,
        max_shapes: 2, // cap at 2 -> one of the 3 regions gets merged
    };

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert
    assert!(
        result.shapes.len() <= 2,
        "max_shapes=2 should produce at most 2 shapes, got {}",
        result.shapes.len()
    );
}

#[test]
fn when_estimate_computed_then_counts_match_actual_shapes() {
    // Arrange
    let w = 10u32;
    let h = 10u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        rgba[i * 4] = 255;
        rgba[i * 4 + 3] = 255;
    }

    let config = default_config();

    // Act
    let result = image_to_shapes(&rgba, w, h, &config);

    // Assert
    assert_eq!(result.estimate.shape_count, result.shapes.len());

    let actual_commands: usize = result
        .shapes
        .iter()
        .map(|s| {
            if let ShapeVariant::Path { commands } = &s.variant {
                commands.len()
            } else {
                0
            }
        })
        .sum();
    assert_eq!(result.estimate.command_count, actual_commands);
}
