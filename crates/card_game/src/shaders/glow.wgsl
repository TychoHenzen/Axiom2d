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

// Integer-based hash for cell orientation — avoids sin() precision issues
fn hash21(p: vec2<f32>) -> f32 {
    var n = u32(p.x * 73.0 + p.y * 157.0);
    n = n ^ (n >> 13u);
    n = n * 0x5bd1e995u;
    n = n ^ (n >> 15u);
    return f32(n & 0xFFFFu) / 65535.0;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // Only render on art fragments (UV guard)
    if in.uv.x + in.uv.y < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Sparkle point field: UV divided into grid cells, each with a random facet normal.
    // Pointer acts as a point light — different cells catch the light at different angles,
    // producing scattered glinting like glitter or confetti foil.
    let scale = 7.0;
    let cell = floor(in.uv * scale);
    let cell_frac = fract(in.uv * scale);

    // Per-cell random facet orientation from integer hash
    let phase = hash21(cell);
    let cell_angle = phase * 6.28318;
    let facet_n = normalize(vec3<f32>(cos(cell_angle) * 0.45, sin(cell_angle) * 0.45, 1.0));

    // Light direction from pointer
    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);
    let delta = pointer - in.world_pos;
    let dist = length(delta);
    let light_xy = delta / max(dist, 1.0);
    let light_dir = normalize(vec3<f32>(light_xy * 0.8, 0.6));

    // Blinn-Phong specular per cell
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(facet_n, half_vec), 0.0), 80.0);

    // Circular mask: sparkle visible only near cell center (round dot, not square)
    let to_center = cell_frac - vec2<f32>(0.5);
    let spot = smoothstep(0.45, 0.2, length(to_center));

    // Proximity boost — sparkles brighter when pointer is close
    let proximity = 1.0 / (1.0 + dist * 0.005);

    let sparkle = spec * spot * proximity;
    return vec4<f32>(1.0, 1.0, 1.0, clamp(sparkle * 0.9, 0.0, 0.9));
}
