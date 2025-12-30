// Pattern ID Prepass Shader
// Renders object pattern IDs to a texture for per-object ASCII rendering

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct PatternUniform {
    pattern_id: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(0) @binding(0) var<uniform> view: mat4x4<f32>;
// Group 1 contains both mesh transform and pattern ID
@group(1) @binding(0) var<uniform> mesh: mat4x4<f32>;
@group(1) @binding(1) var<uniform> pattern: PatternUniform;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view * mesh * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Encode pattern ID in red channel (0-255 range mapped to 0-1)
    let pattern_value = f32(pattern.pattern_id) / 255.0;
    return vec4<f32>(pattern_value, 0.0, 0.0, 1.0);
}
