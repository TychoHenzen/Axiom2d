use bevy_ecs::prelude::{Component, Entity, Query, Without};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_scene::prelude::Visible;
use glam::Vec2;

use crate::card::reader::SignatureSpace;

const CABLE_COLOR: Color = Color {
    r: 0.7,
    g: 0.6,
    b: 0.3,
    a: 0.9,
};
pub(crate) const CABLE_HALF_THICKNESS: f32 = 1.5;
pub(crate) const CABLE_LOCAL_SORT: i32 = -2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JackDirection {
    Input,
    Output,
}

#[derive(Component, Debug, Clone)]
pub struct Jack<T: Clone + Send + Sync + 'static> {
    pub direction: JackDirection,
    pub data: Option<T>,
}

#[derive(Component, Debug, Clone)]
pub struct Cable {
    pub source: Entity,
    pub dest: Entity,
}

pub(crate) fn cable_visuals(a: Vec2, b: Vec2) -> (Transform2D, Shape) {
    let midpoint = (a + b) * 0.5;
    let half_delta = (b - a) * 0.5;
    let dir = (b - a).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x) * CABLE_HALF_THICKNESS;

    (
        Transform2D {
            position: midpoint,
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    -half_delta - perp,
                    -half_delta + perp,
                    half_delta + perp,
                    half_delta - perp,
                ],
            },
            color: CABLE_COLOR,
        },
    )
}

pub fn cable_render_system(
    mut cables: Query<(&Cable, &mut Transform2D, &mut Shape, &mut Visible)>,
    transforms: Query<&Transform2D, Without<Cable>>,
) {
    for (cable, mut transform, mut shape, mut visible) in &mut cables {
        let Ok(src_t) = transforms.get(cable.source) else {
            visible.0 = false;
            continue;
        };
        let Ok(dst_t) = transforms.get(cable.dest) else {
            visible.0 = false;
            continue;
        };

        let (next_transform, next_shape) = cable_visuals(src_t.position, dst_t.position);
        *transform = next_transform;
        *shape = next_shape;
        visible.0 = true;
    }
}

pub fn signature_space_propagation_system(
    cables: Query<&Cable>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
) {
    let updates: Vec<(Entity, Option<SignatureSpace>)> = cables
        .iter()
        .filter_map(|cable| {
            let data = jacks.get(cable.source).ok()?.data.clone();
            Some((cable.dest, data))
        })
        .collect();

    for (dest, data) in updates {
        if let Ok(mut jack) = jacks.get_mut(dest) {
            jack.data = data;
        }
    }
}
