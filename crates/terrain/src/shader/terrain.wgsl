// === terrain.wgsl — master terrain shader ===

struct MaterialParams {
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    params: vec4<f32>,   // frequency, amplitude, warp, scale
    extra: vec4<f32>,    // type-specific
};

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(1) @binding(0) var<uniform> model: mat4x4<f32>;
@group(2) @binding(0) var<uniform> material: array<vec4<f32>, 16>;

// Material uniform unpacking: 16 vec4s = 256 bytes.
// Layout: [0] = corner_types (as u32 bits), world_pos.xy, seed
//         [1..4] = MaterialParams for corner 0
//         [5..8] = MaterialParams for corner 1
//         [9..12] = MaterialParams for corner 2
//         [13..15] = MaterialParams for corner 3 (partial, expand if needed)
// For Phase 1 (single material preview), we only use material[1..4].

fn unpack_corner_types() -> vec4<u32> {
    let packed = bitcast<u32>(material[0].x);
    return vec4<u32>(
        packed & 0xFFu,
        (packed >> 8u) & 0xFFu,
        (packed >> 16u) & 0xFFu,
        (packed >> 24u) & 0xFFu,
    );
}

fn unpack_world_pos() -> vec2<f32> {
    return material[0].yz;
}

fn unpack_seed() -> f32 {
    return material[0].w;
}

fn unpack_params(offset: u32) -> MaterialParams {
    let base = offset;
    return MaterialParams(
        material[base],
        material[base + 1],
        material[base + 2],
        material[base + 3],
    );
}

// ============================================================
// Noise primitives
// ============================================================

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    let n = vec3<f32>(dot(p, vec2<f32>(127.1, 311.7)),
                       dot(p, vec2<f32>(269.5, 183.3)),
                       dot(p, vec2<f32>(419.2, 371.9)));
    return fract(sin(n.xy) * 43758.5453);
}

fn gradient_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq_p = p;
    for (var i = 0; i < octaves; i++) {
        value += amplitude * gradient_noise(freq_p);
        freq_p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

fn voronoi(p: vec2<f32>) -> vec2<f32> {
    // Returns (distance to nearest cell edge, distance to nearest cell center)
    let n = floor(p);
    let f = fract(p);
    var min_dist = 8.0;
    var min_edge = 8.0;
    var nearest_center = vec2<f32>(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash22(n + g);
            let r = g + o - f;
            let d = dot(r, r);
            if d < min_dist {
                min_dist = d;
                nearest_center = n + g + o;
            }
        }
    }
    min_dist = sqrt(min_dist);
    // Second pass for edge distance
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash22(n + g);
            let r = g + o - f;
            let cell = n + g + o;
            if distance(cell, nearest_center) > 0.001 {
                let edge_d = dot(0.5 * (nearest_center - (n + f) + r), normalize(r - (nearest_center - (n + f))));
                min_edge = min(min_edge, abs(edge_d));
            }
        }
    }
    return vec2<f32>(min_dist, min_edge);
}

fn domain_warp(p: vec2<f32>, strength: f32) -> vec2<f32> {
    let ox = fbm(p + vec2<f32>(0.0, 0.0), 2);
    let oy = fbm(p + vec2<f32>(5.2, 1.3), 2);
    return p + vec2<f32>(ox, oy) * strength;
}

// ============================================================
// Sub-shaders — one per terrain type
// ============================================================

fn grass(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let warp = p.params.z;
    let wind_dir = p.extra.x;

    // Anisotropic stretch along wind direction
    let angle = wind_dir * 6.2832;
    let rot = mat2x2<f32>(cos(angle), -sin(angle), sin(angle), cos(angle));
    let stretched_uv = rot * uv * vec2<f32>(1.0, 2.5);

    // Layered directional noise
    let n1 = fbm(stretched_uv * freq, 3);
    let n2 = gradient_noise(uv * freq * 2.0 + vec2<f32>(17.0, 31.0));

    // Domain warp for organic flow
    let warped = domain_warp(uv * freq * 0.5, warp);
    let n3 = gradient_noise(warped * 3.0);

    let t = clamp(n1 * amp + n2 * 0.2 + n3 * 0.15, 0.0, 1.0);
    return mix(p.color_a.rgb, p.color_b.rgb, t);
}

