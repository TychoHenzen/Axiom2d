// EVOLVE-BLOCK-START
#![cfg(feature = "render")]

use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::{Color, Transform2D};
use engine_render::prelude::{
    PathCommand, Shape, ShapeVariant, Stroke, resolve_commands, sample_cubic, sample_quadratic,
    split_contours,
};
use engine_scene::prelude::{RenderLayer, SortOrder, Visible};
use glam::Vec2;

use super::letters::{letter_a, letter_i, letter_m, letter_o, letter_x};
use super::types::{
    ACCENT_COLOR, LOGO_COLOR, SPLASH_ACCENT_ORDER, SPLASH_BG_ORDER, SPLASH_LETTER_ORDER,
    SPLASH_SIDE_BASE, SplashEntity, SplashScreen,
};

const BG_COLOR: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.08,
    a: 1.0,
};

pub(crate) fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub(crate) fn shade_for_normal(normal: Vec2, light_dir: Vec2, dark: Color, bright: Color) -> Color {
    let t = (normal.dot(light_dir) + 1.0) * 0.5;
    let t = t.clamp(0.0, 1.0);
    let t = t * t * t;
    color_lerp(dark, bright, t)
}

pub(crate) fn segment_normal(from: Vec2, to: Vec2) -> Vec2 {
    let d = to - from;
    Vec2::new(d.y, -d.x).normalize()
}

pub(crate) fn project_point(p: Vec2, vp: Vec2, depth: f32) -> Vec2 {
    p + (vp - p) * depth
}

fn side_quad(a: Vec2, b: Vec2, vp: Vec2, depth: f32) -> Vec<PathCommand> {
    vec![
        PathCommand::MoveTo(a),
        PathCommand::LineTo(b),
        PathCommand::LineTo(project_point(b, vp, depth)),
        PathCommand::LineTo(project_point(a, vp, depth)),
        PathCommand::Close,
    ]
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn build_shaded_side_faces(
    contour: &[PathCommand],
    vp: Vec2,
    depth: f32,
    slices: usize,
    light_dir: Vec2,
    dark: Color,
    bright: Color,
) -> Vec<(Vec<PathCommand>, Color, f32)> {
    let PathCommand::MoveTo(start) = contour[0] else {
        return Vec::new();
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

    result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    result
}

#[allow(clippy::too_many_lines)]
pub(crate) fn spawn_splash_entities(world: &mut bevy_ecs::world::World) {
    world.spawn((
        SplashEntity,
        Transform2D::default(),
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
        SortOrder::new(SPLASH_BG_ORDER),
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
        let resolved = resolve_commands(&commands);

        for contour in split_contours(&resolved) {
            let faces = build_shaded_side_faces(
                &contour,
                vp_local,
                depth,
                slices,
                light_dir,
                dark_color,
                bright_color,
            );
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
                    SortOrder::new(SPLASH_SIDE_BASE + i as i32),
                ));
            }
        }

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
            SortOrder::new(SPLASH_LETTER_ORDER),
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
        SortOrder::new(SPLASH_ACCENT_ORDER),
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
        SortOrder::new(SPLASH_ACCENT_ORDER),
    ));
}

pub fn splash_render_system(
    splash: Res<SplashScreen>,
    mut renderer: ResMut<engine_render::prelude::RendererRes>,
) {
    if splash.done {
        return;
    }
    let (vw, vh) = renderer.viewport_size();
    if vw == 0 || vh == 0 {
        return;
    }
    let vw = vw as f32;
    let vh = vh as f32;

    let splash_camera = engine_render::prelude::Camera2D {
        position: Vec2::new(0.0, 15.0),
        zoom: (vw / 500.0).min(vh / 300.0),
    };
    let uniform = engine_render::prelude::CameraUniform::from_camera(&splash_camera, vw, vh);
    renderer.set_view_projection(uniform.view_proj);
}
// EVOLVE-BLOCK-END
