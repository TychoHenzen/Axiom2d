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

@group(1) @binding(0) var<storage, read_write> cell_indices: array<u32>;
@group(1) @binding(1) var<storage, read_write> cell_counts: array<u32>;
@group(1) @binding(2) var<storage, read_write> cell_offsets: array<u32>;
@group(1) @binding(3) var<storage, read_write> sorted_indices: array<u32>;

fn cell_coords(cell: u32) -> vec2<i32> {
    return vec2<i32>(i32(cell % params.grid_width), i32(cell / params.grid_width));
}

fn cell_id(cx: i32, cy: i32) -> u32 {
    return u32(cy) * params.grid_width + u32(cx);
}

@compute @workgroup_size(256)
fn solve(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let pos_i = positions[i];
    let vel_i = velocities[i];
    let r = params.particle_radius;
    let contact_dist = 2.0 * r;

    var force = vec2<f32>(0.0, params.gravity);

    let my_cell = cell_indices[i];
    let cc = cell_coords(my_cell);

    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let nx = cc.x + dx;
            let ny = cc.y + dy;
            if nx < 0 || ny < 0 || nx >= i32(params.grid_width) || ny >= i32(params.grid_height) {
                continue;
            }
            let neighbor_cell = cell_id(nx, ny);
            let start = cell_offsets[neighbor_cell];
            let count = cell_counts[neighbor_cell];

            for (var k = 0u; k < count; k++) {
                let j = sorted_indices[start + k];
                if j == i { continue; }

                let pos_j = positions[j];
                let delta = pos_i - pos_j;
                let dist = length(delta);
                if dist >= contact_dist || dist < 1e-8 { continue; }

                let overlap = contact_dist - dist;
                let n = delta / dist;

                let rel_vel = vel_i - velocities[j];
                let vn = dot(rel_vel, n);
                let fn_mag = params.spring_k * overlap - params.damping * vn;
                let f_normal = max(fn_mag, 0.0) * n;

                let vt = rel_vel - vn * n;
                let vt_mag = length(vt);
                var f_tangential = vec2<f32>(0.0);
                if vt_mag > 1e-8 {
                    let ft_mag = min(params.friction_mu * max(fn_mag, 0.0), params.damping * vt_mag);
                    f_tangential = -ft_mag * (vt / vt_mag);
                }

                force += f_normal + f_tangential;
            }
        }
    }

    // Velocity update only — position update happens in integrate pass
    velocities[i] = vel_i + force * params.dt;
}
