// ASCII Post-Processing Effect
// Renders the scene as ASCII art
// Supports both global and per-object pattern modes

use bevy::{
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
        Extract, Render, RenderApp, RenderSet,
    },
};

const ASCII_SHADER_PATH: &str = "shaders/ascii.wgsl";

pub struct AsciiRenderPlugin;

impl Plugin for AsciiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<AsciiSettings>::default(),
            UniformComponentPlugin::<AsciiSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(Render, prepare_pattern_texture.in_set(RenderSet::PrepareResources))
            .add_render_graph_node::<ViewNodeRunner<AsciiNode>>(
                Core3d,
                AsciiNodeLabel,
            )
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

        // Get the pattern texture (or fallback to a dummy)
        let pattern_texture_view = world
            .get_resource::<PatternTextureResource>()
            .map(|r| &r.texture_view)
            .unwrap_or(&pipeline.fallback_texture_view);

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
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let fallback_texture_view = fallback_texture.create_view(&TextureViewDescriptor::default());

        let shader = world.load_asset(ASCII_SHADER_PATH);

        let pipeline_id = world.resource_mut::<PipelineCache>().queue_render_pipeline(
            RenderPipelineDescriptor {
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
            },
        );

        Self {
            layout,
            sampler,
            pipeline_id,
            fallback_texture_view,
        }
    }
}

/// Resource holding the pattern texture for per-object mode
#[derive(Resource)]
struct PatternTextureResource {
    texture_view: TextureView,
}

/// System to prepare the pattern texture
fn prepare_pattern_texture(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
) {
    // For now, create a simple fallback texture
    // In a full implementation, this would be populated by a prepass
    // that renders object pattern IDs

    let texture = texture_cache.get(
        &render_device,
        TextureDescriptor {
            label: Some("ascii_pattern_texture"),
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
    );

    let texture_view = texture.default_view.clone();

    commands.insert_resource(PatternTextureResource { texture_view });
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
    /// Padding for GPU alignment
    _padding: Vec2,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self {
            cell_size: Vec2::new(8.0, 14.0),
            resolution: Vec2::new(1280.0, 720.0),
            monochrome: 0.0,
            per_object_mode: 0.0,
            _padding: Vec2::ZERO,
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
pub fn update_ascii_resolution(
    windows: Query<&Window>,
    mut settings: Query<&mut AsciiSettings>,
) {
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_settings_default() {
        let settings = AsciiSettings::default();
        assert_eq!(settings.cell_size, Vec2::new(8.0, 14.0));
        assert_eq!(settings.monochrome, 0.0);
        assert_eq!(settings.per_object_mode, 0.0);
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
