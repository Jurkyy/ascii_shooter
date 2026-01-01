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
    // Global pattern ID (0-3) used when per_object_mode is 0
    global_pattern: f32,
    // Animation time in seconds
    time: f32,
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

// Pattern 4: Matrix Rain - animated falling digital characters
fn get_char_pixel_pattern4(char_index: u32, local_x: u32, local_y: u32, cell_x: f32, cell_y: f32, time: f32) -> f32 {
    // Use same character set as Binary
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

    // Create falling rain effect
    // Each column falls at a different speed based on pseudo-random offset
    let column_seed = fract(sin(cell_x * 12.9898) * 43758.5453);
    let fall_speed = 3.0 + column_seed * 4.0; // Varying speeds
    let fall_offset = column_seed * 50.0; // Stagger start positions

    // Animated character selection based on time and position
    let time_offset = floor(time * 8.0 + cell_y * fall_speed + fall_offset);
    let char_variation = u32(fract(sin(cell_x * 78.233 + time_offset * 45.164) * 43758.5453) * 8.0) + 1u;

    // Use animated character for visible cells, empty for dark areas
    let animated_char = select(0u, char_variation, char_index > 0u);

    var row_bits: u32 = 0u;
    if animated_char == 0u { row_bits = char0[local_y]; }
    else if animated_char == 1u { row_bits = char1[local_y]; }
    else if animated_char == 2u { row_bits = char2[local_y]; }
    else if animated_char == 3u { row_bits = char3[local_y]; }
    else if animated_char == 4u { row_bits = char4[local_y]; }
    else if animated_char == 5u { row_bits = char5[local_y]; }
    else if animated_char == 6u { row_bits = char6[local_y]; }
    else if animated_char == 7u { row_bits = char7[local_y]; }
    else if animated_char == 8u { row_bits = char8[local_y]; }
    else { row_bits = char9[local_y]; }

    let bit_pos = 4u - local_x;
    return f32((row_bits >> bit_pos) & 1u);
}

// Get pixel from character bitmap based on pattern ID
fn get_char_pixel(pattern_id: u32, char_index: u32, local_x: u32, local_y: u32, cell_x: f32, cell_y: f32, time: f32) -> f32 {
    let clamped_x = min(local_x, 4u);
    let clamped_y = min(local_y, 6u);

    if pattern_id == 1u {
        return get_char_pixel_pattern1(char_index, clamped_x, clamped_y);
    } else if pattern_id == 2u {
        return get_char_pixel_pattern2(char_index, clamped_x, clamped_y);
    } else if pattern_id == 3u {
        return get_char_pixel_pattern3(char_index, clamped_x, clamped_y);
    } else if pattern_id == 4u {
        return get_char_pixel_pattern4(char_index, clamped_x, clamped_y, cell_x, cell_y, time);
    } else {
        return get_char_pixel_pattern0(char_index, clamped_x, clamped_y);
    }
}

// ============================================================================
// SIMPLE PROCEDURAL PATTERNS FOR SMALL CELL SIZES
// These work at any resolution and capture the "essence" of each pattern type
// ============================================================================

// Pattern 0 (Standard): Dot/line density based on brightness
fn get_simple_pattern0(local_uv: vec2<f32>, density: f32) -> f32 {
    // Simple ordered dithering - dots appear based on density
    let threshold = fract(local_uv.x * 2.0 + local_uv.y * 3.0);
    return select(0.0, 1.0, density > threshold);
}

// Pattern 1 (Blocks): Checkerboard with density
fn get_simple_pattern1(local_uv: vec2<f32>, density: f32) -> f32 {
    // Checkerboard pattern - classic block look
    let check_x = floor(local_uv.x * 2.0);
    let check_y = floor(local_uv.y * 2.0);
    let checker = (check_x + check_y) % 2.0;
    // Blend checker with solid based on density
    if density < 0.3 {
        return 0.0;
    } else if density < 0.6 {
        return checker;
    } else {
        return 1.0;
    }
}

