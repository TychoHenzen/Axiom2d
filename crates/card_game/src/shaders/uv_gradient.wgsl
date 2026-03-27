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
    let half_size = vec2<f32>(art_params.half_w, art_params.half_h);
    let uv = in.local_pos / (half_size * 2.0) + vec2<f32>(0.5, 0.5);
    let center = vec2<f32>(0.5, 0.5);
    let dist = length((uv - center) / vec2<f32>(0.5, 0.5));
    let edge = smoothstep(0.6, 1.0, dist);
    return vec4<f32>(0.0, 0.0, 0.0, edge * 0.5);
}
