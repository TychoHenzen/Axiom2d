use bevy_ecs::prelude::Component;

use crate::types::Pixels;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: Pixels,
    pub y: Pixels,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    pub dx: Pixels,
    pub dy: Pixels,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_position_constructed_then_stores_values_unchanged() {
        // Act
        let pos = Position {
            x: Pixels(100.0),
            y: Pixels(200.0),
        };

        // Assert
        assert_eq!(pos.x, Pixels(100.0));
        assert_eq!(pos.y, Pixels(200.0));
    }

    #[test]
    fn when_position_spawned_then_queryable_from_world() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();

        // Act
        world.spawn(Position {
            x: Pixels(0.0),
            y: Pixels(0.0),
        });

        // Assert
        assert_eq!(world.query::<&Position>().iter(&world).count(), 1);
    }

    #[test]
    fn when_velocity_constructed_then_stores_values_unchanged() {
        // Act
        let vel = Velocity {
            dx: Pixels(4.0),
            dy: Pixels(-3.0),
        };

        // Assert
        assert_eq!(vel.dx, Pixels(4.0));
        assert_eq!(vel.dy, Pixels(-3.0));
    }

    #[test]
    fn when_velocity_spawned_then_queryable_from_world() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();

        // Act
        world.spawn(Velocity {
            dx: Pixels(1.0),
            dy: Pixels(2.0),
        });

        // Assert
        assert_eq!(world.query::<&Velocity>().iter(&world).count(), 1);
    }
}
