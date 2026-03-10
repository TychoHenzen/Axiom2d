use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    Background,
    World,
    Characters,
    Foreground,
    UI,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SortOrder(pub i32);

#[cfg(test)]
mod tests {
    use super::*;

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
