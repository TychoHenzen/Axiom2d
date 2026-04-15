//! Data-driven zone transition configuration.
//!
//! `ZoneConfig` describes the properties a card should have when in a given zone.
//! This is the recommended approach for new zone-related logic. Existing `drop_on_*`
//! functions in `release.rs` will be migrated incrementally once the pattern is proven.

use engine_scene::prelude::RenderLayer;

use crate::card::component::CardZone;

/// Describes the properties a card should have when in a given zone.
/// Used by reconciliation logic to compute the delta between current
/// state and desired state after a zone transition.
///
/// This is a pure data description — no ECS types, no Commands, no side effects.
pub struct ZoneConfig {
    pub has_physics: bool,
    pub render_layer: RenderLayer,
    pub has_item_form: bool,
}

impl ZoneConfig {
    pub fn for_zone(zone: &CardZone) -> Self {
        match zone {
            CardZone::Table => Self {
                has_physics: true,
                render_layer: RenderLayer::World,
                has_item_form: false,
            },
            CardZone::Hand(_) => Self {
                has_physics: false,
                render_layer: RenderLayer::UI,
                has_item_form: false,
            },
            CardZone::Stash { .. } => Self {
                has_physics: false,
                render_layer: RenderLayer::UI,
                has_item_form: true,
            },
            CardZone::Reader(_) => Self {
                has_physics: false,
                render_layer: RenderLayer::World,
                has_item_form: false,
            },
        }
    }
}
