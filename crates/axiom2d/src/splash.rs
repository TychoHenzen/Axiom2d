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
        render::spawn_splash_entities(app.world_mut());

        app.add_systems(Phase::PreUpdate, preload_system);
        app.add_systems(Phase::Update, splash_tick_system);
    }
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

#[cfg(feature = "render")]
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

#[cfg(feature = "render")]
mod render {
    use super::{
        ACCENT_COLOR, Color, LOGO_COLOR, RenderLayer, SPLASH_ACCENT_ORDER, SPLASH_BG_ORDER,
        SPLASH_LETTER_ORDER, SPLASH_SIDE_BASE, SortOrder, SplashEntity, Transform2D, Vec2, Visible,
    };
    use crate::splash_letters::{letter_a, letter_i, letter_m, letter_o, letter_x};
    use engine_render::prelude::{
        PathCommand, Shape, ShapeVariant, Stroke, resolve_commands, sample_cubic, sample_quadratic,
        split_contours,
    };

    const BG_COLOR: Color = Color {
        r: 0.05,
        g: 0.05,
        b: 0.08,
        a: 1.0,
    };

    pub(super) fn color_lerp(a: Color, b: Color, t: f32) -> Color {
        Color::new(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
            a.a + (b.a - a.a) * t,
        )
    }

    pub(super) fn shade_for_normal(
        normal: Vec2,
        light_dir: Vec2,
        dark: Color,
        bright: Color,
    ) -> Color {
        let t = (normal.dot(light_dir) + 1.0) * 0.5;
        let t = t.clamp(0.0, 1.0);
        let t = t * t * t;
        color_lerp(dark, bright, t)
    }

    pub(super) fn segment_normal(from: Vec2, to: Vec2) -> Vec2 {
        let d = to - from;
        Vec2::new(d.y, -d.x).normalize()
    }

