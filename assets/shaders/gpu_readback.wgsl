#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};

// Compute shader that populates a texture.
@group(0) @binding(0) var texture: texture_storage_2d<rgba32float, write>;

// Function to return a color for a given (x, y) point.
fn f(xy: vec2<f32>) -> vec4<f32> {
    let offset_xy = vec2<f32>(200.0, 100.0);

    var output_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

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
    let noise_amount = 2.0;

    output_color.r *= noise * noise_amount;
    return output_color;
}

// Writes the function value to each pixel of the texture.
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_xy = vec2<i32>(global_id.xy);
    let xy = vec2<f32>(global_id.xy) * 1.0;
    let offset_xy = vec2<f32>(100.0, 100.0);
    textureStore(texture, texture_xy, f(xy + offset_xy));
}
