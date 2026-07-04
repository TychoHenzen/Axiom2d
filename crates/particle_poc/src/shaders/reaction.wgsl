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
@group(0) @binding(2) var<storage, read_write> species: array<u32>;
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

// Red(0) + Blue(1) in contact → both transmute to Green(2)
@compute @workgroup_size(256)
fn react(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let si = species[i];
    // Only Red or Blue can react; Green is inert
    if si > 1u { return; }

    let pos_i = positions[i];
    let contact_dist = 2.0 * params.particle_radius;
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

                let sj = species[j];
                // Need opposite species: Red(0)+Blue(1) or Blue(1)+Red(0)
                if si == sj || sj > 1u { continue; }

                let dist = distance(pos_i, positions[j]);
                if dist < contact_dist {
                    // Transmute both to Green
                    species[i] = 2u;
                    return;
                }
            }
        }
    }
}
