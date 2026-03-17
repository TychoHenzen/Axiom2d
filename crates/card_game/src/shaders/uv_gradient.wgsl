struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
};

@vertex
fn vs_shape(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.local_pos = position;
    let world_pos = model.model * vec4<f32>(position, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    return out;
}

@fragment
fn fs_shape(in: VertexOutput) -> @location(0) vec4<f32> {
    // Art rect half-extents: half_w = CARD_WIDTH * 0.45, half_h = 22.5
    // local_pos ranges from (-half_w, -half_h) to (half_w, half_h)
    let half_size = vec2<f32>(27.0, 22.5);
    let uv = in.local_pos / (half_size * 2.0) + vec2<f32>(0.5, 0.5);
    return vec4<f32>(uv.x, uv.y, 0.4, 1.0);
}
