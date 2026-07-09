// Wire rendering ECS systems — wrap management and ribbon visualization.

use bevy_ecs::prelude::{Entity, Query, Without};
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_scene::prelude::Visible;
use glam::Vec2;

use super::geom::{polyline_to_ribbon, segment_crosses_convex_polygon, segment_intersects_convex_polygon, polygon_centroid};
use super::{CableCollider, CABLE_COLOR, CABLE_HALF_THICKNESS, WireEndpoints, WrapAnchor, WrapWire};

/// Update anchor world positions from obstacle transforms each frame.
pub fn wrap_update_system(
    mut wires: Query<&mut WrapWire>,
    colliders: Query<(&Transform2D, &CableCollider), Without<WrapWire>>,
) {
    for mut wrap in &mut wires {
        for anchor in &mut wrap.anchors {
            if let Ok((transform, collider)) = colliders.get(anchor.obstacle)
                && let Some(local_vert) = collider.vertices.get(anchor.vertex_index)
            {
                anchor.position = *local_vert + transform.position;
            }
        }
    }
}

/// Detect wrap and unwrap events based on cable span vs obstacle polygon intersections.
pub fn wrap_detect_system(
    mut wires: Query<(&mut WrapWire, &WireEndpoints)>,
    transforms: Query<&Transform2D, Without<WrapWire>>,
    colliders: Query<(Entity, &Transform2D, &CableCollider), Without<WrapWire>>,
) {
    let obstacles: Vec<(Entity, Vec<Vec2>)> = colliders
        .iter()
        .map(|(entity, t, c)| {
            let world_verts: Vec<Vec2> = c.vertices.iter().map(|v| *v + t.position).collect();
            (entity, world_verts)
        })
        .collect();
    let obstacle_refs: Vec<(Entity, &[Vec2])> =
        obstacles.iter().map(|(e, v)| (*e, v.as_slice())).collect();

    for (mut wrap, endpoints) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src = src_t.position;
        let dst = dst_t.position;

        wrap.detect_unwraps(src, dst, &obstacle_refs);
        wrap.detect_wraps(src, dst, &obstacle_refs);
    }
}

/// Render each wire as a ribbon polygon along its geometric path.
pub fn wire_render_system(
    mut wires: Query<(
        &WireEndpoints,
        Option<&WrapWire>,
        &mut Transform2D,
        &mut Shape,
        &mut Visible,
    )>,
    transforms: Query<&Transform2D, Without<WireEndpoints>>,
) {
    for (endpoints, wrap_wire, mut transform, mut shape, mut visible) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            visible.0 = false;
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            visible.0 = false;
            continue;
        };

        let waypoints = if let Some(wrap) = wrap_wire {
            wrap.waypoints(src_t.position, dst_t.position)
        } else {
            vec![src_t.position, dst_t.position]
        };

        transform.position = Vec2::ZERO;
        transform.rotation = 0.0;
        transform.scale = Vec2::ONE;
        shape.variant = ShapeVariant::Polygon {
            points: polyline_to_ribbon(&waypoints, CABLE_HALF_THICKNESS),
        };
        shape.color = CABLE_COLOR;
        visible.0 = true;
    }
}

// ---------------------------------------------------------------------------
// WrapWire — wrap/unwrap detection logic
// ---------------------------------------------------------------------------

/// Sine of the minimum angular overshoot past the straight-through direction
/// before an anchor is unwrapped via turn reversal.  ~9 degrees — provides
/// hysteresis that prevents rapid wrap/unwrap oscillation when the cable sits
/// near the decision boundary.
const UNWRAP_SIN_THRESHOLD: f32 = 0.15;

impl WrapWire {
    fn boundary_neighbor_index(
        vertex_index: usize,
        boundary_step: i8,
        vertex_count: usize,
    ) -> usize {
        let vertex_count = vertex_count.max(1) as isize;
        let next = vertex_index as isize + boundary_step as isize;
        next.rem_euclid(vertex_count) as usize
    }

