struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
};

struct ArtRegionParams {
    half_w: f32,
    half_h: f32,
    pointer_x: f32,
    pointer_y: f32,
    offset_y: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> art_params: ArtRegionParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_pos: vec2<f32>,
};

@vertex
fn vs_shape(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.local_pos = position;
    out.uv = uv;
    let wp = model.model * vec4<f32>(position, 0.0, 1.0);
    out.world_pos = wp.xy;
    out.position = camera.view_proj * wp;
    out.color = color;
    return out;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // Only render on art fragments
    if in.uv.x + in.uv.y < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Pointer direction relative to this fragment
    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);
    let delta = pointer - in.world_pos;
    let dist = length(delta);
    let dir_norm = delta / max(dist, 1.0);

    // Thin-film interference: phase varies across UV surface based on pointer angle
    let angle = atan2(dir_norm.y, dir_norm.x);
    let phase = dot(in.uv, vec2<f32>(cos(angle), sin(angle)));

    // Subtle hue rotation — shift the base color through a small chromatic range
    let shift = sin(phase * 6.2832) * 0.15;
    let shifted = vec3<f32>(
        in.color.r + shift,
        in.color.g + shift * 0.7,
        in.color.b - shift,
    );

    // Edge iridescence: stronger color shift near UV boundaries
    let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    let edge_boost = 1.0 - smoothstep(0.0, 0.25, edge_dist);
    let edge_shift = sin((phase + 0.5) * 6.2832) * 0.2 * edge_boost;

    let result = vec3<f32>(
        shifted.r + edge_shift * 0.8,
        shifted.g - edge_shift * 0.3,
        shifted.b + edge_shift,
    );

    // Proximity boost — effect intensifies when pointer is close
    let proximity = 1.0 / (1.0 + dist * 0.005);
    let strength = 0.4 + proximity * 0.6;

    let final_color = mix(in.color.rgb, result, strength);
    return vec4<f32>(final_color, 0.55);
}
