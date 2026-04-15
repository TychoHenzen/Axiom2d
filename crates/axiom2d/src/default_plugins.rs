#[cfg(feature = "render")]
use crate::splash::splash_render_system;
use crate::splash::{SkipSplash, SplashPlugin};
use engine_app::mouse_world_pos_system::mouse_world_pos_system;
use engine_app::prelude::{App, Phase, Plugin};
#[cfg(feature = "audio")]
use engine_audio::{
    audio_res::AudioRes, backend::NullAudioBackend, playback::PlaySound,
    playback::play_sound_system, spatial::spatial_audio_system,
};
use engine_core::prelude::{ClockRes, SystemClock, time_system};
use engine_ecs::prelude::IntoScheduleConfigs;
use engine_input::prelude::{
    InputState, KeyInputEvent, MouseInputEvent, MouseState, input_system, mouse_input_system,
    scroll_clear_system,
};
#[cfg(feature = "physics")]
use engine_physics::prelude::{
    CollisionEvent, PhysicsCommand, PhysicsRes, physics_command_apply_system, physics_step_system,
    physics_sync_system,
};
#[cfg(feature = "render")]
use engine_render::prelude::{
    ClearColor, ShaderRegistry, camera_prepare_system, clear_system, post_process_system,
    shader_prepare_system, upload_atlas_system,
};
#[cfg(feature = "render")]
use engine_render::shape::mesh_cache_system;
use engine_scene::prelude::{
    hierarchy_maintenance_system, transform_propagation_system, visibility_system,
};

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<SkipSplash>().is_none() {
            app.add_plugin(SplashPlugin);
        } else {
            // splash_render_system requires Res<SplashScreen>. When splash is
            // skipped, insert a done screen so the system early-returns.
            use crate::splash::SplashScreen;
            app.world_mut().insert_resource(SplashScreen {
                elapsed: 0.0,
                duration: 0.0,
                done: true,
            });
        }
        register_core_resources(app);
        register_core_systems(app);
        register_physics(app);
        register_post_update_systems(app);
        register_render(app);
    }
}

fn register_core_resources(app: &mut App) {
    app.world_mut().insert_resource(InputState::default());
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<KeyInputEvent>::default());
    app.world_mut().insert_resource(MouseState::default());
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<MouseInputEvent>::default());
    app.world_mut()
        .insert_resource(ClockRes::new(Box::new(SystemClock::default())));
}

fn register_core_systems(app: &mut App) {
    app.add_systems(Phase::Input, (input_system, mouse_input_system));
    app.add_systems(Phase::Input, time_system);
}

fn register_physics(app: &mut App) {
    #[cfg(feature = "physics")]
    {
        if app.world().get_resource::<PhysicsRes>().is_none() {
            app.world_mut().insert_resource(PhysicsRes::new(Box::new(
                engine_physics::prelude::NullPhysicsBackend::default(),
            )));
        }
        app.world_mut()
            .insert_resource(engine_core::prelude::EventBus::<CollisionEvent>::default());
        app.world_mut()
            .insert_resource(engine_core::prelude::EventBus::<PhysicsCommand>::default());
        app.add_systems(
            Phase::FixedUpdate,
            (
                physics_command_apply_system,
                physics_step_system,
                physics_sync_system,
            )
                .chain(),
        );
    }
}

fn register_post_update_systems(app: &mut App) {
    #[cfg(feature = "audio")]
    {
        app.world_mut()
            .insert_resource(AudioRes::new(Box::new(NullAudioBackend::new())));
        app.world_mut()
            .insert_resource(engine_core::prelude::EventBus::<PlaySound>::default());
        app.add_systems(Phase::LateUpdate, play_sound_system);
    }

    #[cfg(not(feature = "audio"))]
    app.add_systems(
        Phase::LateUpdate,
        (
            hierarchy_maintenance_system,
            transform_propagation_system,
            visibility_system,
            mouse_world_pos_system,
            scroll_clear_system,
        )
            .chain(),
    );

    #[cfg(feature = "audio")]
    app.add_systems(
        Phase::LateUpdate,
        (
            hierarchy_maintenance_system,
            transform_propagation_system,
            visibility_system,
            spatial_audio_system,
            mouse_world_pos_system,
            scroll_clear_system,
        )
            .chain(),
    );
}

fn register_render(app: &mut App) {
    #[cfg(feature = "render")]
    {
        app.world_mut().insert_resource(ClearColor::default());
        app.world_mut().insert_resource(ShaderRegistry::default());
        app.world_mut()
            .insert_resource(engine_ui::draw_command::DrawQueue::default());
        app.add_systems(Phase::LateUpdate, mesh_cache_system);
        app.add_systems(
            Phase::Render,
            (
                clear_system,
                upload_atlas_system,
                camera_prepare_system,
                shader_prepare_system,
                splash_render_system,
            )
                .chain(),
        );
        app.add_systems(
            Phase::PostRender,
            (
                engine_ui::unified_render::unified_render_system,
                post_process_system,
            )
                .chain(),
        );
    }
}
