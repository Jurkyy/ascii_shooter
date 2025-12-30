// ASCII Post-Processing Effect with Per-Object Pattern Support
// Renders the scene as ASCII art with optional per-object character patterns

mod pattern_material;

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::query::QueryItem,
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode,
            ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{RenderLayers, ViewTarget},
        RenderApp,
        render_asset::RenderAssets,
        texture::GpuImage,
    },
    core_pipeline::core_3d::graph::{Core3d, Node3d},
};

pub use pattern_material::{PatternIdMaterial, PatternMaterialPlugin};

const ASCII_SHADER_PATH: &str = "shaders/ascii.wgsl";

/// Render layer for pattern ID rendering (layer 1)
pub const PATTERN_RENDER_LAYER: usize = 1;

pub struct AsciiRenderPlugin;

impl Plugin for AsciiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<AsciiSettings>::default(),
            UniformComponentPlugin::<AsciiSettings>::default(),
            ExtractResourcePlugin::<PatternRenderTarget>::default(),
            PatternMaterialPlugin,
        ))
        .init_resource::<PatternRenderTarget>()
        .add_systems(Startup, setup_pattern_camera)
        .add_systems(Update, (
            sync_pattern_meshes,
            sync_pattern_camera_transform,
            update_pattern_render_target_size,
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<AsciiNode>>(Core3d, AsciiNodeLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    AsciiNodeLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<AsciiPipeline>();
    }
}

// ============================================================================
// PATTERN CAMERA SYSTEM - Uses render layers for per-object patterns
// ============================================================================

/// Resource holding the pattern render target image handle
#[derive(Resource, Default, Clone, ExtractResource)]
pub struct PatternRenderTarget {
    pub image: Handle<Image>,
}

/// Marker for the pattern camera
#[derive(Component)]
pub struct PatternCamera;

/// Marker for pattern mesh entities (clones of main meshes on layer 1)
#[derive(Component)]
pub struct PatternMesh {
    /// The source entity this pattern mesh mirrors
    pub source: Entity,
}

/// Setup the pattern camera that renders to a texture
fn setup_pattern_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pattern_target: ResMut<PatternRenderTarget>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        depth_or_array_layers: 1,
    };

    // Create the render target image
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("pattern_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);
    pattern_target.image = image_handle.clone();

    // Spawn pattern camera - renders only layer 1
    // Must match main camera projection for correct alignment
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 100.0_f32.to_radians(), // Match main camera FOV
            ..default()
        }),
        Camera {
            order: -1, // Render before main camera
            target: RenderTarget::Image(image_handle.into()),
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderLayers::layer(PATTERN_RENDER_LAYER),
        PatternCamera,
        Msaa::Off,
    ));
}

/// Sync pattern camera transform with main camera
fn sync_pattern_camera_transform(
    main_camera: Query<&GlobalTransform, (With<Camera3d>, Without<PatternCamera>)>,
    mut pattern_camera: Query<&mut Transform, With<PatternCamera>>,
) {
    let Ok(main_transform) = main_camera.single() else {
        return;
    };
    let Ok(mut pattern_transform) = pattern_camera.single_mut() else {
        return;
    };

    // Copy the global transform to local (pattern camera has no parent)
    let (scale, rotation, translation) = main_transform.to_scale_rotation_translation();
    pattern_transform.translation = translation;
    pattern_transform.rotation = rotation;
    pattern_transform.scale = scale;
}

/// Update pattern render target size when window resizes
fn update_pattern_render_target_size(
    windows: Query<&Window>,
    pattern_target: Res<PatternRenderTarget>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let new_size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        depth_or_array_layers: 1,
    };

    if let Some(image) = images.get_mut(&pattern_target.image) {
        if image.texture_descriptor.size != new_size {
            image.resize(new_size);
        }
    }
}

