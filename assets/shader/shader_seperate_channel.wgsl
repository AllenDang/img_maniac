struct MaterialSeperateChannel {
    channel: u32,
    show_outline: u32,
    outline_color: vec4<f32>,
    outline_width: f32,
};

@group(1) @binding(0)
var<uniform> material: MaterialSeperateChannel;

@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var final_color = textureSample(base_color_texture, base_color_sampler, uv);

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

    let outline_width = material.outline_width / 400.0;

    if material.show_outline > 0u {
        if uv.y < outline_width {
            final_color = material.outline_color;
        }

        if uv.x < outline_width {
            final_color = material.outline_color;
        }

        if uv.x > 1.0 - outline_width {
            final_color = material.outline_color;
        }

        if uv.y > 1.0 - outline_width {
            final_color = material.outline_color;
        }
    }


    return final_color;
}
