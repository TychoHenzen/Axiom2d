// EVOLVE-BLOCK-START
// Booster pack opening animation state machine

use bevy_ecs::prelude::{Entity, Resource, World};
use engine_core::prelude::{EventBus, Transform2D};
use engine_core::time::DeltaTime;
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_render::camera::Camera2D;
use engine_scene::render_order::SortOrder;
use engine_scene::sort_propagation::LocalSortOrder;
use glam::Vec2;

use crate::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use crate::card::identity::signature::CardSignature;
use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card::rendering::geometry::TABLE_CARD_SIZE;
use crate::card::rendering::spawn_table_card::spawn_visual_card;

const MOVE_TO_CENTER_DURATION: f32 = 0.3;
const RIPPING_DURATION: f32 = 0.4;
const LOWERING_DURATION: f32 = 0.3;
const REVEAL_DURATION: f32 = 0.5;
const COMPLETING_DURATION: f32 = 0.5;
const FAN_ARC: f32 = std::f32::consts::PI * 0.6;
const FAN_RADIUS: f32 = 80.0;
const OPENING_SCALE: f32 = 3.0;
const FAN_MOVE_DURATION: f32 = 0.8;
const LOWERING_OFFSET: f32 = 200.0;
const CARD_SPAWN_BELOW: f32 = 200.0;
const CARD_REVEAL_ABOVE: f32 = 250.0;

#[derive(Debug, Clone)]
pub enum BoosterOpenPhase {
    MovingToCenter {
        start_pos: Vec2,
        progress: f32,
    },
    Ripping {
        progress: f32,
    },
    LoweringPack {
        progress: f32,
    },
    RevealingCards {
        card_index: usize,
        card_progress: f32,
    },
    Completing {
        progress: f32,
    },
    Done,
}

#[derive(Resource, Debug, Clone)]
pub struct BoosterOpening {
    pub pack_entity: Entity,
    pub phase: BoosterOpenPhase,
    pub cards: Vec<CardSignature>,
    pub original_position: Vec2,
    pub screen_center: Vec2,
    pub spawned_cards: Vec<Entity>,
    pub start_rotation: f32,
    pub camera_start_pos: Vec2,
    /// Per-card progress toward fan position (0.0–1.0). Pushed when a card's
    /// reveal completes and advanced every frame during RevealingCards/Completing.
    pub card_fan_progress: Vec<f32>,
}

impl BoosterOpening {
    #[must_use]
    pub fn new(
        pack_entity: Entity,
        cards: Vec<CardSignature>,
        original_position: Vec2,
        screen_center: Vec2,
        start_rotation: f32,
        camera_start_pos: Vec2,
    ) -> Self {
        Self {
            pack_entity,
            phase: BoosterOpenPhase::MovingToCenter {
                start_pos: original_position,
                progress: 0.0,
            },
            cards,
            original_position,
            screen_center,
            spawned_cards: Vec::new(),
            start_rotation,
            camera_start_pos,
            card_fan_progress: Vec::new(),
        }
    }

    /// Advance the state machine by dt seconds. Pure state transitions — no ECS side effects.
    pub fn advance(&mut self, dt: f32) {
        match &mut self.phase {
            BoosterOpenPhase::MovingToCenter { progress, .. } => {
                *progress += dt / MOVE_TO_CENTER_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::Ripping { progress: 0.0 };
                }
            }
            BoosterOpenPhase::Ripping { progress } => {
                *progress += dt / RIPPING_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::LoweringPack { progress: 0.0 };
                }
            }
            BoosterOpenPhase::LoweringPack { progress } => {
                *progress += dt / LOWERING_DURATION;
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::RevealingCards {
                        card_index: 0,
                        card_progress: 0.0,
                    };
                }
            }
            BoosterOpenPhase::RevealingCards {
                card_index,
                card_progress,
            } => {
                *card_progress += dt / REVEAL_DURATION;

                // Advance fan progress for already-revealed cards
                for p in &mut self.card_fan_progress {
                    *p = (*p + dt / FAN_MOVE_DURATION).min(1.0);
                }

                if *card_progress >= 1.0 {
                    // This card's upward reveal is done — start its fan movement
                    self.card_fan_progress.push(0.0);

                    let next_index = *card_index + 1;
                    if next_index >= self.cards.len() {
                        self.phase = BoosterOpenPhase::Completing { progress: 0.0 };
                    } else {
                        *card_index = next_index;
                        *card_progress = 0.0;
                    }
                }
            }
            BoosterOpenPhase::Completing { progress } => {
                *progress += dt / COMPLETING_DURATION;

                // Continue fan animation for all cards
                for p in &mut self.card_fan_progress {
                    *p = (*p + dt / FAN_MOVE_DURATION).min(1.0);
                }

                let all_fanned = self.card_fan_progress.iter().all(|p| *p >= 1.0);
                if *progress >= 1.0 && all_fanned {
                    self.phase = BoosterOpenPhase::Done;
                }
            }
            BoosterOpenPhase::Done => {}
        }
    }

    /// Compute the fan angle for a card at index out of total.
    fn fan_angle(index: usize, total: usize) -> f32 {
        if total <= 1 {
            0.0
        } else {
            let start = -FAN_ARC / 2.0;
            let step = FAN_ARC / (total - 1) as f32;
            start + step * index as f32
        }
    }

    /// Compute fan position for card at index out of total.
    #[must_use]
    pub fn fan_position(&self, index: usize, total: usize) -> Vec2 {
        let angle = Self::fan_angle(index, total);
        let offset = Vec2::new(angle.sin(), -angle.cos()) * FAN_RADIUS;
        self.original_position + offset
    }

    /// Compute the rotation a card should have at its fan position.
    #[must_use]
    pub fn fan_rotation(index: usize, total: usize) -> f32 {
        Self::fan_angle(index, total)
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        matches!(self.phase, BoosterOpenPhase::Done)
    }
}

