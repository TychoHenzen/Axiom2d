// BoosterMachine device — spawn, drag, seal button

use bevy_ecs::prelude::{
    Component, Entity, Query, Res, ResMut, Resource, Trigger, With, Without, World,
};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::prelude::LocalSortOrder;
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, ClickedEntity};
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, on_socket_clicked};
use crate::card::identity::signature::CardSignature;
use crate::card::reader::SignatureSpace;

const BODY_HALF_W: f32 = 50.0;
const BODY_HALF_H: f32 = 35.0;
const BODY_CORNER_RADIUS: f32 = 4.0;

const BODY_FILL: Color = Color {
    r: 0.18,
    g: 0.14,
    b: 0.10,
    a: 1.0,
};
const BODY_STROKE: Color = Color {
    r: 0.80,
    g: 0.65,
    b: 0.20,
    a: 1.0,
};
const SOCKET_COLOR: Color = Color {
    r: 0.80,
    g: 0.65,
    b: 0.20,
    a: 1.0,
};
const SOCKET_RADIUS: f32 = 8.0;
const BOOSTER_LOCAL_SORT: i32 = -1;
const BOOSTER_SOCKET_LOCAL_SORT: i32 = 1;
const BOOSTER_BUTTON_LOCAL_SORT: i32 = 2;

const BOOSTER_HALF_EXTENTS: Vec2 = Vec2::new(BODY_HALF_W, BODY_HALF_H);
const INPUT_X: f32 = -(BODY_HALF_W + SOCKET_RADIUS + 4.0);
const BUTTON_OFFSET: Vec2 = Vec2::new(20.0, 0.0);
const PACK_SLOT_OFFSET: Vec2 = Vec2::new(-20.0, 0.0);

const BUTTON_HALF_W: f32 = 18.0;
const BUTTON_HALF_H: f32 = 10.0;
const BUTTON_CORNER_RADIUS: f32 = 3.0;
const BUTTON_FILL: Color = Color {
    r: 0.25,
    g: 0.20,
    b: 0.15,
    a: 1.0,
};
const BUTTON_STROKE: Color = Color {
    r: 0.90,
    g: 0.75,
    b: 0.25,
    a: 1.0,
};

