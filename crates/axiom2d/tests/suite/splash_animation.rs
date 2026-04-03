#![allow(clippy::unwrap_used)]

use axiom2d::prelude::*;
use axiom2d::splash::{post_splash_setup_system, preload_system, splash_tick_system};

const FLOAT_EPSILON: f32 = 1e-6;

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

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= FLOAT_EPSILON,
        "expected {expected} +/- {FLOAT_EPSILON}, got {actual}"
    );
}

#[test]
fn when_splash_tick_runs_then_elapsed_increases_by_delta() {
    // Arrange
    let (mut world, mut schedule) = world_with_splash(0.0, 2.0, false, 0.016);

    // Act
    schedule.run(&mut world);

    // Assert
    let splash = world.resource::<SplashScreen>();
    assert_close(splash.elapsed, 0.016);
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
    assert_close(splash.elapsed, 2.5);
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
    assert_close(splash.duration, SPLASH_DURATION);
    assert_close(splash.elapsed, 0.0);
    assert!(!splash.done);
}

#[test]
fn when_splash_plugin_built_then_system_runs_on_redraw() {
    // Arrange
    let mut app = App::new();
    app.add_plugin(DefaultPlugins);
    app.add_plugin(SplashPlugin);
    app.world_mut().insert_resource(ClockRes::new(Box::new({
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
    assert_close(world.resource::<DeltaTime>().0.0, 42.0);
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
    assert_close(world.resource::<DeltaTime>().0.0, 99.0);
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
    assert_close(world.resource::<DeltaTime>().0.0, 42.0);
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
    app.add_plugin(DefaultPlugins);
    app.world_mut().insert_resource(ClockRes::new(Box::new({
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
    assert_close(world.resource::<DeltaTime>().0.0, 0.0);
}
