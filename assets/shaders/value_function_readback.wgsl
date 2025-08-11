#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};
#import "shaders/bezier_curves_1d.wgsl"::{bezier_curve3};

// Compute shader that populates a texture.
@group(0) @binding(0) var texture: texture_storage_2d<rgba32float, write>;

fn smooth_step(t: f32) -> f32 {
  return (
    t * t * (3 - 2 * t)
  );
}

fn smooth_step3(t: f32) -> f32 {
  var t1 = t;
  t1 = smooth_step(t1);
  t1 = smooth_step(t1);
  t1 = smooth_step(t1);
  return t1;
}

// Fractral brownian motion.
fn fbm(xy: vec2<f32>) -> f32 {
    var m2: mat2x2<f32> = mat2x2<f32>(vec2<f32>(0.8, 0.6), vec2<f32>(-0.6, 0.8));
    var p = xy;
    var f: f32 = 0.;
    f = f + 0.5000 * perlin_noise_2d(p); p = m2 * p * 2.02;
    f = f + 0.2500 * perlin_noise_2d(p); p = m2 * p * 2.03;
    f = f + 0.1250 * perlin_noise_2d(p); p = m2 * p * 2.01;
    f = f + 0.0625 * perlin_noise_2d(p);
    return f / 0.9375;
}

// Function to return a color for a given (x, y) point.
fn f(xy: vec2<f32>) -> vec4<f32> {
    let offset_xy = vec2<f32>(1000.0, 500.0);
    let scale = 128.0;
    let level = 0.4;
    let amplitude = 1.0;

    var output_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    let noise1_weight = 1.0;
    let noise1 = fbm((xy + offset_xy) / scale) * noise1_weight;

    let noise2_weight = 0.1;
    let noise2 = perlin_noise_2d(xy / (scale / 2.0)) * noise2_weight;
    
    let noise3_weight = 0.1;
    let noise3 = perlin_noise_2d(xy / (scale / 4.0)) * noise3_weight;

    let noise4_weight = 0.03;
    let noise4 = perlin_noise_2d(xy / (scale / 16.0)) * noise4_weight;

    let noise_sum = noise1 + noise2 + noise3 + noise4;
    let noise_weight = noise1_weight + noise2_weight + noise3_weight + noise4_weight;

    let noise = clamp(amplitude * noise_sum / noise_weight + level, 0.0, 1.0);
    let scaled_noise = smooth_step3(noise);

    let max_height = 2.0;
    output_color.r *= clamp(scaled_noise, 0.5, 1.0) * max_height;

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
