use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without};
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_physics::prelude::{PhysicsCommand, RigidBody};

use crate::card::component::{Card, CardZone};
use crate::card::interaction::drag_state::DragState;
use crate::card::jack_cable::Jack;
use crate::card::reader::components::{CardReader, READER_CARD_SCALE, card_overlaps_reader};
use crate::card::reader::signature_space::{SIGNATURE_SPACE_RADIUS, SignatureSpace};

pub fn card_reader_insert_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut readers: Query<(Entity, &Transform2D, &mut CardReader)>,
    mut cards: Query<(&mut Transform2D, &Card, &mut CardZone), Without<CardReader>>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
    mut physics_commands: ResMut<EventBus<PhysicsCommand>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(drag_info) = drag_state.dragging.as_ref() else {
        return;
    };
    let card_entity = drag_info.entity;

    let Ok((mut card_transform, card, mut card_zone)) = cards.get_mut(card_entity) else {
        return;
    };
    let card_pos = card_transform.position;

    for (reader_entity, reader_transform, mut reader) in &mut readers {
        if reader.loaded.is_some() {
            continue;
        }
        if !card_overlaps_reader(card_pos, reader_transform.position, reader.half_extents) {
            continue;
        }

        card_transform.position = reader_transform.position;
        card_transform.rotation = 0.0;

        commands
            .entity(card_entity)
            .insert(engine_core::scale_spring::ScaleSpring::new(
                READER_CARD_SCALE,
            ));

        physics_commands.push(PhysicsCommand::RemoveBody {
            entity: card_entity,
        });
        commands.entity(card_entity).remove::<RigidBody>();

        *card_zone = CardZone::Reader(reader_entity);
        reader.loaded = Some(card_entity);

        if let Ok(mut jack) = jacks.get_mut(reader.jack_entity) {
            jack.data = Some(SignatureSpace::from_single(
                card.signature,
                SIGNATURE_SPACE_RADIUS,
            ));
        }

        drag_state.dragging = None;
        return;
    }
}
