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
