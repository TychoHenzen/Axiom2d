use bevy_ecs::prelude::Component;

use crate::card::identity::signature::CardSignature;

pub const SIGNATURE_SPACE_RADIUS: f32 = 0.2;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct SignatureSpace {
    pub center: CardSignature,
    pub radius: f32,
}

impl SignatureSpace {
    pub fn contains(&self, point: &CardSignature) -> bool {
        self.center.distance_to(point) <= self.radius
    }
}
