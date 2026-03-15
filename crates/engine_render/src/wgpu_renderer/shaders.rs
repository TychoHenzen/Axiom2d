pub(crate) const SHADER_SRC: &str = "
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    @location(0) quad_pos: vec2<f32>,
    @location(1) world_rect: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let x = quad_pos.x * world_rect.z + world_rect.x;
    let y = quad_pos.y * world_rect.w + world_rect.y;
    let world_pos = vec4<f32>(x, y, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    out.uv = vec2<f32>(
        mix(uv_rect.x, uv_rect.z, quad_pos.x),
        mix(uv_rect.y, uv_rect.w, quad_pos.y),
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex_color * in.color;
}
";

pub(crate) const SHAPE_SHADER_SRC: &str = "
struct ShapeOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_shape(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> ShapeOutput {
    var out: ShapeOutput;
    let world_pos = vec4<f32>(position, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    return out;
}

@fragment
fn fs_shape(in: ShapeOutput) -> @location(0) vec4<f32> {
    return in.color;
}
";

pub(super) const BLOOM_PREAMBLE: &str = "
struct FullscreenOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BloomParams {
    threshold: f32,
    intensity: f32,
    direction: vec2<f32>,
    texel_size: vec2<f32>,
    _pad: vec2<f32>,
};

@vertex
fn vs_fullscreen(@location(0) position: vec2<f32>) -> FullscreenOutput {
    var out: FullscreenOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, -position.y * 0.5 + 0.5);
    return out;
}
";

pub(super) const BLOOM_SHADER_FRAG: &str = "
@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

@fragment
fn fs_brightness(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_input, s_input, in.uv);
    let luminance = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (luminance > params.threshold) {
        return vec4<f32>(color.rgb, 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_blur(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let offset = params.direction * params.texel_size;
    var result = textureSample(t_input, s_input, in.uv) * 0.227027;
    result += textureSample(t_input, s_input, in.uv + offset) * 0.1945946;
    result += textureSample(t_input, s_input, in.uv - offset) * 0.1945946;
    result += textureSample(t_input, s_input, in.uv + offset * 2.0) * 0.1216216;
    result += textureSample(t_input, s_input, in.uv - offset * 2.0) * 0.1216216;
    result += textureSample(t_input, s_input, in.uv + offset * 3.0) * 0.054054;
    result += textureSample(t_input, s_input, in.uv - offset * 3.0) * 0.054054;
    result += textureSample(t_input, s_input, in.uv + offset * 4.0) * 0.016216;
    result += textureSample(t_input, s_input, in.uv - offset * 4.0) * 0.016216;
    return result;
}
";

pub(super) const COMPOSITE_SHADER_FRAG: &str = "
@group(0) @binding(0) var t_scene: texture_2d<f32>;
@group(0) @binding(1) var t_bloom: texture_2d<f32>;
@group(0) @binding(2) var s_composite: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

@fragment
fn fs_composite(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(t_scene, s_composite, in.uv);
    let bloom = textureSample(t_bloom, s_composite, in.uv);
    return vec4<f32>(scene.rgb + bloom.rgb * params.intensity, scene.a);
}
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_shape_shader_parsed_then_no_error() {
        // Act
        let result = naga::front::wgsl::parse_str(SHAPE_SHADER_SRC);

        // Assert
        assert!(result.is_ok(), "WGSL parse error: {result:?}");
    }
}
