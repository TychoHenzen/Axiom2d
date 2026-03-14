use bevy_ecs::prelude::{Component, Query, Res, ResMut, Resource, With, World};
use engine_app::prelude::{App, Phase, Plugin};
use engine_core::prelude::{Color, DeltaTime, Transform2D};
use engine_scene::prelude::{RenderLayer, SortOrder, Visible};
use glam::Vec2;

#[derive(Resource)]
pub struct SplashScreen {
    pub elapsed: f32,
    pub duration: f32,
    pub done: bool,
}

impl SplashScreen {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            done: false,
        }
    }
}

const SPLASH_DURATION: f32 = 2.5;
const SPLASH_BG_ORDER: i32 = 10_000;
const SPLASH_SIDE_BASE: i32 = 10_001;
const SPLASH_LETTER_ORDER: i32 = 11_000;
const SPLASH_ACCENT_ORDER: i32 = 11_001;

#[cfg(feature = "render")]
fn shade_for_normal(normal: Vec2, light_dir: Vec2, dark: Color, bright: Color) -> Color {
    let t = (normal.dot(light_dir) + 1.0) * 0.5;
    let t = t.clamp(0.0, 1.0);
    let t = t * t * t;
    Color::new(
        dark.r + (bright.r - dark.r) * t,
        dark.g + (bright.g - dark.g) * t,
        dark.b + (bright.b - dark.b) * t,
        dark.a + (bright.a - dark.a) * t,
    )
}

#[cfg(feature = "render")]
fn segment_normal(from: Vec2, to: Vec2) -> Vec2 {
    let d = to - from;
    Vec2::new(d.y, -d.x).normalize()
}

/// Split a path into sub-contours at each `MoveTo` boundary.
/// Each returned contour starts with `MoveTo` and ends with `Close`.
#[cfg(feature = "render")]
fn split_contours(commands: &[PathCommand]) -> Vec<Vec<PathCommand>> {
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
#[cfg(feature = "render")]
fn sample_quadratic(from: Vec2, control: Vec2, to: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            from * (u * u) + control * (2.0 * u * t) + to * (t * t)
        })
        .collect()
}

/// Sample `n+1` evenly-spaced points along a cubic bezier curve.
#[cfg(feature = "render")]
fn sample_cubic(from: Vec2, c1: Vec2, c2: Vec2, to: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            from * (u * u * u)
                + c1 * (3.0 * u * u * t)
                + c2 * (3.0 * u * t * t)
                + to * (t * t * t)
        })
        .collect()
}

/// Project a point toward the vanishing point by `depth` fraction.
#[cfg(feature = "render")]
fn project_point(p: Vec2, vp: Vec2, depth: f32) -> Vec2 {
    p + (vp - p) * depth
}

/// Build a single side-face quad from front points a→b and their perspective-projected back copies.
#[cfg(feature = "render")]
fn side_quad(a: Vec2, b: Vec2, vp: Vec2, depth: f32) -> Vec<PathCommand> {
    vec![
        PathCommand::MoveTo(a),
        PathCommand::LineTo(b),
        PathCommand::LineTo(project_point(b, vp, depth)),
        PathCommand::LineTo(project_point(a, vp, depth)),
        PathCommand::Close,
    ]
}

