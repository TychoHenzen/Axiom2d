use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{TextureId, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::shape::ColorMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::art::ShapeRepository;
use crate::card::art_selection::select_art_for_signature;
use crate::card::component::Card;
use crate::card::component::CardLabel;
use crate::card::component::CardZone;
use crate::card::identity::base_type::BaseCardTypeRegistry;
use crate::card::identity::card_description::generate_card_description;
use crate::card::identity::card_name::generate_card_name;
use crate::card::identity::definition::CardDefinition;
use crate::card::identity::residual::ResidualStats;
use crate::card::identity::signature::CardSignature;
use crate::card::identity::signature_profile::SignatureProfile;
use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::interaction::physics_helpers::warn_on_physics_bool;
use crate::card::rendering::bake::{bake_back_face, bake_front_face};
use crate::card::rendering::baked_mesh::BakedCardMesh;

pub(crate) const CARD_CORNER_RADIUS: f32 = 5.0;

mod overlay;
mod text;

#[cfg(test)]
pub(crate) use overlay::build_gem_overlay;
pub(crate) use text::{TEXT_COLOR, fit_name_font_size};

pub fn spawn_visual_card(
    world: &mut World,
    def: &CardDefinition,
    position: Vec2,
    card_size: Vec2,
    face_up: bool,
    signature: CardSignature,
) -> Entity {
    let half = card_size * 0.5;
    let card = Card {
        face_texture: TextureId(0),
        back_texture: TextureId(0),
        face_up,
        signature,
    };
    let (profile, stats) = {
        let registry = world.get_resource::<BaseCardTypeRegistry>();
        let profile = registry.map_or_else(
            || SignatureProfile::without_archetype(&signature),
            |reg| SignatureProfile::new(&signature, reg),
        );
        let stats = registry
            .and_then(|reg| reg.best_match(&signature))
            .map(|base_type| ResidualStats::from_card(&signature, base_type));
        (profile, stats)
    };

    let card_name = generate_card_name(&profile, &signature);
    let description = stats
        .as_ref()
        .map(generate_card_description)
        .filter(|d| !d.is_empty())
        .unwrap_or(card_name.subtitle);
    let label = CardLabel {
        name: card_name.title,
        description,
    };

    let root = world
        .spawn((
            card,
            def.clone(),
            label.clone(),
            CardZone::Table,
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
            Collider::Aabb(half),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id();

    if let Some(mut physics) = world.get_resource_mut::<PhysicsRes>() {
        warn_on_physics_bool(
            "add_body",
            root,
            physics.add_body(root, &RigidBody::Dynamic, position),
        );
        warn_on_physics_bool(
            "add_collider",
            root,
            physics.add_collider(root, &Collider::Aabb(half)),
        );
        physics
            .set_damping(root, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG)
            .expect("freshly spawned card should have physics body");
    }

    if let Some(stats) = stats {
        world.entity_mut(root).insert(stats);
    }

    let art_shapes = world
        .get_resource::<ShapeRepository>()
        .and_then(|repo| select_art_for_signature(&signature, repo))
        .map(|entry| entry.shapes().to_vec());
    let baked = BakedCardMesh {
        front: bake_front_face(&card.signature, card_size, &label, art_shapes.as_deref()),
        back: bake_back_face(card_size),
    };
    let initial_mesh = if face_up {
        baked.front.clone()
    } else {
        baked.back.clone()
    };
    let mesh_overlays =
        overlay::build_mesh_overlays(world, card_size, &card.signature, face_up, &baked.front);
    world
        .entity_mut(root)
        .insert((baked, mesh_overlays, ColorMesh(initial_mesh)));

    root
}
