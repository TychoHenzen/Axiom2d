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

// Compute a discrete facet normal from UV position.
// The octagon is divided into 8 sectors radiating from center (0.5, 0.5).
// Each sector gets a constant outward-tilted normal, creating flat facet faces
// that catch light at distinct angles — the hallmark of a cut gemstone.
fn facet_normal(uv: vec2<f32>) -> vec3<f32> {
    let centered = uv - vec2<f32>(0.5, 0.5);
    let angle = atan2(centered.y, centered.x);

    // Quantize into one of 8 sectors (each 45 degrees / PI/4 radians)
    let sector = floor((angle + 3.14159265) / (3.14159265 / 4.0));
    let sector_angle = -3.14159265 + (sector + 0.5) * (3.14159265 / 4.0);

    // Each facet tilts outward from center at a fixed angle
    let tilt = 0.35;
    let facet_dir = vec2<f32>(cos(sector_angle), sin(sector_angle));
    return normalize(vec3<f32>(facet_dir * tilt, 1.0));
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.uv.x + in.uv.y < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let normal = facet_normal(in.uv);

    // Light direction from pointer position
    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);
    let delta = pointer - in.world_pos;
    let dist = length(delta);
    let light_xy = delta / max(dist, 1.0);
    let light_dir = normalize(vec3<f32>(light_xy * 0.7, 0.6));

    // Blinn-Phong specular
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 48.0);

    // Proximity falloff
    let proximity = 1.0 / (1.0 + dist * 0.004);

    // Distance from UV center — gems are brighter near edges (crown facets)
    let centered = in.uv - vec2<f32>(0.5, 0.5);
    let edge_boost = smoothstep(0.1, 0.45, length(centered));

    // Scale by tier specular intensity from uniform
    let intensity = art_params.extra0;
    let highlight = spec * proximity * intensity * (0.6 + edge_boost * 0.4);

    return vec4<f32>(1.0, 1.0, 1.0, highlight * 0.8);
}
