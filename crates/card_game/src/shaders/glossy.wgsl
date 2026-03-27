struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
};

struct ArtRegionParams {
    half_w: f32,
    half_h: f32,
    time: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> art_params: ArtRegionParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) uv: vec2<f32>,
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
    let world_pos = model.model * vec4<f32>(position, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    return out;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = art_params.time;

    if in.uv.x + in.uv.y < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Bright specular sweep across each shape's UV space.
    // A narrow band of light slides diagonally through each shape.
    let sweep_phase = fract(in.uv.x * 0.6 + in.uv.y * 0.4 - t * 0.25);
    let glint = exp(-pow((sweep_phase - 0.5) * 5.0, 2.0));

    // Second slower sweep in the other direction for sparkle
    let sweep2 = fract(in.uv.x * 0.3 - in.uv.y * 0.7 - t * 0.15);
    let glint2 = exp(-pow((sweep2 - 0.5) * 6.0, 2.0));

    // Shape edge darkening: gives depth, makes shapes look glossy/lacquered
    let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    let darken = smoothstep(0.0, 0.15, edge_dist);

    let spec = max(glint, glint2 * 0.6);

    // Darken the base slightly, then add bright white specular on top
    let darkened = in.color.rgb * (0.7 + darken * 0.3);
    let result = darkened + vec3<f32>(spec * 0.9);

    return vec4<f32>(result, 0.6 + spec * 0.3);
}
