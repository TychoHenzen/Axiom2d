use bevy_ecs::entity::Entity;
use engine_core::prelude::Event;

use crate::mixer::MixerTrack;

/// Per-channel gain factors computed by the spatial audio system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialGains {
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone)]
pub struct PlaySound {
    pub name: String,
    pub track: MixerTrack,
    pub emitter: Option<Entity>,
    pub spatial_gains: Option<SpatialGains>,
}

impl Event for PlaySound {}

impl PlaySound {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            track: MixerTrack::Sfx,
            emitter: None,
            spatial_gains: None,
        }
    }

    pub fn on_track(name: impl Into<String>, track: MixerTrack) -> Self {
        Self {
            name: name.into(),
            track,
            emitter: None,
            spatial_gains: None,
        }
    }

    pub fn at_emitter(name: impl Into<String>, emitter: Entity) -> Self {
        Self {
            name: name.into(),
            track: MixerTrack::Sfx,
            emitter: Some(emitter),
            spatial_gains: None,
        }
    }
}
