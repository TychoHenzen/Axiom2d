use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderLayer {
    Background,
    World,
    Characters,
    Foreground,
    UI,
}

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct SortOrder(pub i32);

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_render_layer_variants_serialized_to_ron_then_each_deserializes_to_matching_variant() {
        for layer in [
            RenderLayer::Background,
            RenderLayer::World,
            RenderLayer::Characters,
            RenderLayer::Foreground,
            RenderLayer::UI,
        ] {
            let ron = ron::to_string(&layer).unwrap();
            let back: RenderLayer = ron::from_str(&ron).unwrap();
            assert_eq!(layer, back);
        }
    }

    #[test]
    fn when_sort_order_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let order = SortOrder(-42);

        // Act
        let ron = ron::to_string(&order).unwrap();
        let back: SortOrder = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(order, back);
    }

    #[test]
    fn when_render_layers_compared_then_background_less_than_world_less_than_characters_less_than_foreground_less_than_ui()
     {
        assert!(RenderLayer::Background < RenderLayer::World);
        assert!(RenderLayer::World < RenderLayer::Characters);
        assert!(RenderLayer::Characters < RenderLayer::Foreground);
        assert!(RenderLayer::Foreground < RenderLayer::UI);
    }

    #[test]
    fn when_sort_order_values_compared_then_lower_i32_value_sorts_before_higher() {
        assert!(SortOrder(-1) < SortOrder(1));
        assert!(SortOrder(i32::MIN) < SortOrder(i32::MAX));
    }

    #[test]
    fn when_entities_sorted_by_render_layer_and_sort_order_then_order_is_deterministic() {
        // Arrange
        let mut items = vec![
            (RenderLayer::World, SortOrder(1)),
            (RenderLayer::Background, SortOrder(0)),
            (RenderLayer::World, SortOrder(0)),
            (RenderLayer::UI, SortOrder(-1)),
        ];

        // Act
        items.sort();

        // Assert
        assert_eq!(
            items,
            vec![
                (RenderLayer::Background, SortOrder(0)),
                (RenderLayer::World, SortOrder(0)),
                (RenderLayer::World, SortOrder(1)),
                (RenderLayer::UI, SortOrder(-1)),
            ]
        );
    }
}
