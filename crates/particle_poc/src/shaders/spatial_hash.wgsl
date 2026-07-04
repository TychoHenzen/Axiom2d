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

// Pass 3: prefix sum over cell_counts → cell_offsets, then clear cell_counts
// Sequential scan — 25600 cells is trivial for one thread.
@compute @workgroup_size(1)
fn prefix_scan() {
    let total_cells = params.grid_width * params.grid_height;
    var running = 0u;
    for (var i = 0u; i < total_cells; i++) {
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
