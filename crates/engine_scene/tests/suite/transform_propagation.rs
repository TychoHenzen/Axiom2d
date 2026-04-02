#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::IntoScheduleConfigs;
use engine_core::prelude::Transform2D;
use engine_scene::hierarchy::{ChildOf, Children, hierarchy_maintenance_system};
use engine_scene::transform_propagation::{GlobalTransform2D, transform_propagation_system};
use glam::{Affine2, Vec2};

fn run_hierarchy_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);
}

fn run_transform_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(transform_propagation_system);
    schedule.run(world);
}

#[derive(Resource, Default)]
struct ChangedGlobalTransformCapture(usize);

fn capture_changed_global_transforms(
    mut capture: ResMut<ChangedGlobalTransformCapture>,
    query: Query<Entity, Changed<GlobalTransform2D>>,
) {
    capture.0 = query.iter().count();
}

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

/// @doc: Root entities (no `ChildOf`) copy `Transform2D` directly to `GlobalTransform2D`
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

/// @doc: `GlobalTransform2D` = parent.global * child.local — standard affine composition
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

fn spawn_child_at(world: &mut World, parent: Entity, x: f32, y: f32) -> Entity {
    world
        .spawn((
            Transform2D {
                position: Vec2::new(x, y),
                ..Transform2D::default()
            },
            ChildOf(parent),
        ))
        .id()
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
    let child_a = spawn_child_at(&mut world, parent, 1.0, 0.0);
    let child_b = spawn_child_at(&mut world, parent, 0.0, 2.0);
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

#[test]
fn when_transform_system_reruns_without_changes_then_global_transform_is_not_marked_changed() {
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
    run_hierarchy_system(&mut world);
    world.insert_resource(ChangedGlobalTransformCapture::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            transform_propagation_system,
            capture_changed_global_transforms,
        )
            .chain(),
    );
    schedule.run(&mut world);

    // Act
    schedule.run(&mut world);

    // Assert
    let changed = world.resource::<ChangedGlobalTransformCapture>().0;
    assert!(
        changed == 0,
        "unchanged hierarchy should not rewrite GlobalTransform2D, but {changed} entities were marked changed"
    );

    let parent_global = world.get::<GlobalTransform2D>(parent).unwrap();
    assert!((parent_global.0.translation.x - 10.0).abs() < 1e-6);
    let child_global = world.get::<GlobalTransform2D>(child).unwrap();
    assert!((child_global.0.translation.x - 15.0).abs() < 1e-6);
}