fn placeholder_definition() -> CardDefinition {
    CardDefinition {
        card_type: CardType::Creature,
        name: String::new(),
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: String::new(),
        },
        art: art_descriptor_default(CardType::Creature),
    }
}

fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Pan the camera toward a target position.
fn pan_camera(world: &mut World, target: Vec2, start: Vec2, t: f32) {
    let pos = lerp(start, target, t);
    let cam_entity = {
        let mut q = world.query::<(Entity, &Camera2D)>();
        q.iter(world).next().map(|(e, _)| e)
    };
    if let Some(ce) = cam_entity
        && let Some(mut cam) = world.get_mut::<Camera2D>(ce)
    {
        cam.position = pos;
    }
}

/// Apply pack transform: position, rotation, scale.
fn set_pack_transform(world: &mut World, entity: Entity, pos: Vec2, rot: f32, scale: f32) {
    if let Some(mut t) = world.get_mut::<Transform2D>(entity) {
        t.position = pos;
        t.rotation = rot;
        t.scale = Vec2::splat(scale);
    }
}

/// Exclusive system that drives the booster opening animation.
///
/// Runs in `Phase::Animate`. Removes the `BoosterOpening` resource, advances
/// the state machine, applies visual effects, and re-inserts the resource
/// for all non-Done phases.
pub fn booster_opening_system(world: &mut World) {
    let dt = world.get_resource::<DeltaTime>().map_or(0.0, |dt| dt.0.0);

    let Some(mut opening) = world.remove_resource::<BoosterOpening>() else {
        return;
    };

    opening.advance(dt);

    match &opening.phase {
        BoosterOpenPhase::MovingToCenter {
            start_pos,
            progress,
        } => {
            let progress = *progress;
            let pos = lerp(*start_pos, opening.screen_center, progress);
            let rotation = lerp_f32(opening.start_rotation, 0.0, progress);
            let scale = lerp_f32(1.0, OPENING_SCALE, progress);

            set_pack_transform(world, opening.pack_entity, pos, rotation, scale);
            pan_camera(
                world,
                opening.screen_center,
                opening.camera_start_pos,
                progress,
            );
        }
        BoosterOpenPhase::Ripping { .. } => {
            // Pack stays at screen_center, zero rotation, full scale
            set_pack_transform(
                world,
                opening.pack_entity,
                opening.screen_center,
                0.0,
                OPENING_SCALE,
            );
        }
        BoosterOpenPhase::LoweringPack { progress } => {
            let base = opening.screen_center;
            let target = base + Vec2::new(0.0, LOWERING_OFFSET);
            let pos = lerp(base, target, *progress);
            set_pack_transform(world, opening.pack_entity, pos, 0.0, OPENING_SCALE);
        }
        BoosterOpenPhase::RevealingCards {
            card_index,
            card_progress,
        } => {
            let card_index = *card_index;
            let card_progress = *card_progress;
            let lowered_pos = opening.screen_center + Vec2::new(0.0, LOWERING_OFFSET);
            let card_spawn = lowered_pos + Vec2::new(0.0, CARD_SPAWN_BELOW);
            let card_reveal = lowered_pos - Vec2::new(0.0, CARD_REVEAL_ABOVE);
            let total_cards = opening.cards.len();

            // Keep pack at lowered position, full scale
            set_pack_transform(world, opening.pack_entity, lowered_pos, 0.0, OPENING_SCALE);

            // Spawn a new card entity when card_index advances past spawned_cards.len()
            if opening.spawned_cards.len() <= card_index {
                let signature = opening.cards[card_index];
                let def = placeholder_definition();
                let entity =
                    spawn_visual_card(world, &def, card_spawn, TABLE_CARD_SIZE, true, signature);

                // Remove physics — we'll re-add in the Done phase
                if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
                    bus.push(PhysicsCommand::RemoveBody { entity });
                }
                world.entity_mut(entity).remove::<RigidBody>();

                // Set sort order so cards render front-to-back in fan order
                world
                    .entity_mut(entity)
                    .insert(LocalSortOrder(500 + card_index as i32));

                if let Some(mut transform) = world.get_mut::<Transform2D>(entity) {
                    transform.scale = Vec2::splat(OPENING_SCALE);
                }

                opening.spawned_cards.push(entity);
            }

            // Animate the current card upward
            if let Some(&card_entity) = opening.spawned_cards.get(card_index) {
                let pos = lerp(card_spawn, card_reveal, card_progress);
                if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                    transform.position = pos;
                    transform.scale = Vec2::splat(OPENING_SCALE);
                }
            }

            // Animate already-revealed cards toward their fan positions
            for i in 0..card_index {
                if let (Some(&card_entity), Some(&fan_progress)) = (
                    opening.spawned_cards.get(i),
                    opening.card_fan_progress.get(i),
                ) {
                    let fan_target = opening.fan_position(i, total_cards);
                    let fan_rot = BoosterOpening::fan_rotation(i, total_cards);
                    let pos = lerp(card_reveal, fan_target, fan_progress);
                    let scale = lerp_f32(OPENING_SCALE, 1.0, fan_progress);
                    let rotation = lerp_f32(0.0, fan_rot, fan_progress);
                    if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                        transform.position = pos;
                        transform.scale = Vec2::splat(scale);
                        transform.rotation = rotation;
                    }
                }
            }
        }
        BoosterOpenPhase::Completing { progress } => {
            let total = opening.spawned_cards.len();
            let progress = *progress;
            let lowered_pos = opening.screen_center + Vec2::new(0.0, LOWERING_OFFSET);
            let card_reveal = lowered_pos - Vec2::new(0.0, CARD_REVEAL_ABOVE);

            // Continue animating all cards toward their fan positions
            for (i, &card_entity) in opening.spawned_cards.iter().enumerate() {
                let fan_progress = opening.card_fan_progress.get(i).copied().unwrap_or(0.0);
                let fan_target = opening.fan_position(i, total);
                let fan_rot = BoosterOpening::fan_rotation(i, total);
                let pos = lerp(card_reveal, fan_target, fan_progress);
                let scale = lerp_f32(OPENING_SCALE, 1.0, fan_progress);
                let rotation = lerp_f32(0.0, fan_rot, fan_progress);
                if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                    transform.position = pos;
                    transform.scale = Vec2::splat(scale);
                    transform.rotation = rotation;
                }
            }

            // Slide pack off-screen downward
            let pack_start = lowered_pos;
            let off_screen = opening.screen_center + Vec2::new(0.0, 600.0);
            let pack_pos = lerp(pack_start, off_screen, progress);
            set_pack_transform(world, opening.pack_entity, pack_pos, 0.0, OPENING_SCALE);
        }
        BoosterOpenPhase::Done => {
            let half = TABLE_CARD_SIZE * 0.5;
            let card_positions: Vec<(Entity, Vec2)> = opening
                .spawned_cards
                .iter()
                .map(|&e| {
                    let pos = world
                        .get::<Transform2D>(e)
                        .map_or(opening.original_position, |t| t.position);
                    (e, pos)
                })
                .collect();

            // Reset sort order, scale, and rotation on all cards
            for &(entity, _) in &card_positions {
                if let Some(mut transform) = world.get_mut::<Transform2D>(entity) {
                    transform.scale = Vec2::ONE;
                    transform.rotation = 0.0;
                }
                let mut em = world.entity_mut(entity);
                em.insert(SortOrder::default());
                em.remove::<LocalSortOrder>();
            }

            // Give all spawned cards physics bodies
            if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
                for (card_entity, position) in &card_positions {
                    bus.push(PhysicsCommand::AddBody {
                        entity: *card_entity,
                        body_type: RigidBody::Dynamic,
                        position: *position,
                    });
                    bus.push(PhysicsCommand::AddCollider {
                        entity: *card_entity,
                        collider: Collider::Aabb(half),
                    });
                    bus.push(PhysicsCommand::SetDamping {
                        entity: *card_entity,
                        linear: BASE_LINEAR_DRAG,
                        angular: BASE_ANGULAR_DRAG,
                    });
                    bus.push(PhysicsCommand::SetCollisionGroup {
                        entity: *card_entity,
                        membership: CARD_COLLISION_GROUP,
                        filter: CARD_COLLISION_FILTER,
                    });
                }
            }

            for &(card_entity, _) in &card_positions {
                world.entity_mut(card_entity).insert(RigidBody::Dynamic);
            }

            world.despawn(opening.pack_entity);

            return;
        }
    }

    world.insert_resource(opening);
}
// EVOLVE-BLOCK-END
