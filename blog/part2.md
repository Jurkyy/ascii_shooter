# Part 2: The ASCII Shader

*Turning 3D into terminal aesthetics*

---

The movement felt great, but it looked like... a generic 3D game. Time for the ASCII post-processing shader that gives this project its visual identity. The goal: not just a global ASCII filter, but per-object patterns where walls use block characters, tech panels show binary, and different materials have distinct visual identities.

---

## The Basic Idea

ASCII art works by mapping brightness to characters. Dark areas use sparse characters (`.`, `:`) while bright areas use dense characters (`#`, `@`, `█`). The shader needs to:

1. Divide the screen into character-sized cells
2. Sample the average color of each cell
3. Pick a character based on brightness
4. Render the character bitmap at that position

For per-object patterns, we need an additional step: a second render pass that writes pattern IDs to a texture, so the ASCII shader knows which character set to use for each pixel.

---

## Part 1: Setting Up a Post-Process Pass in Bevy

Bevy 0.16's render pipeline is powerful but complex. Post-process effects use `ViewNode` - a render graph node that operates on a camera's view:

```rust
#[derive(Default)]
struct AsciiNode;

impl ViewNode for AsciiNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static AsciiSettings,
        &'static DynamicUniformIndex<AsciiSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<AsciiPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        // Get the pattern texture for per-object mode
        let pattern_texture_view = if let Some(pattern_target) =
            world.get_resource::<PatternRenderTarget>()
        {
            let gpu_images = world.resource::<RenderAssets<GpuImage>>();
            if let Some(gpu_image) = gpu_images.get(&pattern_target.image) {
                &gpu_image.texture_view
            } else {
                &pipeline.fallback_texture_view
            }
        } else {
            &pipeline.fallback_texture_view
        };

        // Create bind group with both screen texture and pattern texture
        let bind_group = render_context.render_device().create_bind_group(
            "ascii_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                settings_binding.clone(),
                pattern_texture_view,
            )),
        );

        // Draw fullscreen triangle
        let mut render_pass = render_context.begin_tracked_render_pass(/* ... */);
        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
```

The key insight: `view_target.post_process_write()` gives you the current frame as input and a destination texture for output. The pattern texture is sampled alongside it to determine which character set to use.

---

## Part 2: Character Bitmaps in WGSL

WGSL doesn't have texture atlases in the traditional sense for this use case. Instead, I encoded character bitmaps directly as bit patterns.

Each character is a 5x7 bitmap stored as an array of 7 integers, where each integer represents a row of 5 bits:

```wgsl
// Pattern 0: Standard ASCII density ramp " .:-=+*#%@"
fn get_char_pixel_pattern0(char_index: u32, local_x: u32, local_y: u32) -> f32 {
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
    // ... etc

    let bit_pos = 4u - local_x;
    return f32((row_bits >> bit_pos) & 1u);
}
```

The number `31` is `11111` in binary - a full row. The number `4` is `00100` - just the center pixel.

### Why Not Use a Texture Atlas?

1. **No texture load/decode overhead** - the patterns are compiled into the shader
2. **No filtering artifacts** - nearest-neighbor sampling on tiny characters is tricky
3. **Multiple pattern sets are trivial** - just define more functions

---

## Part 3: Four Character Patterns

The game has four pattern sets, each giving a different visual feel:

### Pattern 0: Standard ASCII
The classic ` .:-=+*#%@` density ramp. Clean, readable, familiar.

### Pattern 1: Block Characters
Box-drawing style patterns - checkerboards, half-blocks, solid fills. More geometric, less textual.

```wgsl
let char1 = array<u32, 7>(0u, 10u, 0u, 10u, 0u, 10u, 0u);        // sparse dots
let char2 = array<u32, 7>(21u, 0u, 21u, 0u, 21u, 0u, 21u);       // checker
let char3 = array<u32, 7>(21u, 10u, 21u, 10u, 21u, 10u, 21u);    // dense checker
```

### Pattern 2: Slashes and Lines
Diagonal lines, X patterns, grid meshes. Adds movement and energy.

### Pattern 3: Binary/Digital
Numbers 0-9. Matrix rain vibes.

```wgsl
let char2 = array<u32, 7>(14u, 17u, 17u, 17u, 17u, 17u, 14u);    // 0
let char3 = array<u32, 7>(4u, 12u, 4u, 4u, 4u, 4u, 14u);         // 1
```

---

## Part 4: Per-Object Patterns with Render Layers

This is where it gets interesting. I wanted different objects to use different character sets - walls using blocks, tech panels using binary, organic shapes using standard ASCII. The solution: render layers with a second camera.

### The Pattern Material

First, a custom material that outputs the pattern ID as a color value:

```rust
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PatternIdMaterial {
    #[uniform(0)]
    pub pattern_id: f32,
}

impl Material for PatternIdMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/pattern_material.wgsl".into()
    }
}
```

The shader encodes the pattern ID (0-3) in the red channel:

```wgsl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Encode pattern ID: 0=0.0, 1=0.25, 2=0.5, 3=0.75
    let pattern_value = pattern.pattern_id / 4.0;
    return vec4<f32>(pattern_value, 0.0, 0.0, 1.0);
}
```

### The Pattern Camera

A second camera renders *only* objects with pattern materials to a texture:

```rust
pub const PATTERN_RENDER_LAYER: usize = 1;

fn setup_pattern_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pattern_target: ResMut<PatternRenderTarget>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else { return };

    // Create render target image
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("pattern_render_target"),
            size,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            ..default()
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);
    pattern_target.image = image_handle.clone();

    // Pattern camera - only sees layer 1, renders before main camera
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 100.0_f32.to_radians(), // Must match main camera!
            ..default()
        }),
        Camera {
            order: -1,
            target: RenderTarget::Image(image_handle.into()),
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderLayers::layer(PATTERN_RENDER_LAYER),
        PatternCamera,
        Msaa::Off, // Critical: MSAA must be off for clean pattern IDs
    ));
}
```

