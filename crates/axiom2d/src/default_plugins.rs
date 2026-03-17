use crate::splash::SplashPlugin;
#[cfg(feature = "render")]
use crate::splash::splash_render_system;
use engine_app::mouse_world_pos_system::mouse_world_pos_system;
use engine_app::prelude::{App, Phase, Plugin};
#[cfg(feature = "audio")]
use engine_audio::{
    audio_res::AudioRes, backend::NullAudioBackend, playback::PlaySoundBuffer,
    playback::play_sound_system, spatial::spatial_audio_system,
};
use engine_core::prelude::{ClockRes, SystemClock, time_system};
use engine_ecs::prelude::IntoScheduleConfigs;
use engine_input::prelude::{
    InputEventBuffer, InputState, MouseEventBuffer, MouseState, input_system, mouse_input_system,
    scroll_clear_system,
};
#[cfg(feature = "physics")]
use engine_physics::prelude::{
    CollisionEventBuffer, PhysicsRes, physics_step_system, physics_sync_system,
};
#[cfg(feature = "render")]
use engine_render::prelude::{
    ClearColor, ShaderRegistry, camera_prepare_system, clear_system, post_process_system,
    shader_prepare_system, shape_render_system, sprite_render_system, upload_atlas_system,
};
use engine_scene::prelude::{
    hierarchy_maintenance_system, transform_propagation_system, visibility_system,
};

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugin(SplashPlugin);
        register_core_resources(app);
        register_core_systems(app);
        register_physics(app);
        register_post_update_systems(app);
        register_render(app);
    }
}

fn register_core_resources(app: &mut App) {
    app.world_mut().insert_resource(InputState::default());
    app.world_mut().insert_resource(InputEventBuffer::default());
    app.world_mut().insert_resource(MouseState::default());
    app.world_mut().insert_resource(MouseEventBuffer::default());
    app.world_mut()
        .insert_resource(ClockRes::new(Box::new(SystemClock::new())));
}

fn register_core_systems(app: &mut App) {
    app.add_systems(Phase::Input, (input_system, mouse_input_system));
    app.add_systems(Phase::PreUpdate, time_system);
}

fn register_physics(app: &mut App) {
    #[cfg(feature = "physics")]
    {
        app.world_mut().insert_resource(PhysicsRes::new(Box::new(
            engine_physics::prelude::NullPhysicsBackend::new(),
        )));
        app.world_mut()
            .insert_resource(CollisionEventBuffer::default());
        app.add_systems(
            Phase::PreUpdate,
            (physics_step_system, physics_sync_system).chain(),
        );
    }
}

