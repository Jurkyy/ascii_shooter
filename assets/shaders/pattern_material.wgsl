// Pattern ID Material Shader
// Outputs the pattern ID as a color value for the ASCII shader to read

#import bevy_pbr::forward_io::VertexOutput

struct PatternIdUniform {
    pattern_id: f32,
}

@group(2) @binding(0) var<uniform> pattern: PatternIdUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Encode pattern ID in red channel (0-5 mapped to 0.0-0.833 range)
    // We use 1/6 steps: 0=0.0, 1=0.167, 2=0.333, 3=0.5, 4=0.667, 5=0.833
    let pattern_value = pattern.pattern_id / 6.0;
    return vec4<f32>(pattern_value, 0.0, 0.0, 1.0);
}
