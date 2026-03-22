use bevy_ecs::prelude::Component;

/// Marker for cards in stash item-form (rendered as stash grid slots rather
/// than full table cards).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct CardItemForm;
