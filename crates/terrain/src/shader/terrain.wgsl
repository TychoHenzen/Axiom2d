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

fn unpack_type_id() -> u32 {
    return bitcast<u32>(material[0].x);
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
    let type_id = unpack_type_id();
    let world_pos = unpack_world_pos();
    let seed = unpack_seed();
    let params = unpack_params(1u);

    // Phase 1: single material on full quad
    let world_uv = world_pos + in.uv * 10.0 + vec2<f32>(seed * 17.0, seed * 31.0);
    let color = eval_terrain(world_uv, type_id, params);
    return vec4<f32>(color, 1.0);
}
