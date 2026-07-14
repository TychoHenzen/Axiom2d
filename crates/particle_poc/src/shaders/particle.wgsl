struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) @interpolate(flat) is_cursor: u32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read> species: array<u32>;

struct RenderParams {
    screen_width: f32,
    screen_height: f32,
    particle_radius: f32,
    particle_count: u32,
    cursor_x: f32,
    cursor_y: f32,
    cursor_radius: f32,
    cursor_active: u32,
}
@group(0) @binding(2) var<uniform> params: RenderParams;

fn species_color(s: u32) -> vec3<f32> {
    switch s {
        case 0u: { return vec3<f32>(0.9, 0.2, 0.2); } // Red
        case 1u: { return vec3<f32>(0.2, 0.4, 0.9); } // Blue
        case 2u: { return vec3<f32>(0.2, 0.9, 0.3); } // Green (Red+Blue reaction)
        case 3u: { return vec3<f32>(0.9, 0.9, 0.2); } // Yellow (Red → Grinder)
        case 4u: { return vec3<f32>(0.7, 0.2, 0.9); } // Purple (Blue → Heater)
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
    let uv = QUAD_UVS[corner];

    var pos: vec2<f32>;
    var col: vec3<f32>;
    var r: f32;
    var is_cursor: u32 = 0u;

    let aspect = params.screen_width / params.screen_height;
    if params.cursor_active != 0u && particle_id >= params.particle_count {
        // Cursor quad — white circle outline at cursor position.
        pos = vec2<f32>(params.cursor_x, params.cursor_y);
        col = vec3<f32>(1.0, 1.0, 1.0);
        r = params.cursor_radius;
        is_cursor = 1u;
    } else {
        pos = positions[particle_id];
        col = species_color(species[particle_id]);
        r = params.particle_radius;
    }

    // Divide X by aspect so world-space distances map to equal screen pixels in both axes.
    let clip_x = (pos.x + uv.x * r) / aspect;
    let clip_y = pos.y + uv.y * r;

    var out: VertexOut;
    out.clip_pos = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.uv = uv;
    out.color = col;
    out.is_cursor = is_cursor;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if dist > 1.0 { discard; }
    if in.is_cursor != 0u {
        // Cursor: outline ring only.
        let inner = smoothstep(0.7, 0.85, dist);
        let outer = smoothstep(0.85, 1.0, dist);
        let alpha = (1.0 - outer) * inner;
        if alpha < 0.01 { discard; }
        return vec4<f32>(1.0, 1.0, 1.0, alpha * 0.8);
    }
    let edge = smoothstep(0.85, 1.0, dist);
    let col = in.color * (1.0 - 0.3 * edge);
    return vec4<f32>(col, 1.0);
}
