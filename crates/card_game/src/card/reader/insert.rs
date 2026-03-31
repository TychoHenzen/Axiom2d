use bevy_ecs::prelude::{Commands, Query, Res, ResMut, Without};
use engine_core::prelude::{EventBus, Transform2D};
use engine_physics::prelude::{PhysicsRes, RigidBody};

use crate::card::component::{Card, CardZone};
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::physics_helpers::warn_on_physics_result;
use crate::card::reader::components::{
    CardReader, CardReaderInsertIntent, OutputJack, READER_CARD_SCALE, card_overlaps_reader,
};

pub fn card_reader_insert_intent_system(
    mouse: Res<engine_input::prelude::MouseState>,
    drag_state: Res<DragState>,
    readers: Query<(bevy_ecs::prelude::Entity, &Transform2D, &CardReader)>,
    cards: Query<&Transform2D, (bevy_ecs::prelude::With<Card>, Without<CardReader>)>,
    mut intents: ResMut<EventBus<CardReaderInsertIntent>>,
) {
    use engine_input::mouse_button::MouseButton;

    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(drag_info) = drag_state.dragging.as_ref() else {
        return;
    };
    let card_entity = drag_info.entity;

    let Ok(card_transform) = cards.get(card_entity) else {
        return;
    };
    let card_pos = card_transform.position;

    for (reader_entity, reader_transform, reader) in &readers {
        if reader.loaded.is_some() {
            continue;
        }
        if !card_overlaps_reader(card_pos, reader_transform.position, reader.half_extents) {
            continue;
        }

        intents.push(CardReaderInsertIntent {
            card_entity,
            reader_entity,
        });
        return;
    }
}

pub fn apply_card_reader_insert_intents_system(
    mut drag_state: ResMut<DragState>,
    mut readers: Query<(bevy_ecs::prelude::Entity, &Transform2D, &mut CardReader)>,
    mut cards: Query<(&mut Transform2D, &Card, &mut CardZone), Without<CardReader>>,
    mut jacks: Query<&mut OutputJack>,
    mut physics: ResMut<PhysicsRes>,
    mut intents: ResMut<EventBus<CardReaderInsertIntent>>,
    mut commands: Commands,
) {
    for intent in intents.drain() {
        let Some(drag_info) = drag_state.dragging.as_ref() else {
            continue;
        };
        if drag_info.entity != intent.card_entity {
            continue;
        }

        let Ok((reader_entity, reader_transform, mut reader)) =
            readers.get_mut(intent.reader_entity)
        else {
            continue;
        };
        if reader_entity != intent.reader_entity || reader.loaded.is_some() {
            continue;
        }

        let Ok((mut card_transform, card, mut card_zone)) = cards.get_mut(intent.card_entity)
        else {
            continue;
        };
        if !card_overlaps_reader(
            card_transform.position,
            reader_transform.position,
            reader.half_extents,
        ) {
            continue;
        }

        card_transform.position = reader_transform.position;
        card_transform.rotation = 0.0;

        commands
            .entity(intent.card_entity)
            .insert(engine_core::scale_spring::ScaleSpring::new(
                READER_CARD_SCALE,
            ));

        warn_on_physics_result(
            "remove_body",
            intent.card_entity,
            physics.remove_body(intent.card_entity),
        );
        commands.entity(intent.card_entity).remove::<RigidBody>();

        *card_zone = CardZone::Reader(intent.reader_entity);
        reader.loaded = Some(intent.card_entity);

        if let Ok(mut jack) = jacks.get_mut(reader.jack_entity) {
            jack.data = Some(card.signature);
        }

        drag_state.dragging = None;
    }
}
