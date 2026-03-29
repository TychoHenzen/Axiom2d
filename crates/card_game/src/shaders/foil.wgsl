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
    @location(4) card_center: vec2<f32>,
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
    // Card center = model origin in world space (translation column)
    let origin = model.model * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.card_center = origin.xy;
    return out;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // Only render on art fragments (UV guard)
    if in.uv.x + in.uv.y < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Cosine environment reflection: the card surface acts as a metallic mirror.
    // Pointer offset from card center tilts the virtual surface normal, sweeping
    // different regions of a synthetic multi-frequency cosine environment across
    // the card. The result is a smooth, rich color gradient that shifts dramatically
    // as the pointer moves — like premium foil or a chrome surface.

    let pointer = vec2<f32>(art_params.pointer_x, art_params.pointer_y);

    // Surface tilt: pointer offset from card center biases the virtual normal
    let tilt = (pointer - in.card_center) * 0.006;
    let base_n = normalize(vec3<f32>(tilt, 1.0));

    // Per-fragment UV variation adds surface micro-detail
    let uv_offset = (in.uv - vec2<f32>(0.5)) * 0.3;
    let local_n = normalize(base_n + vec3<f32>(uv_offset, 0.0));

    // Synthetic environment: 3 cosine terms at different spatial frequencies
    // create a rich color-field gradient (each channel at different phase offset)
    let env_r = cos(local_n.x * 9.0 + local_n.y * 4.0) * 0.5 + 0.5;
    let env_g = cos(local_n.x * 4.0 - local_n.y * 8.0 + 1.047) * 0.5 + 0.5;
    let env_b = cos(local_n.x * 6.0 + local_n.y * 6.0 + 2.094) * 0.5 + 0.5;
    let env_color = vec3<f32>(env_r, env_g, env_b);

    // Blend environment color with card base color to preserve card identity
    let identity_tinted = mix(env_color, env_color * (in.color.rgb * 0.6 + 0.4), 0.4);

    // Specular highlight: bright flash when pointer is aligned with surface normal
    let ptr_delta = pointer - in.world_pos;
    let ptr_dist = length(ptr_delta);
    let light_dir = normalize(vec3<f32>((ptr_delta) / max(ptr_dist, 1.0) * 0.7, 0.6));
    let half_v = normalize(light_dir + vec3<f32>(0.0, 0.0, 1.0));
    let spec = pow(max(dot(local_n, half_v), 0.0), 24.0) * 0.4;

    // Proximity boost
    let proximity = 0.6 + 0.4 / (1.0 + ptr_dist * 0.005);

    let foiled = identity_tinted * proximity + vec3<f32>(spec);

    // Edge shimmer: UV boundary brightening for premium feel
    let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    let rim = (1.0 - smoothstep(0.0, 0.15, edge_dist)) * 0.15;

    return vec4<f32>(foiled + vec3<f32>(rim), 0.7 * proximity);
}
