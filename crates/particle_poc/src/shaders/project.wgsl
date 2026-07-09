struct Params {
    particle_count: u32,
    dt: f32,
    gravity: f32,
    particle_radius: f32,
    wall_min_x: f32,
    wall_min_y: f32,
    wall_max_x: f32,
    wall_max_y: f32,
    friction_mu: f32,
    grid_cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    disable_velocity_cap: u32,
    sub_steps: u32,
}

const MAX_MACHINES: u32 = 16u;

struct Machine {
    pos_x: f32,
    pos_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    kind: u32,
    input_species: u32,
    output_species: u32,
    angular_velocity: f32,
}

struct MachineParams {
    count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    machines: array<Machine, MAX_MACHINES>,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> corrections: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> prev_positions: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(8) var<storage, read> machine_params: MachineParams;

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

fn morton_encode(x: u32, y: u32) -> u32 {
    var result = 0u;
    for (var i = 0u; i < 8u; i++) {
        result |= ((x >> i) & 1u) << (2u * i);
        result |= ((y >> i) & 1u) << (2u * i + 1u);
    }
    return result;
}

fn morton_decode(key: u32) -> vec2<i32> {
    var x = 0u;
    var y = 0u;
    for (var i = 0u; i < 8u; i++) {
        x |= ((key >> (2u * i)) & 1u) << i;
        y |= ((key >> (2u * i + 1u)) & 1u) << i;
    }
    return vec2<i32>(i32(x), i32(y));
}

const SOR_OMEGA: f32 = 1.2;

fn world_from_local(local: vec2<f32>, cos_a: f32, sin_a: f32) -> vec2<f32> {
    return vec2(local.x * cos_a - local.y * sin_a,
                local.x * sin_a + local.y * cos_a);
}

// Unified PBD contact projection covering particle-particle + machine contacts.
// Machine contacts (capsule body, paddle) get 0.5*overlap (same as
// particle-particle) and share SOR dilution + per-substep cap. Paddle contacts
// add Coulomb friction on tangential velocity relative to belt surface.
@compute @workgroup_size(256)
fn project(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let pos_i = positions[i];
    let disp_i = pos_i - prev_positions[i];
    let r = params.particle_radius;
    let contact_dist = 2.0 * r;

    var correction = vec2<f32>(0.0);
    var contact_count = 0u;

    // --- particle-particle contacts ---
    let my_morton = cell_indices[i];
    let cc = morton_decode(my_morton);

    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let nx = cc.x + dx;
            let ny = cc.y + dy;
            if nx < 0 || ny < 0 || nx >= i32(params.grid_width) || ny >= i32(params.grid_height) {
                continue;
            }
            let neighbor_cell = morton_encode(u32(nx), u32(ny));
            let bucket = neighbor_cell % (params.grid_width * params.grid_height);
            let start = cell_offsets[bucket];
            let count = cell_counts[bucket];

            for (var k = 0u; k < count; k++) {
                let j = sorted_indices[start + k];
                if j == i { continue; }

                let pos_j = positions[j];
                let delta = pos_i - pos_j;
                let dist_sq = dot(delta, delta);
                let contact_sq = contact_dist * contact_dist;
                if dist_sq >= contact_sq { continue; }

                let dist = sqrt(dist_sq);
                let overlap = contact_dist - dist;
                contact_count += 1u;

                var n: vec2<f32>;
                if dist < 1e-6 {
                    n = select(vec2<f32>(-1.0, 0.0), vec2<f32>(1.0, 0.0), i > j);
                } else {
                    n = delta / dist;
                }

                correction += 0.5 * overlap * n;

                // Positional friction: oppose tangential relative motion.
                let rel_disp = disp_i - (pos_j - prev_positions[j]);
                let tangential = rel_disp - dot(rel_disp, n) * n;
                let t_len = length(tangential);
                if t_len > 1e-9 {
                    let scale = min(params.friction_mu * overlap / t_len, 1.0);
                    correction -= 0.5 * tangential * scale;
                }
            }
        }
    }

    // --- machine contacts (capsule body + paddle) ---
    // Static rigid bodies: apply directly, not SOR-diluted.
    // Stale transforms (1x/frame): scale by 1/sub_steps to match old 1x/frame push.
    var machine_corr = vec2<f32>(0.0);

    for (var m = 0u; m < machine_params.count; m++) {
        let mach = machine_params.machines[m];
        if mach.kind != 0u && mach.kind != 3u { continue; }

        let to_particle = pos_i - vec2(mach.pos_x, mach.pos_y);
        let local_x =  to_particle.x * mach.cos_angle + to_particle.y * mach.sin_angle;
        let local_y = -to_particle.x * mach.sin_angle + to_particle.y * mach.cos_angle;
        let c = mach.cos_angle;
        let s = mach.sin_angle;

        var n: vec2<f32>;
        var overlap: f32;
        var is_paddle = false;

        if mach.kind == 0u {
            // Capsule body: long axis is body Y (half_height), perpendicular is body X (half_width).
            // Clamp position along the belt (local_y), check perpendicular distance (local_x).
            let tr = mach.half_width + r;
            let t = clamp(local_y, -mach.half_height, mach.half_height);
            let delta = vec2(local_x, local_y - t);
            let dist = length(delta);
            if dist >= tr { continue; }
            if dist < 1e-9 {
                n = world_from_local(vec2(1.0, 0.0), c, s);
            } else {
                n = world_from_local(delta / dist, c, s);
            }
            overlap = tr - dist;
        } else {
            let hw = mach.half_width + r;
            let hh = mach.half_height + r;
            let frame_disp = mach.angular_velocity * params.dt * f32(params.sub_steps);
            let closest_x = clamp(local_x, -(hw + frame_disp), hw + frame_disp);
            let closest_y = clamp(local_y, -hh, hh);
            let local_delta = vec2(local_x - closest_x, local_y - closest_y);
            let dist = length(local_delta);
            if dist >= r || dist < 1e-9 { continue; }
            n = world_from_local(local_delta / dist, c, s);
            overlap = r - dist;
            is_paddle = true;
        }

        let compliance = 0.5 / f32(params.sub_steps);
        machine_corr += overlap * n * compliance;

        if is_paddle {
            let surf_disp = vec2(c, s) * mach.angular_velocity * params.dt;
            let rel_disp = disp_i - surf_disp;
            let tang_rel = rel_disp - dot(rel_disp, n) * n;
            let tang_len = length(tang_rel);
            if tang_len > 1e-9 {
                let scale = min(params.friction_mu * overlap * compliance / tang_len, 1.0);
                machine_corr -= 0.5 * tang_rel * scale;
            }
        }
    }

    correction *= SOR_OMEGA / f32(max(contact_count, 1u));

    // Machine correction: separate SOR over machine contacts only.
    let mach_mag = length(machine_corr);
    if mach_mag > 0.0 {
        machine_corr *= SOR_OMEGA;
        let mach_mag2 = length(machine_corr);
        if mach_mag2 > r {
            machine_corr *= r / mach_mag2;
        }
        correction += machine_corr;
    }

    // Cap per-substep correction.
    let max_corr = r;
    let mag = length(correction);
    if mag > max_corr {
        correction *= max_corr / mag;
    }
    corrections[i] = correction;
}

// Apply unified PBD correction, clamp walls, recompute velocity.
// Safety velocity cap defaults on; optional disable for root-cause testing.
const MAX_SPEED: f32 = 1.9;

@compute @workgroup_size(256)
fn apply(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let r = params.particle_radius;
    let prev = prev_positions[i];

    var pos = positions[i] + corrections[i];

    // Wall clamp.
    pos.x = clamp(pos.x, params.wall_min_x + r, params.wall_max_x - r);
    pos.y = clamp(pos.y, params.wall_min_y + r, params.wall_max_y - r);

    // Safety velocity cap — disable for root-cause testing.
    if params.disable_velocity_cap == 0u {
        let v = (pos - prev) / params.dt;
        let speed = length(v);
        if speed > MAX_SPEED {
            pos = prev + (v / speed) * MAX_SPEED * params.dt;
        }
    }

    positions[i] = pos;
    velocities[i] = (pos - prev_positions[i]) / params.dt;
}