    fn select_wrap_vertex(
        verts: &[Vec2],
        hit: Vec2,
        prev_anchor: Option<&WrapAnchor>,
    ) -> Option<usize> {
        let n = verts.len();
        if let Some(prev) = prev_anchor {
            if prev.vertex_index < n {
                return Some(Self::boundary_neighbor_index(
                    prev.vertex_index,
                    prev.boundary_step,
                    n,
                ));
            }
            return None;
        }

        verts
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                ((*a - hit).length_squared()).total_cmp(&((*b - hit).length_squared()))
            })
            .map(|(idx, _)| idx)
    }

    /// Check each span for polygon intersections and insert wrap anchors.
    ///
    /// Each iteration of the outer loop adds at most one anchor, then restarts
    /// the scan from scratch with updated pins.
    pub fn detect_wraps(&mut self, src: Vec2, dst: Vec2, obstacles: &[(Entity, &[Vec2])]) {
        let max_iters = obstacles.iter().map(|(_, v)| v.len()).sum::<usize>().max(1);
        for _ in 0..max_iters {
            let mut pins: Vec<Vec2> = Vec::with_capacity(self.anchors.len() + 2);
            pins.push(src);
            for anchor in &self.anchors {
                pins.push(anchor.position);
            }
            pins.push(dst);

            let mut found_any = false;
            let mut insert_idx = 0;
            let mut i = 0;
            while i < pins.len() - 1 {
                let span_a = pins[i];
                let span_b = pins[i + 1];

                let mut found = None;
                'obstacle: for &(entity, verts) in obstacles {
                    let n = verts.len();
                    if n < 3 {
                        continue;
                    }
                    let Some(first_t) = segment_crosses_convex_polygon(span_a, span_b, verts)
                    else {
                        continue 'obstacle;
                    };
                    let span_dir = span_b - span_a;
                    let hit = span_a + span_dir * first_t;

                    let prev_anchor = if insert_idx > 0 {
                        let a = &self.anchors[insert_idx - 1];
                        if a.obstacle == entity { Some(a) } else { None }
                    } else {
                        None
                    };

                    let best_idx = Self::select_wrap_vertex(verts, hit, prev_anchor);

                    if let Some(vidx) = best_idx
                        && !self
                            .anchors
                            .iter()
                            .any(|a| a.obstacle == entity && a.vertex_index == vidx)
                    {
                        let v = verts[vidx];
                        let bend = (v - span_a).perp_dot(span_b - v);
                        let wrap_sign = if bend.abs() > 1e-6 {
                            bend.signum()
                        } else if let Some(prev) = prev_anchor {
                            prev.wrap_sign
                        } else {
                            let c = span_dir.perp_dot(polygon_centroid(verts) - span_a);
                            if c.abs() > 1e-6 { -c.signum() } else { 1.0 }
                        };
                        let boundary_step = if let Some(prev) = prev_anchor {
                            prev.boundary_step
                        } else if wrap_sign > 0.0 {
                            1i8
                        } else {
                            -1i8
                        };
                        found = Some(WrapAnchor {
                            position: v,
                            obstacle: entity,
                            vertex_index: vidx,
                            boundary_step,
                            wrap_sign,
                        });
                        break 'obstacle;
                    }
                }

                if let Some(anchor) = found {
                    self.anchors.insert(insert_idx, anchor);
                    found_any = true;
                    break;
                }
                insert_idx += 1;
                i += 1;
            }

            if !found_any {
                break;
            }
        }
    }

    /// Remove anchors only after the cable has clearly swung past the wrap point.
    pub fn detect_unwraps(&mut self, src: Vec2, dst: Vec2, obstacles: &[(Entity, &[Vec2])]) {
        loop {
            let mut changed = false;
            let mut i = self.anchors.len();
            while i > 0 {
                i -= 1;
                let prev = if i == 0 {
                    src
                } else {
                    self.anchors[i - 1].position
                };
                let next = if i + 1 < self.anchors.len() {
                    self.anchors[i + 1].position
                } else {
                    dst
                };

                let to_anchor = self.anchors[i].position - prev;
                let from_anchor = next - self.anchors[i].position;
                let cross = to_anchor.perp_dot(from_anchor);

                let to_len = to_anchor.length();
                let from_len = from_anchor.length();
                let sin_angle = if to_len > 1e-6 && from_len > 1e-6 {
                    cross / (to_len * from_len)
                } else {
                    0.0
                };
                let turn_reversed = sin_angle * self.anchors[i].wrap_sign < -UNWRAP_SIN_THRESHOLD;
                let same_obstacle_prev =
                    i > 0 && self.anchors[i - 1].obstacle == self.anchors[i].obstacle;
                let shortcut_start = if same_obstacle_prev {
                    prev + (next - prev) * 0.005
                } else {
                    prev
                };
                let shortcut_clear = obstacles
                    .iter()
                    .find(|(entity, _)| *entity == self.anchors[i].obstacle)
                    .is_none_or(|(_, verts)| {
                        !segment_intersects_convex_polygon(shortcut_start, next, verts)
                    });

                if turn_reversed || shortcut_clear {
                    self.anchors.remove(i);
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    /// Compute the shortest geometric path from `src` through all anchors to `dst`.
    pub fn shortest_path(&self, src: Vec2, dst: Vec2) -> f32 {
        self.waypoints(src, dst)
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).length())
            .sum()
    }

    /// Return the full waypoint list: `[src, anchor1, ..., anchorN, dst]`.
    pub fn waypoints(&self, src: Vec2, dst: Vec2) -> Vec<Vec2> {
        let mut waypoints = Vec::with_capacity(self.anchors.len() + 2);
        waypoints.push(src);
        waypoints.extend(self.anchors.iter().map(|anchor| anchor.position));
        waypoints.push(dst);
        waypoints
    }
}