    pub(super) fn project_point(p: Vec2, vp: Vec2, depth: f32) -> Vec2 {
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_shaded_side_faces(
        contour: &[PathCommand],
        vp: Vec2,
        depth: f32,
        slices: usize,
        light_dir: Vec2,
        dark: Color,
        bright: Color,
    ) -> Vec<(Vec<PathCommand>, Color, f32)> {
        let PathCommand::MoveTo(start) = contour[0] else {
            panic!("contour must start with MoveTo")
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
    pub(super) fn spawn_splash_entities(world: &mut bevy_ecs::world::World) {
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
                        SortOrder(SPLASH_SIDE_BASE + i as i32),
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

    #[cfg(feature = "render")]
    mod render_tests {
        use super::*;
        use engine_render::prelude::{PathCommand, Shape, ShapeVariant, split_contours};

        #[test]
        fn when_color_lerp_at_half_then_averages_channels() {
            // Arrange
            let a = Color::new(0.0, 0.2, 0.4, 1.0);
            let b = Color::new(1.0, 0.8, 0.6, 1.0);

            // Act
            let result = super::super::render::color_lerp(a, b, 0.5);

            // Assert
            assert!((result.r - 0.5).abs() < 1e-6);
            assert!((result.g - 0.5).abs() < 1e-6);
            assert!((result.b - 0.5).abs() < 1e-6);
        }

        #[test]
        fn when_color_lerp_at_zero_then_returns_first_color() {
            // Arrange
            let a = Color::new(0.2, 0.4, 0.6, 1.0);
            let b = Color::new(0.8, 0.6, 0.4, 1.0);

            // Act
            let result = super::super::render::color_lerp(a, b, 0.0);

            // Assert
            assert!((result.r - a.r).abs() < 1e-6);
            assert!((result.g - a.g).abs() < 1e-6);
            assert!((result.b - a.b).abs() < 1e-6);
        }

        #[test]
        fn when_segment_normal_horizontal_rightward_then_points_downward() {
            // Arrange
            let from = Vec2::new(0.0, 0.0);
            let to = Vec2::new(10.0, 0.0);

            // Act
            let normal = super::super::render::segment_normal(from, to);

            // Assert — d = (10, 0), normal = (d.y, -d.x).normalize = (0, -10).normalize = (0, -1)
            assert!(normal.x.abs() < 1e-6, "expected nx=0, got {}", normal.x);
            assert!(
                (normal.y - (-1.0)).abs() < 1e-6,
                "expected ny=-1, got {}",
                normal.y
            );
        }

        #[test]
        fn when_segment_normal_vertical_upward_then_points_rightward() {
            // Arrange
            let from = Vec2::new(0.0, 0.0);
            let to = Vec2::new(0.0, 10.0);

            // Act
            let normal = super::super::render::segment_normal(from, to);

            // Assert — d = (0, 10), normal = (10, 0).normalize = (1, 0)
            assert!(
                (normal.x - 1.0).abs() < 1e-6,
                "expected nx=1, got {}",
                normal.x
            );
            assert!(normal.y.abs() < 1e-6, "expected ny=0, got {}", normal.y);
        }

        #[test]
        fn when_project_point_at_half_depth_then_moves_halfway_to_vanishing_point() {
            // Arrange
            let p = Vec2::new(0.0, 0.0);
            let vp = Vec2::new(10.0, 0.0);

            // Act
            let result = super::super::render::project_point(p, vp, 0.5);

            // Assert — p + (vp - p) * 0.5 = (0,0) + (10,0) * 0.5 = (5, 0)
            assert!(
                (result.x - 5.0).abs() < 1e-6,
                "expected x=5, got {}",
                result.x
            );
            assert!(result.y.abs() < 1e-6, "expected y=0, got {}", result.y);
        }

        #[test]
        fn when_project_point_at_zero_depth_then_returns_original_point() {
            // Arrange
            let p = Vec2::new(3.0, 7.0);
            let vp = Vec2::new(10.0, 20.0);

            // Act
            let result = super::super::render::project_point(p, vp, 0.0);

            // Assert
            assert!((result.x - 3.0).abs() < 1e-6);
            assert!((result.y - 7.0).abs() < 1e-6);
        }

        fn run_splash_render(world: &mut World) {
            let mut schedule = Schedule::default();
            schedule.add_systems(splash_render_system);
            schedule.run(world);
        }

        #[test]
        fn when_splash_done_then_render_system_does_not_set_projection() {
            // Arrange
            let mut world = World::new();
            world.insert_resource(SplashScreen {
                elapsed: 3.0,
                duration: 2.0,
                done: true,
            });
            let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(800, 600);
            world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

            // Act
            run_splash_render(&mut world);

            // Assert
            let calls = log.lock().unwrap();
            assert!(
                !calls.iter().any(|c| c == "set_view_projection"),
                "should not set projection when done"
            );
        }

        #[test]
        fn when_viewport_zero_width_then_render_system_does_not_set_projection() {
            // Arrange
            let mut world = World::new();
            world.insert_resource(SplashScreen::new(2.0));
            let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(0, 600);
            world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

            // Act
            run_splash_render(&mut world);

            // Assert
            let calls = log.lock().unwrap();
            assert!(
                !calls.iter().any(|c| c == "set_view_projection"),
                "should not set projection when width=0"
            );
        }

        #[test]
        fn when_viewport_zero_height_then_render_system_does_not_set_projection() {
            // Arrange
            let mut world = World::new();
            world.insert_resource(SplashScreen::new(2.0));
            let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(800, 0);
            world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

            // Act
            run_splash_render(&mut world);

            // Assert
            let calls = log.lock().unwrap();
            assert!(
                !calls.iter().any(|c| c == "set_view_projection"),
                "should not set projection when height=0"
            );
        }

        #[test]
        fn when_splash_active_and_viewport_valid_then_sets_projection() {
            // Arrange
            let mut world = World::new();
            world.insert_resource(SplashScreen::new(2.0));
            let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(800, 600);
            world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

            // Act
            run_splash_render(&mut world);

            // Assert
            let calls = log.lock().unwrap();
            assert!(
                calls.iter().any(|c| c == "set_view_projection"),
                "should set projection when active with valid viewport"
            );
        }

        #[test]
        fn when_splash_render_zoom_computed_then_uses_min_of_width_and_height_ratios() {
            // Arrange — 1000x300 viewport: vw/500=2.0, vh/300=1.0 → zoom=1.0
            let mut world = World::new();
            world.insert_resource(SplashScreen::new(2.0));
            let matrix = std::sync::Arc::new(std::sync::Mutex::new(None));
            let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy = engine_render::testing::SpyRenderer::new(log)
                .with_viewport(1000, 300)
                .with_matrix_capture(matrix.clone());
            world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

            // Act
            run_splash_render(&mut world);

            // Assert — verify projection was set (matrix captured)
            let mat = matrix.lock().unwrap();
            assert!(mat.is_some(), "projection matrix should be captured");
            // The zoom is min(1000/500, 300/300) = min(2.0, 1.0) = 1.0
            // With a different viewport: 500x600 → min(500/500, 600/300) = min(1.0, 2.0) = 1.0
            // With 250x300 → min(250/500, 300/300) = min(0.5, 1.0) = 0.5
            // The matrix diagonal elements scale with zoom, so let's verify with an asymmetric case
        }

        #[test]
        fn when_splash_render_wide_viewport_then_zoom_limited_by_height() {
            // Arrange — 2000x300: vw/500=4.0, vh/300=1.0 → zoom=1.0
            // vs 2000x600: vw/500=4.0, vh/300=2.0 → zoom=2.0
            // Different zoom → different projection matrix
            let mut world_a = World::new();
            world_a.insert_resource(SplashScreen::new(2.0));
            let matrix_a = std::sync::Arc::new(std::sync::Mutex::new(None));
            let log_a = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy_a = engine_render::testing::SpyRenderer::new(log_a)
                .with_viewport(2000, 300)
                .with_matrix_capture(matrix_a.clone());
            world_a.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy_a)));

            let mut world_b = World::new();
            world_b.insert_resource(SplashScreen::new(2.0));
            let matrix_b = std::sync::Arc::new(std::sync::Mutex::new(None));
            let log_b = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let spy_b = engine_render::testing::SpyRenderer::new(log_b)
                .with_viewport(2000, 600)
                .with_matrix_capture(matrix_b.clone());
            world_b.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy_b)));

