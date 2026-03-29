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
    extra0: f32,
    extra1: f32,
    extra2: f32,
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
    if in.uv.x + in.uv.y < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Pointer direction relative to this fragment in world space
    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);
    let delta = pointer - in.world_pos;
    let dist = length(delta);

    // Project pointer direction onto UV diagonal for glint position
    let dir_norm = delta / max(dist, 1.0);
    let sweep_pos = dot(in.uv, vec2<f32>(0.6, 0.4)) + dot(dir_norm, vec2<f32>(0.5, 0.5)) * 0.3;
    let glint = exp(-pow((sweep_pos - 0.5) * 5.0, 2.0));

    // Secondary glint from the perpendicular direction
    let sweep2 = dot(in.uv, vec2<f32>(0.3, -0.7)) + dot(dir_norm, vec2<f32>(-0.5, 0.5)) * 0.3;
    let glint2 = exp(-pow((sweep2 - 0.5) * 6.0, 2.0));

    // Edge darkening for glossy/lacquered depth
    let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    let darken = smoothstep(0.0, 0.15, edge_dist);

    // Proximity boost — glints are brighter when pointer is close
    let proximity = 1.0 / (1.0 + dist * 0.005);

    let spec = max(glint, glint2 * 0.6) * proximity;

    let darkened = in.color.rgb * (0.7 + darken * 0.3);
    let result = darkened + vec3<f32>(spec * 0.9);

    return vec4<f32>(result, 0.6 + spec * 0.3);
}
