// Bond formation: Green(2) particles form mutual bonds with nearby green neighbors.
// Two-pass algorithm eliminates unilateral bonds that cause asymmetric forces.
//
// Pass 1 (proposal): Each green particle proposes a bond to its closest green neighbor.
//   Writes to own bond slots only — no cross-thread writes.
// Pass 2 (resolve): Each green particle validates its proposals are reciprocal.
//   If partner J doesn't have a bond pointing back to i, clears the bond from i.
//   Reads partner slots (read-only cross-thread), writes own slots only — no atomics.
//
// Only mutual bonds survive, ensuring equal-opposite corrections in solve_bonds.
//
// bonds is a flat array: particle i owns slots [i*4+0, i*4+1, i*4+2, i*4+3].

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

struct BondSlot {
    partner: u32,
    rest: f32,
}

const GREEN_SPECIES: u32 = 2u;
const INVALID_BOND: u32 = 0xFFFFFFFFu;
const BOND_FORMATION_MULTIPLIER: f32 = 3.0;
const SLOTS_PER_PARTICLE: u32 = 4u;

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> species: array<u32>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(10) var<storage, read_write> bonds: array<BondSlot>;

@group(1) @binding(0) var<storage, read_write> cell_indices: array<u32>;
@group(1) @binding(1) var<storage, read_write> cell_counts: array<u32>;
@group(1) @binding(2) var<storage, read_write> cell_offsets: array<u32>;
@group(1) @binding(3) var<storage, read_write> sorted_indices: array<u32>;

fn morton_decode(key: u32) -> vec2<i32> {
    var x = 0u;
    var y = 0u;
    for (var i = 0u; i < 8u; i++) {
        x |= ((key >> (2u * i)) & 1u) << i;
        y |= ((key >> (2u * i + 1u)) & 1u) << i;
    }
    return vec2<i32>(i32(x), i32(y));
}

fn morton_encode(x: u32, y: u32) -> u32 {
    var result = 0u;
    for (var i = 0u; i < 8u; i++) {
        result |= ((x >> i) & 1u) << (2u * i);
        result |= ((y >> i) & 1u) << (2u * i + 1u);
    }
    return result;
}

fn has_bond(me: u32, other: u32) -> bool {
    let base = me * SLOTS_PER_PARTICLE;
    return bonds[base + 0u].partner == other
        || bonds[base + 1u].partner == other
        || bonds[base + 2u].partner == other
        || bonds[base + 3u].partner == other;
}

fn slot_partner(pidx: u32, slot: u32) -> u32 {
    return bonds[pidx * SLOTS_PER_PARTICLE + slot].partner;
}

fn slot_rest(pidx: u32, slot: u32) -> f32 {
    return bonds[pidx * SLOTS_PER_PARTICLE + slot].rest;
}

fn set_slot(pidx: u32, slot: u32, partner: u32, rest: f32) {
    let idx = pidx * SLOTS_PER_PARTICLE + slot;
    bonds[idx].partner = partner;
    bonds[idx].rest = rest;
}

fn any_slot_points_to(pidx: u32, tgt: u32) -> bool {
    let base = pidx * SLOTS_PER_PARTICLE;
    return bonds[base + 0u].partner == tgt
        || bonds[base + 1u].partner == tgt
        || bonds[base + 2u].partner == tgt
        || bonds[base + 3u].partner == tgt;
}

// Pass 1: Each green particle proposes a bond to its closest green neighbor.
// Writes to own bond slots only — unilateral proposal, validated in resolve pass.
@compute @workgroup_size(256)
fn form_bonds_propose(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }
    if species[i] != GREEN_SPECIES { return; }

    let base = i * SLOTS_PER_PARTICLE;
    // Find first free slot index (0-3).
    var free_slot: u32 = INVALID_BOND;
    for (var s = 0u; s < SLOTS_PER_PARTICLE; s++) {
        if bonds[base + s].partner == INVALID_BOND {
            free_slot = s;
            break;
        }
    }
    if free_slot == INVALID_BOND { return; }

    let pos_i = positions[i];
    let r = params.particle_radius;
    let form_dist = BOND_FORMATION_MULTIPLIER * r;
    let form_dist_sq = form_dist * form_dist;

    let my_morton = cell_indices[i];
    let cc = morton_decode(my_morton);

    var best_dist_sq = form_dist_sq;
    var best_j = INVALID_BOND;

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
                if species[j] != GREEN_SPECIES { continue; }
                if has_bond(i, j) { continue; }

                let delta = pos_i - positions[j];
                let dist_sq = dot(delta, delta);
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    best_j = j;
                }
            }
        }
    }

    if best_j == INVALID_BOND { return; }

    // Use natural touching distance as rest length, not current squished
    // distance. Using sqrt(best_dist_sq) when particles are compressed by
    // PBD would record a rest length shorter than the natural spacing,
    // causing bonds to fight PBD un-crushing and inject energy.
    let rest_len = 2.0 * params.particle_radius;

    // Write proposal to own free slot only. No cross-thread writes.
    set_slot(i, free_slot, best_j, rest_len);
}

// Pass 2: Validate each proposed bond is mutual.
// For each bond from i to partner p, check if p has a bond back to i.
// If not reciprocal, clear the bond. Each thread writes only its own slots.
@compute @workgroup_size(256)
fn form_bonds_resolve(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }
    if species[i] != GREEN_SPECIES { return; }

    let base = i * SLOTS_PER_PARTICLE;
    for (var s = 0u; s < SLOTS_PER_PARTICLE; s++) {
        let p = bonds[base + s].partner;
        if p == INVALID_BOND { continue; }
        if !any_slot_points_to(p, i) {
            bonds[base + s].partner = INVALID_BOND;
            bonds[base + s].rest = 0.0;
        }
    }
}
