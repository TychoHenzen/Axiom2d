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
@group(0) @binding(7) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    var vel = velocities[i];
    var pos = positions[i] + vel * params.dt;
    let r = params.particle_radius;
    let restitution = 0.3;

    if pos.x - r < params.wall_min_x {
        pos.x = params.wall_min_x + r;
        vel.x = abs(vel.x) * restitution;
    }
    if pos.x + r > params.wall_max_x {
        pos.x = params.wall_max_x - r;
        vel.x = -abs(vel.x) * restitution;
    }
    if pos.y - r < params.wall_min_y {
        pos.y = params.wall_min_y + r;
        vel.y = abs(vel.y) * restitution;
    }
    if pos.y + r > params.wall_max_y {
        pos.y = params.wall_max_y - r;
        vel.y = -abs(vel.y) * restitution;
    }

    positions[i] = pos;
    velocities[i] = vel;
}
