// Custom material for rendering pattern IDs to a texture
// This material outputs the pattern ID in the red channel

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    pbr::{Material, MaterialPlugin},
};

/// Material that renders a pattern ID value
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PatternIdMaterial {
    #[uniform(0)]
    pub pattern_id: f32,
}

impl Material for PatternIdMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/pattern_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

/// Plugin to register the pattern material
pub struct PatternMaterialPlugin;

impl Plugin for PatternMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PatternIdMaterial>::default());
    }
}
