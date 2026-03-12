use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBody {
    Dynamic,
    Static,
    Kinematic,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_rigid_body_variants_serialized_to_ron_then_each_deserializes_to_matching_variant() {
        for body in [RigidBody::Dynamic, RigidBody::Static, RigidBody::Kinematic] {
            let ron = ron::to_string(&body).unwrap();
            let back: RigidBody = ron::from_str(&ron).unwrap();
            assert_eq!(body, back);
        }
    }
}
