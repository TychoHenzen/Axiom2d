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
const SPLASH_SORT_ORDER: i32 = 10_000;

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
const BG_COLOR: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.08,
    a: 1.0,
};

#[cfg(feature = "render")]
fn spawn_splash_entities(world: &mut bevy_ecs::world::World) {
    use engine_render::prelude::{PathCommand, Shape, ShapeVariant};

    // Full-screen dark background
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
        SortOrder(SPLASH_SORT_ORDER),
    ));

    // "AXIOM" letters positioned left-to-right, centered at origin
    let letters: [(f32, Vec<PathCommand>); 5] = [
        (-140.0, letter_a()),
        (-60.0, letter_x()),
        (0.0, letter_i()),
        (58.0, letter_o()),
        (140.0, letter_m()),
    ];

    for (x, commands) in letters {
        world.spawn((
            SplashEntity,
            Transform2D {
                position: Vec2::new(x, -10.0),
                ..Default::default()
            },
            Shape {
                variant: ShapeVariant::Path { commands },
                color: LOGO_COLOR,
            },
            Visible(true),
            RenderLayer::UI,
            SortOrder(SPLASH_SORT_ORDER + 1),
        ));
    }

    // Accent line under the text — tapered lens shape
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
        SortOrder(SPLASH_SORT_ORDER + 2),
    ));

    // Small accent diamond centered below the accent line
    world.spawn((
        SplashEntity,
        Transform2D {
            position: Vec2::new(0.0, 85.0),
            ..Default::default()
        },
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(0.0, -6.0),
                    Vec2::new(6.0, 0.0),
                    Vec2::new(0.0, 6.0),
                    Vec2::new(-6.0, 0.0),
                ],
            },
            color: ACCENT_COLOR,
        },
        Visible(true),
        RenderLayer::UI,
        SortOrder(SPLASH_SORT_ORDER + 2),
    ));
}

/// Futuristic angular "A" — outer contour with crossbar notch + triangular inner hole.
#[cfg(feature = "render")]
fn letter_a() -> Vec<engine_render::prelude::PathCommand> {
    use engine_render::prelude::PathCommand;
    vec![
        // Outer contour
        PathCommand::MoveTo(Vec2::new(0.0, -60.0)),
        PathCommand::LineTo(Vec2::new(35.0, 60.0)),
        PathCommand::LineTo(Vec2::new(23.0, 60.0)),
        PathCommand::LineTo(Vec2::new(10.0, 18.0)),
        PathCommand::LineTo(Vec2::new(-10.0, 18.0)),
        PathCommand::LineTo(Vec2::new(-23.0, 60.0)),
        PathCommand::LineTo(Vec2::new(-35.0, 60.0)),
        PathCommand::Close,
        // Inner triangular hole (EvenOdd fill)
        PathCommand::MoveTo(Vec2::new(0.0, -34.0)),
        PathCommand::LineTo(Vec2::new(10.0, 10.0)),
        PathCommand::LineTo(Vec2::new(-10.0, 10.0)),
        PathCommand::Close,
    ]
}

/// Futuristic "X" — two crossing diagonal bars with EvenOdd diamond cutout at center.
#[cfg(feature = "render")]
fn letter_x() -> Vec<engine_render::prelude::PathCommand> {
    use engine_render::prelude::PathCommand;
    vec![
        // Bar 1: top-left to bottom-right
        PathCommand::MoveTo(Vec2::new(-31.0, -60.0)),
        PathCommand::LineTo(Vec2::new(-19.0, -60.0)),
        PathCommand::LineTo(Vec2::new(31.0, 60.0)),
        PathCommand::LineTo(Vec2::new(19.0, 60.0)),
        PathCommand::Close,
        // Bar 2: top-right to bottom-left
        PathCommand::MoveTo(Vec2::new(19.0, -60.0)),
        PathCommand::LineTo(Vec2::new(31.0, -60.0)),
        PathCommand::LineTo(Vec2::new(-19.0, 60.0)),
        PathCommand::LineTo(Vec2::new(-31.0, 60.0)),
        PathCommand::Close,
    ]
}

/// Futuristic "I" — vertical bar with angular serif caps.
#[cfg(feature = "render")]
fn letter_i() -> Vec<engine_render::prelude::PathCommand> {
    use engine_render::prelude::PathCommand;
    vec![
        PathCommand::MoveTo(Vec2::new(-15.0, -60.0)),
        PathCommand::LineTo(Vec2::new(15.0, -60.0)),
        PathCommand::LineTo(Vec2::new(15.0, -48.0)),
        PathCommand::LineTo(Vec2::new(6.0, -48.0)),
        PathCommand::LineTo(Vec2::new(6.0, 48.0)),
        PathCommand::LineTo(Vec2::new(15.0, 48.0)),
        PathCommand::LineTo(Vec2::new(15.0, 60.0)),
        PathCommand::LineTo(Vec2::new(-15.0, 60.0)),
        PathCommand::LineTo(Vec2::new(-15.0, 48.0)),
        PathCommand::LineTo(Vec2::new(-6.0, 48.0)),
        PathCommand::LineTo(Vec2::new(-6.0, -48.0)),
        PathCommand::LineTo(Vec2::new(-15.0, -48.0)),
        PathCommand::Close,
    ]
}

