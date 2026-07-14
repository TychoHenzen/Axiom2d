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

const MAX_SPECIES: u32 = 8u;
const MAX_MACHINES: u32 = 16u;

struct ReactionMatrix {
    results: array<u32, 64>, // MAX_SPECIES * MAX_SPECIES = 64
}

struct Machine {
    pos_x: f32,
    pos_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    kind: u32,
    input_species: u32,
    output_species: u32,
    angular_velocity: f32,
}

struct MachineParams {
    count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    machines: array<Machine, MAX_MACHINES>,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> species: array<u32>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(8) var<storage, read> machine_params: MachineParams;
@group(0) @binding(9) var<storage, read> reaction_matrix: ReactionMatrix;
@group(1) @binding(5) var<storage, read_write> machine_counters: array<atomic<u32>>;

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

    // Machine sensor transmutation: Grinder/Heater machines check if particle
    // center is inside their bounds. If species matches input_species, transmute.
    for (var m = 0u; m < machine_params.count; m++) {
        let mach = machine_params.machines[m];
        if mach.kind == 0u || mach.kind == 3u { continue; } // Hub/Paddle — no transmutation
        if si != mach.input_species { continue; }

        // Transform particle into machine local space and check AABB containment.
        let rx = pos_i.x - mach.pos_x;
        let ry = pos_i.y - mach.pos_y;
        let lx =  rx * mach.cos_angle + ry * mach.sin_angle;
        let ly = -rx * mach.sin_angle + ry * mach.cos_angle;
        if lx >= -mach.half_width && lx <= mach.half_width
            && ly >= -mach.half_height && ly <= mach.half_height
        {
            species[i] = mach.output_species;
            atomicAdd(&machine_counters[m], 1u);
            return;
        }
    }

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