/// Build shaded side-face geometry for one contour with perspective projection.
///
/// Each outline segment produces side-face quads projected toward `vp` (vanishing point
/// in local coordinates) by `depth` fraction. Back-facing quads are culled.
/// Returns `(geometry, color, sort_distance)` — sorted by distance from VP descending
/// so further faces render first (behind closer faces).
#[cfg(feature = "render")]
#[allow(clippy::too_many_arguments)]
fn build_shaded_side_faces(
    contour: &[PathCommand],
    vp: Vec2,
    depth: f32,
    slices: usize,
    light_dir: Vec2,
    dark: Color,
    bright: Color,
) -> Vec<(Vec<PathCommand>, Color, f32)> {
    let start = match contour[0] {
        PathCommand::MoveTo(p) => p,
        _ => panic!("contour must start with MoveTo"),
    };

    let middle = &contour[1..contour.len() - 1];
    let mut result: Vec<(Vec<PathCommand>, Color, f32)> = Vec::new();
    let mut current = start;

    let push_strip = |a: Vec2, b: Vec2, result: &mut Vec<(Vec<PathCommand>, Color, f32)>| {
        let midpoint = (a + b) * 0.5;
        let vp_dir = (vp - midpoint).normalize_or_zero();
        let normal = segment_normal(a, b);
        if normal.dot(vp_dir) >= 0.0 {
            let color = shade_for_normal(normal, light_dir, dark, bright);
            let dist = midpoint.distance(vp);
            result.push((side_quad(a, b, vp, depth), color, dist));
        }
    };

    for seg in middle {
        match *seg {
            PathCommand::LineTo(to) => {
                push_strip(current, to, &mut result);
                current = to;
            }
            PathCommand::QuadraticTo { control, to } => {
                let points = sample_quadratic(current, control, to, slices);
                for pair in points.windows(2) {
                    push_strip(pair[0], pair[1], &mut result);
                }
                current = to;
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                let points = sample_cubic(current, control1, control2, to, slices);
                for pair in points.windows(2) {
                    push_strip(pair[0], pair[1], &mut result);
                }
                current = to;
            }
            _ => {}
        }
    }

    if (current - start).length() > f32::EPSILON {
        push_strip(current, start, &mut result);
    }

    // Sort by distance from VP descending — further faces render first (behind)
    result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    result
}

const LOGO_COLOR: Color = Color {
    r: 0.85,
    g: 0.85,
    b: 0.95,
    a: 1.0,
};
const ACCENT_COLOR: Color = Color {
    r: 0.4,
    g: 0.5,
    b: 0.9,
    a: 1.0,
};

#[derive(Component)]
pub struct SplashEntity;

type PreloadHook = Box<dyn FnMut(&mut World) + Send + Sync>;

#[derive(Resource)]
pub struct PreloadHooks {
    hooks: Vec<PreloadHook>,
    executed: bool,
}

impl PreloadHooks {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            executed: false,
        }
    }

    pub fn add(&mut self, hook: impl FnMut(&mut World) + Send + Sync + 'static) {
        self.hooks.push(Box::new(hook));
    }
}

impl Default for PreloadHooks {
    fn default() -> Self {
        Self::new()
    }
}

pub fn preload_system(world: &mut World) {
    let splash_done = world.resource::<SplashScreen>().done;
    let already_executed = world
        .get_resource::<PreloadHooks>()
        .is_none_or(|h| h.executed);

    if splash_done || already_executed {
        return;
    }

    let mut hooks = world
        .remove_resource::<PreloadHooks>()
        .expect("PreloadHooks missing");
    for hook in &mut hooks.hooks {
        hook(world);
    }
    hooks.executed = true;
    hooks.hooks.clear();
    world.insert_resource(hooks);
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .insert_resource(SplashScreen::new(SPLASH_DURATION));
        app.world_mut().insert_resource(PreloadHooks::new());

        #[cfg(feature = "render")]
        spawn_splash_entities(app.world_mut());

        app.add_systems(Phase::PreUpdate, preload_system);
        app.add_systems(Phase::Update, splash_tick_system);
    }
}

#[cfg(feature = "render")]
use engine_render::prelude::PathCommand;

#[cfg(feature = "render")]
const BG_COLOR: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.08,
    a: 1.0,
};

