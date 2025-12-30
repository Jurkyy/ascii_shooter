// ASCII Post-Processing Effect
// Renders the scene as ASCII art

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
        view::ViewTarget,
        RenderApp,
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

        let bind_group = render_context.render_device().create_bind_group(
            "ascii_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                settings_binding.clone(),
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
}

impl FromWorld for AsciiPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "ascii_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<AsciiSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

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
        }
    }
}

/// ASCII rendering settings - attach to camera to enable effect
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct AsciiSettings {
    /// Character cell size in pixels (width, height)
    pub cell_size: Vec2,
    /// Screen resolution (set automatically)
    pub resolution: Vec2,
    /// 0.0 = colored, 1.0 = monochrome green
    pub monochrome: f32,
    /// Padding for GPU alignment
    _padding: Vec3,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self {
            cell_size: Vec2::new(8.0, 14.0),
            resolution: Vec2::new(1280.0, 720.0),
            monochrome: 0.0,
            _padding: Vec3::ZERO,
        }
    }
}

impl AsciiSettings {
    pub fn new(cell_width: f32, cell_height: f32) -> Self {
        Self {
            cell_size: Vec2::new(cell_width, cell_height),
            ..default()
        }
    }

    pub fn monochrome() -> Self {
        Self {
            monochrome: 1.0,
            ..default()
        }
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