/// Futuristic "O" — beveled rectangle with smooth QuadraticTo corners + inner hole.
#[cfg(feature = "render")]
fn letter_o() -> Vec<engine_render::prelude::PathCommand> {
    use engine_render::prelude::PathCommand;
    vec![
        // Outer contour with rounded corners
        PathCommand::MoveTo(Vec2::new(-12.0, -60.0)),
        PathCommand::LineTo(Vec2::new(12.0, -60.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(30.0, -60.0),
            to: Vec2::new(30.0, -42.0),
        },
        PathCommand::LineTo(Vec2::new(30.0, 42.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(30.0, 60.0),
            to: Vec2::new(12.0, 60.0),
        },
        PathCommand::LineTo(Vec2::new(-12.0, 60.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(-30.0, 60.0),
            to: Vec2::new(-30.0, 42.0),
        },
        PathCommand::LineTo(Vec2::new(-30.0, -42.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(-30.0, -60.0),
            to: Vec2::new(-12.0, -60.0),
        },
        PathCommand::Close,
        // Inner hole with rounded corners
        PathCommand::MoveTo(Vec2::new(-2.0, -48.0)),
        PathCommand::LineTo(Vec2::new(2.0, -48.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(18.0, -48.0),
            to: Vec2::new(18.0, -32.0),
        },
        PathCommand::LineTo(Vec2::new(18.0, 32.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(18.0, 48.0),
            to: Vec2::new(2.0, 48.0),
        },
        PathCommand::LineTo(Vec2::new(-2.0, 48.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(-18.0, 48.0),
            to: Vec2::new(-18.0, 32.0),
        },
        PathCommand::LineTo(Vec2::new(-18.0, -32.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(-18.0, -48.0),
            to: Vec2::new(-2.0, -48.0),
        },
        PathCommand::Close,
    ]
}

/// Futuristic "M" — angular M with sharp peaks and center valley.
#[cfg(feature = "render")]
fn letter_m() -> Vec<engine_render::prelude::PathCommand> {
    use engine_render::prelude::PathCommand;
    vec![
        PathCommand::MoveTo(Vec2::new(-38.0, 60.0)),
        PathCommand::LineTo(Vec2::new(-38.0, -60.0)),
        PathCommand::LineTo(Vec2::new(-22.0, -60.0)),
        PathCommand::LineTo(Vec2::new(0.0, -20.0)),
        PathCommand::LineTo(Vec2::new(22.0, -60.0)),
        PathCommand::LineTo(Vec2::new(38.0, -60.0)),
        PathCommand::LineTo(Vec2::new(38.0, 60.0)),
        PathCommand::LineTo(Vec2::new(26.0, 60.0)),
        PathCommand::LineTo(Vec2::new(26.0, -40.0)),
        PathCommand::LineTo(Vec2::new(6.0, -4.0)),
        PathCommand::LineTo(Vec2::new(-6.0, -4.0)),
        PathCommand::LineTo(Vec2::new(-26.0, -40.0)),
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
    fn when_splash_entities_spawned_then_eight_entities_created() {
        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — 1 background + 5 letters + 1 accent line + 1 accent diamond
        let count = world.query::<&SplashEntity>().iter(&world).count();
        assert_eq!(count, 8);
    }

    #[test]
    fn when_splash_entities_spawned_then_five_path_shapes_exist() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — the 5 letter entities use ShapeVariant::Path
        let path_count = world
            .query::<(&SplashEntity, &Shape)>()
            .iter(&world)
            .filter(|(_, shape)| matches!(shape.variant, ShapeVariant::Path { .. }))
            .count();
        assert_eq!(path_count, 6); // 5 letters + 1 accent line
    }

    #[test]
    fn when_splash_entities_spawned_then_letters_positioned_left_to_right() {
        use engine_render::prelude::{Shape, ShapeVariant};

        // Arrange / Act
        let mut world = World::new();
        spawn_splash_entities(&mut world);

        // Assert — letter entities (Path shapes on SPLASH_SORT_ORDER+1) are ordered left-to-right
        let mut letter_xs: Vec<f32> = world
            .query::<(&SplashEntity, &Transform2D, &Shape, &SortOrder)>()
            .iter(&world)
            .filter(|(_, _, shape, order)| {
                matches!(shape.variant, ShapeVariant::Path { .. })
                    && order.0 == SPLASH_SORT_ORDER + 1
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
}
