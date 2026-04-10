// EVOLVE-BLOCK-START
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Query, Res, ResMut, Resource};
use engine_core::prelude::EventBus;
use engine_input::mouse::MouseState;
use engine_input::prelude::MouseButton;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use crate::is_hidden;
use crate::layout::anchor_offset;
use crate::ui_event::UiEvent;
use crate::widget::{Button, UiNode};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Interaction {
    #[default]
    None,
    Hovered,
    Pressed,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FocusState {
    pub focused: Option<Entity>,
}

type InteractionQuery<'w> = (
    Entity,
    &'w UiNode,
    &'w GlobalTransform2D,
    &'w mut Interaction,
    Option<&'w EffectiveVisibility>,
    Option<&'w Button>,
);

pub fn ui_interaction_system(
    mouse: Res<MouseState>,
    mut query: Query<InteractionQuery>,
    mut events: ResMut<EventBus<UiEvent>>,
    mut focus: ResMut<FocusState>,
) {
    let pos = mouse.world_pos();
    for (entity, node, transform, mut interaction, visibility, button) in &mut query {
        let disabled = button.is_some_and(|b| b.disabled);
        let hidden = is_hidden(visibility);
        let prev = *interaction;
        if hidden || disabled || !hit_test(node, transform, pos) {
            *interaction = Interaction::None;
            if !disabled && prev != Interaction::None {
                events.push(UiEvent::HoverExit(entity));
            }
            continue;
        }
        *interaction = compute_interaction(&mouse);
        if prev == Interaction::None {
            events.push(UiEvent::HoverEnter(entity));
        }
        if mouse.just_pressed(MouseButton::Left) {
            events.push(UiEvent::Clicked(entity));
            update_focus(entity, &mut focus, &mut events);
        }
    }
}

fn hit_test(node: &UiNode, transform: &GlobalTransform2D, pos: glam::Vec2) -> bool {
    let offset = anchor_offset(node.anchor, node.size);
    let top_left = transform.0.translation + offset;
    pos.x >= top_left.x
        && pos.x <= top_left.x + node.size.x
        && pos.y >= top_left.y
        && pos.y <= top_left.y + node.size.y
}

fn compute_interaction(mouse: &MouseState) -> Interaction {
    if mouse.pressed(MouseButton::Left) {
        Interaction::Pressed
    } else {
        Interaction::Hovered
    }
}

fn update_focus(entity: Entity, focus: &mut FocusState, events: &mut EventBus<UiEvent>) {
    let old = focus.focused;
    if old != Some(entity) {
        if let Some(prev) = old {
            events.push(UiEvent::FocusLost(prev));
        }
        events.push(UiEvent::FocusGained(entity));
        focus.focused = Some(entity);
    }
}
// EVOLVE-BLOCK-END
