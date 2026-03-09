use bevy_ecs::prelude::Resource;
use engine_core::types::Pixels;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct WindowSize {
    pub width: Pixels,
    pub height: Pixels,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: Pixels(0.0),
            height: Pixels(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_default_then_zero_by_zero() {
        // Act
        let size = WindowSize::default();

        // Assert
        assert_eq!(size.width, Pixels(0.0));
        assert_eq!(size.height, Pixels(0.0));
    }

    #[test]
    fn when_constructed_with_dimensions_then_stores_values_unchanged() {
        // Act
        let size = WindowSize {
            width: Pixels(1280.0),
            height: Pixels(720.0),
        };

        // Assert
        assert_eq!(size.width, Pixels(1280.0));
        assert_eq!(size.height, Pixels(720.0));
    }

    #[test]
    fn when_inserted_into_world_then_retrievable_as_resource() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();

        // Act
        world.insert_resource(WindowSize::default());

        // Assert
        let _res = world.resource::<WindowSize>();
    }
}
