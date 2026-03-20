use bevy_ecs::prelude::{Query, Res, ResMut, With, World};
use engine_app::prelude::{App, Phase, Plugin};
use engine_core::prelude::DeltaTime;
use engine_scene::prelude::Visible;

use super::types::{PostSplashSetup, PreloadHooks, SPLASH_DURATION, SplashEntity, SplashScreen};

pub fn preload_system(world: &mut World) {
    let splash_done = world.resource::<SplashScreen>().done;
    let already_executed = world
        .get_resource::<PreloadHooks>()
        .is_none_or(|h| h.executed);

    if splash_done || already_executed {
        return;
    }

    let Some(mut hooks) = world.remove_resource::<PreloadHooks>() else {
        return;
    };
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
        app.world_mut().insert_resource(PostSplashSetup::new());

        #[cfg(feature = "render")]
        super::render::spawn_splash_entities(app.world_mut());

        app.add_systems(Phase::PreUpdate, preload_system);
        app.add_systems(Phase::PreUpdate, post_splash_setup_system);
        app.add_systems(Phase::Update, splash_tick_system);
    }
}

pub fn post_splash_setup_system(world: &mut World) {
    let splash_done = world.get_resource::<SplashScreen>().is_none_or(|s| s.done);
    if !splash_done {
        return;
    }

    let already_executed = world
        .get_resource::<PostSplashSetup>()
        .is_none_or(|h| h.executed);
    if already_executed {
        return;
    }

    let Some(mut setup) = world.remove_resource::<PostSplashSetup>() else {
        return;
    };
    for hook in &mut setup.hooks {
        hook(world);
    }
    setup.executed = true;
    setup.hooks.clear();
    world.insert_resource(setup);
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
                let mut clock = engine_core::time::FakeClock::default();
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
    fn when_splash_done_then_post_splash_hooks_run() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 3.0,
            duration: 2.0,
            done: true,
        });
        let mut setup = PostSplashSetup::new();
        setup.add(|w: &mut World| {
            w.insert_resource(DeltaTime(Seconds(99.0)));
        });
        world.insert_resource(setup);
        world.insert_resource(DeltaTime(Seconds(0.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(post_splash_setup_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!((world.resource::<DeltaTime>().0.0 - 99.0).abs() < f32::EPSILON);
        assert!(world.resource::<PostSplashSetup>().executed);
    }

    #[test]
    fn when_post_splash_hooks_already_executed_then_not_run_again() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 3.0,
            duration: 2.0,
            done: true,
        });
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let counter = std::sync::Arc::clone(&call_count);
        let mut setup = PostSplashSetup::new();
        setup.add(move |_: &mut World| {
            *counter.lock().unwrap() += 1;
        });
        setup.executed = true;
        world.insert_resource(setup);

        let mut schedule = Schedule::default();
        schedule.add_systems(post_splash_setup_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(*call_count.lock().unwrap(), 0);
    }

    #[test]
    fn when_no_splash_screen_resource_then_post_splash_hooks_run_immediately() {
        // Arrange
        let mut world = World::new();
        let mut setup = PostSplashSetup::new();
        setup.add(|w: &mut World| {
            w.insert_resource(DeltaTime(Seconds(42.0)));
        });
        world.insert_resource(setup);
        world.insert_resource(DeltaTime(Seconds(0.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(post_splash_setup_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!((world.resource::<DeltaTime>().0.0 - 42.0).abs() < f32::EPSILON);
        assert!(world.resource::<PostSplashSetup>().executed);
    }

    #[test]
    fn when_post_splash_hook_runs_then_can_spawn_entities() {
        // Arrange
        #[derive(bevy_ecs::prelude::Component)]
        struct Marker;

        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 3.0,
            duration: 2.0,
            done: true,
        });
        let mut setup = PostSplashSetup::new();
        setup.add(|w: &mut World| {
            w.spawn(Marker);
        });
        world.insert_resource(setup);

        let mut schedule = Schedule::default();
        schedule.add_systems(post_splash_setup_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let count = world.query::<&Marker>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn when_multiple_post_splash_hooks_then_run_in_order() {
        // Arrange
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 3.0,
            duration: 2.0,
            done: true,
        });
        let mut setup = PostSplashSetup::new();
        for i in 0..3 {
            let order = std::sync::Arc::clone(&order);
            setup.add(move |_: &mut World| {
                order.lock().unwrap().push(i);
            });
        }
        world.insert_resource(setup);

        let mut schedule = Schedule::default();
        schedule.add_systems(post_splash_setup_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn when_splash_plugin_built_then_post_splash_setup_is_present() {
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(SplashPlugin);

        // Assert
        assert!(app.world().get_resource::<PostSplashSetup>().is_some());
    }

    #[test]
    fn when_splash_done_and_post_splash_hook_registered_then_hook_runs_on_redraw() {
        // Arrange
        let mut app = App::new();
        app.add_plugin(crate::default_plugins::DefaultPlugins);
        app.world_mut()
            .insert_resource(engine_core::prelude::ClockRes::new(Box::new({
                let mut clock = engine_core::time::FakeClock::default();
                clock.advance(Seconds(0.1));
                clock
            })));
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(
                engine_render::prelude::NullRenderer,
            )));
        app.world_mut().resource_mut::<SplashScreen>().done = true;

        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let counter = std::sync::Arc::clone(&call_count);
        app.world_mut()
            .resource_mut::<PostSplashSetup>()
            .add(move |_: &mut World| {
                *counter.lock().unwrap() += 1;
            });

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn when_preload_and_post_splash_coexist_then_preload_runs_during_and_post_splash_runs_after() {
        // Arrange
        let preload_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let post_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));

        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 0.5,
            duration: 2.0,
            done: false,
        });

        let pc = std::sync::Arc::clone(&preload_count);
        let mut hooks = PreloadHooks::new();
        hooks.add(move |_: &mut World| {
            *pc.lock().unwrap() += 1;
        });
        world.insert_resource(hooks);

        let psc = std::sync::Arc::clone(&post_count);
        let mut setup = PostSplashSetup::new();
        setup.add(move |_: &mut World| {
            *psc.lock().unwrap() += 1;
        });
        world.insert_resource(setup);

        let mut schedule = Schedule::default();
        schedule.add_systems((preload_system, post_splash_setup_system));

        // Act — run while splash not done
        schedule.run(&mut world);

        // Assert — preload ran, post-splash did not
        assert_eq!(*preload_count.lock().unwrap(), 1);
        assert_eq!(*post_count.lock().unwrap(), 0);

        // Act — mark splash done and run again
        world.resource_mut::<SplashScreen>().done = true;
        schedule.run(&mut world);

        // Assert — post-splash ran now
        assert_eq!(*post_count.lock().unwrap(), 1);
    }

    #[test]
    fn when_splash_not_done_then_post_splash_hooks_do_not_run() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SplashScreen {
            elapsed: 0.5,
            duration: 2.0,
            done: false,
        });
        let mut setup = PostSplashSetup::new();
        setup.add(|w: &mut World| {
            w.insert_resource(DeltaTime(Seconds(99.0)));
        });
        world.insert_resource(setup);
        world.insert_resource(DeltaTime(Seconds(0.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(post_splash_setup_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!((world.resource::<DeltaTime>().0.0).abs() < f32::EPSILON);
    }
}
