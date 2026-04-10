// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{
    Commands, Component, Entity, Query, Res, ResMut, Resource, Trigger, With, Without,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_scene::prelude::{LocalSortOrder, Visible};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::click_resolve::ClickedEntity;
use crate::card::jack_cable::{
    CABLE_LOCAL_SORT, Cable, Jack, JackDirection, WireEndpoints, WrapWire,
};
use crate::card::reader::SignatureSpace;

const SOCKET_LOCAL_SORT: i32 = 1;

#[derive(Component, Debug, Clone)]
pub struct JackSocket {
    pub radius: f32,
    pub color: Color,
    pub connected_cable: Option<Entity>,
}

impl JackSocket {
    pub fn is_occupied(&self) -> bool {
        self.connected_cable.is_some()
    }
}

#[derive(Component, Debug)]
pub struct CableFreeEnd;

#[derive(Resource, Debug, Default)]
pub struct PendingCable {
    pub source: Option<Entity>,
    pub origin_cable: Option<Entity>,
    pub free_end: Option<Entity>,
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

/// Observer registered on each `JackSocket` entity at spawn time.
pub fn on_socket_clicked(
    trigger: Trigger<ClickedEntity>,
    mut pending: ResMut<PendingCable>,
    mut sockets: Query<&mut JackSocket>,
    transforms: Query<&Transform2D, Without<CableFreeEnd>>,
    cables: Query<&Cable>,
    mut wire_endpoints: Query<&mut WireEndpoints>,
    mut commands: Commands,
) {
    let clicked = trigger.target();
    let Ok(mut socket) = sockets.get_mut(clicked) else {
        pending.source = Some(clicked);
        return;
    };

    if !socket.is_occupied() {
        pending.source = Some(clicked);
        let pos = transforms
            .get(clicked)
            .map(|t| t.position)
            .unwrap_or(Vec2::ZERO);
        let free_end = commands
            .spawn((
                CableFreeEnd,
                Transform2D {
                    position: pos,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
            ))
            .id();
        let cable_entity = commands
            .spawn((
                WireEndpoints {
                    source: clicked,
                    dest: free_end,
                },
                Transform2D {
                    position: pos,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                Shape {
                    variant: ShapeVariant::Polygon {
                        points: vec![Vec2::ZERO],
                    },
                    color: Color::WHITE,
                },
                Visible(true),
                RenderLayer::World,
                SortOrder::default(),
                LocalSortOrder(CABLE_LOCAL_SORT),
                WrapWire::new(),
            ))
            .id();
        pending.origin_cable = Some(cable_entity);
        pending.free_end = Some(free_end);
        return;
    }

    // Disconnect: pick up the existing cable from this socket
    let Some(cable_entity) = socket.connected_cable.take() else {
        pending.source = Some(clicked);
        return;
    };

    let Ok(cable) = cables.get(cable_entity) else {
        pending.source = Some(clicked);
        return;
    };

    let other_socket = if cable.dest == clicked {
        cable.source
    } else {
        cable.dest
    };

    // Spawn a free end at the clicked socket's position and rewire the rope to it
    let pos = transforms
        .get(clicked)
        .map(|t| t.position)
        .unwrap_or(Vec2::ZERO);
    let free_end = commands
        .spawn((
            CableFreeEnd,
            Transform2D {
                position: pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Rewire the wire endpoint from the clicked socket to the free end
    if let Ok(mut endpoints) = wire_endpoints.get_mut(cable_entity) {
        if endpoints.dest == clicked {
            endpoints.dest = free_end;
        } else {
            endpoints.source = free_end;
        }
    }

    pending.source = Some(other_socket);
    pending.origin_cable = Some(cable_entity);
    pending.free_end = Some(free_end);
}

pub fn pending_cable_drag_system(
    mouse: Res<MouseState>,
    pending: Res<PendingCable>,
    mut free_ends: Query<&mut Transform2D, With<CableFreeEnd>>,
) {
    let Some(free_end_entity) = pending.free_end else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    if let Ok(mut free_t) = free_ends.get_mut(free_end_entity) {
        free_t.position = mouse.world_pos();
    }
    // wire_render_system picks up the new free-end position automatically
}

pub fn jack_socket_release_system(
    mouse: Res<MouseState>,
    mut pending: ResMut<PendingCable>,
    mut sockets: Query<(Entity, &mut JackSocket, &Transform2D)>,
    jacks: Query<&Jack<SignatureSpace>>,
    mut cables: Query<&mut Cable>,
    mut rope_endpoints: Query<&mut WireEndpoints>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(source_entity) = pending.source.take() else {
        pending.origin_cable = None;
        pending.free_end = None;
        return;
    };
    let origin_cable = pending.origin_cable.take();
    let free_end = pending.free_end.take();

    let source_transform = sockets.get(source_entity).map(|(_, _, t)| t.position).ok();
    if source_transform.is_none() {
        if let Some(cable_entity) = origin_cable {
            for (_, mut socket, _) in &mut sockets {
                if socket.connected_cable == Some(cable_entity) {
                    socket.connected_cable = None;
                }
            }
            commands.entity(cable_entity).despawn();
        }
        if let Some(fe) = free_end {
            commands.entity(fe).despawn();
        }
        return;
    }

    let cursor = mouse.world_pos();

    let mut target: Option<(Entity, Entity, Entity)> = None;
    for (entity, socket, transform) in sockets.iter() {
        if entity == source_entity {
            continue;
        }
        if socket.is_occupied() {
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
        target = Some((entity, output, input));
        break;
    }

    let Some((dest_entity, output, input)) = target else {
        // No valid target — despawn the pending rope and free end
        if let Some(cable_entity) = origin_cable {
            // Clear any socket that references this cable
            for (_, mut socket, _) in &mut sockets {
                if socket.connected_cable == Some(cable_entity) {
                    socket.connected_cable = None;
                }
            }
            commands.entity(cable_entity).despawn();
        }
        if let Some(fe) = free_end {
            commands.entity(fe).despawn();
        }
        return;
    };
    // Despawn the free-end cursor tracker
    if let Some(fe) = free_end {
        commands.entity(fe).despawn();
    }

    let Some(cable_entity) = origin_cable else {
        return;
    };
    // Finalize the pending wire: add Cable component, update endpoints
    if let Ok(mut cable) = cables.get_mut(cable_entity) {
        cable.source = output;
        cable.dest = input;
    } else {
        commands.entity(cable_entity).insert(Cable {
            source: output,
            dest: input,
        });
    }
    if let Ok(mut endpoints) = rope_endpoints.get_mut(cable_entity) {
        endpoints.source = output;
        endpoints.dest = input;
    }
    if let Ok((_, mut dest_socket, _)) = sockets.get_mut(dest_entity) {
        dest_socket.connected_cable = Some(cable_entity);
    }
    if let Ok((_, mut src_socket, _)) = sockets.get_mut(source_entity) {
        src_socket.connected_cable = Some(cable_entity);
    }
}
// EVOLVE-BLOCK-END
