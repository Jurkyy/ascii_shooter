// ASCII Post-Processing Shader
// Converts the rendered scene into ASCII art
// Supports both global and per-object pattern modes

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct AsciiSettings {
    // Character cell size in pixels
    cell_size: vec2<f32>,
    // Screen resolution
    resolution: vec2<f32>,
    // 0 = colored, 1 = monochrome green
    monochrome: f32,
    // 0 = global pattern, 1 = per-object patterns
    per_object_mode: f32,
    // Padding for alignment
    _padding: vec2<f32>,
}
@group(0) @binding(2) var<uniform> settings: AsciiSettings;

// Pattern ID texture (only used in per-object mode)
@group(0) @binding(3) var pattern_texture: texture_2d<f32>;

// ============================================================================
// CHARACTER PATTERN DEFINITIONS
// Each pattern is a different set of ASCII characters for different aesthetics
// ============================================================================

// Pattern 0: Standard ASCII density ramp " .:-=+*#%@"
fn get_char_pixel_pattern0(char_index: u32, local_x: u32, local_y: u32) -> f32 {
    // Character bitmaps (5x7)
    let char0 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 0u, 0u);           // space
    let char1 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 4u, 4u);           // .
    let char2 = array<u32, 7>(0u, 4u, 4u, 0u, 4u, 4u, 0u);           // :
    let char3 = array<u32, 7>(0u, 0u, 0u, 31u, 0u, 0u, 0u);          // -
    let char4 = array<u32, 7>(0u, 0u, 31u, 0u, 31u, 0u, 0u);         // =
    let char5 = array<u32, 7>(0u, 4u, 4u, 31u, 4u, 4u, 0u);          // +
    let char6 = array<u32, 7>(0u, 21u, 14u, 31u, 14u, 21u, 0u);      // *
    let char7 = array<u32, 7>(10u, 31u, 10u, 10u, 31u, 10u, 0u);     // #
    let char8 = array<u32, 7>(19u, 19u, 4u, 4u, 25u, 25u, 0u);       // %
    let char9 = array<u32, 7>(14u, 17u, 23u, 21u, 23u, 16u, 14u);    // @

    var row_bits: u32 = 0u;
    if char_index == 0u { row_bits = char0[local_y]; }
    else if char_index == 1u { row_bits = char1[local_y]; }
    else if char_index == 2u { row_bits = char2[local_y]; }
    else if char_index == 3u { row_bits = char3[local_y]; }
    else if char_index == 4u { row_bits = char4[local_y]; }
    else if char_index == 5u { row_bits = char5[local_y]; }
    else if char_index == 6u { row_bits = char6[local_y]; }
    else if char_index == 7u { row_bits = char7[local_y]; }
    else if char_index == 8u { row_bits = char8[local_y]; }
    else { row_bits = char9[local_y]; }

    let bit_pos = 4u - local_x;
    return f32((row_bits >> bit_pos) & 1u);
}

// Pattern 1: Block/Box drawing characters (more solid look)
fn get_char_pixel_pattern1(char_index: u32, local_x: u32, local_y: u32) -> f32 {
    // Block patterns: empty, dots, light, medium, heavy, solid
    let char0 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 0u, 0u);           // empty
    let char1 = array<u32, 7>(0u, 10u, 0u, 10u, 0u, 10u, 0u);        // sparse dots
    let char2 = array<u32, 7>(21u, 0u, 21u, 0u, 21u, 0u, 21u);       // checker
    let char3 = array<u32, 7>(21u, 10u, 21u, 10u, 21u, 10u, 21u);    // dense checker
    let char4 = array<u32, 7>(0u, 0u, 0u, 0u, 31u, 31u, 31u);        // lower half
    let char5 = array<u32, 7>(31u, 31u, 31u, 0u, 0u, 0u, 0u);        // upper half
    let char6 = array<u32, 7>(14u, 31u, 31u, 31u, 31u, 31u, 14u);    // rounded block
    let char7 = array<u32, 7>(31u, 17u, 17u, 17u, 17u, 17u, 31u);    // box outline
    let char8 = array<u32, 7>(31u, 31u, 31u, 31u, 31u, 31u, 31u);    // solid
    let char9 = array<u32, 7>(31u, 31u, 31u, 31u, 31u, 31u, 31u);    // solid

    var row_bits: u32 = 0u;
    if char_index == 0u { row_bits = char0[local_y]; }
    else if char_index == 1u { row_bits = char1[local_y]; }
    else if char_index == 2u { row_bits = char2[local_y]; }
    else if char_index == 3u { row_bits = char3[local_y]; }
    else if char_index == 4u { row_bits = char4[local_y]; }
    else if char_index == 5u { row_bits = char5[local_y]; }
    else if char_index == 6u { row_bits = char6[local_y]; }
    else if char_index == 7u { row_bits = char7[local_y]; }
    else if char_index == 8u { row_bits = char8[local_y]; }
    else { row_bits = char9[local_y]; }

    let bit_pos = 4u - local_x;
    return f32((row_bits >> bit_pos) & 1u);
}

