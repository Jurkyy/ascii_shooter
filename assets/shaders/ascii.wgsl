// ASCII Post-Processing Shader
// Converts the rendered scene into ASCII art

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
    // Padding for alignment
    _padding: vec3<f32>,
}
@group(0) @binding(2) var<uniform> settings: AsciiSettings;

// ASCII character patterns (5x7 bitmap encoded as bits)
// Characters: " .:-=+*#%@" (10 levels from dark to bright)
// Each character is a 5-wide x 7-tall bitmap
// Stored as array of 7 rows, each row is 5 bits

// Get pixel from character bitmap
fn get_char_pixel(char_index: u32, local_x: u32, local_y: u32) -> f32 {
    // Character bitmaps (5x7) - encoded as integers
    // Row 0 is top, bit 0 is left

    // Space (empty)
    let char0 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 0u, 0u);
    // .
    let char1 = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 4u, 4u);
    // :
    let char2 = array<u32, 7>(0u, 4u, 4u, 0u, 4u, 4u, 0u);
    // -
    let char3 = array<u32, 7>(0u, 0u, 0u, 31u, 0u, 0u, 0u);
    // =
    let char4 = array<u32, 7>(0u, 0u, 31u, 0u, 31u, 0u, 0u);
    // +
    let char5 = array<u32, 7>(0u, 4u, 4u, 31u, 4u, 4u, 0u);
    // *
    let char6 = array<u32, 7>(0u, 21u, 14u, 31u, 14u, 21u, 0u);
    // #
    let char7 = array<u32, 7>(10u, 31u, 10u, 10u, 31u, 10u, 0u);
    // %
    let char8 = array<u32, 7>(19u, 19u, 4u, 4u, 25u, 25u, 0u);
    // @
    let char9 = array<u32, 7>(14u, 17u, 23u, 21u, 23u, 16u, 14u);

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

    // Check if bit at local_x is set (bit 4 is leftmost, bit 0 is rightmost)
    let bit_pos = 4u - local_x;
    let is_set = (row_bits >> bit_pos) & 1u;

    return f32(is_set);
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

    // Get brightness and map to character index (0-9)
    let brightness = luminance(avg_color);
    let char_index = u32(clamp(brightness * 10.0, 0.0, 9.0));

    // Get the pixel value from the character bitmap
    let char_pixel = get_char_pixel(char_index, char_local_x, char_local_y);

    // Output color
    var output_color: vec3<f32>;

    if settings.monochrome > 0.5 {
        // Classic green terminal look
        output_color = vec3<f32>(0.0, 1.0, 0.3) * char_pixel * brightness;
    } else {
        // Colored ASCII - use original color tinted by character
        output_color = avg_color * char_pixel;
    }

    // Add slight background for visibility
    let bg_color = avg_color * 0.1;
    output_color = mix(bg_color, output_color, char_pixel);

    return vec4<f32>(output_color, 1.0);
}
