#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput};
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};

@group(2) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let offset_xy = vec2<f32>(200.0, 100.0);
    let xy = mesh.world_position.xy + offset_xy;

    var output_color = color;

    let noise1_res = 128.0;
    let noise1_weight = 3.0;
    let noise1 = perlin_noise_2d(xy / noise1_res) * noise1_weight;

    let noise2_res = 64.0;
    let noise2_weight = 2.0;
    let noise2 = perlin_noise_2d(xy / noise2_res) * noise2_weight;
    
    let noise3_res = 16.0;
    let noise3_weight = 1.0;
    let noise3 = perlin_noise_2d(xy / noise3_res) * noise3_weight;

    let noise4_res = 8.0;
    let noise4_weight = 1.0;
    let noise4 = perlin_noise_2d(xy / noise4_weight) * noise4_weight;

    let noise = (
      (noise1 + noise2 + noise3 + noise4) /
      (noise1_weight + noise2_weight + noise3_weight + noise4_weight)
    );
    let noise_amount = 1.0;

    // output_color.r *= mix(output_color.r, noise, noise_amount);
    output_color.r *= noise * noise_amount;
    output_color.g *= noise * noise_amount;
    output_color.b *= noise * noise_amount;
    return output_color;
}
