use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionName(pub String);

#[derive(Resource, Debug, Clone, Default)]
pub struct ActionMap {
    bindings: HashMap<ActionName, Vec<KeyCode>>,
}

impl ActionMap {
    pub fn bind(&mut self, action: &str, keys: Vec<KeyCode>) {
        self.bindings.insert(ActionName(action.to_string()), keys);
    }

    pub fn bindings_for(&self, action: &str) -> &[KeyCode] {
        self.bindings
            .get(&ActionName(action.to_string()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_multiple_keys_bound_to_same_action_then_all_keys_returned() {
        // Arrange
        let mut map = ActionMap::default();

        // Act
        map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

        // Assert
        assert_eq!(
            map.bindings_for("move_right"),
            &[KeyCode::ArrowRight, KeyCode::KeyD]
        );
    }

    #[test]
    fn when_single_key_bound_to_action_then_bindings_for_returns_that_key() {
        // Arrange
        let mut map = ActionMap::default();

        // Act
        map.bind("jump", vec![KeyCode::Space]);

        // Assert
        assert_eq!(map.bindings_for("jump"), &[KeyCode::Space]);
    }

    #[test]
    fn when_action_map_inserted_into_world_then_retrievable_as_resource() {
        // Arrange
        let mut world = bevy_ecs::prelude::World::new();

        // Act
        world.insert_resource(ActionMap::default());

        // Assert
        assert!(world.get_resource::<ActionMap>().is_some());
    }

    #[test]
    fn when_action_map_default_then_no_bindings_exist() {
        // Arrange / Act
        let map = ActionMap::default();

        // Assert
        assert!(map.bindings_for("jump").is_empty());
    }

    #[test]
    fn when_action_name_cloned_then_clone_eq_and_debug_hold() {
        // Arrange
        let a = ActionName("move_left".to_string());

        // Act
        let b = a.clone();

        // Assert
        assert_eq!(a, b);
        assert_eq!(format!("{:?}", a), "ActionName(\"move_left\")");
    }

    #[test]
    fn when_action_name_and_action_map_exported_from_prelude_then_importable() {
        use crate::prelude::{ActionMap, ActionName};
        let _name = ActionName("test".to_string());
        let _map = ActionMap::default();
    }

    #[test]
    fn when_constructed_from_str_then_stores_name() {
        // Act
        let action = ActionName("jump".to_string());

        // Assert
        assert_eq!(action.0, "jump");
    }
}
