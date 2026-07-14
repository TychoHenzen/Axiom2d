// SDF wall visualization — renders a full-world quad sampling the SDF
// texture. Red where sdf_val < 0 (solid wall), transparent elsewhere.

struct VizParams {
    screen_width: f32,
    screen_height: f32,
    world_min_x: f32,
    world_min_y: f32,
    world_max_x: f32,
    world_max_y: f32,
}

@group(0) @binding(0) var sdf_tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: VizParams;

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_uv: vec2<f32>,
}

// Full-screen triangle covering NDC [-1,1]×[-1,1] with 3 vertices.
// Two triangles = 6 vertices.
const FULLSCREEN_UVS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOut {
    let uv = FULLSCREEN_UVS[vid];
    var out: VertexOut;
    out.clip_pos = vec4<f32>(uv, 0.0, 1.0);
    // Map NDC to world-space UV for SDF sampling.
    let aspect = params.screen_width / params.screen_height;
    let wx = uv.x * aspect;
    let wy = uv.y;
    let world_u = (wx - params.world_min_x) / (params.world_max_x - params.world_min_x);
    let world_v = (wy - params.world_min_y) / (params.world_max_y - params.world_min_y);
    out.world_uv = vec2<f32>(world_u, world_v);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Only render within world bounds.
    if in.world_uv.x < 0.0 || in.world_uv.x > 1.0 || in.world_uv.y < 0.0 || in.world_uv.y > 1.0 {
        discard;
    }
    let sdf_val = textureLoad(sdf_tex, vec2<u32>(u32(in.world_uv.x * 255.99), u32(in.world_uv.y * 255.99)), 0).x;
    if sdf_val < 0.0 {
        // Red tint, opacity proportional to how solid the wall is.
        let alpha = min(-sdf_val, 1.0) * 0.4;
        return vec4<f32>(0.9, 0.15, 0.15, alpha);
    }
    // FXC (Windows D3D compiler) requires explicit final return.
    return vec4<f32>(0.0);
}
