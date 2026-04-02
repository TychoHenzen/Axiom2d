#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use bevy_ecs::schedule::IntoScheduleConfigs;
use engine_core::prelude::Transform2D;
use engine_scene::prelude::{ChildOf, GlobalTransform2D, hierarchy_maintenance_system};
use engine_ui::layout::{FlexDirection, FlexLayout, Margin, ui_layout_system};
use engine_ui::widget::UiNode;
use glam::{Affine2, Vec2};

fn run_layout(world: &mut World) {
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
