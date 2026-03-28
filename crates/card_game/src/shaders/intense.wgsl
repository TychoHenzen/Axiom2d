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

// Holographic liquid shimmer — adapted from Shadertoy.
// Uses world_pos so the pattern is pinned to world space: rotating the card
// shifts which part of the holographic field is visible, like tilting a foil card.

const PI: f32 = 3.14159265359;

fn rot2(a: f32) -> mat2x2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c, s, -s, c);
}

// Dave Hoskins hash
fn hash4(p_in: vec2<f32>) -> vec4<f32> {
    var p4 = fract(vec4<f32>(p_in.x, p_in.y, p_in.x, p_in.y) * vec4<f32>(0.1031, 0.1030, 0.0973, 0.1099));
    p4 += dot(p4, p4.wzxy + 19.19);
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

// Value noise (4-channel)
fn noise4(p_in: vec2<f32>) -> vec4<f32> {
    let p = p_in * 200.0;
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash4(i + vec2<f32>(0.0, 0.0)), hash4(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash4(i + vec2<f32>(0.0, 1.0)), hash4(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y,
    );
}

// Domain-warped noise producing liquid distortion
fn liquid(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    p = rot2(0.1) * p * vec2<f32>(2.5, 0.5);
    p += noise4(p).rg * 0.01;
    p = rot2(0.2) * p * 2.5;
    p += noise4(p).ba * 0.01;
    p *= 6.5;
    p += noise4(p).rg * 0.005;
    return noise4(p * 0.1).a;
}

// Height for normal computation
fn height(p: vec3<f32>) -> f32 {
    return p.z - liquid(p.xy) * 0.001;
}

// Normal from central differences
fn calc_normal(uv: vec2<f32>) -> vec3<f32> {
    let e = 0.0001;
    let p = vec3<f32>(uv, 0.0);
    return normalize(vec3<f32>(
        height(p - vec3<f32>(e, 0.0, 0.0)) - height(p + vec3<f32>(e, 0.0, 0.0)),
        height(p - vec3<f32>(0.0, e, 0.0)) - height(p + vec3<f32>(0.0, e, 0.0)),
        height(p - vec3<f32>(0.0, 0.0, e)) - height(p + vec3<f32>(0.0, 0.0, e)),
    ));
}

// Rainbow cubemap — shifted hues for holographic look
fn cubemap(dir: vec3<f32>) -> vec3<f32> {
    var color = cos(dir * vec3<f32>(1.0, 9.0, 2.0) + vec3<f32>(2.0, 3.0, 1.0)) * 0.5 + 0.5;
    color = color * vec3<f32>(0.8, 0.3, 0.7) + vec3<f32>(0.2);
    color *= dir.y * 0.5 + 0.5;
    color += exp(6.0 * dir.y - 2.0) * 0.05;
    color = pow(color, vec3<f32>(1.0 / 2.2));
    return color;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // World-space coordinates scaled to noise domain.
    // Because this uses world_pos (not UV), the holographic pattern is pinned
    // to the world. Rotating the card reveals different parts of the pattern,
    // like tilting a real holographic card under a light.
    let wp = in.world_pos * 0.005;

    let norm = calc_normal(wp * 0.02);
    var dir = normalize(vec3<f32>(wp, 0.2));
    dir = reflect(dir, norm);

    // Fixed world-angle rotation — keeps highlight at a consistent direction
    dir = vec3<f32>(rot2(0.8) * dir.xz, dir.y).xzy;

    var color = cubemap(dir);
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    // Vignette: fade toward card edges for a natural look
    let centered = in.uv - 0.5;
    let vignette = 1.0 - dot(centered, centered) * 2.0;

    // Faint overlay — just enough to catch the eye
    let alpha = clamp(vignette * 0.4, 0.0, 0.4);

    return vec4<f32>(color, alpha);
}
