use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBody {
    Dynamic,
    Static,
    Kinematic,
}
