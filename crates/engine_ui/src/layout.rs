use bevy_ecs::prelude::Query;
use engine_core::prelude::Transform2D;
use engine_scene::prelude::{Children, GlobalTransform2D};

use crate::flex_layout::{FlexLayout, compute_flex_offsets};
use crate::margin::Margin;
use crate::ui_node::UiNode;

pub fn ui_layout_system(
    parents: Query<(&FlexLayout, &Children, &GlobalTransform2D)>,
    mut children: Query<(&UiNode, &mut Transform2D)>,
) {
    for (layout, child_entities, parent_global) in &parents {
        let child_data: Vec<(glam::Vec2, Margin)> = child_entities
            .0
            .iter()
            .filter_map(|&e| {
                children
                    .get(e)
                    .ok()
                    .map(|(node, _)| (node.size, node.margin))
            })
            .collect();

        let offsets = compute_flex_offsets(layout, &child_data);
        let origin = parent_global.0.translation;

        for (entity, offset) in child_entities.0.iter().zip(offsets) {
            if let Ok((_, mut transform)) = children.get_mut(*entity) {
                transform.position = origin + offset;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};
    use engine_scene::prelude::ChildOf;
    use glam::{Affine2, Vec2};

    use crate::flex_layout::FlexDirection;

    fn run_layout(world: &mut World) {
        use bevy_ecs::schedule::IntoScheduleConfigs;
        use engine_scene::prelude::hierarchy_maintenance_system;

        let mut schedule = Schedule::default();
        schedule.add_systems((hierarchy_maintenance_system, ui_layout_system).chain());
        schedule.run(world);
    }

    fn spawn_flex_parent(
        world: &mut World,
        layout: FlexLayout,
        translation: Vec2,
    ) -> bevy_ecs::entity::Entity {
        world
            .spawn((
                layout,
                Transform2D {
                    position: translation,
                    ..Transform2D::default()
                },
                GlobalTransform2D(Affine2::from_translation(translation)),
            ))
            .id()
    }

    fn spawn_ui_child(
        world: &mut World,
        parent: bevy_ecs::entity::Entity,
        size: Vec2,
        margin: Margin,
    ) -> bevy_ecs::entity::Entity {
        world
            .spawn((
                UiNode {
                    size,
                    margin,
                    ..UiNode::default()
                },
                Transform2D::default(),
                ChildOf(parent),
            ))
            .id()
    }

    #[test]
    fn when_row_layout_then_first_child_at_origin() {
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        let child_a = spawn_ui_child(&mut world, parent, Vec2::new(60.0, 30.0), Margin::default());
        spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_a).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::ZERO);
    }

    #[test]
    fn when_row_layout_then_second_child_offset_by_first_width() {
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(&mut world, parent, Vec2::new(60.0, 30.0), Margin::default());
        let child_b = spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(60.0, 0.0));
    }

    #[test]
    fn when_row_layout_with_gap_then_gap_included() {
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 10.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(&mut world, parent, Vec2::new(60.0, 30.0), Margin::default());
        let child_b = spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(70.0, 0.0));
    }

    #[test]
    fn when_column_layout_then_vertical_stacking() {
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Column,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(&mut world, parent, Vec2::new(50.0, 30.0), Margin::default());
        let child_b = spawn_ui_child(&mut world, parent, Vec2::new(50.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(0.0, 30.0));
    }

    #[test]
    fn when_parent_offset_then_children_relative() {
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::new(200.0, 100.0),
        );
        let child = spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(200.0, 100.0));
    }

    #[test]
    fn when_child_has_margin_then_margin_in_spacing() {
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(
            &mut world,
            parent,
            Vec2::new(40.0, 20.0),
            Margin {
                right: 8.0,
                ..Margin::default()
            },
        );
        let child_b = spawn_ui_child(
            &mut world,
            parent,
            Vec2::new(40.0, 20.0),
            Margin {
                left: 4.0,
                ..Margin::default()
            },
        );

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(52.0, 0.0));
    }
}