// Pattern 2: Slashes and lines (diagonal feel)
fn get_char_pixel_pattern2(char_index: u32, local_x: u32, local_y: u32) -> f32 {
    let char0 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 0u, 0u);           // empty
    let char1 = array<u32, 7>(0u, 0u, 0u, 4u, 0u, 0u, 0u);           // center dot
    let char2 = array<u32, 7>(1u, 2u, 4u, 8u, 16u, 0u, 0u);          // /
    let char3 = array<u32, 7>(16u, 8u, 4u, 2u, 1u, 0u, 0u);          // \
    let char4 = array<u32, 7>(17u, 10u, 4u, 10u, 17u, 0u, 0u);       // X light
    let char5 = array<u32, 7>(0u, 0u, 31u, 0u, 31u, 0u, 0u);         // ==
    let char6 = array<u32, 7>(4u, 4u, 31u, 4u, 31u, 4u, 4u);         // grid
    let char7 = array<u32, 7>(31u, 17u, 21u, 17u, 21u, 17u, 31u);    // mesh
    let char8 = array<u32, 7>(17u, 27u, 31u, 27u, 31u, 27u, 17u);    // dense X
    let char9 = array<u32, 7>(31u, 31u, 31u, 31u, 31u, 31u, 31u);    // solid

    var row_bits: u32 = 0u;
    if char_index == 0u { row_bits = char0[local_y]; }
    else if char_index == 1u { row_bits = char1[local_y]; }
    else if char_index == 2u { row_bits = char2[local_y]; }
    else if char_index == 3u { row_bits = char3[local_y]; }
    else if char_index == 4u { row_bits = char4[local_y]; }
    else if char_index == 5u { row_bits = char5[local_y]; }
    else if char_index == 6u { row_bits = char6[local_y]; }
    else if char_index == 7u { row_bits = char7[local_y]; }
    else if char_index == 8u { row_bits = char8[local_y]; }
    else { row_bits = char9[local_y]; }

    let bit_pos = 4u - local_x;
    return f32((row_bits >> bit_pos) & 1u);
}

// Pattern 3: Binary/Digital (0s and 1s feel)
fn get_char_pixel_pattern3(char_index: u32, local_x: u32, local_y: u32) -> f32 {
    // 0, 1, and digital patterns
    let char0 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 0u, 0u);           // empty
    let char1 = array<u32, 7>(0u, 4u, 4u, 4u, 4u, 0u, 4u);           // !
    let char2 = array<u32, 7>(14u, 17u, 17u, 17u, 17u, 17u, 14u);    // 0
    let char3 = array<u32, 7>(4u, 12u, 4u, 4u, 4u, 4u, 14u);         // 1
    let char4 = array<u32, 7>(14u, 17u, 1u, 14u, 16u, 16u, 31u);     // 2
    let char5 = array<u32, 7>(31u, 1u, 1u, 14u, 1u, 1u, 31u);        // 3
    let char6 = array<u32, 7>(17u, 17u, 17u, 31u, 1u, 1u, 1u);       // 4
    let char7 = array<u32, 7>(31u, 16u, 16u, 30u, 1u, 17u, 14u);     // 5
    let char8 = array<u32, 7>(14u, 17u, 31u, 17u, 17u, 17u, 14u);    // 8
    let char9 = array<u32, 7>(31u, 31u, 31u, 31u, 31u, 31u, 31u);    // solid

    var row_bits: u32 = 0u;
    if char_index == 0u { row_bits = char0[local_y]; }
    else if char_index == 1u { row_bits = char1[local_y]; }
    else if char_index == 2u { row_bits = char2[local_y]; }
    else if char_index == 3u { row_bits = char3[local_y]; }
    else if char_index == 4u { row_bits = char4[local_y]; }
    else if char_index == 5u { row_bits = char5[local_y]; }
    else if char_index == 6u { row_bits = char6[local_y]; }
    else if char_index == 7u { row_bits = char7[local_y]; }
    else if char_index == 8u { row_bits = char8[local_y]; }
    else { row_bits = char9[local_y]; }

    let bit_pos = 4u - local_x;
    return f32((row_bits >> bit_pos) & 1u);
}

