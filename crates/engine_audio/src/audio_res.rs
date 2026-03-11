use bevy_ecs::prelude::Resource;

use crate::audio_backend::AudioBackend;

#[derive(Resource)]
pub struct AudioRes(Box<dyn AudioBackend + Send + Sync>);

impl AudioRes {
    #[must_use]
    pub fn new(backend: Box<dyn AudioBackend + Send + Sync>) -> Self {
        Self(backend)
    }
}

impl std::ops::Deref for AudioRes {
    type Target = dyn AudioBackend;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl std::ops::DerefMut for AudioRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}
