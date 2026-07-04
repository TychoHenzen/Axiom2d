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
@group(0) @binding(3) var<storage, read_write> corrections: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> prev_positions: array<vec2<f32>>;
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
                if dist >= contact_dist { continue; }

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

    positions[i] = pos;
    velocities[i] = (pos - prev_positions[i]) / params.dt;
}