/// Sync pattern meshes - create/update pattern mesh entities for objects with AsciiPatternId
fn sync_pattern_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<PatternIdMaterial>>,
    // Objects with pattern IDs that need pattern meshes
    pattern_objects: Query<
        (Entity, &Mesh3d, &GlobalTransform, &AsciiPatternId),
        Changed<GlobalTransform>,
    >,
    // New objects that need pattern meshes created
    new_pattern_objects: Query<
        (Entity, &Mesh3d, &GlobalTransform, &AsciiPatternId),
        Added<AsciiPatternId>,
    >,
    // Existing pattern meshes
    mut pattern_meshes: Query<(Entity, &PatternMesh, &mut Transform)>,
    // All pattern objects (for cleanup check)
    all_pattern_objects: Query<Entity, With<AsciiPatternId>>,
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
            Transform {
                translation,
                rotation,
                scale,
            },
            RenderLayers::layer(PATTERN_RENDER_LAYER),
            PatternMesh { source: entity },
        ));
    }

    // Update transforms for existing pattern meshes
    for (entity, _mesh, global_transform, _) in &pattern_objects {
        for (_, pattern_mesh, mut transform) in &mut pattern_meshes {
            if pattern_mesh.source == entity {
                let (scale, rotation, translation) = global_transform.to_scale_rotation_translation();
                transform.translation = translation;
                transform.rotation = rotation;
                transform.scale = scale;
            }
        }
    }

    // Clean up orphaned pattern meshes
    for (pattern_entity, pattern_mesh, _) in &pattern_meshes {
        if all_pattern_objects.get(pattern_mesh.source).is_err() {
            commands.entity(pattern_entity).despawn();
        }
    }
}

// ============================================================================
// ASCII POST-PROCESS
// ============================================================================

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct AsciiNodeLabel;

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

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<AsciiSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        // Try to get the pattern texture from the render target
        let pattern_texture_view = if let Some(pattern_target) = world.get_resource::<PatternRenderTarget>() {
            let gpu_images = world.resource::<RenderAssets<GpuImage>>();
            if let Some(gpu_image) = gpu_images.get(&pattern_target.image) {
                &gpu_image.texture_view
            } else {
                &pipeline.fallback_texture_view
            }
        } else {
            &pipeline.fallback_texture_view
        };

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

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ascii_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct AsciiPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
    fallback_texture_view: TextureView,
}