// Pattern 2 (Slashes): Diagonal stripes
fn get_simple_pattern2(local_uv: vec2<f32>, density: f32) -> f32 {
    // Diagonal stripe pattern
    let diag = fract(local_uv.x + local_uv.y);
    let stripe_width = 0.3 + density * 0.4; // Wider stripes at higher density
    return select(0.0, 1.0, diag < stripe_width);
}

// Pattern 3 (Binary): Vertical lines like "1"s
fn get_simple_pattern3(local_uv: vec2<f32>, density: f32) -> f32 {
    // Vertical lines with horizontal segments - looks like "1"s
    let vert = abs(local_uv.x - 0.5) < 0.15; // Center vertical line
    let top_hook = local_uv.y < 0.25 && local_uv.x < 0.5 && local_uv.x > 0.2;
    let bottom_base = local_uv.y > 0.8;

    if density < 0.2 {
        return 0.0;
    } else if density < 0.5 {
        return select(0.0, 1.0, vert);
    } else if density < 0.8 {
        return select(0.0, 1.0, vert || top_hook);
    } else {
        return select(0.0, 1.0, vert || top_hook || bottom_base);
    }
}

// Get simple procedural pattern pixel
fn get_simple_pattern(pattern_id: u32, local_uv: vec2<f32>, density: f32) -> f32 {
    if pattern_id == 1u {
        return get_simple_pattern1(local_uv, density);
    } else if pattern_id == 2u {
        return get_simple_pattern2(local_uv, density);
    } else if pattern_id == 3u {
        return get_simple_pattern3(local_uv, density);
    } else {
        return get_simple_pattern0(local_uv, density);
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
    var pattern_id: u32 = u32(settings.global_pattern);

    if settings.per_object_mode > 0.5 {
        // Per-object mode: sample pattern ID from pattern texture
        // Pattern ID is encoded in the red channel as value / 5.0 (0-4 → 0.0-0.8)
        let pattern_sample = textureSample(pattern_texture, texture_sampler, cell_center_uv);
        // Decode: multiply by 5 and round (add 0.5 for rounding)
        pattern_id = u32(pattern_sample.r * 5.0 + 0.5);
    }

    // Get brightness and map to character index (0-9)
    let brightness = luminance(avg_color);
    // Boost darker areas so patterns are visible even on dim surfaces
    let boosted_brightness = pow(brightness, 0.7); // Gamma correction to lift shadows
    let char_index = u32(clamp(boosted_brightness * 10.0, 0.0, 9.0));

    // Render bitmap characters at Classic (8x14) size for all resolutions
    // This ensures patterns are equally legible regardless of cell size
    var char_pixel: f32;
    if settings.cell_size.x < 8.0 {
        // Render characters at fixed Classic size, tiled across the screen
        let reference_size = vec2<f32>(8.0, 14.0);
        let tiled_pos = fract(pixel_coord / reference_size) * reference_size;
        let ref_char_x = u32(tiled_pos.x / reference_size.x * 5.0);
        let ref_char_y = u32(tiled_pos.y / reference_size.y * 7.0);
        let ref_cell = floor(pixel_coord / reference_size);
        char_pixel = get_char_pixel(pattern_id, char_index, ref_char_x, ref_char_y, ref_cell.x, ref_cell.y, settings.time);
    } else {
        // Use cell-sized character bitmaps for Classic and larger
        char_pixel = get_char_pixel(pattern_id, char_index, char_local_x, char_local_y, cell_coord.x, cell_coord.y, settings.time);
    }

    // Boost brightness for better visibility
    let boosted_color = avg_color * 2.0;

    // Output color
    var output_color: vec3<f32>;
    var bg_color: vec3<f32>;

    if settings.monochrome > 0.5 {
        // Classic green terminal look
        let green = vec3<f32>(0.0, 1.0, 0.3);
        output_color = green * char_pixel * brightness * 1.5;
        bg_color = green * brightness * 0.1;
    } else {
        // Colored ASCII - use original color tinted by character
        // Higher contrast: brighter characters, darker background
        output_color = boosted_color * char_pixel;
        bg_color = boosted_color * 0.15;
    }

    // Background fill for non-character pixels
    output_color = mix(bg_color, output_color, char_pixel);

    return vec4<f32>(output_color, 1.0);
}