#[allow(clippy::too_many_lines)]
fn register_post_update_systems(app: &mut App) {
    #[cfg(not(feature = "audio"))]
    app.add_systems(
        Phase::PostUpdate,
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
    {
        app.world_mut()
            .insert_resource(AudioRes::new(Box::new(NullAudioBackend::new())));
        app.world_mut().insert_resource(PlaySoundBuffer::default());
        app.add_systems(Phase::PreUpdate, play_sound_system);
        app.add_systems(
            Phase::PostUpdate,
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
}

fn register_render(app: &mut App) {
    #[cfg(feature = "render")]
    {
        app.world_mut().insert_resource(ClearColor::default());
        app.world_mut().insert_resource(ShaderRegistry::default());
        app.add_systems(
            Phase::Render,
            (
                clear_system,
                upload_atlas_system,
                camera_prepare_system,
                shader_prepare_system,
                splash_render_system,
                sprite_render_system,
                shape_render_system,
                post_process_system,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use engine_core::time::FakeClock;
    use engine_input::prelude::KeyCode;
    use winit::event::ElementState;

    fn app_with_default_plugins() -> App {
        let mut app = App::new();
        app.add_plugin(DefaultPlugins);
        app.world_mut()
            .insert_resource(ClockRes::new(Box::new(FakeClock::new())));
        #[cfg(feature = "render")]
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(
                engine_render::prelude::NullRenderer,
            )));
        app
    }

    #[test]
    fn when_key_pressed_and_frame_runs_then_input_state_reflects_key() {
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Pressed);

        // Act
        app.handle_redraw();

        // Assert
        assert!(
            app.world()
                .resource::<InputState>()
                .just_pressed(KeyCode::Space)
        );
    }

    #[test]
    fn when_frame_advanced_with_fake_clock_then_delta_time_is_updated() {
        // Arrange
        let mut app = app_with_default_plugins();
        let mut fake = FakeClock::new();
        fake.advance(engine_core::prelude::Seconds(0.016));
        app.world_mut()
            .insert_resource(ClockRes::new(Box::new(fake)));

        // Act
        app.handle_redraw();

        // Assert
        let dt = app.world().resource::<engine_core::prelude::DeltaTime>();
        assert_eq!(dt.0, engine_core::prelude::Seconds(0.016));
    }

    #[test]
    fn when_child_of_entity_spawned_then_children_component_created_after_frame() {
        // Arrange
        let mut app = app_with_default_plugins();
        let parent = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn(engine_scene::prelude::ChildOf(parent));

        // Act
        app.handle_redraw();

        // Assert
        assert!(
            app.world()
                .get::<engine_scene::prelude::Children>(parent)
                .is_some()
        );
    }

    #[test]
    fn when_entity_has_transform2d_then_global_transform_set_after_frame() {
        use engine_core::prelude::{Transform2D, Vec2};

        // Arrange
        let mut app = app_with_default_plugins();
        let entity = app
            .world_mut()
            .spawn(Transform2D {
                position: Vec2::new(100.0, 200.0),
                ..Default::default()
            })
            .id();

        // Act
        app.handle_redraw();

        // Assert
        let global = app
            .world()
            .get::<engine_scene::prelude::GlobalTransform2D>(entity)
            .expect("GlobalTransform2D should be set");
        assert_eq!(global.0.translation, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn when_entity_has_visible_false_then_effective_visibility_false_after_frame() {
        // Arrange
        let mut app = app_with_default_plugins();
        let entity = app
            .world_mut()
            .spawn(engine_scene::prelude::Visible(false))
            .id();

        // Act
        app.handle_redraw();

        // Assert
        let eff = app
            .world()
            .get::<engine_scene::prelude::EffectiveVisibility>(entity)
            .expect("EffectiveVisibility should be set");
        assert!(!eff.0);
    }

    #[cfg(feature = "render")]
    #[test]
    fn when_renderer_injected_and_frame_runs_then_clear_called() {
        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log));

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.contains(&"clear".to_string()));
    }

    #[cfg(feature = "render")]
    #[test]
    fn when_sprite_entity_exists_and_frame_runs_then_draw_sprite_called() {
        use engine_core::prelude::{Color, Pixels, TextureId};
        use engine_render::prelude::{RendererRes, Sprite};
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log))
            .with_viewport(800, 600);

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));
        app.world_mut().spawn((
            Sprite {
                texture: TextureId(0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::WHITE,
                width: Pixels(32.0),
                height: Pixels(32.0),
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.iter().any(|c| c.starts_with("draw_sprite")));
    }

    #[cfg(feature = "render")]
    #[test]
    fn when_shape_entity_exists_and_frame_runs_then_draw_shape_called() {
        use engine_core::prelude::Color;
        use engine_render::prelude::{RendererRes, Shape, ShapeVariant};
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log))
            .with_viewport(800, 600);

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));
        app.world_mut().spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::WHITE,
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.iter().any(|c| c.starts_with("draw_shape")));
    }

    #[cfg(feature = "render")]
    #[test]
    fn when_atlas_inserted_and_frame_runs_then_upload_atlas_called() {
        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log));

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));
        app.world_mut()
            .insert_resource(engine_render::prelude::TextureAtlas {
                data: vec![255; 4],
                width: 1,
                height: 1,
                lookups: std::collections::HashMap::default(),
            });

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.contains(&"upload_atlas".to_string()));
    }

    #[cfg(feature = "render")]
    #[test]
    fn when_atlas_uploaded_then_draw_sprite_also_called_same_frame() {
        use engine_core::prelude::{Color, Pixels, TextureId};
        use engine_render::prelude::{RendererRes, Sprite};
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log))
            .with_viewport(800, 600);

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));
        app.world_mut()
            .insert_resource(engine_render::prelude::TextureAtlas {
                data: vec![255; 4],
                width: 1,
                height: 1,
                lookups: std::collections::HashMap::default(),
            });
        app.world_mut().spawn((
            Sprite {
                texture: TextureId(0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::WHITE,
                width: Pixels(32.0),
                height: Pixels(32.0),
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        let upload_idx = calls.iter().position(|c| c == "upload_atlas");
        let sprite_idx = calls.iter().position(|c| c == "draw_sprite");
        assert!(upload_idx.is_some(), "upload_atlas should be called");
        assert!(sprite_idx.is_some(), "draw_sprite should be called");
        assert!(
            upload_idx.unwrap() < sprite_idx.unwrap(),
            "upload_atlas should run before draw_sprite"
        );
    }

    #[cfg(feature = "audio")]
    #[test]
    fn when_audio_feature_on_then_play_sound_buffer_is_present() {
        // Arrange
        let app = app_with_default_plugins();

        // Assert
        assert!(
            app.world()
                .get_resource::<engine_audio::playback::PlaySoundBuffer>()
                .is_some()
        );
    }

    #[cfg(feature = "audio")]
    #[test]
    fn when_audio_feature_on_then_audio_res_is_present() {
        // Arrange
        let app = app_with_default_plugins();

        // Assert
        assert!(
            app.world()
                .get_resource::<engine_audio::audio_res::AudioRes>()
                .is_some()
        );
    }

    #[test]
    fn when_mouse_button_pressed_and_frame_runs_then_mouse_state_reflects_button() {
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<engine_input::prelude::MouseEventBuffer>()
            .push(winit::event::MouseButton::Left, ElementState::Pressed);

        // Act
        app.handle_redraw();

        // Assert
        assert!(
            app.world()
                .resource::<engine_input::prelude::MouseState>()
                .just_pressed(winit::event::MouseButton::Left)
        );
    }

    #[test]
    fn when_mouse_button_pressed_and_two_frames_run_then_just_pressed_cleared() {
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<engine_input::prelude::MouseEventBuffer>()
            .push(winit::event::MouseButton::Left, ElementState::Pressed);
        app.handle_redraw();

        // Act
        app.handle_redraw();

        // Assert
        let mouse = app.world().resource::<engine_input::prelude::MouseState>();
        assert!(!mouse.just_pressed(winit::event::MouseButton::Left));
        assert!(mouse.pressed(winit::event::MouseButton::Left));
    }

    #[cfg(feature = "render")]
    #[test]
    fn when_shader_registered_and_frame_runs_then_compile_shader_called() {
        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log));

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));
        app.world_mut()
            .resource_mut::<engine_render::prelude::ShaderRegistry>()
            .register("test_shader_source");

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(
            calls.iter().any(|c| c == "compile_shader"),
            "compile_shader should be called when shaders are registered"
        );
    }

    #[test]
    fn when_key_pressed_and_second_frame_runs_then_just_pressed_is_false() {
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Pressed);
        app.handle_redraw(); // first frame — just_pressed should be true

        // Act
        app.handle_redraw(); // second frame — no new events

        // Assert
        assert!(
            !app.world()
                .resource::<InputState>()
                .just_pressed(KeyCode::Space)
        );
        assert!(app.world().resource::<InputState>().pressed(KeyCode::Space));
    }
}
