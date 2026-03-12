use engine_audio::prelude::AudioEmitter;
use engine_core::prelude::Transform2D;
use engine_physics::prelude::{Collider, RigidBody};
use engine_render::prelude::{BloomSettings, Camera2D, Material2d, Shape, Sprite};
use engine_scene::prelude::{RenderLayer, SortOrder, Visible};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SceneNodeDef {
    pub name: String,
    pub transform: Transform2D,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Visible>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_layer: Option<RenderLayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<Sprite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<Shape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<Camera2D>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rigid_body: Option<RigidBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collider: Option<Collider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material2d>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bloom_settings: Option<BloomSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_emitter: Option<AudioEmitter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneDef {
    pub nodes: Vec<SceneNodeDef>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use engine_core::color::Color;
    use engine_core::types::{Pixels, TextureId};
    use engine_render::prelude::{BlendMode, ShaderHandle, ShapeVariant, TextureBinding};
    use glam::Vec2;

    use super::*;

    fn minimal_node(name: &str) -> SceneNodeDef {
        SceneNodeDef {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    #[test]
    fn when_scene_node_def_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let node = SceneNodeDef {
            name: "player".to_owned(),
            transform: Transform2D {
                position: Vec2::new(100.0, 200.0),
                rotation: 0.5,
                scale: Vec2::ONE,
            },
            render_layer: Some(RenderLayer::Characters),
            sort_order: Some(SortOrder(5)),
            ..minimal_node("player")
        };

        // Act
        let ron = ron::to_string(&node).unwrap();
        let back: SceneNodeDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(node, back);
    }

    #[test]
    fn when_scene_node_with_none_sprite_serialized_then_roundtrips_as_none() {
        // Arrange
        let node = minimal_node("empty");

        // Act
        let ron = ron::to_string(&node).unwrap();
        let back: SceneNodeDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(back.sprite, None);
    }

    #[test]
    fn when_scene_node_with_some_sprite_serialized_then_roundtrips_with_matching_fields() {
        // Arrange
        let sprite = Sprite {
            texture: TextureId(3),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(32.0),
            height: Pixels(32.0),
        };
        let mut node = minimal_node("hero");
        node.sprite = Some(sprite);

        // Act
        let ron = ron::to_string(&node).unwrap();
        let back: SceneNodeDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(back.sprite, Some(sprite));
    }

    #[test]
    fn when_scene_def_with_parent_child_serialized_then_parent_index_is_preserved() {
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                minimal_node("parent"),
                SceneNodeDef {
                    parent_index: Some(0),
                    ..minimal_node("child")
                },
            ],
        };

        // Act
        let ron = ron::to_string(&scene).unwrap();
        let back: SceneDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(back.nodes.len(), 2);
        assert_eq!(back.nodes[0].parent_index, None);
        assert_eq!(back.nodes[1].parent_index, Some(0));
    }

    #[test]
    fn when_invalid_ron_deserialized_as_scene_def_then_returns_error() {
        // Arrange
        let bad_ron = "{ nodes: [ { broken }";

        // Act
        let result = ron::from_str::<SceneDef>(bad_ron);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn when_minimal_scene_def_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                minimal_node("parent"),
                SceneNodeDef {
                    parent_index: Some(0),
                    ..minimal_node("child")
                },
            ],
        };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_node_def_with_all_fields_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            name: "full".to_owned(),
            transform: Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 1.0,
                scale: Vec2::splat(2.0),
            },
            parent_index: Some(0),
            visible: Some(Visible(true)),
            render_layer: Some(RenderLayer::Foreground),
            sort_order: Some(SortOrder(3)),
            sprite: Some(Sprite {
                texture: TextureId(1),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::RED,
                width: Pixels(16.0),
                height: Pixels(16.0),
            }),
            shape: Some(Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::BLUE,
            }),
            camera: Some(Camera2D {
                position: Vec2::new(50.0, 50.0),
                zoom: 1.5,
            }),
            rigid_body: Some(RigidBody::Dynamic),
            collider: Some(Collider::Circle(5.0)),
            material: Some(Material2d::default()),
            bloom_settings: Some(BloomSettings::default()),
            audio_emitter: Some(AudioEmitter {
                volume: 0.9,
                max_distance: 300.0,
            }),
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_convex_polygon_collider_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            collider: Some(Collider::ConvexPolygon(vec![
                Vec2::new(0.0, 10.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(0.0, -10.0),
                Vec2::new(-10.0, 0.0),
            ])),
            ..minimal_node("diamond")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_shape_variants_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                SceneNodeDef {
                    shape: Some(Shape {
                        variant: ShapeVariant::Circle { radius: 25.0 },
                        color: Color::RED,
                    }),
                    ..minimal_node("circle_entity")
                },
                SceneNodeDef {
                    shape: Some(Shape {
                        variant: ShapeVariant::Polygon {
                            points: vec![
                                Vec2::new(0.0, 0.0),
                                Vec2::new(50.0, 0.0),
                                Vec2::new(25.0, 43.3),
                            ],
                        },
                        color: Color::GREEN,
                    }),
                    ..minimal_node("triangle_entity")
                },
            ],
        };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_audio_emitter_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            audio_emitter: Some(AudioEmitter {
                volume: 0.75,
                max_distance: 500.0,
            }),
            ..minimal_node("speaker")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_material_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            material: Some(Material2d {
                blend_mode: BlendMode::Additive,
                shader: ShaderHandle(3),
                textures: vec![
                    TextureBinding {
                        texture: TextureId(0),
                        binding: 0,
                    },
                    TextureBinding {
                        texture: TextureId(1),
                        binding: 1,
                    },
                ],
                uniforms: vec![64, 128, 255],
            }),
            ..minimal_node("glowing")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_def_with_all_component_types_roundtrips_then_all_fields_survive() {
        // Arrange
        let node = SceneNodeDef {
            name: "full".to_owned(),
            transform: Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 1.0,
                scale: Vec2::splat(2.0),
            },
            visible: Some(Visible(true)),
            render_layer: Some(RenderLayer::Foreground),
            sort_order: Some(SortOrder(3)),
            sprite: Some(Sprite {
                texture: TextureId(1),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::RED,
                width: Pixels(16.0),
                height: Pixels(16.0),
            }),
            shape: Some(Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::BLUE,
            }),
            camera: Some(Camera2D {
                position: Vec2::new(50.0, 50.0),
                zoom: 1.5,
            }),
            rigid_body: Some(RigidBody::Dynamic),
            collider: Some(Collider::Circle(5.0)),
            material: Some(Material2d::default()),
            bloom_settings: Some(BloomSettings::default()),
            audio_emitter: Some(AudioEmitter {
                volume: 0.9,
                max_distance: 300.0,
            }),
            ..Default::default()
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let ron = ron::to_string(&scene).unwrap();
        let back: SceneDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(scene, back);
    }
}
