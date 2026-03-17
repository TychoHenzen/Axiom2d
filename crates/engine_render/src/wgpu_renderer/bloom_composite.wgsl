@group(0) @binding(0) var t_scene: texture_2d<f32>;
@group(0) @binding(1) var t_bloom: texture_2d<f32>;
@group(0) @binding(2) var s_composite: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

@fragment
fn fs_composite(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(t_scene, s_composite, in.uv);
    let bloom = textureSample(t_bloom, s_composite, in.uv);
    return vec4<f32>(scene.rgb + bloom.rgb * params.intensity, scene.a);
}
