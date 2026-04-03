use bevy_ecs::prelude::{Component, Entity, Query, ResMut};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{IDENTITY_MODEL, QUAD_INDICES, RendererRes};

use crate::card::reader::SignatureSpace;

const CABLE_COLOR: Color = Color {
    r: 0.7,
    g: 0.6,
    b: 0.3,
    a: 0.9,
};
const CABLE_HALF_THICKNESS: f32 = 1.5;

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

pub fn cable_render_system(
    cables: Query<&Cable>,
    transforms: Query<&Transform2D>,
    mut renderer: ResMut<RendererRes>,
) {
    for cable in &cables {
        let Ok(src_t) = transforms.get(cable.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(cable.dest) else {
            continue;
        };

        let a = src_t.position;
        let b = dst_t.position;
        let dir = (b - a).normalize_or_zero();
        let perp = glam::Vec2::new(-dir.y, dir.x) * CABLE_HALF_THICKNESS;

        let verts = [
            [a.x - perp.x, a.y - perp.y],
            [a.x + perp.x, a.y + perp.y],
            [b.x + perp.x, b.y + perp.y],
            [b.x - perp.x, b.y - perp.y],
        ];
        renderer.draw_shape(&verts, &QUAD_INDICES, CABLE_COLOR, IDENTITY_MODEL);
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
