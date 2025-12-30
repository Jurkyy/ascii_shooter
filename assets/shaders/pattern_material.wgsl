// Pattern ID Material Shader
// Outputs the pattern ID as a color value for the ASCII shader to read

#import bevy_pbr::forward_io::VertexOutput

struct PatternIdUniform {
    pattern_id: f32,
}

@group(2) @binding(0) var<uniform> pattern: PatternIdUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Encode pattern ID in red channel (0-3 mapped to 0.0-1.0 range)
    // We use 0.25 steps: 0=0.0, 1=0.25, 2=0.5, 3=0.75
    let pattern_value = pattern.pattern_id / 4.0;
    return vec4<f32>(pattern_value, 0.0, 0.0, 1.0);
}
