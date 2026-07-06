struct MachineRender {
    pos_x: f32,
    pos_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
}

struct RenderParams {
    screen_width: f32,
    screen_height: f32,
    machine_count: u32,
}

@group(0) @binding(0) var<storage, read> machines: array<MachineRender>;
@group(0) @binding(1) var<uniform> params: RenderParams;

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
}

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
    let machine_id = vid / 6u;
    let corner = vid % 6u;

    let m = machines[machine_id];
    let uv = QUAD_UVS[corner];

    // Local position on the quad
    let lx = uv.x * m.half_width;
    let ly = uv.y * m.half_height;

    // Rotate and translate to world space
    let wx = lx * m.cos_angle - ly * m.sin_angle + m.pos_x;
    let wy = lx * m.sin_angle + ly * m.cos_angle + m.pos_y;

    let aspect = params.screen_width / params.screen_height;
    let clip_x = wx / aspect;

    var out: VertexOut;
    out.clip_pos = vec4<f32>(clip_x, wy, 0.0, 1.0);
    out.color = vec3<f32>(m.color_r, m.color_g, m.color_b);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
