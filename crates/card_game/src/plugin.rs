use crate::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};
use crate::card::interaction::camera_drag::{
    CameraDragState, camera_drag_system, camera_zoom_system,
};
use crate::card::interaction::damping::card_damping_system;
use crate::card::interaction::drag::card_drag_system;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::flip::card_flip_system;
use crate::card::interaction::flip_animation::{flip_animation_system, sync_scale_spring_lock_x};
use crate::card::interaction::pick::card_pick_system;
use crate::card::interaction::release::card_release_system;
use crate::card::rendering::art_shader::{
    register_card_art_shader, register_gem_shader, register_tier_shaders, register_variant_shaders,
    shader_pointer_system,
};
use crate::card::rendering::baked_render::baked_card_sync_system;
use crate::card::rendering::debug_spawn::{DebugSpawnRng, debug_spawn_system};
use crate::card::rendering::drop_zone_glow::hand_drop_zone_render_system;
use crate::card::rendering::render_layer::card_render_layer_system;
use crate::hand::Hand;
use crate::hand::layout::hand_layout_system;
use crate::stash::boundary::stash_boundary_system;
use crate::stash::grid::StashGrid;
use crate::stash::hover::{
    StashHoverPreview, stash_hover_preview_render_system, stash_hover_preview_system,
};
use crate::stash::layout::stash_layout_system;
use crate::stash::pages::{stash_tab_click_system, stash_tab_render_system};
use crate::stash::render::stash_render_system;
use crate::stash::toggle::{StashVisible, stash_toggle_system};
use bevy_ecs::schedule::IntoScheduleConfigs;
use engine_app::prelude::{App, Phase, Plugin};
use engine_core::prelude::Color;
use engine_core::scale_spring::scale_spring_system;
use engine_physics::prelude::physics_sync_system;
use engine_render::prelude::{
    ClearColor, ShaderRegistry, ShapeRenderDisabled, shape_render_system,
};
use engine_scene::sort_propagation::hierarchy_sort_system;
use engine_ui::unified_render::unified_render_system;

pub struct CardGamePlugin;

impl Plugin for CardGamePlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        world.insert_resource(ClearColor(Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        }));
        world.insert_resource(DragState::default());
        world.insert_resource(CameraDragState::default());
        world.insert_resource(StashVisible::default());
        world.insert_resource(Hand::new(10));
        world.insert_resource(StashGrid::new(10, 10, 3));
        world.insert_resource(StashHoverPreview::default());
        let mut registry = BaseCardTypeRegistry::new();
        populate_default_types(&mut registry);
        world.insert_resource(registry);

        world.insert_resource(DebugSpawnRng::default());
        world.insert_resource(ShapeRenderDisabled);

        {
            let (art_shader, variant_shaders, tier_shaders, gem_shader) = {
                let mut shader_reg = world.resource_mut::<ShaderRegistry>();
                let art = register_card_art_shader(&mut shader_reg);
                let variants = register_variant_shaders(&mut shader_reg);
                let tiers = register_tier_shaders(&mut shader_reg);
                let gem = register_gem_shader(&mut shader_reg);
                (art, variants, tiers, gem)
            };
            world.insert_resource(art_shader);
            world.insert_resource(variant_shaders);
            world.insert_resource(tier_shaders);
            world.insert_resource(gem_shader);
        }

        register_systems(app);
    }
}

