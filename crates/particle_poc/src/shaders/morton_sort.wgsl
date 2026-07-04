// Spatial reordering: Morton (Z-order) sort for cache coherency.
// Buckets indexed by Morton key; neighbor lookup also uses Morton keys
// to match the counting bucket.

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

fn morton_encode(x: u32, y: u32) -> u32 {
    var r = 0u;
    for (var i = 0u; i < 8u; i++) {
        r |= ((x >> i) & 1u) << (2u * i);
        r |= ((y >> i) & 1u) << (2u * i + 1u);
    }
    return r;
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

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;

@group(1) @binding(0) var<storage, read_write> cell_indices: array<u32>;
@group(1) @binding(1) var<storage, read_write> cell_counts: array<atomic<u32>>;
@group(1) @binding(2) var<storage, read_write> cell_offsets: array<u32>;
@group(1) @binding(3) var<storage, read_write> sorted_indices: array<u32>;
@group(1) @binding(4) var<storage, read_write> morton_keys: array<u32>;

const GRID_SIZE: u32 = 65536u; // 256 * 256

@compute @workgroup_size(256)
fn morton_keys_kernel(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let pos = positions[i];
    let gx = u32(clamp(
        i32(floor((pos.x - params.wall_min_x) / params.grid_cell_size)),
        0, i32(params.grid_width - 1u),
    ));
    let gy = u32(clamp(
        i32(floor((pos.y - params.wall_min_y) / params.grid_cell_size)),
        0, i32(params.grid_height - 1u),
    ));

    let key = morton_encode(gx, gy);
    let bucket = key % GRID_SIZE;
    morton_keys[i] = key;
    cell_indices[i] = bucket;
}

@compute @workgroup_size(256)
fn morton_count(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let bucket = cell_indices[i];
    cell_indices[i] = atomicAdd(&cell_counts[bucket], 1u);
}

@compute @workgroup_size(256)
fn morton_scatter(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let local_offset = cell_indices[i];
    let bucket = morton_keys[i] % GRID_SIZE;
    sorted_indices[cell_offsets[bucket] + local_offset] = i;
    cell_indices[i] = morton_keys[i];
    // Re-populate cell_counts so project/reaction can iterate neighbors
    atomicAdd(&cell_counts[bucket], 1u);
}
