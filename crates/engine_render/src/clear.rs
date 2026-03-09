use bevy_ecs::prelude::{Res, ResMut, Resource};
use engine_core::color::Color;

use crate::renderer::RendererRes;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ClearColor(pub Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(Color::new(0.392, 0.584, 0.929, 1.0))
    }
}

pub fn clear_system(color: Res<ClearColor>, mut renderer: ResMut<RendererRes>) {
    renderer.clear(color.0);
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine_core::color::Color;

    use super::*;
    use crate::renderer::RendererRes;
    use crate::testing::SpyRenderer;

    #[test]
    fn when_constructed_with_color_then_stores_color_unchanged() {
        // Arrange
        let color = Color::new(0.1, 0.5, 0.9, 1.0);

        // Act
        let clear = ClearColor(color);

        // Assert
        assert_eq!(clear.0, color);
    }

    #[test]
    fn when_default_then_color_is_cornflower_blue() {
        // Act
        let clear = ClearColor::default();

        // Assert
        assert_eq!(clear.0, Color::new(0.392, 0.584, 0.929, 1.0));
    }

    #[test]
    fn when_inserted_into_world_then_retrievable_as_resource() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();

        // Act
        world.insert_resource(ClearColor::default());

        // Assert
        let _res = world.resource::<ClearColor>();
    }

    #[test]
    fn when_clear_system_runs_then_renderer_clear_receives_clear_color_value() {
        // Arrange
        let expected_color = Color::new(0.1, 0.2, 0.3, 1.0);
        let log = Arc::new(Mutex::new(Vec::new()));
        let color_capture = Arc::new(Mutex::new(None));
        let spy = SpyRenderer::with_color_capture(log.clone(), color_capture.clone());

        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(ClearColor(expected_color));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(clear_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(*color_capture.lock().unwrap(), Some(expected_color));
    }
}
