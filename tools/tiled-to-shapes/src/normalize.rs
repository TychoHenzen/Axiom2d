use engine_render::prelude::PathCommand;
use engine_render::shape::{Shape, ShapeVariant};
use glam::Vec2;

/// Transform shapes from img-to-shape engine coordinates to `[0,1]²` tile space.
///
/// img-to-shape outputs shapes in center-origin, Y-up, pixel-scale coordinates:
///   - center is (0, 0)
///   - positive Y is up
///   - units are pixels (of the post-upscale image)
///
/// Normalized formula:
///   `norm_x = (engine_x + half_width)  / width`
///   `norm_y = (engine_y + half_height) / height`
///
/// Returns one `Vec<PathCommand>` per shape, with coordinates in `[0,1]²`.
pub fn normalize_shapes(shapes: &[Shape], width: f32, height: f32) -> Vec<Vec<PathCommand>> {
    let half_w = width / 2.0;
    let half_h = height / 2.0;

    shapes
        .iter()
        .filter_map(|s| {
            if let ShapeVariant::Path { commands } = &s.variant {
                Some(normalize_commands(commands, half_w, half_h, width, height))
            } else {
                None
            }
        })
        .collect()
}

/// Normalize a single command list in-place.
fn normalize_commands(
    commands: &[PathCommand],
    half_w: f32,
    half_h: f32,
    width: f32,
    height: f32,
) -> Vec<PathCommand> {
    commands
        .iter()
        .map(|cmd| normalize_command(cmd, half_w, half_h, width, height))
        .collect()
}

fn normalize_point(v: Vec2, half_w: f32, half_h: f32, width: f32, height: f32) -> Vec2 {
    Vec2::new((v.x + half_w) / width, (v.y + half_h) / height)
}

fn normalize_command(
    cmd: &PathCommand,
    half_w: f32,
    half_h: f32,
    width: f32,
    height: f32,
) -> PathCommand {
    match cmd {
        PathCommand::MoveTo(v) => {
            PathCommand::MoveTo(normalize_point(*v, half_w, half_h, width, height))
        }
        PathCommand::LineTo(v) => {
            PathCommand::LineTo(normalize_point(*v, half_w, half_h, width, height))
        }
        PathCommand::QuadraticTo { control, to } => PathCommand::QuadraticTo {
            control: normalize_point(*control, half_w, half_h, width, height),
            to: normalize_point(*to, half_w, half_h, width, height),
        },
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => PathCommand::CubicTo {
            control1: normalize_point(*control1, half_w, half_h, width, height),
            control2: normalize_point(*control2, half_w, half_h, width, height),
            to: normalize_point(*to, half_w, half_h, width, height),
        },
        PathCommand::Close | PathCommand::Reverse => cmd.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::color::Color;
    use engine_render::shape::{Shape, ShapeVariant};
    use glam::Vec2;

    fn make_shape_at(x: f32, y: f32) -> Shape {
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![PathCommand::MoveTo(Vec2::new(x, y)), PathCommand::Close],
            },
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        }
    }

    #[test]
    fn when_center_origin_then_normalizes_to_half() {
        // Arrange — a shape at engine-space center (0, 0)
        let shapes = vec![make_shape_at(0.0, 0.0)];
        // Act
        let result = normalize_shapes(&shapes, 64.0, 64.0);
        // Assert — (0,0) + (32,32) / 64 = (0.5, 0.5)
        assert_eq!(result.len(), 1);
        if let PathCommand::MoveTo(v) = result[0][0] {
            assert!((v.x - 0.5).abs() < 1e-5, "x: {}", v.x);
            assert!((v.y - 0.5).abs() < 1e-5, "y: {}", v.y);
        } else {
            panic!("expected MoveTo");
        }
    }

    #[test]
    fn when_normalized_then_all_coords_in_unit_square() {
        // Arrange — corners of a 64×64 pixel-scale tile
        let shapes = vec![
            make_shape_at(-32.0, -32.0), // BL corner
            make_shape_at(32.0, 32.0), // TR corner (note: -0.5 offset because range is [-half, half))
            make_shape_at(-32.0, 32.0), // TL
            make_shape_at(32.0, -32.0), // BR
        ];
        // Act
        let result = normalize_shapes(&shapes, 64.0, 64.0);
        // Assert — all MoveTo points in [0, 1]
        for cmds in &result {
            for cmd in cmds {
                if let PathCommand::MoveTo(v) = cmd {
                    assert!(v.x >= 0.0 && v.x <= 1.0, "x={} out of [0,1]", v.x);
                    assert!(v.y >= 0.0 && v.y <= 1.0, "y={} out of [0,1]", v.y);
                }
            }
        }
    }
}
