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
    kill_y: f32,
    _pad_final: u32,
    _pad_final2: u32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;

@group(1) @binding(0) var<storage, read_write> cell_indices: array<u32>;
@group(1) @binding(1) var<storage, read_write> cell_counts: array<atomic<u32>>;
@group(1) @binding(2) var<storage, read_write> cell_offsets: array<u32>;
@group(1) @binding(3) var<storage, read_write> sorted_indices: array<u32>;

// Pass 1: Clear grid
@compute @workgroup_size(256)
fn clear_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let total_cells = params.grid_width * params.grid_height;
    if i >= total_cells { return; }
    atomicStore(&cell_counts[i], 0u);
    cell_offsets[i] = 0u;
}

// Pass 2: prefix sum over cell_counts → cell_offsets, then clear cell_counts.
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