pub use crate::card::art_shader::{CardArtShader, register_card_art_shader};
pub use crate::card::base_type::{
    BaseCardType, BaseCardTypeRegistry, CardCategory, populate_default_types,
};
pub use crate::card::camera_drag::{CameraDragState, camera_drag_system, camera_zoom_system};
pub use crate::card::card_name::{CardName, generate_card_name};
pub use crate::card::component::Card;
pub use crate::card::damping::card_damping_system;
pub use crate::card::definition::{
    CardAbilities, CardDefinition, CardStats, CardType, Keyword, Rarity, art_descriptor_default,
};
pub use crate::card::drag::card_drag_system;
pub use crate::card::drag_state::DragState;
pub use crate::card::flip::card_flip_system;
pub use crate::card::flip_animation::{flip_animation_system, sync_scale_spring_lock_x};
pub use crate::card::gem_sockets::{
    MAX_GEM_RADIUS, MIN_GEM_RADIUS, aspect_color, gem_border_positions, gem_radius,
};
pub use crate::card::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
pub use crate::card::item_form::card_item_form_visibility_system;
pub use crate::card::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP, card_pick_system};
pub use crate::card::release::card_release_system;
pub use crate::card::render_layer::card_render_layer_system;
pub use crate::card::residual::{ModifierType, ResidualModifier, ResidualStats};
pub use crate::card::signature::{Aspect, CardSignature, Element};
pub use crate::card::signature_profile::{SignatureProfile, Tier};
pub use crate::card::spawn_table_card::spawn_visual_card;
pub use crate::card::visual_params::{CardVisualParams, PATTERN_COUNT, generate_card_visuals};
pub use crate::hand::Hand;
pub use crate::hand::layout::hand_layout_system;
pub use crate::stash::boundary::stash_boundary_system;
pub use crate::stash::grid::StashGrid;
pub use crate::stash::hover::{
    StashHoverPreview, stash_hover_preview_render_system, stash_hover_preview_system,
};
pub use crate::stash::layout::stash_layout_system;
pub use crate::stash::pages::{stash_tab_click_system, stash_tab_render_system};
pub use crate::stash::render::stash_render_system;
pub use crate::stash::toggle::{StashVisible, stash_toggle_system};

pub use crate::plugin::CardGamePlugin;

// Engine re-exports used by the binary crate
pub use engine_core::scale_spring::{ScaleSpring, scale_spring_system};
pub use engine_scene::sort_propagation::hierarchy_sort_system;
pub use engine_ui::unified_render::unified_render_system;
