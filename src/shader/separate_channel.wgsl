#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct MaterialSeparateChannel {
    channel: u32,
    show_outline: u32,
    outline_color: vec4<f32>,
    outline_width: f32,
    quad_ratio: f32,
};

@group(2) @binding(0)
var<uniform> material: MaterialSeparateChannel;

@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    var final_color = textureSample(base_color_texture, base_color_sampler, in.uv);

    var blend = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    switch material.channel {
        case 1u: {
            blend.y = 0.0;
            blend.z = 0.0;
        }
        case 2u: {
            blend.x = 0.0;
            blend.z = 0.0;
        }
        case 3u: {
            blend.x = 0.0;
            blend.y = 0.0;
        }
        case 4u: {
            blend.x = 0.0;
            blend.y = 0.0;
            blend.z = 0.0;
        }
        default : {}
    }

    final_color = final_color * blend;

    if material.show_outline > 0u {
        let outline_width = material.outline_width / 400.0;

        var outline_width_top_buttom = outline_width;
        var outline_width_left_right = outline_width;

        if material.quad_ratio > 1.0 {
            outline_width_left_right /= material.quad_ratio;
        }

        if material.quad_ratio < 1.0 {
            outline_width_top_buttom /= material.quad_ratio;
        }

        if in.uv.y < outline_width_top_buttom || in.uv.y > 1.0 - outline_width_top_buttom {
            final_color = material.outline_color;
        }

        if in.uv.x < outline_width_left_right || in.uv.x > 1.0 - outline_width_left_right {
            final_color = material.outline_color;
        }
    }


    return final_color;
}
