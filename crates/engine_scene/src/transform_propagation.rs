use bevy_ecs::prelude::*;
use engine_core::prelude::Transform2D;
use glam::Affine2;

use crate::hierarchy::{ChildOf, Children};

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform2D(pub Affine2);

pub fn transform_propagation_system(
    roots: Query<(Entity, &Transform2D), Without<ChildOf>>,
    children_query: Query<&Children>,
    transforms: Query<&Transform2D>,
    mut commands: Commands,
) {
    for (entity, transform) in &roots {
        let global = GlobalTransform2D(transform.to_affine2());
        commands.entity(entity).insert(global);
        propagate_to_children(entity, &global, &children_query, &transforms, &mut commands);
    }
}

fn propagate_to_children(
    parent: Entity,
    parent_global: &GlobalTransform2D,
    children_query: &Query<&Children>,
    transforms: &Query<&Transform2D>,
    commands: &mut Commands,
) {
    if let Ok(children) = children_query.get(parent) {
        for &child in &children.0 {
            if let Ok(local) = transforms.get(child) {
                let global = GlobalTransform2D(parent_global.0 * local.to_affine2());
                commands.entity(child).insert(global);
                propagate_to_children(child, &global, children_query, transforms, commands);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::run_transform_system;
    use engine_core::prelude::Transform2D;

    use crate::hierarchy::{ChildOf, Children};
    use crate::test_helpers::run_hierarchy_system;
    use glam::Vec2;

    #[test]
    fn when_root_entity_has_transform2d_then_propagation_system_inserts_global_transform() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Transform2D::default()).id();

        // Act
        run_transform_system(&mut world);

        // Assert
        assert!(world.get::<GlobalTransform2D>(entity).is_some());
    }

    #[test]
    fn when_root_entity_has_identity_transform_then_global_transform_equals_affine2_identity() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Transform2D::default()).id();

        // Act
        run_transform_system(&mut world);

        // Assert
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert_eq!(global.0, Affine2::IDENTITY);
    }

    #[test]
    fn when_root_entity_has_translation_only_then_global_transform_matches() {
        // Arrange
        let mut world = World::new();
        let t = Transform2D {
            position: Vec2::new(10.0, 20.0),
            ..Transform2D::default()
        };
        let entity = world.spawn(t).id();

        // Act
        run_transform_system(&mut world);

        // Assert
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert!((global.0.translation.x - 10.0).abs() < 1e-6);
        assert!((global.0.translation.y - 20.0).abs() < 1e-6);
    }

    #[test]
    fn when_entity_has_no_transform2d_then_propagation_system_does_not_insert_global_transform() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Act
        run_transform_system(&mut world);

        // Assert
        assert!(world.get::<GlobalTransform2D>(entity).is_none());
    }

    #[test]
    fn when_child_has_identity_transform_then_global_transform_equals_parent() {
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(5.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world.spawn((Transform2D::default(), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 5.0).abs() < 1e-6);
        assert!((child_global.0.translation.y).abs() < 1e-6);
    }

    #[test]
    fn when_child_has_translation_and_parent_has_translation_then_both_accumulate() {
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(10.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(5.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 15.0).abs() < 1e-6);
        assert!((child_global.0.translation.y).abs() < 1e-6);
    }

    #[test]
    fn when_parent_has_scale_and_child_has_translation_then_child_position_is_scaled() {
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                scale: Vec2::splat(2.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(3.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 6.0).abs() < 1e-6);
    }

    #[test]
    fn when_three_level_hierarchy_then_grandchild_accumulates_all_ancestors() {
        // Arrange
        let mut world = World::new();
        let root = world
            .spawn(Transform2D {
                position: Vec2::new(1.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(2.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(root),
            ))
            .id();
        let grandchild = world
            .spawn((
                Transform2D {
                    position: Vec2::new(3.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(child),
            ))
            .id();
        world.entity_mut(root).insert(Children(vec![child]));
        world.entity_mut(child).insert(Children(vec![grandchild]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let grandchild_global = world.get::<GlobalTransform2D>(grandchild).unwrap();
        assert!((grandchild_global.0.translation.x - 6.0).abs() < 1e-6);
    }

    #[test]
    fn when_two_siblings_then_each_gets_independent_global_transform() {
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(10.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child_a = world
            .spawn((
                Transform2D {
                    position: Vec2::new(1.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        let child_b = world
            .spawn((
                Transform2D {
                    position: Vec2::new(0.0, 2.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        world
            .entity_mut(parent)
            .insert(Children(vec![child_a, child_b]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let a_global = world.get::<GlobalTransform2D>(child_a).unwrap();
        assert!((a_global.0.translation.x - 11.0).abs() < 1e-6);
        assert!((a_global.0.translation.y).abs() < 1e-6);
        let b_global = world.get::<GlobalTransform2D>(child_b).unwrap();
        assert!((b_global.0.translation.x - 10.0).abs() < 1e-6);
        assert!((b_global.0.translation.y - 2.0).abs() < 1e-6);
    }

    #[test]
    fn when_multiple_root_entities_then_each_gets_independent_global_transform() {
        // Arrange
        let mut world = World::new();
        let root_a = world
            .spawn(Transform2D {
                position: Vec2::new(5.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let root_b = world
            .spawn(Transform2D {
                position: Vec2::new(0.0, 7.0),
                ..Transform2D::default()
            })
            .id();

        // Act
        run_transform_system(&mut world);

        // Assert
        let a_global = world.get::<GlobalTransform2D>(root_a).unwrap();
        assert!((a_global.0.translation.x - 5.0).abs() < 1e-6);
        let b_global = world.get::<GlobalTransform2D>(root_b).unwrap();
        assert!((b_global.0.translation.y - 7.0).abs() < 1e-6);
    }

    #[test]
    fn when_hierarchy_system_runs_before_propagation_then_children_receive_global_transform() {
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(10.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(5.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();

        // Act
        run_hierarchy_system(&mut world);
        run_transform_system(&mut world);

        // Assert
        let parent_global = world.get::<GlobalTransform2D>(parent).unwrap();
        assert!((parent_global.0.translation.x - 10.0).abs() < 1e-6);
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 15.0).abs() < 1e-6);
    }

    #[test]
    fn when_transform_updated_and_system_reruns_then_global_transform_reflects_new_value() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn(Transform2D {
                position: Vec2::new(1.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        run_transform_system(&mut world);
        world
            .entity_mut(entity)
            .get_mut::<Transform2D>()
            .unwrap()
            .position = Vec2::new(99.0, 0.0);

        // Act
        run_transform_system(&mut world);

        // Assert
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert!((global.0.translation.x - 99.0).abs() < 1e-6);
    }
}
