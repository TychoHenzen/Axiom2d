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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use fundsp::hacker32::*;
    use fundsp::prelude::AudioUnit;

    use super::*;

    fn test_effect() -> SoundEffect {
        SoundEffect::new(|| Box::new(dc(0.5)) as Box<dyn AudioUnit>)
    }

    #[test]
    fn when_empty_library_then_get_returns_none() {
        // Arrange
        let library = SoundLibrary::default();

        // Act
        let result = library.get("explosion");

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_registered_then_get_with_same_name_returns_some() {
        // Arrange
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());

        // Act
        let result = library.get("beep");

        // Assert
        assert!(result.is_some());
    }

    #[test]
    fn when_registered_then_get_with_different_name_returns_none() {
        // Arrange
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());

        // Act
        let result = library.get("boom");

        // Assert
        assert!(result.is_none());
    }
}
