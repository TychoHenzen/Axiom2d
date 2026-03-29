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

    // Analytical bevel normal: compute slope direction toward nearest UV edge.
    // Distances to each edge of the UV [0,1] square.
    let dl = in.uv.x;         // left
    let dr = 1.0 - in.uv.x;  // right
    let db = in.uv.y;         // bottom
    let dt = 1.0 - in.uv.y;  // top

    // Find the nearest edge distance and its inward-facing direction.
    // The surface slopes downward toward the nearest edge.
    var min_dist = dl;
    var slope_dir = vec2<f32>(-1.0, 0.0);

    if dr < min_dist {
        min_dist = dr;
        slope_dir = vec2<f32>(1.0, 0.0);
    }
    if db < min_dist {
        min_dist = db;
        slope_dir = vec2<f32>(0.0, -1.0);
    }
    if dt < min_dist {
        min_dist = dt;
        slope_dir = vec2<f32>(0.0, 1.0);
    }

    // Bevel zone: only slope near edges, flat plateau in the center.
    let bevel_width = 0.15;
    let bevel_strength = 1.0 - smoothstep(0.0, bevel_width, min_dist);

    // Build the surface normal: tilts toward slope_dir at edges, straight up in center.
    let tilt = bevel_strength * 0.8;
    let n = normalize(vec3<f32>(slope_dir * tilt, 1.0));

    // Light direction from pointer
    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);
    let delta = pointer - in.world_pos;
    let dist = length(delta);
    let light_xy = delta / max(dist, 1.0);
    let light_dir = normalize(vec3<f32>(light_xy * 0.7, 0.6));

    // Diffuse
    let n_dot_l = dot(n, light_dir);
    let diffuse = smoothstep(-0.2, 1.0, n_dot_l);

    // Specular — Blinn-Phong
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(n, half_vec), 0.0), 32.0);

    let proximity = 1.0 / (1.0 + dist * 0.005);

    // Output as overlay tint (only affects underlying card via alpha blend)
    let light_delta = (diffuse - 0.5) * 0.3 * bevel_strength + spec * 0.6 * proximity;

    if light_delta >= 0.0 {
        return vec4<f32>(1.0, 1.0, 1.0, light_delta);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, -light_delta);
    }
}
