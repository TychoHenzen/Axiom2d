use std::collections::HashMap;

use bevy_ecs::resource::Resource;

use super::effect::SoundEffect;

#[derive(Resource, Default)]
pub struct SoundLibrary {
    effects: HashMap<String, SoundEffect>,
}

impl SoundLibrary {
    pub fn register(&mut self, name: &str, effect: SoundEffect) {
        self.effects.insert(name.to_owned(), effect);
    }

    pub fn get(&self, name: &str) -> Option<&SoundEffect> {
        self.effects.get(name)
    }
}
