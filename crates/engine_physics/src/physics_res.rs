use bevy_ecs::prelude::Resource;

use crate::physics_backend::PhysicsBackend;

#[derive(Resource)]
pub struct PhysicsRes(Box<dyn PhysicsBackend + Send + Sync>);

impl PhysicsRes {
    #[must_use]
    pub fn new(backend: Box<dyn PhysicsBackend + Send + Sync>) -> Self {
        Self(backend)
    }
}

impl std::ops::Deref for PhysicsRes {
    type Target = dyn PhysicsBackend;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl std::ops::DerefMut for PhysicsRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}
