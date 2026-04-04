use bevy_ecs::prelude::{Commands, Query, Res, ResMut, With};
use engine_core::prelude::{EventBus, Transform2D};
use engine_physics::prelude::{PhysicsCommand, RigidBody};

use crate::card::component::{Card, CardZone};
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card::jack_cable::Jack;
use crate::card::reader::components::CardReader;
use crate::card::reader::signature_space::SignatureSpace;

pub fn card_reader_eject_system(
    drag_state: Res<DragState>,
    mut readers: Query<&mut CardReader>,
    mut cards: Query<&mut CardZone, With<Card>>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
    mut physics_commands: ResMut<EventBus<PhysicsCommand>>,
    transforms: Query<&Transform2D>,
    colliders: Query<&engine_physics::prelude::Collider>,
    mut commands: Commands,
) {
    let Some(drag_info) = drag_state.dragging.as_ref() else {
        return;
    };
    let card_entity = drag_info.entity;

    let Ok(mut card_zone) = cards.get_mut(card_entity) else {
        return;
    };
    let CardZone::Reader(reader_entity) = *card_zone else {
        return;
    };

    *card_zone = CardZone::Table;
    commands
        .entity(card_entity)
        .insert(engine_core::scale_spring::ScaleSpring::new(1.0))
        .insert(RigidBody::Dynamic);

    if let Ok(transform) = transforms.get(card_entity)
        && let Ok(collider) = colliders.get(card_entity)
    {
        activate_physics_body(
            card_entity,
            transform.position,
            collider,
            &mut physics_commands,
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
