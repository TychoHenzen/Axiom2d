// Bond formation: Green(2) particles form bonds with nearby green neighbors.
// Each particle can have up to 2 bonds (MAX_BONDS_PER_PARTICLE).
// Each thread only writes to its own bond_slot_*[i] — no cross-thread writes.
// Bonds are unilateral declarations; solve_bonds enforces them regardless of
// whether the partner reciprocates (lower-index particle applies force).

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

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> species: array<u32>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(10) var<storage, read_write> bond_slot_a: array<BondSlot>;
@group(0) @binding(11) var<storage, read_write> bond_slot_b: array<BondSlot>;

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
    return bond_slot_a[me].partner == other || bond_slot_b[me].partner == other;
}

@compute @workgroup_size(256)
fn form_bonds(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }
    if species[i] != GREEN_SPECIES { return; }

    // Only bond if we have a free slot.
    let free_a = bond_slot_a[i].partner == INVALID_BOND;
    let free_b = bond_slot_b[i].partner == INVALID_BOND;
    if !free_a && !free_b { return; }

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

    let rest_len = sqrt(best_dist_sq);

    // Write to own free slot only. No cross-thread writes.
    if free_a {
        bond_slot_a[i].partner = best_j;
        bond_slot_a[i].rest = rest_len;
    } else {
        bond_slot_b[i].partner = best_j;
        bond_slot_b[i].rest = rest_len;
    }
}