### Syncing Pattern Meshes

For each object with an `AsciiPatternId`, we create a mirror mesh on layer 1 with the pattern material:

```rust
fn sync_pattern_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<PatternIdMaterial>>,
    new_pattern_objects: Query<
        (Entity, &Mesh3d, &GlobalTransform, &AsciiPatternId),
        Added<AsciiPatternId>,
    >,
    mut pattern_meshes: Query<(Entity, &PatternMesh, &mut Transform)>,
) {
    // Create pattern meshes for new objects
    for (entity, mesh, transform, pattern_id) in &new_pattern_objects {
        let pattern_material = materials.add(PatternIdMaterial {
            pattern_id: pattern_id.pattern.as_id() as f32,
        });

        let (scale, rotation, translation) = transform.to_scale_rotation_translation();

        commands.spawn((
            Mesh3d(mesh.0.clone()),
            MeshMaterial3d(pattern_material),
            Transform { translation, rotation, scale },
            RenderLayers::layer(PATTERN_RENDER_LAYER),
            PatternMesh { source: entity },
        ));
    }

    // Update transforms each frame...
}
```

### Sampling in the ASCII Shader

The ASCII shader samples the pattern texture to determine which character set to use:

```wgsl
// Determine pattern ID
var pattern_id: u32 = u32(settings.global_pattern);

if settings.per_object_mode > 0.5 {
    // Per-object mode: sample from pattern texture
    let pattern_sample = textureSample(pattern_texture, texture_sampler, cell_center_uv);
    // Decode: pattern was encoded as value/4.0
    pattern_id = u32(pattern_sample.r * 4.0 + 0.5);
}

let char_pixel = get_char_pixel(pattern_id, char_index, char_local_x, char_local_y);
```

### Extracting to the Render World

Bevy's main app and render app are separate. The pattern texture handle must be extracted:

```rust
#[derive(Resource, Default, Clone, ExtractResource)]
pub struct PatternRenderTarget {
    pub image: Handle<Image>,
}

// In plugin build:
app.add_plugins(ExtractResourcePlugin::<PatternRenderTarget>::default());
```

Then in the render node, we look up the GPU image:

```rust
let pattern_texture_view = if let Some(pattern_target) =
    world.get_resource::<PatternRenderTarget>()
{
    let gpu_images = world.resource::<RenderAssets<GpuImage>>();
    gpu_images.get(&pattern_target.image)
        .map(|img| &img.texture_view)
        .unwrap_or(&pipeline.fallback_texture_view)
} else {
    &pipeline.fallback_texture_view
};
```

---

## Part 5: Runtime Controls

Being able to tweak settings without recompiling saved hours:

| Key | Action |
|-----|--------|
| F1 | Cycle resolution presets (Ultra/HighRes/Classic/Chunky) |
| F2 | Toggle monochrome green mode |
| F3 | Toggle per-object patterns |
| F4 | Cycle global pattern (when per-object is off) |

### Resolution Presets

```rust
pub enum AsciiPreset {
    Ultra,      // 3x5 cells - maximum detail
    HighRes,    // 5x9 cells - good balance
    Classic,    // 8x14 cells - traditional terminal
    Chunky,     // 12x20 cells - retro, large characters
}
```

Ultra at 3x5 is crisp enough to read details in the distance. Classic at 8x14 has that "my terminal in 1995" look.

### Visibility Tuning

Dark objects were hard to see. Two fixes:

1. **Gamma correction** lifts shadows without blowing out highlights:
```wgsl
let boosted_brightness = pow(brightness, 0.7);
```

2. **Background fill** prevents pure black pixels:
```wgsl
bg_color = boosted_color * 0.15;
output_color = mix(bg_color, output_color, char_pixel);
```

---

## Part 6: Movement Tuning

While working on the shader, I refined the movement. The bunny hop speed buildup was too aggressive - reducing air acceleration from 20.0 to 15.0 gives a more satisfying skill curve:

```rust
sv_airaccelerate: 15.0,  // Down from 20.0
```

Now there's clear progression: new players get some speed bonus, practiced players can still hit the cap, but it takes sustained technique rather than button mashing.

---

## Lessons Learned

1. **Render layers solve the MSAA problem.** A separate camera on its own layer with MSAA disabled can render pattern IDs cleanly, even when the main camera uses MSAA.

2. **Bevy's render world is separate.** Resources and assets need explicit extraction via `ExtractResource` or `ExtractComponent`.

3. **Camera projections must match.** The pattern camera needs identical FOV and aspect ratio or the textures won't align.

4. **Encode data efficiently.** Pattern IDs 0-3 fit in a single color channel with room to spare.

5. **Multi-sampling hides aliasing.** A single cell-center sample flickers. Five samples is cheap and smooth.

6. **Runtime controls are essential.** F-keys for instant parameter changes saved hours of iteration.

---

## Current State

The ASCII rendering is fully working:
- Four resolution presets
- Four character patterns
- Per-object pattern support via render layers
- Monochrome and colored modes
- All switchable at runtime

What's next:
- Combat prototype (hitscan weapons, basic damage)
- Enemies (something to shoot at)
- Game loop (win/lose conditions, waves)

---

*The source code is available at [github.com/Jurkyy/ascii_shooter](https://github.com/Jurkyy/ascii_shooter)*
