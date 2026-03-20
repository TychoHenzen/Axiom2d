use crate::card::art_shader::register_card_art_shader;
use crate::card::camera_drag::{CameraDragState, camera_drag_system, camera_zoom_system};
use crate::card::damping::card_damping_system;
use crate::card::drag::card_drag_system;
use crate::card::drag_state::DragState;
use crate::card::flip::card_flip_system;
use crate::card::flip_animation::{flip_animation_system, sync_scale_spring_lock_x};
use crate::card::item_form::card_item_form_visibility_system;
use crate::card::pick::card_pick_system;
use crate::card::release::card_release_system;
use crate::card::render_layer::card_render_layer_system;
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
use engine_render::prelude::{ClearColor, ShaderRegistry, shape_render_system};
use engine_scene::sort_propagation::sort_propagation_system;
use engine_ui::text_render::text_render_system;

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

        let art_shader = register_card_art_shader(&mut world.resource_mut::<ShaderRegistry>());
        world.insert_resource(art_shader);

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
    .add_systems(Phase::Update, stash_hover_preview_system)
    .add_systems(
        Phase::PostUpdate,
        (
            card_item_form_visibility_system,
            stash_layout_system,
            sort_propagation_system,
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
    .add_systems(Phase::Render, text_render_system.after(shape_render_system));
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::art_shader::CardArtShader;
    use engine_physics::prelude::PhysicsRes;

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

    #[test]
    fn when_plugin_built_then_clear_color_is_dark_gray() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());

        // Act
        app.add_plugin(CardGamePlugin);

        // Assert
        let clear = app.world().get_resource::<ClearColor>().unwrap();
        assert_eq!(
            clear.0,
            Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0
            }
        );
    }

    #[test]
    fn when_plugin_built_then_stash_grid_is_10x10_with_3_pages() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(ShaderRegistry::default());

        // Act
        app.add_plugin(CardGamePlugin);

        // Assert
        let grid = app.world().get_resource::<StashGrid>().unwrap();
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 10);
        assert_eq!(grid.page_count(), 3);
    }
}
