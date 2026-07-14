struct SdfParams {
    resolution: u32,
    world_min_x: f32,
    world_min_y: f32,
    world_max_x: f32,
    world_max_y: f32,
}

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

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> prev_positions: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(11) var sdf_tex: texture_2d<f32>;
@group(0) @binding(12) var<uniform> sdf_params: SdfParams;

// Sample SDF texture at world position with bilinear interpolation.
// Returns signed distance in texel units: positive = free, negative = wall.
// Bilinear avoids nearest-neighbor quantization that causes gradient jitter
// and false verify-reverts at collision boundaries.
fn sample_sdf(pos: vec2<f32>) -> f32 {
    let inv_w = 1.0 / (sdf_params.world_max_x - sdf_params.world_min_x);
    let inv_h = 1.0 / (sdf_params.world_max_y - sdf_params.world_min_y);
    let u = clamp((pos.x - sdf_params.world_min_x) * inv_w, 0.0, 1.0);
    let v = clamp((pos.y - sdf_params.world_min_y) * inv_h, 0.0, 1.0);
    let sres = f32(sdf_params.resolution);
    let px = u * sres - 0.5;
    let py = v * sres - 0.5;
    let max_i = i32(sdf_params.resolution) - 1;
    let x0 = u32(clamp(i32(floor(px)), 0, max_i));
    let y0 = u32(clamp(i32(floor(py)), 0, max_i));
    let x1 = u32(clamp(i32(x0) + 1, 0, max_i));
    let y1 = u32(clamp(i32(y0) + 1, 0, max_i));
    let fx = clamp(px - floor(px), 0.0, 1.0);
    let fy = clamp(py - floor(py), 0.0, 1.0);
    let v00 = textureLoad(sdf_tex, vec2<u32>(x0, y0), 0).x;
    let v10 = textureLoad(sdf_tex, vec2<u32>(x1, y0), 0).x;
    let v01 = textureLoad(sdf_tex, vec2<u32>(x0, y1), 0).x;
    let v11 = textureLoad(sdf_tex, vec2<u32>(x1, y1), 0).x;
    return mix(mix(v00, v10, fx), mix(v01, v11, fx), fy);
}

// SDF gradient via central differences. Points away from wall interior
// (toward free space). Not normalized.
fn sdf_gradient(pos: vec2<f32>) -> vec2<f32> {
    let cell_size = (sdf_params.world_max_x - sdf_params.world_min_x) / f32(sdf_params.resolution);
    let eps = cell_size;
    let gx = sample_sdf(pos + vec2(eps, 0.0)) - sample_sdf(pos - vec2(eps, 0.0));
    let gy = sample_sdf(pos + vec2(0.0, eps)) - sample_sdf(pos - vec2(0.0, eps));
    return vec2(gx, gy);
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }

    let r = params.particle_radius;
    let cell_size = (sdf_params.world_max_x - sdf_params.world_min_x) / f32(sdf_params.resolution);
    let r_texels = r / cell_size;

    // --- Pre-integration: push particle out of SDF wall ---
    // positions[i] comes from previous apply pass, should be clean.
    // Defense in depth: if still in wall, try gradient push, verify.
    let saved_pos = positions[i];
    {
        let d = sample_sdf(positions[i]);
        if d < r_texels {
            let grad = sdf_gradient(positions[i]);
            let glen = length(grad);
            if glen > 1e-6 {
                let n = grad / glen;
                positions[i] += n * (r_texels - d) * cell_size;
                let vn = dot(velocities[i], n);
                if vn < 0.0 {
                    velocities[i] -= vn * n;
                }
            }
            let d2 = sample_sdf(positions[i]);
            if d2 < r_texels {
                // Push failed — freeze particle at original position.
                positions[i] = saved_pos;
                velocities[i] = vec2<f32>(0.0);
            }
        }
    }

    // Velocity integration (gravity).
    let vel = velocities[i] + vec2<f32>(0.0, params.gravity) * params.dt;
    let pos = positions[i];
    prev_positions[i] = pos;

    var p = pos + vel * params.dt;

    // --- Post-integration: prevent tunneling into SDF wall ---
    {
        let d = sample_sdf(p);
        if d < r_texels {
            let grad = sdf_gradient(p);
            let glen = length(grad);
            if glen > 1e-6 {
                let n = grad / glen;
                p += n * (r_texels - d) * cell_size;
            }
            let d2 = sample_sdf(p);
            if d2 < r_texels {
                p = pos;
            }
        }
    }

    p.x = clamp(p.x, params.wall_min_x + r, params.wall_max_x - r);
    p.y = clamp(p.y, params.wall_min_y + r, params.wall_max_y - r);
    positions[i] = p;
}