impl FromWorld for AsciiPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "ascii_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // Screen texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // Sampler
                    sampler(SamplerBindingType::Filtering),
                    // Settings uniform
                    uniform_buffer::<AsciiSettings>(true),
                    // Pattern ID texture (for per-object mode)
                    texture_2d(TextureSampleType::Float { filterable: true }),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Create a 1x1 fallback texture for when per-object mode is disabled
        let fallback_texture = render_device.create_texture(&TextureDescriptor {
            label: Some("ascii_fallback_pattern_texture"),
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let fallback_texture_view =
            fallback_texture.create_view(&TextureViewDescriptor::default());

        let shader = world.load_asset(ASCII_SHADER_PATH);

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("ascii_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            sampler,
            pipeline_id,
            fallback_texture_view,
        }
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// ASCII rendering settings - attach to camera to enable effect
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct AsciiSettings {
    /// Character cell size in pixels (width, height)
    pub cell_size: Vec2,
    /// Screen resolution (set automatically)
    pub resolution: Vec2,
    /// 0.0 = colored, 1.0 = monochrome green
    pub monochrome: f32,
    /// 0.0 = global pattern, 1.0 = per-object patterns
    pub per_object_mode: f32,
    /// Global pattern ID (0-3) used when per_object_mode is 0
    pub global_pattern: f32,
    /// Padding for GPU alignment
    _padding: f32,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self {
            cell_size: Vec2::new(3.0, 5.0),
            resolution: Vec2::new(1280.0, 720.0),
            monochrome: 0.0,
            per_object_mode: 0.0,
            global_pattern: 0.0,
            _padding: 0.0,
        }
    }
}

impl AsciiSettings {
    /// Create settings with custom cell size
    pub fn new(cell_width: f32, cell_height: f32) -> Self {
        Self {
            cell_size: Vec2::new(cell_width, cell_height),
            ..default()
        }
    }

    /// Create monochrome (green terminal) settings
    pub fn monochrome() -> Self {
        Self {
            monochrome: 1.0,
            ..default()
        }
    }

    /// Enable per-object pattern mode
    pub fn with_per_object_patterns(mut self) -> Self {
        self.per_object_mode = 1.0;
        self
    }

    /// Set monochrome mode
    pub fn with_monochrome(mut self, enabled: bool) -> Self {
        self.monochrome = if enabled { 1.0 } else { 0.0 };
        self
    }

    /// Apply a preset to these settings
    pub fn apply_preset(&mut self, preset: AsciiPreset) {
        match preset {
            AsciiPreset::Ultra => {
                self.cell_size = Vec2::new(3.0, 5.0);
            }
            AsciiPreset::HighRes => {
                self.cell_size = Vec2::new(5.0, 9.0);
            }
            AsciiPreset::Classic => {
                self.cell_size = Vec2::new(8.0, 14.0);
            }
            AsciiPreset::Chunky => {
                self.cell_size = Vec2::new(12.0, 20.0);
            }
        }
    }
}

/// Visual presets for ASCII rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Resource)]
pub enum AsciiPreset {
    /// Ultra resolution - tiny characters, maximum detail (3x5)
    #[default]
    Ultra,
    /// High resolution - small characters, more detail (5x9)
    HighRes,
    /// Classic look - medium characters (8x14)
    Classic,
    /// Chunky retro - large characters (12x20)
    Chunky,
}

impl AsciiPreset {
    /// Cycle to the next preset
    pub fn next(self) -> Self {
        match self {
            AsciiPreset::Ultra => AsciiPreset::HighRes,
            AsciiPreset::HighRes => AsciiPreset::Classic,
            AsciiPreset::Classic => AsciiPreset::Chunky,
            AsciiPreset::Chunky => AsciiPreset::Ultra,
        }
    }

    /// Get display name for this preset
    pub fn name(&self) -> &'static str {
        match self {
            AsciiPreset::Ultra => "Ultra (3x5)",
            AsciiPreset::HighRes => "High-Res (5x9)",
            AsciiPreset::Classic => "Classic (8x14)",
            AsciiPreset::Chunky => "Chunky (12x20)",
        }
    }
}

/// ASCII pattern types for per-object rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AsciiPattern {
    /// Standard ASCII: " .:-=+*#%@"
    #[default]
    Standard = 0,
    /// Block/Box characters (solid look)
    Blocks = 1,
    /// Slashes and diagonals
    Slashes = 2,
    /// Binary/Digital (0s and 1s)
    Binary = 3,
}

impl AsciiPattern {
    /// Get the pattern ID as a u8 for GPU encoding
    pub fn as_id(&self) -> u8 {
        *self as u8
    }
}

/// Component to assign an ASCII pattern to an object
/// When AsciiSettings::per_object_mode is enabled, objects with this component
/// will use their specified pattern instead of the global pattern
#[derive(Component, Clone, Copy, Default)]
pub struct AsciiPatternId {
    pub pattern: AsciiPattern,
}

impl AsciiPatternId {
    pub fn new(pattern: AsciiPattern) -> Self {
        Self { pattern }
    }

    pub fn standard() -> Self {
        Self::new(AsciiPattern::Standard)
    }

    pub fn blocks() -> Self {
        Self::new(AsciiPattern::Blocks)
    }

    pub fn slashes() -> Self {
        Self::new(AsciiPattern::Slashes)
    }

    pub fn binary() -> Self {
        Self::new(AsciiPattern::Binary)
    }
}

