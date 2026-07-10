use bevy_ecs::prelude::{
    Component, Entity, Query, Res, ResMut, Resource, Trigger, With, Without, World,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rect_polygon, rounded_rect_path};
use engine_scene::prelude::{ChildOf, LocalSortOrder, SpawnChildExt, Visible};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::identity::signature::Element;
use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, ClickedEntity};
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, on_socket_clicked};
use crate::card::reader::SignatureSpace;
use crate::card::screen_geometry::{build_signal_polyline, clipped_signal_circle};

const DISPLAY_COUNT: usize = 4;
const PANEL_HALF: f32 = 50.0;
const PANEL_SPACING: f32 = 110.0;

const PANEL_OFFSETS: [(f32, f32); DISPLAY_COUNT] = [
    (-PANEL_SPACING * 0.5, -PANEL_SPACING * 0.5),
    (PANEL_SPACING * 0.5, -PANEL_SPACING * 0.5),
    (-PANEL_SPACING * 0.5, PANEL_SPACING * 0.5),
    (PANEL_SPACING * 0.5, PANEL_SPACING * 0.5),
];

// Body dimensions — large enough to contain the 2×2 panel grid.
const BODY_HALF_W: f32 = PANEL_SPACING * 0.5 + PANEL_HALF + 10.0;
const BODY_HALF_H: f32 = PANEL_SPACING * 0.5 + PANEL_HALF + 10.0;
const BODY_CORNER_RADIUS: f32 = 6.0;

const BODY_FILL: Color = Color {
    r: 0.10,
    g: 0.12,
    b: 0.16,
    a: 1.0,
};
const BODY_STROKE: Color = Color {
    r: 0.30,
    g: 0.45,
    b: 0.65,
    a: 1.0,
};
const PANEL_FILL: Color = Color {
    r: 0.05,
    g: 0.07,
    b: 0.10,
    a: 1.0,
};
const SIGNAL_COLOR: Color = Color {
    r: 0.4,
    g: 0.9,
    b: 0.4,
    a: 1.0,
};
const SOCKET_COLOR: Color = Color {
    r: 0.4,
    g: 0.7,
    b: 0.9,
    a: 1.0,
};
const SOCKET_RADIUS: f32 = 8.0;
const JACK_OFFSET: Vec2 = Vec2::new(-(BODY_HALF_W + SOCKET_RADIUS + 4.0), 0.0);
const SCREEN_LOCAL_SORT: i32 = -1;
const SCREEN_PANEL_LOCAL_SORT: i32 = 1;
const SCREEN_DOT_LOCAL_SORT: i32 = 2;
const SCREEN_SOCKET_LOCAL_SORT: i32 = 1;

#[derive(Component)]
pub struct ScreenDevice {
    pub signature_input: Entity,
}

#[derive(Component)]
pub struct ScreenSignalShape {
    display_index: usize,
}

/// Project all control points of a signal into panel space using the selected axis pair.
pub(crate) fn project_signal_points(space: &SignatureSpace, display_index: usize) -> Vec<Vec2> {
    let (x_element, y_element) = panel_axes(display_index);
    let mut projected = Vec::with_capacity(space.control_points.len());
    for cp in &space.control_points {
        projected.push(Vec2::new(
            cp[x_element] * PANEL_HALF,
            cp[y_element] * PANEL_HALF,
        ));
    }
    projected
}

pub fn display_axes(space: &SignatureSpace, display_index: usize) -> (f32, f32) {
    let (x_element, y_element) = panel_axes(display_index);
    (
        space.control_points[0][x_element],
        space.control_points[0][y_element],
    )
}

fn panel_axes(display_index: usize) -> (Element, Element) {
    let x_element = Element::ALL[display_index * 2];
    let y_element = Element::ALL[display_index * 2 + 1];
    (x_element, y_element)
}

fn panel_offset(display_index: usize) -> Vec2 {
    let (x, y) = PANEL_OFFSETS[display_index];
    Vec2::new(x, y)
}

