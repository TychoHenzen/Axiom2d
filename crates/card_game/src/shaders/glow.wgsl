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
    // Fragments with valid UVs are art shape geometry — fully transparent
    if in.uv.x + in.uv.y > 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Distance from this fragment to the art region rectangle
    let art_center = vec2<f32>(0.0, art_params.offset_y);
    let art_half = vec2<f32>(art_params.half_w, art_params.half_h);
    let d = abs(in.local_pos - art_center) - art_half;
    let outside = max(d, vec2<f32>(0.0));
    let dist = length(outside);

    // Tight bloom: only visible within a few pixels of the art boundary
    let bloom_spread = 4.0;
    let bloom = exp(-dist * dist / (bloom_spread * bloom_spread));

    // Cut off anything too far from the art edge
    if bloom < 0.05 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Warm white glow
    let glow_color = mix(vec3<f32>(1.0, 0.95, 0.85), in.color.rgb, 0.3);

    return vec4<f32>(glow_color, bloom * 0.5);
}
