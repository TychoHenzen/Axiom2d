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

fn spectral_gaussian(x: f32, center: f32, width: f32) -> f32 {
    return exp(-((x - center) * (x - center)) / (width * width));
}

fn spectral_color(w: f32) -> vec3<f32> {
    let x = clamp((w - 400.0) / 300.0, 0.0, 1.0);
    let r = spectral_gaussian(x, 0.70, 0.18) + spectral_gaussian(x, 0.10, 0.12);
    let g = spectral_gaussian(x, 0.50, 0.16);
    let b = spectral_gaussian(x, 0.28, 0.15) + spectral_gaussian(x, 0.66, 0.08);
    return vec3<f32>(r, g, b);
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    let has_uv = step(0.001, in.uv.x + in.uv.y);

    if has_uv < 0.5 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Pointer direction relative to this fragment
    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);
    let delta = pointer - in.world_pos;
    let dist = length(delta);
    let dir_norm = delta / max(dist, 1.0);

    // Rainbow phase driven by viewing angle — shifts as pointer moves
    let angle = atan2(dir_norm.y, dir_norm.x);
    let sweep_dir = vec2<f32>(cos(angle), sin(angle));
    let phase = fract(dot(in.uv, sweep_dir) * 1.5 + angle * 0.15);
    let wavelength = 400.0 + phase * 300.0;
    let rainbow = spectral_color(wavelength);

    // UV edge shimmer at shape boundaries
    let uv_edge = length(fwidth(in.uv)) * 4.0;
    let edge_flash = smoothstep(0.3, 1.5, uv_edge);

    // Shape depth: brighter at edges
    let shape_depth = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y)) * 2.0;
    let rim = 1.0 - smoothstep(0.0, 0.3, shape_depth);

    // Proximity boost
    let proximity = 1.0 / (1.0 + dist * 0.005);

    let mix_str = (0.45 + edge_flash * 0.3 + rim * 0.1) * (0.6 + proximity * 0.4);
    let foiled = mix(in.color.rgb, rainbow, mix_str);

    let alpha = 0.5 + edge_flash * 0.3 + rim * 0.1;

    return vec4<f32>(foiled, alpha);
}
