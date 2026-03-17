struct FullscreenOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BloomParams {
    threshold: f32,
    intensity: f32,
    direction: vec2<f32>,
    texel_size: vec2<f32>,
    _pad: vec2<f32>,
};

@vertex
fn vs_fullscreen(@location(0) position: vec2<f32>) -> FullscreenOutput {
    var out: FullscreenOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, -position.y * 0.5 + 0.5);
    return out;
}
