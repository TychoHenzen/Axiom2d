use bevy_ecs::prelude::{
    Component, Entity, Query, Res, ResMut, Resource, Trigger, With, Without, World,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::prelude::{LocalSortOrder, SpawnChildExt};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, ClickedEntity};
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, on_socket_clicked};
use crate::card::reader::SignatureSpace;

const BODY_HALF_W: f32 = 40.0;
const BODY_HALF_H: f32 = 30.0;
const BODY_CORNER_RADIUS: f32 = 4.0;

const BODY_FILL: Color = Color {
    r: 0.12,
    g: 0.10,
    b: 0.16,
    a: 1.0,
};
const BODY_STROKE: Color = Color {
    r: 0.50,
    g: 0.35,
    b: 0.65,
    a: 1.0,
};
const SOCKET_COLOR: Color = Color {
    r: 0.4,
    g: 0.7,
    b: 0.9,
    a: 1.0,
};
const MERGE_LINE_COLOR: Color = Color {
    r: 0.35,
    g: 0.25,
    b: 0.55,
    a: 0.6,
};
const SOCKET_RADIUS: f32 = 8.0;
const SOCKET_SPACING: f32 = 20.0;
const COMBINER_LOCAL_SORT: i32 = -1;
const COMBINER_SOCKET_LOCAL_SORT: i32 = 1;
const COMBINER_DECOR_LOCAL_SORT: i32 = 0;

const COMBINER_HALF_EXTENTS: Vec2 = Vec2::new(BODY_HALF_W, BODY_HALF_H);
const INPUT_X: f32 = -(BODY_HALF_W + SOCKET_RADIUS + 4.0);
const OUTPUT_X: f32 = BODY_HALF_W + SOCKET_RADIUS + 4.0;

#[derive(Component, Debug)]
pub struct CombinerDevice {
    pub input_a: Entity,
    pub input_b: Entity,
    pub output: Entity,
}

pub fn combiner_system(
    devices: Query<&CombinerDevice>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
) {
    let updates: Vec<(Entity, Option<SignatureSpace>)> = devices
        .iter()
        .filter_map(|device| {
            let data_a = jacks.get(device.input_a).ok()?.data.clone();
            let data_b = jacks.get(device.input_b).ok()?.data.clone();

            let combined = match (data_a, data_b) {
                (None, None) => None,
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (Some(a), Some(b)) => Some(SignatureSpace::combine(&a, &b)),
            };
            Some((device.output, combined))
        })
        .collect();

    for (output, data) in updates {
        if let Ok(mut jack) = jacks.get_mut(output) {
            jack.data = data;
        }
    }
}

fn spawn_socket(world: &mut World, position: Vec2, direction: JackDirection) -> Entity {
    world
        .spawn((
            Jack::<SignatureSpace> {
                direction,
                data: None,
            },
            JackSocket {
                radius: SOCKET_RADIUS,
                color: SOCKET_COLOR,
                connected_cable: None,
            },
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Shape {
                variant: ShapeVariant::Circle {
                    radius: SOCKET_RADIUS,
                },
                color: SOCKET_COLOR,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(COMBINER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id()
}

fn s_curve_points() -> Vec<Vec2> {
    let [(_, left_top), (_, left_bot), (_, right)] = socket_layout();
    let left_top = left_top + Vec2::new(SOCKET_RADIUS, 0.0);
    let left_bot = left_bot + Vec2::new(SOCKET_RADIUS, 0.0);
    let right = right - Vec2::new(SOCKET_RADIUS, 0.0);
    let mid_x = (left_top.x + right.x) * 0.5;
    vec![
        left_top,
        Vec2::new(mid_x, SOCKET_SPACING * 0.25),
        Vec2::new(mid_x, 0.0),
        right,
        Vec2::new(mid_x, 0.0),
        Vec2::new(mid_x, -SOCKET_SPACING * 0.25),
        left_bot,
    ]
}

fn socket_layout() -> [(JackDirection, Vec2); 3] {
    [
        (
            JackDirection::Input,
            Vec2::new(INPUT_X, SOCKET_SPACING * 0.5),
        ),
        (
            JackDirection::Input,
            Vec2::new(INPUT_X, -SOCKET_SPACING * 0.5),
        ),
        (JackDirection::Output, Vec2::new(OUTPUT_X, 0.0)),
    ]
}

pub fn spawn_combiner_device(
    world: &mut World,
    position: Vec2,
) -> (Entity, Entity, Entity, Entity) {
    let [(_, input_a_offset), (_, input_b_offset), (_, output_offset)] = socket_layout();
    let input_a = spawn_socket(world, position + input_a_offset, JackDirection::Input);
    let input_b = spawn_socket(world, position + input_b_offset, JackDirection::Input);
    let output = spawn_socket(world, position + output_offset, JackDirection::Output);

    let device_entity = world
        .spawn((
            CombinerDevice {
                input_a,
                input_b,
                output,
            },
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Shape {
                variant: rounded_rect_path(BODY_HALF_W, BODY_HALF_H, BODY_CORNER_RADIUS),
                color: BODY_FILL,
            },
            Stroke {
                color: BODY_STROKE,
                width: 2.0,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(COMBINER_LOCAL_SORT),
            Clickable(ClickHitShape::Aabb(COMBINER_HALF_EXTENTS)),
            CableCollider::from_aabb(COMBINER_HALF_EXTENTS),
        ))
        .id();
    world.entity_mut(device_entity).observe(on_combiner_clicked);
    world.entity_mut(input_a).observe(on_socket_clicked);
    world.entity_mut(input_b).observe(on_socket_clicked);
    world.entity_mut(output).observe(on_socket_clicked);

    // Decorative S-curve child
    world.spawn_child(
        device_entity,
        (
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: s_curve_points(),
                },
                color: MERGE_LINE_COLOR,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(COMBINER_DECOR_LOCAL_SORT),
        ),
    );

    (device_entity, input_a, input_b, output)
}

fn socket_entities(device: &CombinerDevice) -> [(Entity, Vec2); 3] {
    let [(_, input_a_offset), (_, input_b_offset), (_, output_offset)] = socket_layout();
    [
        (device.input_a, input_a_offset),
        (device.input_b, input_b_offset),
        (device.output, output_offset),
    ]
}

#[derive(Resource, Debug, Default)]
pub struct CombinerDragState {
    pub dragging: Option<CombinerDragInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombinerDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
}

pub fn on_combiner_clicked(
    trigger: Trigger<ClickedEntity>,
    combiners: Query<&Transform2D, With<CombinerDevice>>,
    mut drag: ResMut<CombinerDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = combiners.get(entity) else {
        return;
    };
    drag.dragging = Some(CombinerDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}

pub fn combiner_drag_system(
    mouse: Res<MouseState>,
    drag: Res<CombinerDragState>,
    mut combiner_transforms: Query<&mut Transform2D, With<CombinerDevice>>,
    mut other_transforms: Query<&mut Transform2D, Without<CombinerDevice>>,
    combiners: Query<&CombinerDevice>,
) {
    let Some(info) = &drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = combiner_transforms.get_mut(info.entity) {
        transform.position = target;
    }
    if let Ok(device) = combiners.get(info.entity) {
        for (jack_entity, offset) in socket_entities(device) {
            if let Ok(mut jack_t) = other_transforms.get_mut(jack_entity) {
                jack_t.position = target + offset;
            }
        }
    }
}

pub fn combiner_release_system(mouse: Res<MouseState>, mut drag: ResMut<CombinerDragState>) {
    if drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        drag.dragging = None;
    }
}