pub fn screen_render_system(
    devices: Query<&ScreenDevice>,
    jacks: Query<&Jack<SignatureSpace>>,
    mut shapes: Query<(&ScreenSignalShape, &ChildOf, &mut Shape, &mut Visible)>,
) {
    for (signal_shape, parent, mut shape, mut visible) in &mut shapes {
        let Ok(device) = devices.get(parent.0) else {
            visible.0 = false;
            continue;
        };
        let Ok(jack) = jacks.get(device.signature_input) else {
            visible.0 = false;
            continue;
        };
        let Some(space) = jack.data.as_ref() else {
            visible.0 = false;
            continue;
        };

        let projected = project_signal_points(space, signal_shape.display_index);
        let visual_radius = space.radius * PANEL_HALF;

        if projected.len() == 1 {
            shape.variant = clipped_signal_circle(projected[0], visual_radius);
        } else {
            shape.variant = build_signal_polyline(&projected, visual_radius);
        }
        shape.color = SIGNAL_COLOR;
        visible.0 = true;
    }
}

/// Spawns a screen device at `position`.
///
/// The device and its child panel/dot entities render through the unified
/// shape pipeline. `screen_render_system` only updates the dot visuals.
///
/// Returns `(device_entity, jack_entity)`.
pub fn spawn_screen_device(world: &mut World, position: Vec2) -> (Entity, Entity) {
    let jack_entity = spawn_screen_jack(world, position);
    let device_entity = spawn_screen_body(world, position, jack_entity);
    spawn_screen_panels(world, device_entity);
    (device_entity, jack_entity)
}

