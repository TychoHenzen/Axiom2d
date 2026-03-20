use engine_render::prelude::PathCommand;
use glam::Vec2;

#[allow(clippy::too_many_lines)]
pub(crate) fn letter_a() -> Vec<PathCommand> {
    vec![
        // Outer contour — curved apex and gently bowed legs
        PathCommand::MoveTo(Vec2::new(-6.0, -60.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(0.0, -68.0),
            to: Vec2::new(6.0, -60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(14.0, -38.0),
            control2: Vec2::new(26.0, -6.0),
            to: Vec2::new(35.0, 60.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(29.0, 60.0),
            to: Vec2::new(23.0, 60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(18.0, 36.0),
            control2: Vec2::new(14.0, 26.0),
            to: Vec2::new(12.0, 18.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(6.0, 16.0),
            to: Vec2::new(0.0, 16.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(-6.0, 16.0),
            to: Vec2::new(-12.0, 18.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-14.0, 26.0),
            control2: Vec2::new(-18.0, 36.0),
            to: Vec2::new(-23.0, 60.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(-29.0, 60.0),
            to: Vec2::new(-35.0, 60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-26.0, -6.0),
            control2: Vec2::new(-14.0, -38.0),
            to: Vec2::new(-6.0, -60.0),
        },
        PathCommand::Close,
        // Inner rounded cutout (EvenOdd fill)
        PathCommand::MoveTo(Vec2::new(0.0, -30.0)),
        PathCommand::CubicTo {
            control1: Vec2::new(4.0, -18.0),
            control2: Vec2::new(8.0, -4.0),
            to: Vec2::new(10.0, 8.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(0.0, 6.0),
            to: Vec2::new(-10.0, 8.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-8.0, -4.0),
            control2: Vec2::new(-4.0, -18.0),
            to: Vec2::new(0.0, -30.0),
        },
        PathCommand::Reverse,
        PathCommand::Close,
    ]
}

#[allow(clippy::too_many_lines)]
pub(crate) fn letter_x() -> Vec<PathCommand> {
    vec![
        // Bar 1: top-left to bottom-right with S-curve
        PathCommand::MoveTo(Vec2::new(-31.0, -60.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(-22.0, -62.0),
            to: Vec2::new(-19.0, -60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-6.0, -30.0),
            control2: Vec2::new(6.0, 30.0),
            to: Vec2::new(19.0, 60.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(22.0, 62.0),
            to: Vec2::new(31.0, 60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(18.0, 30.0),
            control2: Vec2::new(-18.0, -30.0),
            to: Vec2::new(-31.0, -60.0),
        },
        PathCommand::Close,
        // Bar 2: top-right to bottom-left with S-curve
        PathCommand::MoveTo(Vec2::new(19.0, -60.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(22.0, -62.0),
            to: Vec2::new(31.0, -60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(18.0, -30.0),
            control2: Vec2::new(-18.0, 30.0),
            to: Vec2::new(-31.0, 60.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(-22.0, 62.0),
            to: Vec2::new(-19.0, 60.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-6.0, 30.0),
            control2: Vec2::new(6.0, -30.0),
            to: Vec2::new(19.0, -60.0),
        },
        PathCommand::Close,
    ]
}

pub(crate) fn letter_i() -> Vec<PathCommand> {
    vec![
        // Top serif — left edge
        PathCommand::MoveTo(Vec2::new(-15.0, -60.0)),
        PathCommand::LineTo(Vec2::new(15.0, -60.0)),
        // Top-right corner rounds into stem
        PathCommand::QuadraticTo {
            control: Vec2::new(15.0, -48.0),
            to: Vec2::new(6.0, -48.0),
        },
        // Right side of stem
        PathCommand::LineTo(Vec2::new(6.0, 48.0)),
        // Bottom-right corner rounds out to serif
        PathCommand::QuadraticTo {
            control: Vec2::new(15.0, 48.0),
            to: Vec2::new(15.0, 60.0),
        },
        PathCommand::LineTo(Vec2::new(-15.0, 60.0)),
        // Bottom-left corner rounds into stem
        PathCommand::QuadraticTo {
            control: Vec2::new(-15.0, 48.0),
            to: Vec2::new(-6.0, 48.0),
        },
        // Left side of stem
        PathCommand::LineTo(Vec2::new(-6.0, -48.0)),
        // Top-left corner rounds out to serif
        PathCommand::QuadraticTo {
            control: Vec2::new(-15.0, -48.0),
            to: Vec2::new(-15.0, -60.0),
        },
        PathCommand::Close,
    ]
}

#[allow(clippy::similar_names, clippy::too_many_lines)]
pub(crate) fn letter_o() -> Vec<PathCommand> {
    // Approximate an ellipse with 4 cubic bezier segments (kappa ≈ 0.5522847)
    let kappa = 0.552_284_7_f32;
    let outer_rx = 30.0_f32;
    let outer_ry = 60.0_f32;
    let outer_kx = outer_rx * kappa;
    let outer_ky = outer_ry * kappa;
    let inner_rx = 18.0_f32;
    let inner_ry = 48.0_f32;
    let inner_kx = inner_rx * kappa;
    let inner_ky = inner_ry * kappa;
    vec![
        // Outer ellipse (top → right → bottom → left)
        PathCommand::MoveTo(Vec2::new(0.0, -outer_ry)),
        PathCommand::CubicTo {
            control1: Vec2::new(outer_kx, -outer_ry),
            control2: Vec2::new(outer_rx, -outer_ky),
            to: Vec2::new(outer_rx, 0.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(outer_rx, outer_ky),
            control2: Vec2::new(outer_kx, outer_ry),
            to: Vec2::new(0.0, outer_ry),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-outer_kx, outer_ry),
            control2: Vec2::new(-outer_rx, outer_ky),
            to: Vec2::new(-outer_rx, 0.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-outer_rx, -outer_ky),
            control2: Vec2::new(-outer_kx, -outer_ry),
            to: Vec2::new(0.0, -outer_ry),
        },
        PathCommand::Close,
        // Inner ellipse hole (EvenOdd)
        PathCommand::MoveTo(Vec2::new(0.0, -inner_ry)),
        PathCommand::CubicTo {
            control1: Vec2::new(inner_kx, -inner_ry),
            control2: Vec2::new(inner_rx, -inner_ky),
            to: Vec2::new(inner_rx, 0.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(inner_rx, inner_ky),
            control2: Vec2::new(inner_kx, inner_ry),
            to: Vec2::new(0.0, inner_ry),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-inner_kx, inner_ry),
            control2: Vec2::new(-inner_rx, inner_ky),
            to: Vec2::new(-inner_rx, 0.0),
        },
        PathCommand::CubicTo {
            control1: Vec2::new(-inner_rx, -inner_ky),
            control2: Vec2::new(-inner_kx, -inner_ry),
            to: Vec2::new(0.0, -inner_ry),
        },
        PathCommand::Reverse,
        PathCommand::Close,
    ]
}

#[allow(clippy::too_many_lines)]
pub(crate) fn letter_m() -> Vec<PathCommand> {
    vec![
        // Start bottom-left
        PathCommand::MoveTo(Vec2::new(-38.0, 60.0)),
        PathCommand::LineTo(Vec2::new(-38.0, -50.0)),
        // Left peak — smooth arch up
        PathCommand::QuadraticTo {
            control: Vec2::new(-38.0, -62.0),
            to: Vec2::new(-30.0, -60.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(-22.0, -58.0),
            to: Vec2::new(-16.0, -48.0),
        },
        // Down into center valley — smooth curve
        PathCommand::CubicTo {
            control1: Vec2::new(-6.0, -24.0),
            control2: Vec2::new(-4.0, -16.0),
            to: Vec2::new(0.0, -12.0),
        },
        // Up from valley to right peak
        PathCommand::CubicTo {
            control1: Vec2::new(4.0, -16.0),
            control2: Vec2::new(6.0, -24.0),
            to: Vec2::new(16.0, -48.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(22.0, -58.0),
            to: Vec2::new(30.0, -60.0),
        },
        PathCommand::QuadraticTo {
            control: Vec2::new(38.0, -62.0),
            to: Vec2::new(38.0, -50.0),
        },
        // Right stem down
        PathCommand::LineTo(Vec2::new(38.0, 60.0)),
        PathCommand::LineTo(Vec2::new(26.0, 60.0)),
        PathCommand::LineTo(Vec2::new(26.0, -34.0)),
        // Inner right slope with curve
        PathCommand::CubicTo {
            control1: Vec2::new(18.0, -20.0),
            control2: Vec2::new(10.0, -10.0),
            to: Vec2::new(6.0, -4.0),
        },
        // Inner valley
        PathCommand::QuadraticTo {
            control: Vec2::new(0.0, 2.0),
            to: Vec2::new(-6.0, -4.0),
        },
        // Inner left slope with curve
        PathCommand::CubicTo {
            control1: Vec2::new(-10.0, -10.0),
            control2: Vec2::new(-18.0, -20.0),
            to: Vec2::new(-26.0, -34.0),
        },
        PathCommand::LineTo(Vec2::new(-26.0, 60.0)),
        PathCommand::Close,
    ]
}
