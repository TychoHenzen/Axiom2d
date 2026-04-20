use bevy_ecs::prelude::{Entity, Local, Query, Res, ResMut};
use engine_core::color::Color;
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsRes;
use engine_render::prelude::{QUAD_INDICES, UNIT_QUAD, affine2_to_mat4};
use engine_render::shape::TessellatedMesh;
use engine_scene::prelude::{GlobalTransform2D, RenderLayer, SortOrder};
use engine_ui::draw_command::{DrawCommand, DrawQueue};

use crate::card::component::CardZone;
use crate::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};

const TINT_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 0.3,
};

pub fn debug_sleep_indicator_system(
    query: Query<(Entity, &CardZone, &GlobalTransform2D, Option<&SortOrder>)>,
    input: Res<InputState>,
    physics: Res<PhysicsRes>,
    mut draw_queue: ResMut<DrawQueue>,
    mut enabled: Local<bool>,
) {
    if input.just_pressed(KeyCode::F9) {
        *enabled = !*enabled;
    }
    if !*enabled {
        return;
    }

    for (entity, zone, transform, sort) in &query {
        if !matches!(zone, CardZone::Table) {
            continue;
        }
        if physics.is_body_sleeping(entity) != Some(true) {
            continue;
        }
        let scaled = transform.0
            * glam::Affine2::from_scale(glam::Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT));
        let model = affine2_to_mat4(&scaled);
        let sort_order = sort.copied().unwrap_or_default();
        draw_queue.push(
            RenderLayer::World,
            SortOrder::new(sort_order.value() + 1),
            DrawCommand::Shape {
                mesh: TessellatedMesh {
                    vertices: UNIT_QUAD.to_vec(),
                    indices: QUAD_INDICES.to_vec(),
                },
                color: TINT_COLOR,
                model,
                material: None,
                stroke: None,
            },
        );
    }
}
