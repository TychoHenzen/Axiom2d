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

// Port of https://www.shadertoy.com/view/4syXRD
// Adapted to WGSL with per-card seed from model matrix translation.

const WAVYNESS: f32 = 0.12;
const SCALE: vec2<f32> = vec2<f32>(3.0, 3.0);
const LAYERS: i32 = 5;
const BASE_FREQUENCY: vec2<f32> = vec2<f32>(0.5, 0.5);
const FREQUENCY_STEP: vec2<f32> = vec2<f32>(0.25, 0.25);

fn rotate2d(p: vec2<f32>, a: f32) -> vec2<f32> {
    let sa = sin(a);
    let ca = cos(a);
    return vec2<f32>(ca * p.x + sa * p.y, -sa * p.x + ca * p.y);
}

fn scratch(uv_in: vec2<f32>, seed: vec2<f32>) -> f32 {
    let sx = floor(sin(seed.x * 51024.0) * 3104.0);
    let sy = floor(sin(seed.y * 1324.0) * 554.0);

    var uv = uv_in * 2.0 - 1.0;
    uv = rotate2d(uv, sx + sy);
    uv += sin(sx - sy);
    uv = clamp(uv * 0.5 + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));

    let s1 = sin(sx + uv.y * 3.1415) * WAVYNESS;
    let s2 = sin(sy + uv.y * 3.1415) * WAVYNESS;

    let x = sign(0.01 - abs(uv.x - 0.5 + s2 + s1));
    return clamp(((1.0 - pow(uv.y, 2.0)) * uv.y) * 2.5 * x, 0.0, 1.0);
}

fn scratch_layer(uv: vec2<f32>, frequency: vec2<f32>, offset: vec2<f32>, angle: f32) -> f32 {
    let rotated = rotate2d(uv, angle);
    let scaled = rotated * frequency + offset;
    return scratch(fract(scaled), floor(scaled));
}

fn scratches(uv: vec2<f32>, seed_offset: vec2<f32>) -> f32 {
    var p = uv * SCALE + seed_offset;
    var frequency = BASE_FREQUENCY;
    var result = 0.0;
    for (var i = 0; i < LAYERS; i++) {
        let fi = f32(i);
        result = max(result, scratch_layer(p, frequency, vec2<f32>(fi, fi), fi * 3145.0));
        frequency += FREQUENCY_STEP;
    }
    return clamp(result, 0.0, 1.0);
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // Per-card seed from signature hash, written into extra0 by the CPU.
    // Spread it into a 2D offset so each card gets unique scratch placement.
    let seed = art_params.extra0;
    let seed_offset = vec2<f32>(
        fract(sin(seed * 0.0001) * 43758.5453),
        fract(sin(seed * 0.0002) * 28461.7231),
    ) * 100.0;

    let s = scratches(in.uv, seed_offset);

    // Semi-transparent white scratch marks
    return vec4<f32>(0.9, 0.88, 0.82, s * 0.7);
}