// Get pixel from character bitmap based on pattern ID
fn get_char_pixel(pattern_id: u32, char_index: u32, local_x: u32, local_y: u32) -> f32 {
    let clamped_x = min(local_x, 4u);
    let clamped_y = min(local_y, 6u);

    if pattern_id == 1u {
        return get_char_pixel_pattern1(char_index, clamped_x, clamped_y);
    } else if pattern_id == 2u {
        return get_char_pixel_pattern2(char_index, clamped_x, clamped_y);
    } else if pattern_id == 3u {
        return get_char_pixel_pattern3(char_index, clamped_x, clamped_y);
    } else {
        return get_char_pixel_pattern0(char_index, clamped_x, clamped_y);
    }
}

// Calculate luminance from RGB
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pixel_coord = in.uv * settings.resolution;

    // Which character cell are we in?
    let cell_coord = floor(pixel_coord / settings.cell_size);

    // Local position within the cell (0 to cell_size)
    let local_pos = pixel_coord - cell_coord * settings.cell_size;

    // Map to character bitmap coordinates (5x7)
    let char_local_x = u32(local_pos.x / settings.cell_size.x * 5.0);
    let char_local_y = u32(local_pos.y / settings.cell_size.y * 7.0);

    // Sample the center of this cell to get average color
    let cell_center_uv = (cell_coord + 0.5) * settings.cell_size / settings.resolution;

    // Sample multiple points for better average
    let sample_offset = settings.cell_size / settings.resolution * 0.25;
    var total_color = vec3<f32>(0.0);
    total_color += textureSample(screen_texture, texture_sampler, cell_center_uv).rgb;
    total_color += textureSample(screen_texture, texture_sampler, cell_center_uv + vec2(-sample_offset.x, -sample_offset.y)).rgb;
    total_color += textureSample(screen_texture, texture_sampler, cell_center_uv + vec2(sample_offset.x, -sample_offset.y)).rgb;
    total_color += textureSample(screen_texture, texture_sampler, cell_center_uv + vec2(-sample_offset.x, sample_offset.y)).rgb;
    total_color += textureSample(screen_texture, texture_sampler, cell_center_uv + vec2(sample_offset.x, sample_offset.y)).rgb;
    let avg_color = total_color / 5.0;

    // Determine pattern ID
    var pattern_id: u32 = 0u;

    if settings.per_object_mode > 0.5 {
        // Per-object mode: sample pattern ID from pattern texture
        // Pattern ID is encoded in the red channel (0-255 mapped to 0-1)
        let pattern_sample = textureSample(pattern_texture, texture_sampler, cell_center_uv);
        pattern_id = u32(pattern_sample.r * 255.0 + 0.5);
    }

    // Get brightness and map to character index (0-9)
    let brightness = luminance(avg_color);
    let char_index = u32(clamp(brightness * 10.0, 0.0, 9.0));

    // Get the pixel value from the character bitmap
    let char_pixel = get_char_pixel(pattern_id, char_index, char_local_x, char_local_y);

    // Boost brightness for better visibility
    let boosted_color = avg_color * 1.4;

    // Output color
    var output_color: vec3<f32>;
    var bg_color: vec3<f32>;

    if settings.monochrome > 0.5 {
        // Classic green terminal look
        let green = vec3<f32>(0.0, 1.0, 0.3);
        output_color = green * char_pixel * brightness * 1.3;
        bg_color = green * brightness * 0.15;
    } else {
        // Colored ASCII - use original color tinted by character
        output_color = boosted_color * char_pixel;
        bg_color = boosted_color * 0.35;
    }

    // Background fill for non-character pixels
    output_color = mix(bg_color, output_color, char_pixel);

    return vec4<f32>(output_color, 1.0);
}