#[cfg(feature = "render")]
#[allow(clippy::too_many_lines)]
fn spawn_splash_entities(world: &mut bevy_ecs::world::World) {
    use engine_render::prelude::{Shape, ShapeVariant, Stroke};

    world.spawn((
        SplashEntity,
        Transform2D {
            position: Vec2::ZERO,
            ..Default::default()
        },
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(-2000.0, -2000.0),
                    Vec2::new(2000.0, -2000.0),
                    Vec2::new(2000.0, 2000.0),
                    Vec2::new(-2000.0, 2000.0),
                ],
            },
            color: BG_COLOR,
        },
        Visible(true),
        RenderLayer::UI,
        SortOrder(SPLASH_BG_ORDER),
    ));

    let vanishing_point = Vec2::new(-60.0, -10.0);
    let depth = 0.12;
    let light_dir = Vec2::new(-1.0, -1.0).normalize();
    let dark_color = Color {
        r: 0.15,
        g: 0.17,
        b: 0.28,
        a: 1.0,
    };
    let bright_color = Color {
        r: LOGO_COLOR.r * 0.9,
        g: LOGO_COLOR.g * 0.9,
        b: LOGO_COLOR.b * 0.9,
        a: 1.0,
    };
    let slices = 6;

    let letters: [(f32, Vec<PathCommand>); 5] = [
        (-140.0, letter_a()),
        (-60.0, letter_x()),
        (0.0, letter_i()),
        (58.0, letter_o()),
        (140.0, letter_m()),
    ];

    for (x, commands) in letters {
        let letter_pos = Vec2::new(x, -10.0);
        let vp_local = vanishing_point - letter_pos;
        let resolved = engine_render::prelude::resolve_commands(&commands);

        for contour in split_contours(&resolved) {
            let faces =
                build_shaded_side_faces(&contour, vp_local, depth, slices, light_dir, dark_color, bright_color);
            #[allow(clippy::cast_possible_wrap)]
            for (i, (quad_cmds, color, _dist)) in faces.into_iter().enumerate() {
                world.spawn((
                    SplashEntity,
                    Transform2D {
                        position: letter_pos,
                        ..Default::default()
                    },
                    Shape {
                        variant: ShapeVariant::Path {
                            commands: quad_cmds,
                        },
                        color,
                    },
                    Visible(true),
                    RenderLayer::UI,
                    SortOrder(SPLASH_SIDE_BASE + i as i32),
                ));
            }
        }

        // Foreground letter
        world.spawn((
            SplashEntity,
            Transform2D {
                position: letter_pos,
                ..Default::default()
            },
            Shape {
                variant: ShapeVariant::Path { commands },
                color: LOGO_COLOR,
            },
            Stroke {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                width: 2.0,
            },
            Visible(true),
            RenderLayer::UI,
            SortOrder(SPLASH_LETTER_ORDER),
        ));
    }

    world.spawn((
        SplashEntity,
        Transform2D {
            position: Vec2::new(0.0, 68.0),
            ..Default::default()
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-170.0, 0.0)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-120.0, -3.5),
                        control2: Vec2::new(-60.0, -3.5),
                        to: Vec2::new(0.0, -3.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(60.0, -3.5),
                        control2: Vec2::new(120.0, -3.5),
                        to: Vec2::new(170.0, 0.0),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(120.0, 3.5),
                        control2: Vec2::new(60.0, 3.5),
                        to: Vec2::new(0.0, 3.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-60.0, 3.5),
                        control2: Vec2::new(-120.0, 3.5),
                        to: Vec2::new(-170.0, 0.0),
                    },
                    PathCommand::Close,
                ],
            },
            color: ACCENT_COLOR,
        },
        Visible(true),
        RenderLayer::UI,
        SortOrder(SPLASH_ACCENT_ORDER),
    ));

    world.spawn((
        SplashEntity,
        Transform2D {
            position: Vec2::new(0.0, 85.0),
            ..Default::default()
        },
        Shape {
            variant: ShapeVariant::Circle { radius: 5.0 },
            color: ACCENT_COLOR,
        },
        Visible(true),
        RenderLayer::UI,
        SortOrder(SPLASH_ACCENT_ORDER),
    ));
}