#[derive(Component, Debug)]
pub struct BoosterMachine {
    pub signal_input: Entity,
    pub button_entity: Entity,
    pub output_pack: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct BoosterSealButton {
    pub machine_entity: Entity,
}

#[derive(Resource, Debug, Default)]
pub struct SealButtonPressed(pub Option<Entity>);

/// Spawns a booster machine device at `position`.
///
/// Returns `(device_entity, input_jack)`.
pub fn spawn_booster_machine(world: &mut World, position: Vec2) -> (Entity, Entity) {
    // Reserve the device entity ID first so we can reference it in child components.
    let device_entity = world.spawn_empty().id();

    let input_jack = world
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
                position: position + Vec2::new(INPUT_X, 0.0),
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
            LocalSortOrder(BOOSTER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(SOCKET_RADIUS)),
        ))
        .id();

    let button_entity = world
        .spawn((
            BoosterSealButton {
                machine_entity: device_entity,
            },
            Transform2D {
                position: position + BUTTON_OFFSET,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Shape {
                variant: rounded_rect_path(BUTTON_HALF_W, BUTTON_HALF_H, BUTTON_CORNER_RADIUS),
                color: BUTTON_FILL,
            },
            Stroke {
                color: BUTTON_STROKE,
                width: 1.5,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(BOOSTER_BUTTON_LOCAL_SORT),
            Clickable(ClickHitShape::Aabb(Vec2::new(BUTTON_HALF_W, BUTTON_HALF_H))),
        ))
        .id();

    world.entity_mut(device_entity).insert((
        BoosterMachine {
            signal_input: input_jack,
            button_entity,
            output_pack: None,
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
        LocalSortOrder(BOOSTER_LOCAL_SORT),
        Clickable(ClickHitShape::Aabb(BOOSTER_HALF_EXTENTS)),
        CableCollider::from_aabb(BOOSTER_HALF_EXTENTS),
    ));

    world.entity_mut(device_entity).observe(on_booster_clicked);
    world.entity_mut(input_jack).observe(on_socket_clicked);
    world
        .entity_mut(button_entity)
        .observe(on_seal_button_clicked);

    (device_entity, input_jack)
}

#[derive(Resource, Debug, Default)]
pub struct BoosterDragState {
    pub dragging: Option<BoosterDragInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoosterDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
}

pub fn on_booster_clicked(
    trigger: Trigger<ClickedEntity>,
    boosters: Query<&Transform2D, With<BoosterMachine>>,
    mut drag: ResMut<BoosterDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = boosters.get(entity) else {
        return;
    };
    drag.dragging = Some(BoosterDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}

pub fn booster_drag_system(
    mouse: Res<MouseState>,
    drag: Res<BoosterDragState>,
    mut booster_transforms: Query<&mut Transform2D, With<BoosterMachine>>,
    mut other_transforms: Query<&mut Transform2D, Without<BoosterMachine>>,
    boosters: Query<&BoosterMachine>,
) {
    let Some(info) = &drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = booster_transforms.get_mut(info.entity) {
        transform.position = target;
    }
    if let Ok(machine) = boosters.get(info.entity) {
        // Move input jack
        if let Ok(mut jack_t) = other_transforms.get_mut(machine.signal_input) {
            jack_t.position = target + Vec2::new(INPUT_X, 0.0);
        }
        // Move seal button
        if let Ok(mut btn_t) = other_transforms.get_mut(machine.button_entity) {
            btn_t.position = target + BUTTON_OFFSET;
        }
        // Move output pack if present
        if let Some(pack_entity) = machine.output_pack
            && let Ok(mut pack_t) = other_transforms.get_mut(pack_entity)
        {
            pack_t.position = target + PACK_SLOT_OFFSET;
        }
    }
}

pub fn booster_release_system(mouse: Res<MouseState>, mut drag: ResMut<BoosterDragState>) {
    if drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        drag.dragging = None;
    }
}

fn on_seal_button_clicked(
    trigger: Trigger<ClickedEntity>,
    buttons: Query<&BoosterSealButton>,
    mut pressed: ResMut<SealButtonPressed>,
) {
    let entity = trigger.target();
    if let Ok(button) = buttons.get(entity) {
        pressed.0 = Some(button.machine_entity);
    }
}

/// Exclusive system that seals a booster pack when the seal button is pressed.
///
/// Reads `SealButtonPressed`, samples signatures from the input signal, spawns a
/// `BoosterPack` in the machine's output slot, and destroys all source cards.
pub fn booster_seal_system(world: &mut World) {
    use crate::booster::pack::spawn_booster_pack;
    use crate::booster::sampling::sample_signatures_from_space;
    use crate::card::component::Card;
    use crate::card::identity::signature::Rarity;
    use crate::card::reader::CardReader;
    use engine_core::prelude::EventBus;
    use engine_core::scale_spring::ScaleSpring;
    use engine_physics::prelude::{PhysicsCommand, RigidBody};
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    // 1. Read and clear SealButtonPressed. If no device entity, return.
    let device_entity = {
        let Some(mut pressed) = world.get_resource_mut::<SealButtonPressed>() else {
            return;
        };
        let entity = pressed.0.take();
        match entity {
            Some(e) => e,
            None => return,
        }
    };

    // 2. Get BoosterMachine. If output_pack.is_some(), return (slot occupied).
    let (signal_input, position) = {
        let Some(machine) = world.get::<BoosterMachine>(device_entity) else {
            return;
        };
        if machine.output_pack.is_some() {
            return;
        }
        let signal_input = machine.signal_input;
        let position = world
            .get::<Transform2D>(device_entity)
            .map_or(Vec2::ZERO, |t| t.position);
        (signal_input, position)
    };

    // 3. Read Jack<SignatureSpace> from machine.signal_input. If no data, return.
    let space = {
        let Some(jack) = world.get::<Jack<SignatureSpace>>(signal_input) else {
            return;
        };
        match jack.data.clone() {
            Some(s) => s,
            None => return,
        }
    };

    // 4. If source_cards is empty, return.
    if space.source_cards.is_empty() {
        return;
    }

    // RNG seeding from control points
    let seed_bytes: u64 = space
        .control_points
        .iter()
        .flat_map(CardSignature::axes)
        .fold(0u64, |acc, v: f32| acc.wrapping_add(u64::from(v.to_bits())));
    let mut rng = ChaCha8Rng::seed_from_u64(seed_bytes);

    // 5. Determine card count with rarity bonus
    let base_count: usize = rng.gen_range(5..=15);

    let mut rarity_bonus: usize = 0;
    for &card_entity in &space.source_cards {
        if let Some(card) = world.get::<Card>(card_entity) {
            rarity_bonus += match card.signature.rarity() {
                Rarity::Common => 0,
                Rarity::Uncommon => 1,
                Rarity::Rare => 2,
                Rarity::Epic => 3,
                Rarity::Legendary => 4,
            };
        }
    }
    let count = (base_count + rarity_bonus).min(15);

    // 6. Sample signatures
    let signatures = sample_signatures_from_space(&space, count, &mut rng);

    // 7. Spawn pack at machine position + offset
    let pack_position = position + PACK_SLOT_OFFSET;
    let pack_entity = spawn_booster_pack(world, pack_position, signatures);

    // 8. Scale down the pack in slot
    world.entity_mut(pack_entity).insert(ScaleSpring::new(0.5));

    // 9. Remove physics from pack while in slot
    if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
        bus.push(PhysicsCommand::RemoveBody {
            entity: pack_entity,
        });
    }
    world.entity_mut(pack_entity).remove::<RigidBody>();

    // 10. Set machine.output_pack
    if let Some(mut machine) = world.get_mut::<BoosterMachine>(device_entity) {
        machine.output_pack = Some(pack_entity);
    }

    // 11. Destroy source cards
    let source_cards = space.source_cards.clone();
    for &card_entity in &source_cards {
        // Find the CardReader that has this card loaded
        let mut reader_to_clear: Option<(Entity, Entity)> = None;
        for (reader_entity, reader) in world.query::<(Entity, &CardReader)>().iter(world) {
            if reader.loaded == Some(card_entity) {
                reader_to_clear = Some((reader_entity, reader.jack_entity));
                break;
            }
        }
        if let Some((reader_entity, jack_entity)) = reader_to_clear {
            if let Some(mut reader) = world.get_mut::<CardReader>(reader_entity) {
                reader.loaded = None;
            }
            if let Some(mut jack) = world.get_mut::<Jack<SignatureSpace>>(jack_entity) {
                jack.data = None;
            }
        }
        world.despawn(card_entity);
    }
}
