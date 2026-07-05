struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read> species: array<u32>;

struct RenderParams {
    screen_width: f32,
    screen_height: f32,
    particle_radius: f32,
    particle_count: u32,
}
@group(0) @binding(2) var<uniform> params: RenderParams;

fn species_color(s: u32) -> vec3<f32> {
    switch s {
        case 0u: { return vec3<f32>(0.9, 0.2, 0.2); } // Red
        case 1u: { return vec3<f32>(0.2, 0.4, 0.9); } // Blue
        case 2u: { return vec3<f32>(0.2, 0.9, 0.3); } // Green
        default: { return vec3<f32>(1.0, 1.0, 1.0); }
    }
}

// 6 vertices per particle (2 triangles forming a quad)
const QUAD_UVS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOut {
    let particle_id = vid / 6u;
    let corner = vid % 6u;

    let pos = positions[particle_id];
    let uv = QUAD_UVS[corner];

    let aspect = params.screen_width / params.screen_height;
    let r = params.particle_radius;
    // Divide X by aspect so world-space distances map to equal screen pixels in both axes.
    let clip_x = (pos.x + uv.x * r) / aspect;
    let clip_y = pos.y + uv.y * r;

    var out: VertexOut;
    out.clip_pos = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.uv = uv;
    out.color = species_color(species[particle_id]);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if dist > 1.0 { discard; }
    let edge = smoothstep(0.85, 1.0, dist);
    let col = in.color * (1.0 - 0.3 * edge);
    return vec4<f32>(col, 1.0);
}
