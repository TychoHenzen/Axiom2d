use bevy_ecs::prelude::{Component, Entity, Query, ResMut};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{IDENTITY_MODEL, QUAD_INDICES, RendererRes, rect_vertices};

use crate::card::identity::signature::Element;
use crate::card::jack_cable::Jack;
use crate::card::reader::SignatureSpace;

const DISPLAY_COUNT: usize = 4;
const PANEL_HALF: f32 = 50.0;
const PANEL_SPACING: f32 = 110.0;
const DOT_HALF: f32 = 3.0;

const PANEL_OFFSETS: [(f32, f32); DISPLAY_COUNT] = [
    (-PANEL_SPACING * 0.5, -PANEL_SPACING * 0.5),
    (PANEL_SPACING * 0.5, -PANEL_SPACING * 0.5),
    (-PANEL_SPACING * 0.5, PANEL_SPACING * 0.5),
    (PANEL_SPACING * 0.5, PANEL_SPACING * 0.5),
];

const DOT_COLOR: Color = Color {
    r: 0.4,
    g: 0.9,
    b: 0.4,
    a: 1.0,
};

#[derive(Component)]
pub struct ScreenDevice {
    pub signature_input: Entity,
}

pub fn display_axes(space: &SignatureSpace, display_index: usize) -> (f32, f32) {
    let x_element = Element::ALL[display_index * 2];
    let y_element = Element::ALL[display_index * 2 + 1];
    (space.center[x_element], space.center[y_element])
}

pub fn screen_render_system(
    devices: Query<(&ScreenDevice, &Transform2D)>,
    jacks: Query<&Jack<SignatureSpace>>,
    mut renderer: ResMut<RendererRes>,
) {
    for (device, transform) in &devices {
        let Ok(jack) = jacks.get(device.signature_input) else {
            continue;
        };
        let Some(ref space) = jack.data else {
            continue;
        };

        for (i, &(panel_ox, panel_oy)) in PANEL_OFFSETS.iter().enumerate() {
            let (ax, ay) = display_axes(space, i);

            let dot_x = transform.position.x + panel_ox + ax * PANEL_HALF;
            let dot_y = transform.position.y + panel_oy + ay * PANEL_HALF;

            let verts = rect_vertices(
                dot_x - DOT_HALF,
                dot_y - DOT_HALF,
                DOT_HALF * 2.0,
                DOT_HALF * 2.0,
            );
            renderer.draw_shape(&verts, &QUAD_INDICES, DOT_COLOR, IDENTITY_MODEL);
        }
    }
}
