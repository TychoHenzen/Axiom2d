// Detection shaders: per-frame outlier velocity scanning and paddle phasing detection.
// Uses its own bind group (detection_bgl) to avoid exceeding storage buffer limits.

struct DetParams {
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
}

const MAX_MACHINES: u32 = 16u;

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

@group(0) @binding(0) var<storage, read> det_positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read> det_velocities: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> outlier_amt: atomic<u32>;
@group(0) @binding(3) var<storage, read_write> phasing_amt: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> outlier_data: array<u32>;
@group(0) @binding(5) var<storage, read_write> phasing_data: array<u32>;
@group(0) @binding(6) var<uniform> det_params: DetParams;
@group(0) @binding(7) var<storage, read> det_machine_params: MachineParams;

const OUTLIER_ENTRY_WORDS: u32 = 6u;
const PHASING_ENTRY_WORDS: u32 = 3u;
const MAX_DETECT_OUTLIERS: u32 = 64u;
const MAX_DETECT_PHASING: u32 = 32u;

// Scan all particle velocities, write top-speed entries to outlier buffer.
@compute @workgroup_size(256)
fn detect_outliers(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= det_params.particle_count { return; }

    let vel = det_velocities[i];
    let speed_sq = vel.x * vel.x + vel.y * vel.y;
    if speed_sq < 0.25 { return; }  // threshold: speed > 0.5

    let slot = atomicAdd(&outlier_amt, 1u);
    if slot >= MAX_DETECT_OUTLIERS { return; }

    let base = slot * OUTLIER_ENTRY_WORDS;
    let pos = det_positions[i];
    let speed = sqrt(speed_sq);
    outlier_data[base + 0u] = i;
    outlier_data[base + 1u] = bitcast<u32>(pos.x);
    outlier_data[base + 2u] = bitcast<u32>(pos.y);
    outlier_data[base + 3u] = bitcast<u32>(vel.x);
    outlier_data[base + 4u] = bitcast<u32>(vel.y);
    outlier_data[base + 5u] = bitcast<u32>(speed);
}

// Check every particle against every paddle (kind=3) OBB.
// If particle center is inside expanded paddle OBB, log phasing event.
@compute @workgroup_size(256)
fn detect_phasing(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= det_params.particle_count { return; }

    let pos = det_positions[i];
    let r = det_params.particle_radius;

    for (var m = 0u; m < det_machine_params.count; m++) {
        let mach = det_machine_params.machines[m];
        if mach.kind != 3u { continue; }

        // Transform particle to paddle-local space.
        let to_particle = pos - vec2(mach.pos_x, mach.pos_y);
        let local_x =  to_particle.x * mach.cos_angle + to_particle.y * mach.sin_angle;
        let local_y = -to_particle.x * mach.sin_angle + to_particle.y * mach.cos_angle;

        // Expanded OBB (matching project pass expansion).
        // local_x = tangent direction, local_y = perpendicular.
        let hw = mach.half_width + r;
        let hh = mach.half_height + r;
        let frame_disp = mach.angular_velocity * det_params.dt * f32(det_params.sub_steps);
        let hw_back = hw + frame_disp;

        // Check if particle center is inside expanded OBB.
        if local_x >= -hw_back && local_x <= hw_back && local_y >= -hh && local_y <= hh {
            // Compute penetration relative to unexpanded OBB.
            let closest_x = clamp(local_x, -mach.half_width, mach.half_width);
            let closest_y = clamp(local_y, -mach.half_height, mach.half_height);
            let dx = local_x - closest_x;
            let dy = local_y - closest_y;
            let pen = r - sqrt(dx * dx + dy * dy);
            // Only flag deep penetration (> 1.5 * particle_radius past surface).
            // pen ≈ r for normal contact (particle center at OBB edge).
            if pen <= 1.5 * r { continue; }

            let slot = atomicAdd(&phasing_amt, 1u);
            if slot >= MAX_DETECT_PHASING { continue; }

            let base = slot * PHASING_ENTRY_WORDS;
            phasing_data[base + 0u] = m;
            phasing_data[base + 1u] = i;
            phasing_data[base + 2u] = bitcast<u32>(pen);
        }
    }
}