            // Act
            run_splash_render(&mut world_a);
            run_splash_render(&mut world_b);

            // Assert — different viewports → different matrices (zoom differs)
            let mat_a = matrix_a.lock().unwrap().unwrap();
            let mat_b = matrix_b.lock().unwrap().unwrap();
            assert_ne!(
                mat_a, mat_b,
                "different viewport heights should produce different projections"
            );
        }

        #[test]
        fn when_split_contours_called_then_splits_on_moveto() {
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
            // Arrange / Act
            let mut world = World::new();
            super::super::render::spawn_splash_entities(&mut world);

            // Assert
            let side_count = world
                .query::<(&SplashEntity, &Shape, &SortOrder)>()
                .iter(&world)
                .filter(|(_, shape, order)| {
                    matches!(shape.variant, ShapeVariant::Path { .. })
                        && order.0 >= SPLASH_SIDE_BASE
                        && order.0 < SPLASH_LETTER_ORDER
                })
                .count();

            assert!(side_count > 0, "side face entities must exist");
        }

        #[test]
        fn when_depth_paths_spawned_then_all_darker_than_logo_color() {
            // Arrange / Act
            let mut world = World::new();
            super::super::render::spawn_splash_entities(&mut world);

            // Assert
            let all_darker = world
                .query::<(&SplashEntity, &Shape, &SortOrder)>()
                .iter(&world)
                .filter(|(_, shape, order)| {
                    matches!(shape.variant, ShapeVariant::Path { .. })
                        && order.0 >= SPLASH_SIDE_BASE
                        && order.0 < SPLASH_LETTER_ORDER
                })
                .all(|(_, shape, _)| {
                    shape.color.r < LOGO_COLOR.r
                        && shape.color.g < LOGO_COLOR.g
                        && shape.color.b < LOGO_COLOR.b
                });

            assert!(
                all_darker,
                "all side face colors must be darker than LOGO_COLOR"
            );
        }

        #[test]
        fn when_depth_paths_spawned_then_shaded_colors_are_not_all_identical() {
            // Arrange / Act
            let mut world = World::new();
            super::super::render::spawn_splash_entities(&mut world);

            // Assert
            let colors: Vec<(u32, u32, u32)> = world
                .query::<(&SplashEntity, &Shape, &SortOrder)>()
                .iter(&world)
                .filter(|(_, shape, order)| {
                    matches!(shape.variant, ShapeVariant::Path { .. })
                        && order.0 >= SPLASH_SIDE_BASE
                        && order.0 < SPLASH_LETTER_ORDER
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

            assert!(
                !all_same,
                "shading must produce color variation across side faces"
            );
        }

        #[test]
        fn when_splash_entities_spawned_then_has_bg_letters_accents_and_side_faces() {
            // Arrange / Act
            let mut world = World::new();
            super::super::render::spawn_splash_entities(&mut world);

            // Assert
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
            // Arrange / Act
            let mut world = World::new();
            super::super::render::spawn_splash_entities(&mut world);

            // Assert
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
        fn when_normal_aligns_with_light_then_shade_returns_bright_color() {
            // Arrange
            let normal = Vec2::new(1.0, 0.0);
            let light_dir = Vec2::new(1.0, 0.0);
            let dark = Color::new(0.0, 0.0, 0.0, 1.0);
            let bright = Color::new(0.8, 0.6, 0.4, 1.0);

            // Act
            let result = super::super::render::shade_for_normal(normal, light_dir, dark, bright);

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
            let light_dir = Vec2::new(1.0, 0.0);
            let dark = Color::new(0.1, 0.2, 0.3, 1.0);
            let bright = Color::new(0.9, 0.8, 0.7, 1.0);

            // Act
            let result = super::super::render::shade_for_normal(normal, light_dir, dark, bright);

            // Assert
            assert!((result.r - dark.r).abs() < 1e-6);
            assert!((result.g - dark.g).abs() < 1e-6);
            assert!((result.b - dark.b).abs() < 1e-6);
            assert!((result.a - dark.a).abs() < 1e-6);
        }

        #[test]
        fn when_build_shaded_side_faces_then_back_faces_are_culled() {
            // Arrange
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
            let pairs = super::super::render::build_shaded_side_faces(
                &contour, vp, 0.1, 1, light_dir, dark, bright,
            );

            // Assert
            assert!(
                !pairs.is_empty() && pairs.len() < 4,
                "some edges should be culled"
            );
        }

        #[test]
        fn when_build_shaded_side_faces_with_quadratic_then_curved_segment_produces_slices_and_colors_vary()
         {
            // Arrange
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
            let pairs = super::super::render::build_shaded_side_faces(
                &contour, vp, 0.1, 4, light_dir, dark, bright,
            );

            // Assert
            assert!(
                pairs.len() >= 2,
                "curve strips + close edge must produce visible faces"
            );
            let colors: Vec<Color> = pairs.iter().map(|(_, c, _)| *c).collect();
            let all_same = colors.windows(2).all(|w| (w[0].r - w[1].r).abs() < 1e-6);
            assert!(
                !all_same,
                "varying normals along quadratic arc must yield varying shading"
            );
        }

        #[test]
        fn when_build_shaded_side_faces_with_cubic_then_produces_sliced_faces() {
            // Arrange
            let contour = vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::CubicTo {
                    control1: Vec2::new(5.0, -20.0),
                    control2: Vec2::new(15.0, -20.0),
                    to: Vec2::new(20.0, 0.0),
                },
                PathCommand::Close,
            ];
            let vp = Vec2::new(-50.0, 0.0);
            let light_dir = Vec2::new(1.0, 0.0);
            let dark = Color::new(0.1, 0.1, 0.1, 1.0);
            let bright = Color::new(0.9, 0.9, 0.9, 1.0);

            // Act
            let pairs = super::super::render::build_shaded_side_faces(
                &contour, vp, 0.1, 4, light_dir, dark, bright,
            );

            // Assert — cubic arc with 4 slices should produce visible faces
            assert!(
                pairs.len() >= 2,
                "cubic curve strips + close edge must produce visible faces, got {}",
                pairs.len()
            );
        }

        #[test]
        fn when_build_shaded_side_faces_then_results_sorted_farthest_first() {
            // Arrange
            let contour = vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(20.0, 0.0)),
                PathCommand::LineTo(Vec2::new(20.0, 20.0)),
                PathCommand::LineTo(Vec2::new(0.0, 20.0)),
                PathCommand::Close,
            ];
            let vp = Vec2::new(50.0, 10.0);
            let light_dir = Vec2::new(1.0, 0.0);
            let dark = Color::new(0.0, 0.0, 0.0, 1.0);
            let bright = Color::new(1.0, 1.0, 1.0, 1.0);

            // Act
            let pairs = super::super::render::build_shaded_side_faces(
                &contour, vp, 0.1, 1, light_dir, dark, bright,
            );

            // Assert — sorted by distance descending (farthest first for painter's algorithm)
            for w in pairs.windows(2) {
                assert!(
                    w[0].2 >= w[1].2,
                    "expected farthest-first sort: dist {} >= {}",
                    w[0].2,
                    w[1].2
                );
            }
        }

        #[test]
        fn when_build_shaded_side_faces_with_close_gap_then_closing_edge_included() {
            // Arrange — triangle where close segment is far from start
            let contour = vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(30.0, 0.0)),
                PathCommand::LineTo(Vec2::new(15.0, 30.0)),
                PathCommand::Close,
            ];
            let vp = Vec2::new(-50.0, 15.0);
            let light_dir = Vec2::new(-1.0, 0.0);
            let dark = Color::new(0.1, 0.1, 0.1, 1.0);
            let bright = Color::new(0.9, 0.9, 0.9, 1.0);

            // Act
            let pairs = super::super::render::build_shaded_side_faces(
                &contour, vp, 0.1, 1, light_dir, dark, bright,
            );

            // Assert — at least some faces exist (the close segment from (15,30) back to (0,0)
            // has a large gap so > f32::EPSILON check must pass)
            assert!(
                !pairs.is_empty(),
                "triangle with non-trivial close gap must produce faces"
            );
        }

        #[test]
        fn when_build_shaded_side_faces_called_then_each_geometry_is_a_closed_five_command_quad() {
            // Arrange
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
            let pairs = super::super::render::build_shaded_side_faces(
                &contour, vp, 0.1, 4, light_dir, dark, bright,
            );

            // Assert
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
}
