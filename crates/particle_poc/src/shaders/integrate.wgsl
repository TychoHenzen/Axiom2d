struct Params {
    particle_count: u32,
    dt: f32,
    gravity: f32,
    particle_radius: f32,
    wall_min_x: f32,
    wall_min_y: f32,
    wall_max_x: f32,
    wall_max_y: f32,
    spring_k: f32,
    damping: f32,
    friction_mu: f32,
    grid_cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> prev_positions: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;

// PBD predict step: apply gravity to velocity, advance position, clamp to
// walls. Final velocity is recomputed in the apply pass as (x - x_prev)/dt,
// so it is not stored here. No drag: gravity is scaled so free-fall speed
// stays below one particle radius per substep (see SimParams::gravity).
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let vel = velocities[i] + vec2<f32>(0.0, params.gravity) * params.dt;

    let pos = positions[i];
    prev_positions[i] = pos;

    var p = pos + vel * params.dt;
    let r = params.particle_radius;
    p.x = clamp(p.x, params.wall_min_x + r, params.wall_max_x - r);
    p.y = clamp(p.y, params.wall_min_y + r, params.wall_max_y - r);
    positions[i] = p;
}
