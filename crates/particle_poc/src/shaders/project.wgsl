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
    _pad0: u32,
    _pad1: u32,
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

// Morton (Z-order) encode for 8-bit grid coords (grid up to 256×256).
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

// SOR factor for the contact-count-averaged Jacobi update (Macklin et al.).
// Corrections are averaged over contacts (ω/n), not summed: summing makes a
// particle with ~6 neighbors overshoot 3× and oscillate (measured as KE
// climbing to 10× steady state — the "fountain"), while a fixed under-relaxed
// sum leaves the bulk permanently overlapped (squish). ω slightly above 1
// speeds convergence without re-introducing the oscillation.
// 1.5 was tried and destabilizes: sparse-contact particles (n=1..2) overshoot
// and get ejected from the pile surface (measured vmax spikes 9-14 at rest).
const SOR_OMEGA: f32 = 1.2;

// PBD contact projection (Macklin et al., "Unified Particle Physics").
// For every overlapping pair, each particle computes half the separation
// (equal masses) plus a positional Coulomb friction term derived from the
// tangential relative displacement this substep. Corrections are written to
// a scratch buffer and applied in the next pass, so the pass only reads
// shared state — pair forces stay exactly antisymmetric (Newton's 3rd law).
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

                // Coincident particles: deterministic separation axis from
                // index order so the pair pushes apart, not together.
                var n: vec2<f32>;
                if dist < 1e-6 {
                    n = select(vec2<f32>(-1.0, 0.0), vec2<f32>(1.0, 0.0), i > j);
                } else {
                    n = delta / dist;
                }

                correction += 0.5 * overlap * n;

                // Positional friction: oppose tangential relative motion this
                // substep. Static below mu*overlap (fully cancelled), kinetic
                // above (scaled down). Gives piles a real angle of repose.
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

    correction *= SOR_OMEGA / f32(max(contact_count, 1u));

    // Cap per-substep correction: deep penetrations from fast impacts resolve
    // over a few substeps instead of one violent displacement.
    let max_corr = r;
    let mag = length(correction);
    if mag > max_corr {
        correction *= max_corr / mag;
    }
    corrections[i] = correction;
}

// Apply corrections, clamp to walls, recompute velocity from actual position
// change. Projection is therefore dissipative — it can only remove kinetic
// energy, never add it. Contacts are inelastic, which is correct for sand.
@compute @workgroup_size(256)
fn apply(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    var pos = positions[i] + corrections[i];
    let r = params.particle_radius;
    pos.x = clamp(pos.x, params.wall_min_x + r, params.wall_max_x - r);
    pos.y = clamp(pos.y, params.wall_min_y + r, params.wall_max_y - r);

    // Machine collision: loop over all machines.
    // kind 0 = Capsule body (OBB, static wall — push out only, no velocity)
    // kind 1 = Grinder, kind 2 = Heater (sensor only — transmutation in reaction pass)
    // kind 3 = Paddle (OBB collision + tangential sweep along paddle surface)
    for (var m = 0u; m < machine_params.count; m++) {
        let mach = machine_params.machines[m];

        if mach.kind == 0u {
            // Capsule body: line-segment SDF with rounded ends.
            // Belt is a line segment (0,-l)→(0,+l) in machine-local + radius r.
            let to_particle = pos - vec2(mach.pos_x, mach.pos_y);
            let local_x =  to_particle.x * mach.cos_angle + to_particle.y * mach.sin_angle;
            let local_y = -to_particle.x * mach.sin_angle + to_particle.y * mach.cos_angle;
            let br = mach.half_width;   // belt half-thickness
            let bl = mach.half_height;  // straight half-length
            let tr = br + r;            // total collision radius
            // Closest point on segment (0,-bl) → (0,+bl).
            let t = clamp(local_y, -bl, bl);
            let closest = vec2(0.0, t);
            let delta = vec2(local_x, local_y) - closest;
            let dist = length(delta);

            if dist < tr && dist > 1e-9 {
                let ln = delta / dist;
                let overlap = tr - dist;
                let compliance = 0.5;
                let wx = ln.x * overlap * compliance * mach.cos_angle - ln.y * overlap * compliance * mach.sin_angle;
                let wy = ln.x * overlap * compliance * mach.sin_angle + ln.y * overlap * compliance * mach.cos_angle;
                pos += vec2(wx, wy);
            }
        } else if mach.kind == 3u {
            // Paddle: OBB collision with tangential sweep to bridge frame-to-frame
            // teleport (paddle positions update once per frame on CPU).
            // Forward edge is expanded to catch particles that would slip past
            // when the paddle rotates on end caps (tip displacement ≈ PADDLE_HH * Δθ).
            let to_particle = pos - vec2(mach.pos_x, mach.pos_y);
            let local_x =  to_particle.x * mach.cos_angle + to_particle.y * mach.sin_angle;
            let local_y = -to_particle.x * mach.sin_angle + to_particle.y * mach.cos_angle;
            let hw = mach.half_width + r;
            let hh = mach.half_height + r;
            // Extend forward (+tangent) edge by per-frame displacement to catch
            // particles in the paddle's swept path between position updates.
            let frame_displacement = mach.angular_velocity * params.dt * 16.0;
            let hh_forward = hh + frame_displacement;
            let closest_x = clamp(local_x, -hw, hw);
            let closest_y = clamp(local_y, -hh, hh_forward);
            let local_delta = vec2(local_x - closest_x, local_y - closest_y);
            let dist = length(local_delta);

            if dist < r && dist > 1e-9 {
                let ln = local_delta / dist;
                let overlap = r - dist;
                let compliance = 0.5;
                let wx = ln.x * overlap * compliance * mach.cos_angle - ln.y * overlap * compliance * mach.sin_angle;
                let wy = ln.x * overlap * compliance * mach.sin_angle + ln.y * overlap * compliance * mach.cos_angle;
                pos += vec2(wx, wy);

                // Tangential sweep: carry particles at ~80% of belt speed.
                let surf_speed = mach.angular_velocity * params.dt * 0.8;
                let tang = vec2(mach.cos_angle, mach.sin_angle);
                pos += tang * surf_speed;
            }
        }
        // Grinder/Heater sensors: handled in reaction pass (has species access).
    }

    positions[i] = pos;
    velocities[i] = (pos - prev_positions[i]) / params.dt;
}
