use bevy_ecs::prelude::{
    Commands, Component, Entity, Query, Res, ResMut, Resource, With, Without, World,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_scene::prelude::{LocalSortOrder, Visible};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::drag_state::DragState;
use crate::card::jack_cable::{
    CABLE_HALF_THICKNESS, CABLE_LOCAL_SORT, Cable, Jack, JackDirection, cable_visuals,
};
use crate::card::reader::ReaderDragState;
use crate::card::reader::SignatureSpace;
use crate::card::screen_device::ScreenDragState;

const SOCKET_LOCAL_SORT: i32 = 1;
const PREVIEW_LOCAL_SORT: i32 = 3;

#[derive(Component, Debug, Clone)]
pub struct JackSocket {
    pub radius: f32,
    pub color: Color,
}

#[derive(Component)]
pub struct PendingCablePreview;

pub(crate) fn spawn_pending_cable_preview(world: &mut World) -> Entity {
    world
        .spawn((
            PendingCablePreview,
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![
                        Vec2::new(-CABLE_HALF_THICKNESS, -CABLE_HALF_THICKNESS),
                        Vec2::new(CABLE_HALF_THICKNESS, -CABLE_HALF_THICKNESS),
                        Vec2::new(CABLE_HALF_THICKNESS, CABLE_HALF_THICKNESS),
                        Vec2::new(-CABLE_HALF_THICKNESS, CABLE_HALF_THICKNESS),
                    ],
                },
                color: Color::WHITE,
            },
            Visible(false),
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(PREVIEW_LOCAL_SORT),
        ))
        .id()
}

#[derive(Resource, Debug, Default)]
pub struct PendingCable {
    pub source: Option<Entity>,
}

pub fn jack_socket_render_system(
    sockets: Query<(
        Entity,
        &JackSocket,
        Option<&Shape>,
        Option<&RenderLayer>,
        Option<&SortOrder>,
        Option<&LocalSortOrder>,
        Option<&Visible>,
    )>,
    mut commands: Commands,
) {
    for (entity, socket, shape, layer, sort, local_sort, visible) in &sockets {
        let expected_shape = Shape {
            variant: ShapeVariant::Circle {
                radius: socket.radius,
            },
            color: socket.color,
        };
        let mut entity_commands = commands.entity(entity);
        if shape != Some(&expected_shape) {
            entity_commands.insert(expected_shape.clone());
        }
        if layer.is_none() {
            entity_commands.insert(RenderLayer::World);
        }
        if sort.is_none() {
            entity_commands.insert(SortOrder::default());
        }
        if local_sort.is_none() {
            entity_commands.insert(LocalSortOrder(SOCKET_LOCAL_SORT));
        }
        if visible.is_none() {
            entity_commands.insert(Visible(true));
        }
    }
}

pub fn jack_socket_pick_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    reader_drag: Res<ReaderDragState>,
    screen_drag: Res<ScreenDragState>,
    mut pending: ResMut<PendingCable>,
    sockets: Query<(Entity, &JackSocket, &Transform2D)>,
) {
    if drag_state.dragging.is_some()
        || reader_drag.dragging.is_some()
        || screen_drag.dragging.is_some()
        || pending.source.is_some()
    {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let cursor = mouse.world_pos();
    for (entity, socket, transform) in &sockets {
        if (cursor - transform.position).length() <= socket.radius {
            pending.source = Some(entity);
            return;
        }
    }
}

pub fn pending_cable_drag_system(
    mouse: Res<MouseState>,
    pending: Res<PendingCable>,
    transforms: Query<&Transform2D, Without<PendingCablePreview>>,
    mut preview: Query<(&mut Transform2D, &mut Shape, &mut Visible), With<PendingCablePreview>>,
) {
    let Ok((mut preview_transform, mut preview_shape, mut preview_visible)) = preview.single_mut()
    else {
        return;
    };

    let Some(source_entity) = pending.source else {
        preview_visible.0 = false;
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        preview_visible.0 = false;
        return;
    }
    let Ok(src_t) = transforms.get(source_entity) else {
        preview_visible.0 = false;
        return;
    };

    let (next_transform, mut next_shape) = cable_visuals(src_t.position, mouse.world_pos());
    next_shape.color = Color::WHITE;
    *preview_transform = next_transform;
    *preview_shape = next_shape;
    preview_visible.0 = true;
}

pub fn jack_socket_release_system(
    mouse: Res<MouseState>,
    mut pending: ResMut<PendingCable>,
    sockets: Query<(Entity, &JackSocket, &Transform2D)>,
    jacks: Query<&Jack<SignatureSpace>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let source_entity = match pending.source.take() {
        Some(e) => e,
        None => return,
    };
    let Ok((_, _, source_transform)) = sockets.get(source_entity) else {
        return;
    };
    let cursor = mouse.world_pos();
    for (entity, socket, transform) in &sockets {
        if entity == source_entity {
            continue;
        }
        if (cursor - transform.position).length() > socket.radius {
            continue;
        }
        let Ok(source_jack) = jacks.get(source_entity) else {
            continue;
        };
        let Ok(dest_jack) = jacks.get(entity) else {
            continue;
        };
        let (output, input) = match (source_jack.direction, dest_jack.direction) {
            (JackDirection::Output, JackDirection::Input) => (source_entity, entity),
            (JackDirection::Input, JackDirection::Output) => (entity, source_entity),
            _ => continue,
        };
        let (cable_transform, cable_shape) =
            cable_visuals(source_transform.position, transform.position);
        commands.spawn((
            Cable {
                source: output,
                dest: input,
            },
            cable_transform,
            cable_shape,
            Visible(true),
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(CABLE_LOCAL_SORT),
        ));
        return;
    }
}
