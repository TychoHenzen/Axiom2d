use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Query, Res, ResMut, Resource};
use engine_input::mouse::MouseState;
use engine_input::prelude::MouseButton;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use crate::is_hidden;
use crate::layout::anchor_offset;
use crate::ui_event::{UiEvent, UiEventBuffer};
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

#[allow(clippy::type_complexity)]
pub fn ui_interaction_system(
    mouse: Res<MouseState>,
    mut query: Query<(
        Entity,
        &UiNode,
        &GlobalTransform2D,
        &mut Interaction,
        Option<&EffectiveVisibility>,
        Option<&Button>,
    )>,
    mut events: ResMut<UiEventBuffer>,
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

fn update_focus(entity: Entity, focus: &mut FocusState, events: &mut UiEventBuffer) {
    let old = focus.focused;
    if old != Some(entity) {
        if let Some(prev) = old {
            events.push(UiEvent::FocusLost(prev));
        }
        events.push(UiEvent::FocusGained(entity));
        focus.focused = Some(entity);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};
    use engine_scene::prelude::GlobalTransform2D;
    use glam::{Affine2, Vec2};

    use crate::layout::Anchor;
    use crate::ui_event::{UiEvent, UiEventBuffer};
    use crate::widget::UiNode;

    fn setup_world(world_pos: Vec2) -> World {
        let mut world = World::new();
        let mut mouse = MouseState::default();
        mouse.set_world_pos(world_pos);
        world.insert_resource(mouse);
        world.insert_resource(UiEventBuffer::default());
        world.insert_resource(FocusState::default());
        world
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(ui_interaction_system);
        schedule.run(world);
    }

    /// @doc: AABB hit-test uses `anchor_offset` to compute top-left from node position + size
    #[test]
    fn when_cursor_inside_node_then_interaction_becomes_hovered() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Hovered);
    }

    #[test]
    fn when_cursor_on_node_boundary_then_interaction_becomes_hovered() {
        // Arrange
        let mut world = setup_world(Vec2::new(200.0, 100.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Hovered);
    }

    #[test]
    fn when_cursor_outside_node_then_interaction_remains_none() {
        // Arrange
        let mut world = setup_world(Vec2::new(0.0, 0.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }

    #[test]
    fn when_cursor_inside_and_left_held_then_interaction_becomes_pressed() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Pressed);
    }

    #[test]
    fn when_cursor_outside_and_left_held_then_interaction_remains_none() {
        // Arrange
        let mut world = setup_world(Vec2::new(0.0, 0.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }

    #[test]
    fn when_node_has_center_anchor_then_hit_test_accounts_for_offset() {
        // Arrange
        let mut world = setup_world(Vec2::new(175.0, 180.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::Center,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 200.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Hovered);
    }

    #[test]
    fn when_effective_visibility_false_then_not_hit_tested() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
                EffectiveVisibility(false),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }

    #[test]
    fn when_two_overlapping_nodes_then_both_receive_interaction() {
        // Arrange
        let mut world = setup_world(Vec2::new(50.0, 50.0));

        let entity_a = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                Interaction::default(),
            ))
            .id();

        let entity_b = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let a = *world.entity(entity_a).get::<Interaction>().unwrap();
        let b = *world.entity(entity_b).get::<Interaction>().unwrap();
        assert_eq!(a, Interaction::Hovered);
        assert_eq!(b, Interaction::Hovered);
    }

    #[test]
    fn when_cursor_leaves_node_then_interaction_reverts_to_none() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        run_system(&mut world);
        assert_eq!(
            *world.entity(entity).get::<Interaction>().unwrap(),
            Interaction::Hovered
        );

        // Act
        world
            .resource_mut::<MouseState>()
            .set_world_pos(Vec2::new(0.0, 0.0));
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }

    #[test]
    fn when_just_pressed_inside_then_clicked_event_emitted() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        {
            let mut mouse = world.resource_mut::<MouseState>();
            mouse.press(MouseButton::Left);
        }

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::Clicked(entity)));
    }

    #[test]
    fn when_cursor_enters_node_then_hover_enter_event_emitted() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::HoverEnter(entity)));
    }

    #[test]
    fn when_cursor_leaves_node_then_hover_exit_event_emitted() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        run_system(&mut world);
        let _ = world.resource_mut::<UiEventBuffer>().drain().count();

        // Act
        world
            .resource_mut::<MouseState>()
            .set_world_pos(Vec2::new(0.0, 0.0));
        run_system(&mut world);

        // Assert
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::HoverExit(entity)));
    }

    /// @doc: Click sets FocusState.focused — only one entity has focus at a time
    #[test]
    fn when_node_clicked_then_focus_state_updated() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let focus = world.resource::<FocusState>();
        assert_eq!(focus.focused, Some(entity));
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::FocusGained(entity)));
    }

    #[test]
    fn when_different_node_clicked_then_focus_transfers() {
        // Arrange
        let mut world = setup_world(Vec2::new(50.0, 50.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity_a = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                Interaction::default(),
            ))
            .id();

        let _ = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 200.0))),
                Interaction::default(),
            ))
            .id();

        run_system(&mut world);
        assert_eq!(world.resource::<FocusState>().focused, Some(entity_a));
        let _ = world.resource_mut::<UiEventBuffer>().drain().count();

        // Act — move cursor to entity_b and click
        {
            let mut mouse = world.resource_mut::<MouseState>();
            mouse.set_world_pos(Vec2::new(250.0, 250.0));
            mouse.clear_frame_state();
            mouse.press(MouseButton::Left);
        }
        run_system(&mut world);

        // Assert
        let focus = world.resource::<FocusState>();
        assert_ne!(focus.focused, Some(entity_a));
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::FocusLost(entity_a)));
    }

    #[test]
    fn when_cursor_inside_x_range_but_outside_y_range_then_not_hovered() {
        // Arrange — node at (200, 100), size 100x50, cursor at (250, 200) → inside x, outside y
        let mut world = setup_world(Vec2::new(250.0, 200.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }

    #[test]
    fn when_cursor_inside_y_range_but_outside_x_range_then_not_hovered() {
        // Arrange — node at (200, 100), size 100x50, cursor at (50, 120) → outside x, inside y
        let mut world = setup_world(Vec2::new(50.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }

    #[test]
    fn when_interaction_roundtrip_ron_then_variant_preserved() {
        // Arrange
        let variants = [
            Interaction::None,
            Interaction::Hovered,
            Interaction::Pressed,
        ];

        for variant in variants {
            // Act
            let ron_str = ron::to_string(&variant).unwrap();
            let restored: Interaction = ron::from_str(&ron_str).unwrap();

            // Assert
            assert_eq!(restored, variant);
        }
    }

    #[test]
    fn when_invisible_entity_with_no_prior_hover_then_no_hover_exit_event() {
        // Arrange — entity starts at Interaction::None, is invisible
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let _ = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::None,
                EffectiveVisibility(false),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert — no HoverExit because prev was already None
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(
            !events.iter().any(|e| matches!(e, UiEvent::HoverExit(_))),
            "should not emit HoverExit when prev=None, got {events:?}"
        );
    }

    /// @doc: Disabled buttons are excluded from hit-testing entirely — not just visually dimmed
    #[test]
    fn when_disabled_button_then_interaction_stays_none() {
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
                crate::widget::Button { disabled: true },
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
    }
}