fn stone(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let cell_scale = p.params.w;

    let v = voronoi(uv * cell_scale);
    let crack = smoothstep(0.02, 0.0, v.y);
    let surface = fbm(uv * freq, 3) * amp;
    let base = mix(p.color_a.rgb, p.color_b.rgb, surface);
    return mix(base, base * 0.55, crack);
}

fn water(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let warp_str = p.params.z;
    let cell_scale = p.params.w;

    // Caustic pattern from warped Voronoi
    let warped = domain_warp(uv * freq, warp_str);
    let v = voronoi(warped * cell_scale);
    let caustic = smoothstep(0.3, 0.0, v.x) * amp;

    let surface = fbm(uv * freq * 0.5, 2) * 0.3;
    let base = mix(p.color_a.rgb, p.color_b.rgb, surface);
    return base + vec3<f32>(caustic * 0.15);
}

fn sand(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let ripple_dir = p.extra.x;

    // Directional ripple pattern
    let angle = ripple_dir * 6.2832;
    let dir = vec2<f32>(cos(angle), sin(angle));
    let ripple = sin(dot(uv * freq, dir) * 12.0 + fbm(uv * freq * 0.3, 2) * 4.0) * 0.5 + 0.5;

    // Grain sparkle
    let grain = step(0.97, hash21(floor(uv * freq * 20.0)));

    let t = ripple * amp;
    let base = mix(p.color_a.rgb, p.color_b.rgb, t);
    return base + vec3<f32>(grain * 0.08);
}

fn lava(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let warp_str = p.params.z;
    let cell_scale = p.params.w;

    let v = voronoi(uv * cell_scale);
    let crack_glow = smoothstep(0.08, 0.0, v.y) * amp;

    // Surface variation on cooled plates
    let surface = fbm(uv * freq, 2) * 0.2;
    let crust = mix(p.color_a.rgb, p.color_a.rgb * 1.2, surface);
    let glow = p.color_b.rgb;

    return mix(crust, glow, crack_glow);
}

fn snow(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;

    // Very subtle surface undulation
    let n = fbm(uv * freq, 2) * amp;

    // Sparse sparkle
    let sparkle = step(0.985, hash21(floor(uv * freq * 30.0))) * 0.12;

    let base = mix(p.color_a.rgb, p.color_b.rgb, n);
    return base + vec3<f32>(sparkle);
}

// ============================================================
// Master dispatch
// ============================================================

fn eval_terrain(uv: vec2<f32>, type_id: u32, p: MaterialParams) -> vec3<f32> {
    switch type_id {
        case 0u { return grass(uv, p); }
        case 1u { return stone(uv, p); }
        case 2u { return water(uv, p); }
        case 3u { return sand(uv, p); }
        case 4u { return lava(uv, p); }
        case 5u { return snow(uv, p); }
        default { return p.color_a.rgb; }
    }
}

// ============================================================
// Per-pair transition effects
// ============================================================

fn transition_effect(
    base_color: vec3<f32>,
    world_uv: vec2<f32>,
    my_type: u32,
    neighbor_type: u32,
    dist_to_edge: f32,
) -> vec3<f32> {
    // Only apply effects within a narrow band near the boundary
    let band = 0.15;
    if dist_to_edge > band { return base_color; }
    let t = 1.0 - dist_to_edge / band; // 1.0 at edge, 0.0 at band limit

    // Stone -> Sand: scattered pebbles
    if my_type == 3u && neighbor_type == 1u {
        let pebble = step(0.85, hash21(floor(world_uv * 25.0)));
        return mix(base_color, base_color * 0.65, pebble * t);
    }

    // Lava -> Grass: singed/darkened
    if my_type == 0u && neighbor_type == 4u {
        return mix(base_color, base_color * vec3<f32>(0.4, 0.3, 0.2), t * 0.8);
    }

    // Water -> Sand: wet sand darkening
    if my_type == 3u && neighbor_type == 2u {
        return mix(base_color, base_color * 0.7, t * 0.6);
    }

    // Grass -> Stone: moss/soil strip
    if my_type == 1u && neighbor_type == 0u {
        let moss = gradient_noise(world_uv * 15.0) * t;
        return mix(base_color, vec3<f32>(0.25, 0.35, 0.18), moss * 0.4);
    }

    // Generic fallback: subtle darkening at boundary
    return mix(base_color, base_color * 0.85, t * 0.3);
}

