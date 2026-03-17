@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(1) @binding(0) var<uniform> params: BloomParams;

@fragment
fn fs_brightness(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_input, s_input, in.uv);
    let luminance = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (luminance > params.threshold) {
        return vec4<f32>(color.rgb, 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_blur(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let offset = params.direction * params.texel_size;
    var result = textureSample(t_input, s_input, in.uv) * 0.227027;
    result += textureSample(t_input, s_input, in.uv + offset) * 0.1945946;
    result += textureSample(t_input, s_input, in.uv - offset) * 0.1945946;
    result += textureSample(t_input, s_input, in.uv + offset * 2.0) * 0.1216216;
    result += textureSample(t_input, s_input, in.uv - offset * 2.0) * 0.1216216;
    result += textureSample(t_input, s_input, in.uv + offset * 3.0) * 0.054054;
    result += textureSample(t_input, s_input, in.uv - offset * 3.0) * 0.054054;
    result += textureSample(t_input, s_input, in.uv + offset * 4.0) * 0.016216;
    result += textureSample(t_input, s_input, in.uv - offset * 4.0) * 0.016216;
    return result;
}
