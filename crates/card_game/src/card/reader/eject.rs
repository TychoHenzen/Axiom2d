use bevy_ecs::prelude::{Commands, Query, Res, ResMut, With};
use engine_core::prelude::{EventBus, Transform2D};
use engine_physics::prelude::{PhysicsRes, RigidBody};

use crate::card::component::{Card, CardZone};
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card::reader::components::{CardReader, CardReaderEjectIntent, OutputJack};

pub fn card_reader_eject_intent_system(
    drag_state: Res<DragState>,
    cards: Query<&CardZone, With<Card>>,
    mut intents: ResMut<EventBus<CardReaderEjectIntent>>,
) {
    let Some(drag_info) = drag_state.dragging.as_ref() else {
        return;
    };
    let card_entity = drag_info.entity;

    let Ok(card_zone) = cards.get(card_entity) else {
        return;
    };
    let CardZone::Reader(reader_entity) = *card_zone else {
        return;
    };

    intents.push(CardReaderEjectIntent {
        card_entity,
        reader_entity,
    });
}

pub fn apply_card_reader_eject_intents_system(
    mut readers: Query<&mut CardReader>,
    mut cards: Query<&mut CardZone, With<Card>>,
    mut jacks: Query<&mut OutputJack>,
    mut physics: ResMut<PhysicsRes>,
    transforms: Query<&Transform2D>,
    colliders: Query<&engine_physics::prelude::Collider>,
    mut intents: ResMut<EventBus<CardReaderEjectIntent>>,
    mut commands: Commands,
) {
    for intent in intents.drain() {
        let Ok(mut card_zone) = cards.get_mut(intent.card_entity) else {
            continue;
        };
        let CardZone::Reader(reader_entity) = *card_zone else {
            continue;
        };
        if reader_entity != intent.reader_entity {
            continue;
        }

        *card_zone = CardZone::Table;
        commands
            .entity(intent.card_entity)
            .insert(engine_core::scale_spring::ScaleSpring::new(1.0))
            .insert(RigidBody::Dynamic);

        if let Ok(transform) = transforms.get(intent.card_entity)
            && let Ok(collider) = colliders.get(intent.card_entity)
        {
            activate_physics_body(
                intent.card_entity,
                transform.position,
                collider,
                &mut physics,
                CARD_COLLISION_GROUP,
                CARD_COLLISION_FILTER,
            );
        }

        if let Ok(mut reader) = readers.get_mut(reader_entity) {
            reader.loaded = None;
            if let Ok(mut jack) = jacks.get_mut(reader.jack_entity) {
                jack.data = None;
            }
        }
    }
}