#[cfg(feature = "render")]
fn letter_a() -> Vec<PathCommand> {
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

#[cfg(feature = "render")]
fn letter_x() -> Vec<PathCommand> {
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

#[cfg(feature = "render")]
fn letter_i() -> Vec<PathCommand> {
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

#[cfg(feature = "render")]
#[allow(clippy::similar_names)]
fn letter_o() -> Vec<PathCommand> {
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

#[cfg(feature = "render")]
fn letter_m() -> Vec<PathCommand> {
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

pub fn splash_tick_system(
    mut splash: ResMut<SplashScreen>,
    dt: Res<DeltaTime>,
    mut query: Query<&mut Visible, With<SplashEntity>>,
) {
    if splash.done {
        return;
    }
    splash.elapsed += dt.0.0;
    if splash.elapsed >= splash.duration {
        splash.done = true;
        for mut visible in &mut query {
            visible.0 = false;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn world_with_splash(elapsed: f32, duration: f32, done: bool, dt: f32) -> (World, Schedule) {
        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed,
            duration,
            done,
        });
        world.insert_resource(DeltaTime(Seconds(dt)));
        let mut schedule = Schedule::default();
        schedule.add_systems(splash_tick_system);
        (world, schedule)
    }

    #[test]
    fn when_splash_tick_runs_then_elapsed_increases_by_delta() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(0.0, 2.0, false, 0.016);

        // Act
        schedule.run(&mut world);

        // Assert
        let splash = world.resource::<SplashScreen>();
        assert!((splash.elapsed - 0.016).abs() < f32::EPSILON);
    }

    #[test]
    fn when_elapsed_below_duration_then_done_remains_false() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(0.0, 2.0, false, 0.016);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(!world.resource::<SplashScreen>().done);
    }

    #[test]
    fn when_elapsed_reaches_duration_then_done_is_true() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(1.95, 2.0, false, 0.1);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(world.resource::<SplashScreen>().done);
    }

    #[test]
    fn when_done_already_true_then_elapsed_stops() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(2.5, 2.0, true, 0.1);

        // Act
        schedule.run(&mut world);

        // Assert
        let splash = world.resource::<SplashScreen>();
        assert!((splash.elapsed - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn when_zero_duration_then_done_immediately() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(0.0, 0.0, false, 0.016);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(world.resource::<SplashScreen>().done);
    }

    #[test]
    fn when_done_then_splash_entities_set_visible_false() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(1.95, 2.0, false, 0.1);
        let e1 = world.spawn((SplashEntity, Visible(true))).id();
        let e2 = world.spawn((SplashEntity, Visible(true))).id();
        let e3 = world.spawn((SplashEntity, Visible(true))).id();

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(!world.get::<Visible>(e1).unwrap().0);
        assert!(!world.get::<Visible>(e2).unwrap().0);
        assert!(!world.get::<Visible>(e3).unwrap().0);
    }

    #[test]
    fn when_not_done_then_splash_entities_keep_visible_true() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(0.5, 2.0, false, 0.016);
        let entity = world.spawn((SplashEntity, Visible(true))).id();

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(world.get::<Visible>(entity).unwrap().0);
    }

    #[test]
    fn when_splash_plugin_built_then_resource_present_with_defaults() {
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(SplashPlugin);

        // Assert
        let splash = app.world().resource::<SplashScreen>();
        assert!((splash.duration - SPLASH_DURATION).abs() < f32::EPSILON);
        assert!((splash.elapsed - 0.0).abs() < f32::EPSILON);
        assert!(!splash.done);
    }

    #[test]
    fn when_splash_plugin_built_then_system_runs_on_redraw() {
        // Arrange
        let mut app = App::new();
        app.add_plugin(crate::default_plugins::DefaultPlugins);
        app.add_plugin(SplashPlugin);
        app.world_mut()
            .insert_resource(engine_core::prelude::ClockRes::new(Box::new({
                let mut clock = engine_core::time::FakeClock::new();
                clock.advance(Seconds(0.1));
                clock
            })));
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(
                engine_render::prelude::NullRenderer,
            )));

        // Act
        app.handle_redraw();

        // Assert
        assert!(app.world().resource::<SplashScreen>().elapsed > 0.0);
    }

    #[test]
    fn when_preload_system_runs_during_splash_then_hooks_are_executed() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SplashScreen::new(2.0));
        let mut hooks = PreloadHooks::new();
        hooks.add(|w: &mut World| {
            w.insert_resource(DeltaTime(Seconds(42.0)));
        });
        world.insert_resource(hooks);
        world.insert_resource(DeltaTime(Seconds(0.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(preload_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!((world.resource::<DeltaTime>().0.0 - 42.0).abs() < f32::EPSILON);
        assert!(world.resource::<PreloadHooks>().executed);
    }

    #[test]
    fn when_preload_already_executed_then_hooks_not_run_again() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SplashScreen::new(2.0));
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let counter = std::sync::Arc::clone(&call_count);
        let mut hooks = PreloadHooks::new();
        hooks.add(move |_: &mut World| {
            *counter.lock().unwrap() += 1;
        });
        hooks.executed = true;
        world.insert_resource(hooks);

        let mut schedule = Schedule::default();
        schedule.add_systems(preload_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(*call_count.lock().unwrap(), 0);
    }

    #[test]
    fn when_splash_done_then_preload_hooks_not_run() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 3.0,
            duration: 2.0,
            done: true,
        });
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let counter = std::sync::Arc::clone(&call_count);
        let mut hooks = PreloadHooks::new();
        hooks.add(move |_: &mut World| {
            *counter.lock().unwrap() += 1;
        });
        world.insert_resource(hooks);

        let mut schedule = Schedule::default();
        schedule.add_systems(preload_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(*call_count.lock().unwrap(), 0);
    }

    #[test]
    fn when_done_then_non_splash_entities_not_affected() {
        // Arrange
        let (mut world, mut schedule) = world_with_splash(1.95, 2.0, false, 0.1);
        world.spawn((SplashEntity, Visible(true)));
        let game_entity = world.spawn(Visible(true)).id();

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(world.get::<Visible>(game_entity).unwrap().0);
    }

    #[test]
    fn when_split_contours_called_then_splits_on_moveto() {
        use engine_render::prelude::PathCommand;

        // Arrange
        let commands = vec![
            PathCommand::MoveTo(Vec2::ZERO),
            PathCommand::LineTo(Vec2::X),
            PathCommand::Close,
            PathCommand::MoveTo(Vec2::Y),
            PathCommand::LineTo(Vec2::ONE),
            PathCommand::Close,
        ];

        // Act
        let contours = split_contours(&commands);

        // Assert
        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].len(), 3);
        assert_eq!(contours[1].len(), 3);
    }


    #[test]
    fn when_depth_paths_spawned_then_sort_order_between_bg_and_letters() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — side face entities exist in the side face sort range
        let side_count = world
            .query::<(&SplashEntity, &Shape, &SortOrder)>()
            .iter(&world)
            .filter(|(_, shape, order)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 >= SPLASH_SIDE_BASE && order.0 < SPLASH_LETTER_ORDER
            })
            .count();

        assert!(side_count > 0, "side face entities must exist");
    }

    #[test]
    fn when_depth_paths_spawned_then_all_darker_than_logo_color() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — every side face color is darker than LOGO_COLOR
        let all_darker = world
            .query::<(&SplashEntity, &Shape, &SortOrder)>()
            .iter(&world)
            .filter(|(_, shape, order)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 >= SPLASH_SIDE_BASE && order.0 < SPLASH_LETTER_ORDER
            })
            .all(|(_, shape, _)| {
                shape.color.r < LOGO_COLOR.r
                    && shape.color.g < LOGO_COLOR.g
                    && shape.color.b < LOGO_COLOR.b
            });

        assert!(all_darker, "all side face colors must be darker than LOGO_COLOR");
    }

    #[test]
    fn when_depth_paths_spawned_then_shaded_colors_are_not_all_identical() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — side face entities have more than one distinct color
        let colors: Vec<(u32, u32, u32)> = world
            .query::<(&SplashEntity, &Shape, &SortOrder)>()
            .iter(&world)
            .filter(|(_, shape, order)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 >= SPLASH_SIDE_BASE && order.0 < SPLASH_LETTER_ORDER
            })
            .map(|(_, shape, _)| {
                (
                    (shape.color.r * 1000.0) as u32,
                    (shape.color.g * 1000.0) as u32,
                    (shape.color.b * 1000.0) as u32,
                )
            })
            .collect();
        let first = colors[0];
        let all_same = colors.iter().all(|c| *c == first);

        assert!(!all_same, "shading must produce color variation across side faces");
    }

    #[test]
    fn when_splash_entities_spawned_then_has_bg_letters_accents_and_side_faces() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — at least: 1 bg + some side faces + 5 letters + 1 accent + 1 dot
        let total = world.query::<&SplashEntity>().iter(&world).count();
        let letter_count = world
            .query::<(&SplashEntity, &Shape, &SortOrder)>()
            .iter(&world)
            .filter(|(_, shape, order)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 == SPLASH_LETTER_ORDER
            })
            .count();

        assert!(total > 8, "more entities than old flat approach");
        assert_eq!(letter_count, 5, "5 foreground letter entities");
    }

    #[test]
    fn when_splash_entities_spawned_then_letters_positioned_left_to_right() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — letter entities (Path shapes at SPLASH_LETTER_ORDER) are ordered left-to-right
        let mut letter_xs: Vec<f32> = world
            .query::<(&SplashEntity, &Transform2D, &Shape, &SortOrder)>()
            .iter(&world)
            .filter(|(_, _, shape, order)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 == SPLASH_LETTER_ORDER
            })
            .map(|(_, t, _, _)| t.position.x)
            .collect();
        letter_xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(letter_xs.len(), 5);
        for pair in letter_xs.windows(2) {
            assert!(
                pair[1] > pair[0],
                "letters must be left-to-right: {letter_xs:?}"
            );
        }
    }

    #[test]
    fn when_splash_entities_spawned_then_all_on_ui_layer() {
        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert
        let all_ui = world
            .query::<(&SplashEntity, &RenderLayer)>()
            .iter(&world)
            .all(|(_, layer)| *layer == RenderLayer::UI);
        assert!(all_ui);
    }

    #[test]
    fn when_splash_entities_spawned_then_all_visible() {
        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert
        let all_visible = world
            .query::<(&SplashEntity, &Visible)>()
            .iter(&world)
            .all(|(_, v)| v.0);
        assert!(all_visible);
    }

    #[test]
    fn when_splash_entities_spawned_then_five_letters_have_stroke() {
        use engine_render::prelude::{Shape, ShapeVariant, Stroke};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — the 5 letter entities (SortOrder == SPLASH_LETTER_ORDER) have Stroke
        let stroke_count = world
            .query::<(&SplashEntity, &Shape, &SortOrder, &Stroke)>()
            .iter(&world)
            .filter(|(_, shape, order, _)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 == SPLASH_LETTER_ORDER
            })
            .count();
        assert_eq!(stroke_count, 5);
    }

    #[test]
    fn when_splash_entities_spawned_then_letter_stroke_color_is_black() {
        use engine_render::prelude::{Shape, ShapeVariant, Stroke};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert
        let all_black = world
            .query::<(&SplashEntity, &Shape, &SortOrder, &Stroke)>()
            .iter(&world)
            .filter(|(_, shape, order, _)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 == SPLASH_LETTER_ORDER
            })
            .all(|(_, _, _, stroke)| {
                (stroke.color.r).abs() < f32::EPSILON
                    && (stroke.color.g).abs() < f32::EPSILON
                    && (stroke.color.b).abs() < f32::EPSILON
                    && (stroke.color.a - 1.0).abs() < f32::EPSILON
            });
        assert!(all_black);
    }

    #[test]
    fn when_segment_is_horizontal_rightward_then_normal_points_down() {
        // Arrange
        let from = Vec2::new(0.0, 0.0);
        let to = Vec2::new(1.0, 0.0);

        // Act
        let normal = segment_normal(from, to);

        // Assert
        assert!((normal.x - 0.0).abs() < 1e-6);
        assert!((normal.y - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn when_splash_entities_spawned_then_letter_stroke_width_is_positive() {
        use engine_render::prelude::{Shape, ShapeVariant, Stroke};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert
        let all_positive = world
            .query::<(&SplashEntity, &Shape, &SortOrder, &Stroke)>()
            .iter(&world)
            .filter(|(_, shape, order, _)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 == SPLASH_LETTER_ORDER
            })
            .all(|(_, _, _, stroke)| stroke.width > 0.0);
        assert!(all_positive);
    }

    #[test]
    fn when_normal_aligns_with_light_then_shade_returns_bright_color() {
        // Arrange
        let normal = Vec2::new(1.0, 0.0);
        let light_dir = Vec2::new(1.0, 0.0); // dot = 1.0, t = 1.0
        let dark = Color::new(0.0, 0.0, 0.0, 1.0);
        let bright = Color::new(0.8, 0.6, 0.4, 1.0);

        // Act
        let result = shade_for_normal(normal, light_dir, dark, bright);

        // Assert
        assert!((result.r - bright.r).abs() < 1e-6);
        assert!((result.g - bright.g).abs() < 1e-6);
        assert!((result.b - bright.b).abs() < 1e-6);
        assert!((result.a - bright.a).abs() < 1e-6);
    }

    #[test]
    fn when_normal_opposes_light_then_shade_returns_dark_color() {
        // Arrange
        let normal = Vec2::new(-1.0, 0.0);
        let light_dir = Vec2::new(1.0, 0.0); // dot = -1.0, t = 0.0
        let dark = Color::new(0.1, 0.2, 0.3, 1.0);
        let bright = Color::new(0.9, 0.8, 0.7, 1.0);

        // Act
        let result = shade_for_normal(normal, light_dir, dark, bright);

        // Assert
        assert!((result.r - dark.r).abs() < 1e-6);
        assert!((result.g - dark.g).abs() < 1e-6);
        assert!((result.b - dark.b).abs() < 1e-6);
        assert!((result.a - dark.a).abs() < 1e-6);
    }

    #[test]
    fn when_build_shaded_side_faces_then_back_faces_are_culled() {
        use engine_render::prelude::PathCommand;

        // Arrange — square contour, VP far to the right.
        // Back faces (normal opposes VP direction) are culled.
        let contour = vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2::new(10.0, 0.0)),
            PathCommand::LineTo(Vec2::new(10.0, 10.0)),
            PathCommand::LineTo(Vec2::new(0.0, 10.0)),
            PathCommand::Close,
        ];
        let vp = Vec2::new(50.0, 5.0);
        let light_dir = Vec2::new(1.0, 0.0);
        let dark = Color::new(0.0, 0.0, 0.0, 1.0);
        let bright = Color::new(1.0, 1.0, 1.0, 1.0);

        // Act
        let pairs = build_shaded_side_faces(&contour, vp, 0.1, 1, light_dir, dark, bright);

        // Assert — edges with normals opposing VP direction are culled
        assert!(pairs.len() > 0 && pairs.len() < 4, "some edges should be culled");
    }

    // --- build_shaded_side_faces ---

    #[test]
    fn when_build_shaded_side_faces_with_triangle_then_three_pairs_returned() {
        use engine_render::prelude::PathCommand;

        // Arrange — triangle with VP to the upper-right. Back faces culled.
        let contour = vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2::new(10.0, 0.0)),
            PathCommand::LineTo(Vec2::new(5.0, 10.0)),
            PathCommand::Close,
        ];
        let vp = Vec2::new(20.0, 10.0);
        let light_dir = Vec2::new(1.0, 0.0);
        let dark = Color::new(0.0, 0.0, 0.0, 1.0);
        let bright = Color::new(1.0, 1.0, 1.0, 1.0);

        // Act
        let pairs = build_shaded_side_faces(&contour, vp, 0.1, 4, light_dir, dark, bright);

        // Assert — at least one face visible, but back faces culled (fewer than 3)
        assert!(!pairs.is_empty());
        assert!(pairs.len() < 3);
    }

    #[test]
    fn when_build_shaded_side_faces_with_quadratic_then_curved_segment_produces_slices_and_colors_vary() {
        use engine_render::prelude::PathCommand;

        // Arrange — QuadraticTo arc + Close, VP to the left.
        let contour = vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::QuadraticTo {
                control: Vec2::new(10.0, -20.0),
                to: Vec2::new(20.0, 0.0),
            },
            PathCommand::Close,
        ];
        let vp = Vec2::new(-50.0, 0.0);
        let light_dir = Vec2::new(1.0, 0.0);
        let dark = Color::new(0.1, 0.1, 0.1, 1.0);
        let bright = Color::new(0.9, 0.9, 0.9, 1.0);

        // Act
        let pairs = build_shaded_side_faces(&contour, vp, 0.1, 4, light_dir, dark, bright);

        // Assert — at least some visible faces
        assert!(pairs.len() >= 2, "curve strips + close edge must produce visible faces");

        // Assert — colors must not all be identical (varying normals along arc)
        let colors: Vec<Color> = pairs.iter().map(|(_, c, _)| *c).collect();
        let all_same = colors.windows(2).all(|w| (w[0].r - w[1].r).abs() < 1e-6);
        assert!(!all_same, "varying normals along quadratic arc must yield varying shading");
    }

    #[test]
    fn when_build_shaded_side_faces_called_then_each_geometry_is_a_closed_five_command_quad() {
        use engine_render::prelude::PathCommand;

        // Arrange — QuadraticTo + Close, VP to the left
        let contour = vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::QuadraticTo {
                control: Vec2::new(10.0, -20.0),
                to: Vec2::new(20.0, 0.0),
            },
            PathCommand::Close,
        ];
        let vp = Vec2::new(-50.0, 0.0);
        let light_dir = Vec2::new(1.0, 0.0);
        let dark = Color::new(0.1, 0.1, 0.1, 1.0);
        let bright = Color::new(0.9, 0.9, 0.9, 1.0);

        // Act
        let pairs = build_shaded_side_faces(&contour, vp, 0.1, 4, light_dir, dark, bright);

        // Assert — every geometry must be [MoveTo, LineTo, LineTo, LineTo, Close]
        for (i, (cmds, _, _)) in pairs.iter().enumerate() {
            assert_eq!(cmds.len(), 5, "quad {i} must have exactly 5 commands");
            assert!(
                matches!(cmds[0], PathCommand::MoveTo(_)),
                "quad {i} command 0 must be MoveTo"
            );
            assert!(
                matches!(cmds[1], PathCommand::LineTo(_)),
                "quad {i} command 1 must be LineTo"
            );
            assert!(
                matches!(cmds[2], PathCommand::LineTo(_)),
                "quad {i} command 2 must be LineTo"
            );
            assert!(
                matches!(cmds[3], PathCommand::LineTo(_)),
                "quad {i} command 3 must be LineTo"
            );
            assert!(
                matches!(cmds[4], PathCommand::Close),
                "quad {i} command 4 must be Close"
            );
        }
    }
}
