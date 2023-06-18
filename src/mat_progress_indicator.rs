use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "0d6e2556-e4f3-4f7e-8a2b-024592203604"]
pub struct MaterialProgressIndicator {
    #[uniform(0)]
    pub background_color: Color,

    #[uniform(0)]
    pub aspect_ratio: f32,

    #[uniform(0)]
    pub ring_inner_radius: f32,
    #[uniform(0)]
    pub ring_outer_radius: f32,
    #[uniform(0)]
    pub ring_color: Color,

    #[uniform(0)]
    pub nob_radius: f32,
    #[uniform(0)]
    pub nob_color: Color,
    #[uniform(0)]
    pub nob_rotation_speed: f32,

    #[uniform(0)]
    pub dot_radius: f32,
    #[uniform(0)]
    pub dot_color: Color,
    #[uniform(0)]
    pub dot_distance: f32,
    #[uniform(0)]
    pub dot_margin_top: f32,
    #[uniform(0)]
    pub dot_fade_speed: f32,

    #[uniform(0)]
    pub anti_alias_factor: f32,
}

impl Material for MaterialProgressIndicator {
    fn fragment_shader() -> ShaderRef {
        "shader/shader_progress_indicator.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
