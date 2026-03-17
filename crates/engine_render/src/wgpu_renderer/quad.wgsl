struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    @location(0) quad_pos: vec2<f32>,
    @location(1) world_rect: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let x = quad_pos.x * world_rect.z + world_rect.x;
    let y = quad_pos.y * world_rect.w + world_rect.y;
    let world_pos = vec4<f32>(x, y, 0.0, 1.0);
    out.position = camera.view_proj * world_pos;
    out.color = color;
    out.uv = vec2<f32>(
        mix(uv_rect.x, uv_rect.z, quad_pos.x),
        mix(uv_rect.y, uv_rect.w, quad_pos.y),
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex_color * in.color;
}
