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
use std::f32::consts::TAU;

use crate::card::identity::signature::Element;
use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, ClickedEntity};
use crate::card::interaction::drag_state::DragState;
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, PendingCable, on_socket_clicked};
use crate::card::reader::{ReaderDragState, SignatureSpace};
use crate::stash::grid::StashGrid;
use crate::stash::toggle::StashVisible;

const DISPLAY_COUNT: usize = 4;
const PANEL_HALF: f32 = 50.0;
const PANEL_SPACING: f32 = 110.0;
const SIGNAL_SEGMENTS: usize = 32;

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
const JACK_OFFSET: Vec2 = Vec2::new(BODY_HALF_W + SOCKET_RADIUS + 4.0, 0.0);
const SCREEN_LOCAL_SORT: i32 = -1;
const SCREEN_PANEL_LOCAL_SORT: i32 = 1;
const SCREEN_DOT_LOCAL_SORT: i32 = 2;
const SCREEN_SOCKET_LOCAL_SORT: i32 = 1;

#[derive(Component)]
pub struct ScreenDevice {
    pub signature_input: Entity,
}

#[derive(Component)]
pub struct ScreenSignalDot {
    display_index: usize,
}

pub fn display_axes(space: &SignatureSpace, display_index: usize) -> (f32, f32) {
    let x_element = Element::ALL[display_index * 2];
    let y_element = Element::ALL[display_index * 2 + 1];
    (space.center[x_element], space.center[y_element])
}

fn panel_offset(display_index: usize) -> Vec2 {
    let (x, y) = PANEL_OFFSETS[display_index];
    Vec2::new(x, y)
}

pub fn screen_render_system(
    devices: Query<&ScreenDevice>,
    jacks: Query<&Jack<SignatureSpace>>,
    mut dots: Query<(&ScreenSignalDot, &ChildOf, &mut Shape, &mut Visible)>,
) {
    for (dot, parent, mut shape, mut visible) in &mut dots {
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

        let (ax, ay) = display_axes(space, dot.display_index);
        let center = Vec2::new(ax * PANEL_HALF, ay * PANEL_HALF);
        shape.variant = clipped_signal_circle(center, space.radius * PANEL_HALF);
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
    let jack_entity = world
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

    let device_entity = world
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
            CableCollider {
                half_extents: SCREEN_HALF_EXTENTS,
            },
        ))
        .id();
    world.entity_mut(device_entity).observe(on_screen_clicked);
    world.entity_mut(jack_entity).observe(on_socket_clicked);

    for display_index in 0..DISPLAY_COUNT {
        world.spawn_child(
            device_entity,
            (
                Transform2D {
                    position: panel_offset(display_index),
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
                ScreenSignalDot { display_index },
                Transform2D {
                    position: panel_offset(display_index),
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

    (device_entity, jack_entity)
}

/// The screen's half-extents, for callers that need the bounding box.
pub const SCREEN_HALF_EXTENTS: Vec2 = Vec2::new(BODY_HALF_W, BODY_HALF_H);

#[derive(Resource, Debug, Default)]
pub struct ScreenDragState {
    pub dragging: Option<ScreenDragInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
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
    screen_drag.dragging = Some(ScreenDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}

pub fn screen_pick_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    reader_drag: Res<ReaderDragState>,
    pending: Res<PendingCable>,
    mut screen_drag: ResMut<ScreenDragState>,
    stash_visible: Option<Res<StashVisible>>,
    grid: Option<Res<StashGrid>>,
    screens: Query<(Entity, &Transform2D), With<ScreenDevice>>,
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
    if let (Some(stash_visible), Some(grid)) = (stash_visible, grid)
        && stash_visible.0
        && crate::stash::pages::stash_ui_contains(mouse.screen_pos(), &grid)
    {
        return;
    }
    let cursor = mouse.world_pos();
    for (entity, transform) in &screens {
        let delta = (cursor - transform.position).abs();
        if delta.x <= SCREEN_HALF_EXTENTS.x && delta.y <= SCREEN_HALF_EXTENTS.y {
            screen_drag.dragging = Some(ScreenDragInfo {
                entity,
                grab_offset: cursor - transform.position,
            });
            return;
        }
    }
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

fn clipped_signal_circle(center: Vec2, radius: f32) -> ShapeVariant {
    let circle = circle_polygon(center, radius, SIGNAL_SEGMENTS);
    let clipped = clip_polygon_to_rect(
        &circle,
        Vec2::new(-PANEL_HALF, -PANEL_HALF),
        Vec2::new(PANEL_HALF, PANEL_HALF),
    );
    ShapeVariant::Polygon { points: clipped }
}

fn circle_polygon(center: Vec2, radius: f32, segments: usize) -> Vec<Vec2> {
    (0..segments)
        .map(|index| {
            let angle = TAU * index as f32 / segments as f32;
            center + Vec2::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

fn clip_polygon_to_rect(points: &[Vec2], min: Vec2, max: Vec2) -> Vec<Vec2> {
    let left = clip_polygon(
        points.to_vec(),
        |p| p.x >= min.x,
        |a, b| intersect_x(a, b, min.x),
    );
    let right = clip_polygon(left, |p| p.x <= max.x, |a, b| intersect_x(a, b, max.x));
    let bottom = clip_polygon(right, |p| p.y >= min.y, |a, b| intersect_y(a, b, min.y));
    clip_polygon(bottom, |p| p.y <= max.y, |a, b| intersect_y(a, b, max.y))
}

fn clip_polygon<F, G>(points: Vec<Vec2>, is_inside: F, intersect: G) -> Vec<Vec2>
where
    F: Fn(Vec2) -> bool,
    G: Fn(Vec2, Vec2) -> Vec2,
{
    let Some(mut previous) = points.last().copied() else {
        return points;
    };
    let mut result = Vec::new();
    let mut previous_inside = is_inside(previous);

    for current in points {
        let current_inside = is_inside(current);
        match (previous_inside, current_inside) {
            (true, true) => result.push(current),
            (true, false) => result.push(intersect(previous, current)),
            (false, true) => {
                result.push(intersect(previous, current));
                result.push(current);
            }
            (false, false) => {}
        }
        previous = current;
        previous_inside = current_inside;
    }

    result
}

fn intersect_x(a: Vec2, b: Vec2, x: f32) -> Vec2 {
    let delta = b.x - a.x;
    if delta.abs() <= f32::EPSILON {
        return Vec2::new(x, a.y);
    }
    let t = (x - a.x) / delta;
    a + (b - a) * t
}

fn intersect_y(a: Vec2, b: Vec2, y: f32) -> Vec2 {
    let delta = b.y - a.y;
    if delta.abs() <= f32::EPSILON {
        return Vec2::new(a.x, y);
    }
    let t = (y - a.y) / delta;
    a + (b - a) * t
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
        let signal = SignatureSpace {
            center: CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            radius: SIGNATURE_SPACE_RADIUS,
        };
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
