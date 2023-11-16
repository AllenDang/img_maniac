use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "11CE6B44-B18F-4149-A2E4-3E1D8E602664"]
pub struct MaterialSeparateChannel {
    #[uniform(0)]
    pub channel: u32,
    #[uniform(0)]
    pub show_outline: u32,
    #[uniform(0)]
    pub outline_color: Color,
    #[uniform(0)]
    pub outline_width: f32,
    #[uniform(0)]
    pub quad_ratio: f32,

    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,
}

impl Material for MaterialSeparateChannel {
    fn fragment_shader() -> ShaderRef {
        "shader/shader_separate_channel.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
