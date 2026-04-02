#![allow(clippy::unwrap_used)]

use engine_core::types::TextureId;
use engine_render::material::{BlendMode, Material2d, TextureBinding, effective_shader_handle};
use engine_render::shader::ShaderHandle;

#[test]
fn when_material2d_with_textures_and_uniforms_debug_formatted_then_snapshot_matches() {
    // Arrange
    let material = Material2d {
        blend_mode: BlendMode::Additive,
        shader: ShaderHandle(7),
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
        uniforms: vec![0, 128, 255],
    };

    // Act
    let debug = format!("{material:#?}");

    // Assert
    insta::assert_snapshot!(debug);
}

#[test]
fn when_blend_mode_variants_serialized_to_ron_then_each_deserializes_to_matching_variant() {
    for mode in BlendMode::ALL {
        let ron = ron::to_string(&mode).unwrap();
        let back: BlendMode = ron::from_str(&ron).unwrap();
        assert_eq!(mode, back);
    }
}

#[test]
fn when_comparing_blend_modes_then_alpha_less_than_additive_less_than_multiply() {
    // Arrange
    let alpha = BlendMode::Alpha;
    let additive = BlendMode::Additive;
    let multiply = BlendMode::Multiply;

    // Act / Assert
    assert!(alpha < additive);
    assert!(additive < multiply);
}

#[test]
fn when_effective_shader_handle_with_none_then_returns_default() {
    // Act
    let result = effective_shader_handle(None);

    // Assert
    assert_eq!(result, ShaderHandle(0));
}

#[test]
fn when_effective_shader_handle_with_some_then_returns_material_shader() {
    // Arrange
    let material = Material2d {
        shader: ShaderHandle(99),
        ..Material2d::default()
    };

    // Act
    let result = effective_shader_handle(Some(&material));

    // Assert
    assert_eq!(result, ShaderHandle(99));
}

#[test]
fn when_all_blend_mode_variants_call_index_then_each_returns_discriminant() {
    // Act / Assert
    assert_eq!(BlendMode::Alpha.index(), 0);
    assert_eq!(BlendMode::Additive.index(), 1);
    assert_eq!(BlendMode::Multiply.index(), 2);
}
