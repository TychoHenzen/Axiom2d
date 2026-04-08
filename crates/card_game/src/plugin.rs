use crate::booster::device::{
    BoosterDragState, SealButtonPressed, booster_drag_system, booster_release_system,
    booster_seal_system,
};
use crate::booster::double_click::{DoubleClickState, double_click_detect_system};
use crate::booster::opening::booster_opening_system;
use crate::card::combiner_device::{
    CombinerDragState, combiner_drag_system, combiner_release_system, combiner_system,
};
use crate::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};
use crate::card::interaction::apply::interaction_apply_system;
use crate::card::interaction::camera_drag::{
    CameraDragState, camera_drag_system, camera_zoom_system,
};
use crate::card::interaction::click_resolve::click_resolve_system;
use crate::card::interaction::damping::card_damping_system;
use crate::card::interaction::drag::card_drag_system;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::flip::card_flip_system;
use crate::card::interaction::flip_animation::{flip_animation_system, sync_scale_spring_lock_x};
use crate::card::interaction::intent::InteractionIntent;
use crate::card::interaction::release::card_release_system;
use crate::card::jack_cable::{
    signature_space_propagation_system, wire_render_system, wrap_detect_system, wrap_update_system,
};
use crate::card::jack_socket::{
    PendingCable, jack_socket_release_system, jack_socket_render_system, pending_cable_drag_system,
};
use crate::card::reader::{
    ReaderDragState, card_reader_eject_system, card_reader_insert_system, reader_drag_system,
    reader_glow_system, reader_release_system, reader_rotation_lock_system,
};
use crate::card::rendering::art_shader::{
    register_card_art_shader, register_gem_shader, register_tier_shaders, register_variant_shaders,
    shader_pointer_system,
};
use crate::card::rendering::baked_render::baked_card_render_system;
use crate::card::rendering::debug_spawn::{DebugSpawnRng, debug_spawn_system};
use crate::card::rendering::drop_zone_glow::hand_drop_zone_render_system;
use crate::card::rendering::render_layer::card_render_layer_system;
use crate::card::screen_device::{
    ScreenDragState, screen_drag_system, screen_release_system, screen_render_system,
};
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
use crate::stash::store::{StoreCatalog, StoreWallet, store_buy_system, store_sell_system};
use crate::stash::toggle::{StashVisible, stash_toggle_system};
use bevy_ecs::schedule::IntoScheduleConfigs;
use engine_app::prelude::{App, Phase, Plugin};
use engine_core::prelude::{Color, EventBus};
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
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(CameraDragState::default());
        world.insert_resource(StashVisible::default());
        world.insert_resource(Hand::new(10));
        world.insert_resource(StashGrid::new(10, 10, 3));
        world.insert_resource(StashHoverPreview::default());
        world.insert_resource(StoreWallet::default());
        world.insert_resource(StoreCatalog::default());
        world.insert_resource(DebugSpawnRng::default());
        world.insert_resource(PendingCable::default());
        world.insert_resource(ScreenDragState::default());
        world.insert_resource(CombinerDragState::default());
        world.insert_resource(BoosterDragState::default());
        world.insert_resource(DoubleClickState::default());
        world.insert_resource(SealButtonPressed::default());
        world.insert_resource(EventBus::<InteractionIntent>::default());
        let mut registry = BaseCardTypeRegistry::new();
        populate_default_types(&mut registry);
        world.insert_resource(registry);

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
        app.add_systems(Phase::Update, debug_spawn_system);
    }
}

fn register_systems(app: &mut App) {
    app.add_systems(Phase::Input, click_resolve_system)
        .add_systems(
            Phase::Input,
            double_click_detect_system.after(click_resolve_system),
        )
        .add_systems(
            Phase::FixedUpdate,
            (
                card_damping_system.after(physics_sync_system),
                reader_rotation_lock_system.after(physics_sync_system),
            ),
        )
        .add_systems(
            Phase::Update,
            (
                store_buy_system,
                card_reader_eject_system,
                card_drag_system,
                reader_drag_system,
                screen_drag_system,
                combiner_drag_system,
                booster_drag_system,
                store_sell_system,
                stash_boundary_system,
                card_reader_insert_system,
                card_release_system,
                interaction_apply_system,
                reader_release_system,
                screen_release_system,
                combiner_release_system,
                booster_release_system,
                jack_socket_release_system,
                card_flip_system,
                flip_animation_system,
                booster_seal_system,
            )
                .chain(),
        )
        .add_systems(
            Phase::Update,
            (
                pending_cable_drag_system,
                wrap_update_system,
                wrap_detect_system,
                wire_render_system,
                signature_space_propagation_system,
                combiner_system,
                jack_socket_render_system,
                screen_render_system,
            )
                .chain()
                .after(jack_socket_release_system),
        )
        .add_systems(Phase::Update, (camera_drag_system, camera_zoom_system))
        .add_systems(Phase::Update, (stash_toggle_system, stash_tab_click_system))
        .add_systems(Phase::Update, stash_hover_preview_system)
        .add_systems(Phase::Animate, booster_opening_system)
        .add_systems(
            Phase::LateUpdate,
            (
                shader_pointer_system,
                stash_layout_system,
                hierarchy_sort_system,
                card_render_layer_system,
                hand_layout_system,
                reader_glow_system,
            ),
        )
        .add_systems(
            Phase::LateUpdate,
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
            baked_card_render_system.after(unified_render_system),
        )
        .add_systems(
            Phase::Render,
            hand_drop_zone_render_system.after(shape_render_system),
        );
}