fn spawn_screen_jack(world: &mut World, position: Vec2) -> Entity {
    let entity = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: SOCKET_RADIUS,
                color: SOCKET_COLOR,
                connected_cable: None,
            },
            Transform2D {
                position: position + JACK_OFFSET,
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
            LocalSortOrder(SCREEN_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id();
    world.entity_mut(entity).observe(on_socket_clicked);
    entity
}

fn spawn_screen_body(world: &mut World, position: Vec2, jack_entity: Entity) -> Entity {
    let entity = world
        .spawn((
            ScreenDevice {
                signature_input: jack_entity,
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
            LocalSortOrder(SCREEN_LOCAL_SORT),
            Clickable(ClickHitShape::Aabb(SCREEN_HALF_EXTENTS)),
            CableCollider::from_aabb(SCREEN_HALF_EXTENTS),
        ))
        .id();
    world.entity_mut(entity).observe(on_screen_clicked);
    entity
}

fn spawn_screen_panels(world: &mut World, device_entity: Entity) {
    for display_index in 0..DISPLAY_COUNT {
        let offset = panel_offset(display_index);

        world.spawn_child(
            device_entity,
            (
                Transform2D {
                    position: offset,
                    ..Default::default()
                },
                Shape {
                    variant: rect_polygon(PANEL_HALF, PANEL_HALF),
                    color: PANEL_FILL,
                },
                RenderLayer::World,
                SortOrder::default(),
                LocalSortOrder(SCREEN_PANEL_LOCAL_SORT),
            ),
        );

        world.spawn_child(
            device_entity,
            (
                ScreenSignalShape { display_index },
                Transform2D {
                    position: offset,
                    ..Default::default()
                },
                Shape {
                    variant: rect_polygon(3.0, 3.0),
                    color: SIGNAL_COLOR,
                },
                Visible(false),
                RenderLayer::World,
                SortOrder::default(),
                LocalSortOrder(SCREEN_DOT_LOCAL_SORT),
            ),
        );
    }
}

/// The screen's half-extents, for callers that need the bounding box.
pub const SCREEN_HALF_EXTENTS: Vec2 = Vec2::new(BODY_HALF_W, BODY_HALF_H);

#[derive(Resource, Debug, Default)]
pub struct ScreenDragState {
    pub dragging: Option<crate::card::interaction::drag_state::DeviceDragInfo>,
}

/// Observer registered on each `ScreenDevice` entity at spawn time.
pub fn on_screen_clicked(
    trigger: Trigger<ClickedEntity>,
    screens: Query<&Transform2D, With<ScreenDevice>>,
    mut screen_drag: ResMut<ScreenDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = screens.get(entity) else {
        return;
    };
    screen_drag.dragging = Some(crate::card::interaction::drag_state::DeviceDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}

pub fn screen_drag_system(
    mouse: Res<MouseState>,
    screen_drag: Res<ScreenDragState>,
    mut screen_transforms: Query<&mut Transform2D, With<ScreenDevice>>,
    mut other_transforms: Query<&mut Transform2D, Without<ScreenDevice>>,
    screens: Query<&ScreenDevice>,
) {
    let Some(info) = &screen_drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = screen_transforms.get_mut(info.entity) {
        transform.position = target;
    }
    if let Ok(screen) = screens.get(info.entity)
        && let Ok(mut jack_t) = other_transforms.get_mut(screen.signature_input)
    {
        jack_t.position = target + JACK_OFFSET;
    }
}

pub fn screen_release_system(mouse: Res<MouseState>, mut screen_drag: ResMut<ScreenDragState>) {
    if screen_drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        screen_drag.dragging = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::card::identity::signature::CardSignature;
    use crate::card::reader::SIGNATURE_SPACE_RADIUS;
    use bevy_ecs::prelude::*;
    use engine_render::prelude::{Camera2D, RendererRes, ShapeVariant};
    use engine_render::testing::{ShapeCallLog, SpyRenderer};
    use engine_scene::prelude::{
        hierarchy_maintenance_system, transform_propagation_system, visibility_system,
    };
    use engine_ui::draw_command::DrawQueue;
    use engine_ui::unified_render::unified_render_system;
    use std::sync::{Arc, Mutex};

    fn make_world(jack_data: Option<SignatureSpace>) -> (World, ShapeCallLog, Entity) {
        let mut world = World::new();
        let (device_entity, jack_entity) = spawn_screen_device(&mut world, Vec2::ZERO);
        world
            .get_mut::<Jack<SignatureSpace>>(jack_entity)
            .expect("spawned jack must exist")
            .data = jack_data;

        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
            .with_shape_capture(shape_calls.clone())
            .with_viewport(1024, 768);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(DrawQueue::default());
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        (world, shape_calls, device_entity)
    }

    fn run_visuals(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                hierarchy_maintenance_system,
                screen_render_system,
                transform_propagation_system,
                visibility_system,
                unified_render_system,
            )
                .chain(),
        );
        schedule.run(world);
    }

    #[test]
    fn when_signal_needs_flattening_then_rendered_polygon_hits_panel_edge() {
        // Arrange
        let signal = SignatureSpace::from_single(
            CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            SIGNATURE_SPACE_RADIUS,
            Entity::from_raw(0),
        );
        let (mut world, shape_calls, _device) = make_world(Some(signal));

        // Act
        run_visuals(&mut world);

        // Assert
        let calls = shape_calls.lock().expect("shape log poisoned");
        let signal_calls: Vec<_> = calls
            .iter()
            .filter(|(_, _, color, _)| *color == SIGNAL_COLOR)
            .collect();
        assert_eq!(signal_calls.len(), 4, "four signal panels must be drawn");
        let (vertices, _, _, _) = signal_calls.first().expect("a signal draw call must exist");
        assert!(
            vertices.len() > 4,
            "signal must be rendered as a clipped circle, not a 4-vertex square"
        );
        let max_x = vertices
            .iter()
            .map(|v| v[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (max_x - PANEL_HALF).abs() < 0.5,
            "signal circle must flatten against the panel boundary"
        );
    }

    #[test]
    fn when_signal_is_centered_then_rendered_polygon_stays_inside_panel_bounds() {
        // Arrange
        let center = Vec2::new(0.0, 0.0);

        // Act
        let shape = clipped_signal_circle(center, SIGNATURE_SPACE_RADIUS * PANEL_HALF);

        // Assert
        let ShapeVariant::Polygon { points } = shape else {
            panic!("signal should be a polygon after clipping");
        };
        assert!(points.len() >= 16, "centered circle should stay round");
        for point in points {
            assert!(point.x.abs() <= PANEL_HALF + 0.001);
            assert!(point.y.abs() <= PANEL_HALF + 0.001);
        }
    }
}
