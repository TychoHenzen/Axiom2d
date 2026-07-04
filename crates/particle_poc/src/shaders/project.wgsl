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

struct Conveyor {
    pivot_x: f32,
    pivot_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    angular_velocity: f32,
    _pad0: u32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> corrections: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> prev_positions: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(8) var<uniform> conveyor: Conveyor;

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

    // Rotating arm collision: transform particle into arm's local space,
    // resolve against the axis-aligned box there, then rotate back.
    let to_particle = pos - vec2(conveyor.pivot_x, conveyor.pivot_y);
    // Inverse rotation by -angle
    let local_x =  to_particle.x * conveyor.cos_angle + to_particle.y * conveyor.sin_angle;
    let local_y = -to_particle.x * conveyor.sin_angle + to_particle.y * conveyor.cos_angle;

    // Arm occupies x in [-hw, hw], y in [-hh, hh] in local space (centered on pivot)
    let hw = conveyor.half_width;
    let local_closest_x = clamp(local_x, -hw, hw);
    let local_closest_y = clamp(local_y, -conveyor.half_height, conveyor.half_height);
    let local_delta = vec2(local_x - local_closest_x, local_y - local_closest_y);
    let dist = length(local_delta);

    if dist < r {
        var local_correction: vec2<f32>;
        if dist > 1e-9 {
            local_correction = (r - dist) * local_delta / dist;
        } else {
            // Particle center inside arm — push to nearest edge in local space.
            let to_left   = local_x + hw;
            let to_right  = hw - local_x;
            let to_bottom = local_y + conveyor.half_height;
            let to_top    = conveyor.half_height - local_y;
            let min_edge = min(min(to_left, to_right), min(to_bottom, to_top));
            if min_edge == to_left {
                local_correction = vec2(-(r + local_x + hw), 0.0);
            } else if min_edge == to_right {
                local_correction = vec2(hw - local_x + r, 0.0);
            } else if min_edge == to_bottom {
                local_correction = vec2(0.0, -(r + local_y + conveyor.half_height));
            } else {
                local_correction = vec2(0.0, conveyor.half_height - local_y + r);
            }
        }

        // Rotate correction back to world space
        let wx = local_correction.x * conveyor.cos_angle - local_correction.y * conveyor.sin_angle;
        let wy = local_correction.x * conveyor.sin_angle + local_correction.y * conveyor.cos_angle;
        pos += vec2(wx, wy);

        // Velocity transfer: v = ω × r where r = pos - pivot
        // ω × (rx, ry) = (-ω·ry, ω·rx)
        let rx = pos.x - conveyor.pivot_x;
        let ry = pos.y - conveyor.pivot_y;
        pos.x += -conveyor.angular_velocity * ry * params.dt * 0.5;
        pos.y +=  conveyor.angular_velocity * rx * params.dt * 0.5;
    }

    positions[i] = pos;
    velocities[i] = (pos - prev_positions[i]) / params.dt;
}