// ============================================================
// Auto-tile boundary SDF (corner16 patterns)
// ============================================================

fn autotile_sdf(uv: vec2<f32>, bitmask: u32) -> f32 {
    let x = uv.x;
    let y = uv.y;

    switch bitmask {
        case 0u { return -1.0; }
        case 15u { return 1.0; }
        case 1u { return corner_sdf(x, 1.0 - y); }
        case 2u { return corner_sdf(x, y); }
        case 4u { return corner_sdf(1.0 - x, y); }
        case 8u { return corner_sdf(1.0 - x, 1.0 - y); }
        case 3u { return 0.5 - x; }
        case 6u { return y - 0.5; }
        case 12u { return x - 0.5; }
        case 9u { return 0.5 - y; }
        case 5u { return diagonal_sdf(x, y); }
        case 10u { return diagonal_sdf(y, x); }
        case 7u { return -corner_sdf(1.0 - x, 1.0 - y); }
        case 11u { return -corner_sdf(1.0 - x, y); }
        case 13u { return -corner_sdf(x, y); }
        case 14u { return -corner_sdf(x, 1.0 - y); }
        default { return 0.0; }
    }
}

fn corner_sdf(x: f32, y: f32) -> f32 {
    return length(vec2<f32>(x, y) - vec2<f32>(1.0, 1.0)) - 0.7;
}

fn diagonal_sdf(x: f32, y: f32) -> f32 {
    return (x + y - 1.0) * 0.7071;
}

fn displace_boundary(sdf: f32, world_uv: vec2<f32>, strength: f32) -> f32 {
    let noise = gradient_noise(world_uv * 8.0) * 2.0 - 1.0;
    return sdf + noise * strength;
}

// ============================================================
// Vertex / Fragment entry
// ============================================================

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_proj * model * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let corners = unpack_corner_types();
    let world_pos = unpack_world_pos();
    let seed = unpack_seed();

    let world_uv = world_pos + in.uv * 10.0 + vec2<f32>(seed * 0.17, seed * 0.31);

    // Check if all corners are the same type (uniform tile or Phase 1 single-material)
    if corners.x == corners.y && corners.y == corners.z && corners.z == corners.w {
        let params = unpack_params(1u);
        let color = eval_terrain(world_uv, corners.x, params);
        return vec4<f32>(color, 1.0);
    }

    // Dual-grid: determine primary type (most common corner) and compute bitmask
    let primary = corners.x; // simplified: use NE as primary
    var bitmask = 0u;
    if corners.x != primary { bitmask |= 1u; }
    if corners.y != primary { bitmask |= 2u; }
    if corners.z != primary { bitmask |= 4u; }
    if corners.w != primary { bitmask |= 8u; }

    let raw_sdf = autotile_sdf(in.uv, bitmask);
    let sdf = displace_boundary(raw_sdf, world_uv, 0.06);

    let params_primary = unpack_params(1u);

    // Find the other type
    var other_type = corners.y;
    if corners.y == primary { other_type = corners.z; }
    if other_type == primary { other_type = corners.w; }

    let abs_sdf = abs(sdf);

    if sdf < 0.0 {
        let color = eval_terrain(world_uv, primary, params_primary);
        let result = transition_effect(color, world_uv, primary, other_type, abs_sdf);
        return vec4<f32>(result, 1.0);
    } else {
        let params_other = unpack_params(5u);
        let color = eval_terrain(world_uv, other_type, params_other);
        let result = transition_effect(color, world_uv, other_type, primary, abs_sdf);
        return vec4<f32>(result, 1.0);
    }
}
