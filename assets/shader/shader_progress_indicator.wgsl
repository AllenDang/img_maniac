#import bevy_pbr::mesh_view_bindings

struct ProgressIndicatorParam {
  background_color: vec4<f32>,
  
  aspect_ratio: f32,

  ring_inner_radius: f32,
  ring_outer_radius: f32,
  ring_color: vec4<f32>,

  nob_radius: f32,
  nob_color: vec4<f32>,
  nob_rotation_speed: f32,

  dot_radius: f32,
  dot_color: vec4<f32>,
  dot_distance: f32,
  dot_margin_top: f32,
  dot_fade_speed: f32,

  anti_alias_factor: f32,
}

@group(1) @binding(0)
var<uniform> params: ProgressIndicatorParam;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var final_color = params.background_color;

    var uv = uv * 2.0 - vec2<f32>(1.0, 1.0);

    uv.x *= params.aspect_ratio;

    let dist_from_center = length(uv);

    let nob_pos = vec2<f32>(cos(globals.time * params.nob_rotation_speed), sin(globals.time * params.nob_rotation_speed)) * 0.425;

    // Check if the pixel is within the ring
    if dist_from_center > params.ring_inner_radius && dist_from_center < params.ring_outer_radius {
        let ring_edge_dist = min(dist_from_center - params.ring_inner_radius, params.ring_outer_radius - dist_from_center);
        let ring_alpha = smoothstep(0.0, params.anti_alias_factor, ring_edge_dist);
        final_color = mix(final_color, params.ring_color, ring_alpha);
    }

    // Check if the pixel is within the nob
    let nob_dist = length(uv - nob_pos);
    if nob_dist < params.nob_radius {
        let nob_edge_dist = params.nob_radius - nob_dist;
        let nob_alpha = smoothstep(0.0, params.anti_alias_factor, nob_edge_dist);
        final_color = mix(final_color, params.nob_color, nob_alpha);
    }

    // Calculate the positions of the loading dots
    let total_width = 3.0 * params.dot_distance + 4.0 * params.dot_radius;
    let initial_dot_pos = -0.5 * total_width;

    for (var i = 0u; i < 4u; i = i + 1u) {
        let dot_position = vec2<f32>(initial_dot_pos + f32(i) * (params.dot_radius + params.dot_distance), 0.5 - params.dot_margin_top);

        // Check if the pixel is within one of the loading dots
        let dot_dist = length(uv - dot_position);
        if dot_dist < params.dot_radius {
            let dot_edge_dist = params.dot_radius - dot_dist;
            let opacity = sin(globals.time * params.dot_fade_speed - f32(i) * 0.5) * 0.5 + 0.5;
            let dot_color_with_fade = vec4<f32>(params.dot_color.rgb, params.dot_color.a * opacity);
            let dot_alpha = smoothstep(0.0, params.anti_alias_factor, dot_edge_dist);
            final_color = mix(final_color, dot_color_with_fade, dot_alpha);
        }
    }

    return final_color;
}