/// System to update resolution in settings based on window size
pub fn update_ascii_resolution(windows: Query<&Window>, mut settings: Query<&mut AsciiSettings>) {
    let Ok(window) = windows.single() else {
        return;
    };

    let resolution = Vec2::new(window.width(), window.height());

    for mut setting in &mut settings {
        if setting.resolution != resolution {
            setting.resolution = resolution;
        }
    }
}

/// System to cycle ASCII presets with F1 key
pub fn cycle_ascii_preset(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut preset: ResMut<AsciiPreset>,
    mut settings: Query<&mut AsciiSettings>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        *preset = preset.next();
        info!("ASCII Preset: {}", preset.name());

        for mut setting in &mut settings {
            setting.apply_preset(*preset);
        }
    }
}

/// System to toggle monochrome mode with F2 key
pub fn toggle_ascii_monochrome(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: Query<&mut AsciiSettings>,
) {
    if keyboard.just_pressed(KeyCode::F2) {
        for mut setting in &mut settings {
            let new_mono = if setting.monochrome > 0.5 { 0.0 } else { 1.0 };
            setting.monochrome = new_mono;
            info!("Monochrome: {}", if new_mono > 0.5 { "ON" } else { "OFF" });
        }
    }
}

/// System to toggle per-object pattern mode with F3 key
pub fn toggle_per_object_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: Query<&mut AsciiSettings>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        for mut setting in &mut settings {
            let new_mode = if setting.per_object_mode > 0.5 {
                0.0
            } else {
                1.0
            };
            setting.per_object_mode = new_mode;
            info!(
                "Per-Object Patterns: {}",
                if new_mode > 0.5 { "ON" } else { "OFF" }
            );
        }
    }
}

/// System to cycle global pattern with F4 key
pub fn cycle_global_pattern(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: Query<&mut AsciiSettings>,
) {
    if keyboard.just_pressed(KeyCode::F4) {
        for mut setting in &mut settings {
            let current = setting.global_pattern as u32;
            let next = (current + 1) % 4;
            setting.global_pattern = next as f32;
            let name = match next {
                0 => "Standard",
                1 => "Blocks",
                2 => "Slashes",
                3 => "Binary",
                _ => "Unknown",
            };
            info!("Global Pattern: {} ({})", next, name);
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_settings_default() {
        let settings = AsciiSettings::default();
        assert_eq!(settings.cell_size, Vec2::new(3.0, 5.0));
        assert_eq!(settings.monochrome, 0.0);
        assert_eq!(settings.per_object_mode, 0.0);
        assert_eq!(settings.global_pattern, 0.0);
    }

    #[test]
    fn test_ascii_settings_monochrome() {
        let settings = AsciiSettings::monochrome();
        assert_eq!(settings.monochrome, 1.0);
    }

    #[test]
    fn test_ascii_settings_per_object() {
        let settings = AsciiSettings::default().with_per_object_patterns();
        assert_eq!(settings.per_object_mode, 1.0);
    }

    #[test]
    fn test_ascii_settings_custom_cell_size() {
        let settings = AsciiSettings::new(10.0, 16.0);
        assert_eq!(settings.cell_size, Vec2::new(10.0, 16.0));
    }

    #[test]
    fn test_ascii_pattern_ids() {
        assert_eq!(AsciiPattern::Standard.as_id(), 0);
        assert_eq!(AsciiPattern::Blocks.as_id(), 1);
        assert_eq!(AsciiPattern::Slashes.as_id(), 2);
        assert_eq!(AsciiPattern::Binary.as_id(), 3);
    }

    #[test]
    fn test_ascii_pattern_id_constructors() {
        assert_eq!(AsciiPatternId::standard().pattern, AsciiPattern::Standard);
        assert_eq!(AsciiPatternId::blocks().pattern, AsciiPattern::Blocks);
        assert_eq!(AsciiPatternId::slashes().pattern, AsciiPattern::Slashes);
        assert_eq!(AsciiPatternId::binary().pattern, AsciiPattern::Binary);
    }
}
