// Booster pack opening animation state machine

use bevy_ecs::prelude::{Entity, Resource, World};
use engine_core::prelude::{EventBus, Transform2D};
use engine_core::time::DeltaTime;
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
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
const COMPLETING_DURATION: f32 = 0.3;
const FAN_ARC: f32 = std::f32::consts::PI * 0.6;
const FAN_RADIUS: f32 = 80.0;

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
}

impl BoosterOpening {
    #[must_use]
    pub fn new(
        pack_entity: Entity,
        cards: Vec<CardSignature>,
        original_position: Vec2,
        screen_center: Vec2,
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
                if *card_progress >= 1.0 {
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
                if *progress >= 1.0 {
                    self.phase = BoosterOpenPhase::Done;
                }
            }
            BoosterOpenPhase::Done => {}
        }
    }

    /// Compute fan position for card at index out of total.
    #[must_use]
    pub fn fan_position(&self, index: usize, total: usize) -> Vec2 {
        let angle = if total <= 1 {
            0.0
        } else {
            let start = -FAN_ARC / 2.0;
            let step = FAN_ARC / (total - 1) as f32;
            start + step * index as f32
        };
        // Fan out from original_position; angle 0 = directly above
        let offset = Vec2::new(angle.sin(), -angle.cos()) * FAN_RADIUS;
        self.original_position + offset
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
            let pos = lerp(*start_pos, opening.screen_center, *progress);
            if let Some(mut transform) = world.get_mut::<Transform2D>(opening.pack_entity) {
                transform.position = pos;
            }
        }
        BoosterOpenPhase::Ripping { .. } => {
            // Visual ripping effect — pack stays at center, could add shake/scale later
        }
        BoosterOpenPhase::LoweringPack { progress } => {
            let base = opening.screen_center;
            let target = base + Vec2::new(0.0, 200.0);
            let pos = lerp(base, target, *progress);
            if let Some(mut transform) = world.get_mut::<Transform2D>(opening.pack_entity) {
                transform.position = pos;
            }
        }
        BoosterOpenPhase::RevealingCards {
            card_index,
            card_progress,
        } => {
            let card_index = *card_index;
            let card_progress = *card_progress;

            // Spawn a new card entity when card_index advances past spawned_cards.len()
            if opening.spawned_cards.len() <= card_index {
                let signature = opening.cards[card_index];
                let def = placeholder_definition();
                let spawn_pos = opening.screen_center + Vec2::new(0.0, 50.0);
                let entity =
                    spawn_visual_card(world, &def, spawn_pos, TABLE_CARD_SIZE, true, signature);

                // Remove the physics body that spawn_visual_card adds — we'll re-add
                // physics in the Done phase after the animation completes.
                if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
                    bus.push(PhysicsCommand::RemoveBody { entity });
                }

                opening.spawned_cards.push(entity);
            }

            // Animate the current card upward from below center
            if let Some(&card_entity) = opening.spawned_cards.get(card_index) {
                let start = opening.screen_center + Vec2::new(0.0, 50.0);
                let end = opening.screen_center + Vec2::new(0.0, -60.0);
                let pos = lerp(start, end, card_progress);
                if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                    transform.position = pos;
                }
            }
        }
        BoosterOpenPhase::Completing { progress } => {
            let total = opening.spawned_cards.len();
            let progress = *progress;

            // Move spawned cards toward their fan positions
            for (i, &card_entity) in opening.spawned_cards.iter().enumerate() {
                let current_pos = world
                    .get::<Transform2D>(card_entity)
                    .map_or(opening.screen_center, |t| t.position);
                let fan_target = opening.fan_position(i, total);
                let pos = lerp(current_pos, fan_target, progress);
                if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
                    transform.position = pos;
                }
            }

            // Slide pack off-screen downward
            if let Some(mut transform) = world.get_mut::<Transform2D>(opening.pack_entity) {
                let off_screen = opening.screen_center + Vec2::new(0.0, 600.0);
                transform.position = lerp(transform.position, off_screen, progress);
            }
        }
        BoosterOpenPhase::Done => {
            // Collect card positions before borrowing the EventBus
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

            // Despawn the pack entity
            world.despawn(opening.pack_entity);

            // Do NOT re-insert the resource — animation is complete
            return;
        }
    }

    // Re-insert the resource for all non-Done phases
    world.insert_resource(opening);
}
