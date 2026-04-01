#![allow(clippy::unwrap_used)]

use engine_app::prelude::App;
use engine_physics::prelude::PhysicsRes;
use engine_render::prelude::ShaderRegistry;

use card_game::card::identity::base_type::BaseCardTypeRegistry;
use card_game::card::rendering::art_shader::CardArtShader;
use card_game::hand::Hand;
use card_game::plugin::CardGamePlugin;

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
    use card_game::card::rendering::art_shader::TierShaders;

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