fn register_systems(app: &mut App) {
    app.add_systems(
        Phase::PreUpdate,
        card_damping_system.after(physics_sync_system),
    )
    .add_systems(
        Phase::Update,
        (
            card_pick_system,
            card_drag_system,
            stash_boundary_system,
            card_release_system,
            card_flip_system,
            flip_animation_system,
        )
            .chain(),
    )
    .add_systems(Phase::Update, (camera_drag_system, camera_zoom_system))
    .add_systems(Phase::Update, (stash_toggle_system, stash_tab_click_system))
    .add_systems(Phase::Update, debug_spawn_system)
    .add_systems(Phase::Update, stash_hover_preview_system)
    .add_systems(
        Phase::PostUpdate,
        (
            baked_card_sync_system,
            shader_pointer_system,
            stash_layout_system,
            hierarchy_sort_system,
            card_render_layer_system,
            hand_layout_system,
        ),
    )
    .add_systems(
        Phase::PostUpdate,
        (sync_scale_spring_lock_x, scale_spring_system).chain(),
    )
    .add_systems(
        Phase::Render,
        stash_render_system.after(shape_render_system),
    )
    .add_systems(
        Phase::Render,
        (stash_tab_render_system, stash_hover_preview_render_system).after(stash_render_system),
    )
    .add_systems(
        Phase::Render,
        unified_render_system.after(shape_render_system),
    )
    .add_systems(
        Phase::Render,
        hand_drop_zone_render_system.after(shape_render_system),
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::rendering::art_shader::CardArtShader;
    use engine_physics::prelude::PhysicsRes;

    /// @doc: Hand size is capped at 10 — the fan layout algorithm and card
    /// spacing assume a bounded hand. An uncapped hand would cause cards to
    /// overlap or shrink below readable size in the fan arc.
    #[test]
    fn when_plugin_built_then_hand_max_size_is_10() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());
        app.add_plugin(CardGamePlugin);

        // Act
        let mut hand = app.world_mut().remove_resource::<Hand>().unwrap();
        for _ in 0..10 {
            let entity = app.world_mut().spawn_empty().id();
            hand.add(entity).unwrap();
        }

        // Assert
        assert!(hand.is_full());
    }

    /// @doc: `CardGamePlugin` does not insert `PhysicsRes` — it expects the
    /// engine-level `DefaultPlugins` to provide the physics backend. If the
    /// card game inserted its own, it would overwrite the engine's rapier
    /// backend with a default (null) one, silently disabling all physics.
    #[test]
    fn when_plugin_built_then_physics_res_not_inserted() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());

        // Act
        app.add_plugin(CardGamePlugin);

        // Assert
        assert!(app.world().get_resource::<PhysicsRes>().is_none());
    }

    /// @doc: The plugin pre-populates the `BaseTypeRegistry` with the five
    /// default archetypes. Without this, all cards would have `archetype: None`
    /// and generic names, since `best_match` returns `None` on an empty registry.
    #[test]
    fn when_plugin_built_then_registry_has_default_base_types() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());

        // Act
        app.add_plugin(CardGamePlugin);

        // Assert
        let registry = app.world().get_resource::<BaseCardTypeRegistry>().unwrap();
        assert!(!registry.is_empty());
    }

    /// @doc: `TierShaders` must be registered at plugin startup so tier condition
    /// overlays (worn/shiny) appear on spawned cards. Without this resource,
    /// `build_mesh_overlays` silently skips the tier overlay, making all cards
    /// look Active regardless of their actual intensity tier.
    #[test]
    fn when_plugin_built_then_tier_shaders_resource_is_present_and_resolves() {
        use crate::card::rendering::art_shader::TierShaders;

        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());

        // Act
        app.add_plugin(CardGamePlugin);

        // Assert
        let tiers = app
            .world()
            .get_resource::<TierShaders>()
            .expect("TierShaders resource should be inserted by plugin");
        let registry = app.world().get_resource::<ShaderRegistry>().unwrap();
        assert!(
            registry.lookup(tiers.dormant).is_some(),
            "dormant shader handle should resolve in registry"
        );
        assert!(
            registry.lookup(tiers.intense).is_some(),
            "intense shader handle should resolve in registry"
        );
    }

    #[test]
    fn when_plugin_built_then_card_art_shader_resolves_in_registry() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());

        // Act
        app.add_plugin(CardGamePlugin);

        // Assert
        let shader = app.world().get_resource::<CardArtShader>().unwrap();
        let registry = app.world().get_resource::<ShaderRegistry>().unwrap();
        assert!(registry.lookup(shader.0).is_some());
    }
}
