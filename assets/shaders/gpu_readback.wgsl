// This shader is used for the gpu_readback example
// The actual work it does is not important for the example

// This is the data that lives in the gpu only buffer
@group(0) @binding(0) var texture: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_xy = vec2<i32>(i32(global_id.x), i32(global_id.y));
    // let color = vec4<f32>(global_id.x, global_id.y, global_id.z, 1);
    let color = vec4<u32>(1, 0, 0, 0);
    textureStore(texture, texture_xy, color);
}
