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

    // Height field from UV: raised plateau with steep slopes at edges
    let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    let height = smoothstep(0.0, 0.15, edge_dist);

    // Steep normal at edges from height field derivatives
    let hdx = dpdx(height);
    let hdy = dpdy(height);
    let scale = max(length(vec2<f32>(dpdx(in.local_pos.x), dpdy(in.local_pos.x))), 0.001);
    let n = normalize(vec3<f32>(-hdx / scale * 5.0, -hdy / scale * 5.0, 1.0));

    // Orbiting light — strong directional
    let la = t * 0.3;
    let light_dir = normalize(vec3<f32>(cos(la) * 0.8, sin(la) * 0.8, 0.6));
    let n_dot_l = dot(n, light_dir);

    // Specular highlight on the raised surfaces
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(n, half_vec), 0.0), 16.0);

    // Strong light/shadow contrast
    let diffuse = smoothstep(-0.3, 0.9, n_dot_l);

    // Darken edges (bevel shadow), brighten tops
    let shadow = 0.4 + diffuse * 0.6;
    let lit = in.color.rgb * shadow + vec3<f32>(spec * 0.7);

    return vec4<f32>(lit, 0.7);
}
