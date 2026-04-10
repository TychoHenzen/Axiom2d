// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_render::prelude::{
    Camera2D, RendererRes, resolve_viewport_camera, screen_to_world, unit_quad_model,
};
use engine_render::shape::TessellatedMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use engine_ui::draw_command::{DrawCommand, DrawQueue};

use glam::Vec2;

use crate::card::interaction::drag_state::DragState;

pub(crate) const HAND_DROP_ZONE_HEIGHT: f32 = 120.0;

pub(crate) const HAND_ZONE_GLOW_COLOR: Color = Color {
    r: 0.3,
    g: 0.5,
    b: 0.8,
    a: 0.25,
};

pub fn hand_drop_zone_render_system(
    drag_state: Res<DragState>,
    camera_query: Query<&Camera2D>,
    renderer: Res<RendererRes>,
    mut draw_queue: ResMut<DrawQueue>,
) {
    if drag_state.dragging.is_none() {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let top_left = screen_to_world(Vec2::new(0.0, vh - HAND_DROP_ZONE_HEIGHT), &camera, vw, vh);
    let bottom_right = screen_to_world(Vec2::new(vw, vh), &camera, vw, vh);
    let width = bottom_right.x - top_left.x;
    let height = bottom_right.y - top_left.y;
    let cx = (top_left.x + bottom_right.x) * 0.5;
    let cy = (top_left.y + bottom_right.y) * 0.5;

    let model = unit_quad_model(width, height, cx, cy);
    draw_queue.push(
        RenderLayer::UI,
        SortOrder::new(0),
        DrawCommand::Shape {
            mesh: TessellatedMesh {
                vertices: engine_render::prelude::UNIT_QUAD.to_vec(),
                indices: engine_render::prelude::QUAD_INDICES.to_vec(),
            },
            color: HAND_ZONE_GLOW_COLOR,
            model,
            material: None,
            stroke: None,
        },
    );
}
// EVOLVE-BLOCK-END
