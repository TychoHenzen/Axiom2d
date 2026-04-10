// EVOLVE-BLOCK-START
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadraticTo {
        control: Vec2,
        to: Vec2,
    },
    CubicTo {
        control1: Vec2,
        control2: Vec2,
        to: Vec2,
    },
    Close,
    /// Reverse the winding of all segments since the last `MoveTo`.
    /// Place before `Close` to flip a contour's winding direction inline.
    Reverse,
}

/// Resolve `Reverse` directives in a path command list.
///
/// When `Reverse` is encountered, all segments since the last `MoveTo` are
/// reversed in place (using `reverse_path`), then processing continues.
pub fn resolve_commands(commands: &[PathCommand]) -> Vec<PathCommand> {
    let mut result = Vec::new();
    let mut contour_start = 0;

    for cmd in commands {
        match cmd {
            PathCommand::Reverse => {
                // Find the MoveTo that started this contour
                let contour = &result[contour_start..];
                let mut with_close = contour.to_vec();
                with_close.push(PathCommand::Close);
                let reversed = reverse_path(&with_close);
                result.truncate(contour_start);
                // Push everything from reversed except the trailing Close
                for rc in &reversed[..reversed.len() - 1] {
                    result.push(rc.clone());
                }
            }
            PathCommand::MoveTo(_) => {
                contour_start = result.len();
                result.push(cmd.clone());
            }
            _ => {
                result.push(cmd.clone());
            }
        }
    }

    result
}

/// Reverse the winding order of a single contour (`MoveTo` ... Close).
///
/// Each segment's direction is flipped and the segment order is reversed,
/// so the resulting path traces the same shape in the opposite direction.
pub fn reverse_path(commands: &[PathCommand]) -> Vec<PathCommand> {
    if commands.is_empty() {
        return Vec::new();
    }

    let PathCommand::MoveTo(start) = commands[0] else {
        return commands.to_vec();
    };

    let endpoints = collect_endpoints(start, &commands[1..]);
    // INVARIANT: collect_endpoints always pushes `start` as the first element,
    // so the vec is never empty.
    let last_endpoint = *endpoints.last().expect("path has no segments");
    let mut result = vec![PathCommand::MoveTo(last_endpoint)];

    let segments = &commands[1..];
    let segment_count = segments
        .iter()
        .filter(|c| !matches!(c, PathCommand::Close | PathCommand::Reverse))
        .count();

    for i in (0..segment_count).rev() {
        reverse_segment(&segments[i], endpoints[i], &mut result);
    }

    result.push(PathCommand::Close);
    result
}

fn collect_endpoints(start: Vec2, commands: &[PathCommand]) -> Vec<Vec2> {
    let mut endpoints = vec![start];
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => endpoints.push(p),
            PathCommand::QuadraticTo { to, .. } | PathCommand::CubicTo { to, .. } => {
                endpoints.push(to);
            }
            PathCommand::Close | PathCommand::Reverse => {}
        }
    }
    endpoints
}

fn reverse_segment(cmd: &PathCommand, from: Vec2, result: &mut Vec<PathCommand>) {
    match *cmd {
        PathCommand::LineTo(_) => {
            result.push(PathCommand::LineTo(from));
        }
        PathCommand::QuadraticTo { control, .. } => {
            result.push(PathCommand::QuadraticTo { control, to: from });
        }
        PathCommand::CubicTo {
            control1, control2, ..
        } => {
            result.push(PathCommand::CubicTo {
                control1: control2,
                control2: control1,
                to: from,
            });
        }
        _ => {}
    }
}

/// Split a path into sub-contours at each `MoveTo` boundary.
/// Each returned contour starts with `MoveTo` and ends with `Close`.
pub fn split_contours(commands: &[PathCommand]) -> Vec<Vec<PathCommand>> {
    let mut contours = Vec::new();
    let mut current = Vec::new();
    for cmd in commands {
        if matches!(cmd, PathCommand::MoveTo(_)) && !current.is_empty() {
            contours.push(std::mem::take(&mut current));
        }
        current.push(cmd.clone());
    }
    if !current.is_empty() {
        contours.push(current);
    }
    contours
}

/// Sample `n+1` evenly-spaced points along a quadratic bezier curve.
pub fn sample_quadratic(from: Vec2, control: Vec2, to: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            from * (u * u) + control * (2.0 * u * t) + to * (t * t)
        })
        .collect()
}

/// Sample `n+1` evenly-spaced points along a cubic bezier curve.
pub fn sample_cubic(from: Vec2, c1: Vec2, c2: Vec2, to: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            from * (u * u * u) + c1 * (3.0 * u * u * t) + c2 * (3.0 * u * t * t) + to * (t * t * t)
        })
        .collect()
}
// EVOLVE-BLOCK-END
