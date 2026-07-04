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

const MAX_SPECIES: u32 = 8u;

struct ReactionMatrix {
    results: array<u32, 64>, // MAX_SPECIES * MAX_SPECIES = 64
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> species: array<u32>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(9) var<storage, read> reaction_matrix: ReactionMatrix;

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

// Data-driven reaction: lookup result from reaction_matrix[si * MAX_SPECIES + sj].
// A result of 0 means no reaction. Default setup: Red(0)+Blue(1) → Green(2).
@compute @workgroup_size(256)
fn react(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let si = species[i];
    if si >= MAX_SPECIES { return; }

    let pos_i = positions[i];
    let contact_dist = 2.0 * params.particle_radius;
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

                let sj = species[j];
                if sj >= MAX_SPECIES { continue; }

                let result = reaction_matrix.results[si * MAX_SPECIES + sj];
                if result == 0u { continue; }

                let delta = pos_i - positions[j];
                let dist_sq = dot(delta, delta);
                let contact_sq = contact_dist * contact_dist;
                if dist_sq < contact_sq {
                    species[i] = result;
                    return;
                }
            }
        }
    }
}
