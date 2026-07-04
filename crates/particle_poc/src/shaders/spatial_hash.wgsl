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
@group(0) @binding(7) var<uniform> params: Params;

@group(1) @binding(0) var<storage, read_write> cell_indices: array<u32>;
@group(1) @binding(1) var<storage, read_write> cell_counts: array<atomic<u32>>;
@group(1) @binding(2) var<storage, read_write> cell_offsets: array<u32>;
@group(1) @binding(3) var<storage, read_write> sorted_indices: array<u32>;

fn pos_to_cell(pos: vec2<f32>) -> u32 {
    let gx = clamp(
        i32(floor((pos.x - params.wall_min_x) / params.grid_cell_size)),
        0, i32(params.grid_width) - 1
    );
    let gy = clamp(
        i32(floor((pos.y - params.wall_min_y) / params.grid_cell_size)),
        0, i32(params.grid_height) - 1
    );
    return u32(gy) * params.grid_width + u32(gx);
}

// Pass 1: Clear grid
@compute @workgroup_size(256)
fn clear_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let total_cells = params.grid_width * params.grid_height;
    if i >= total_cells { return; }
    atomicStore(&cell_counts[i], 0u);
    cell_offsets[i] = 0u;
}

// Pass 2: Assign each particle to a cell and count occupancy
@compute @workgroup_size(256)
fn assign_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }
    let cell = pos_to_cell(positions[i]);
    cell_indices[i] = cell;
    atomicAdd(&cell_counts[cell], 1u);
}

// Pass 3: prefix sum over cell_counts → cell_offsets, then clear cell_counts.
// Single workgroup of 256 threads: each thread serially scans its chunk of
// ~100 cells, a Hillis-Steele scan combines the 256 chunk totals, then each
// thread writes final offsets for its chunk. (The previous single-thread
// version serialized 25 600 iterations on one GPU lane and dominated frame
// time once run every substep.)
var<workgroup> chunk_sums: array<u32, 256>;

@compute @workgroup_size(256)
fn prefix_scan(@builtin(local_invocation_id) lid: vec3<u32>) {
    let t = lid.x;
    let total_cells = params.grid_width * params.grid_height;
    let chunk = (total_cells + 255u) / 256u;
    let start = t * chunk;
    let end = min(start + chunk, total_cells);

    var sum = 0u;
    for (var i = start; i < end; i++) {
        sum += atomicLoad(&cell_counts[i]);
    }
    chunk_sums[t] = sum;
    workgroupBarrier();

    // Hillis-Steele inclusive scan over the 256 chunk totals
    for (var offset = 1u; offset < 256u; offset <<= 1u) {
        var v = chunk_sums[t];
        if t >= offset {
            v += chunk_sums[t - offset];
        }
        workgroupBarrier();
        chunk_sums[t] = v;
        workgroupBarrier();
    }

    // Exclusive offset for this chunk = inclusive sum of preceding chunks
    var running = 0u;
    if t > 0u {
        running = chunk_sums[t - 1u];
    }
    for (var i = start; i < end; i++) {
        let count = atomicLoad(&cell_counts[i]);
        cell_offsets[i] = running;
        running += count;
        atomicStore(&cell_counts[i], 0u);
    }
}

// Pass 4: Scatter particles into sorted order using atomic counters
@compute @workgroup_size(256)
fn scatter(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }
    let cell = cell_indices[i];
    let local_offset = atomicAdd(&cell_counts[cell], 1u);
    sorted_indices[cell_offsets[cell] + local_offset] = i;
}
