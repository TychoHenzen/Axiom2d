use bevy_ecs::prelude::Resource;
use engine_render::prelude::{ShaderHandle, ShaderRegistry};

pub const UV_GRADIENT_WGSL: &str = r"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
};

@vertex
fn vs_shape(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.local_pos = position;
    let world_pos = model.model * vec4<f32>(position, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    return out;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // Art rect half-extents: half_w = CARD_WIDTH * 0.45, half_h = 22.5
    // local_pos ranges from (-half_w, -half_h) to (half_w, half_h)
    let half_size = vec2<f32>(27.0, 22.5);
    let uv = in.local_pos / (half_size * 2.0) + vec2<f32>(0.5, 0.5);
    return vec4<f32>(uv.x, uv.y, 0.4, 1.0);
}
";

#[derive(Resource, Debug, Clone, Copy)]
pub struct CardArtShader(pub ShaderHandle);

pub fn register_card_art_shader(registry: &mut ShaderRegistry) -> CardArtShader {
    CardArtShader(registry.register(UV_GRADIENT_WGSL))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_registering_card_art_shader_then_handle_is_retrievable() {
        // Arrange
        let mut registry = ShaderRegistry::new();

        // Act
        let art_shader = register_card_art_shader(&mut registry);

        // Assert
        assert_eq!(registry.lookup(art_shader.0), Some(UV_GRADIENT_WGSL));
    }

    #[test]
    fn when_uv_gradient_shader_parsed_with_naga_then_no_error() {
        // Act
        let result = naga::front::wgsl::parse_str(UV_GRADIENT_WGSL);

        // Assert
        assert!(result.is_ok(), "WGSL parse error: {result:?}");
    }

    #[test]
    fn when_uv_gradient_shader_source_inspected_then_camera_and_model_uniforms_declared() {
        // Assert
        assert!(
            UV_GRADIENT_WGSL.contains("@group(0) @binding(0)"),
            "shader must declare camera uniform at group(0) binding(0)"
        );
        assert!(
            UV_GRADIENT_WGSL.contains("@group(1) @binding(0)"),
            "shader must declare model uniform at group(1) binding(0)"
        );
    }

    #[test]
    fn when_uv_gradient_shader_source_inspected_then_vertex_inputs_at_location0_and_location1() {
        // Assert
        assert!(
            UV_GRADIENT_WGSL.contains("@location(0)"),
            "shader must accept position at @location(0)"
        );
        assert!(
            UV_GRADIENT_WGSL.contains("@location(1)"),
            "shader must accept color at @location(1)"
        );
    }

    #[test]
    fn when_registering_card_art_shader_twice_then_handles_differ() {
        // Arrange
        let mut registry = ShaderRegistry::new();

        // Act
        let first = register_card_art_shader(&mut registry);
        let second = register_card_art_shader(&mut registry);

        // Assert
        assert_ne!(first.0, second.0);
    }
}
