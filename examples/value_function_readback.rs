// Uses GPU readback to get the results from a compute shader.
use bevy_heightmap::compute_shader::{
    ComputeShader, ComputeShaderPlugin, ComputeShaderReadback, ReadbackLimit,
};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        gpu_readback::Readback,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bevy_render::gpu_readback::ReadbackComplete;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ComputeShaderPlugin::<CustomComputeShader>::default(),
    ))
    .insert_resource(ClearColor(Color::BLACK))
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(ComputeShaderReadback::<CustomComputeShader> {
        limit: ReadbackLimit::Finite(1),
        ..default()
    });
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Resource, Clone, Debug, ExtractResource)]
pub struct CustomComputeShader {
    #[storage_texture(0, image_format=Rgba32Uint, access=WriteOnly)]
    texture: Handle<Image>,
}
impl ComputeShader for CustomComputeShader {
    fn compute_shader() -> ShaderRef {
        "shaders/gpu_readback.wgsl".into()
    }
    fn workgroup_size() -> UVec3 {
        UVec3::new(16, 16, 1)
    }
    fn readbacks(&self) -> impl Bundle {
        Readback::texture(self.texture.clone())
    }
    fn on_readback(trigger: Trigger<ReadbackComplete>, mut _commands: Commands) {
        let data: Vec<u32> = trigger.event().to_shader_type();
        info!("Data len: {}", data.len());
        info!("data[0..128] {:?}", &data[0..128]);
    }
}
impl FromWorld for CustomComputeShader {
    fn from_world(world: &mut World) -> Self {
        let workgroup_size = Self::workgroup_size();
        let size = Extent3d {
            width: workgroup_size.x,
            height: workgroup_size.y,
            depth_or_array_layers: workgroup_size.z,
        };
        let mut image = Image::new_uninit(
            size,
            TextureDimension::D2,
            TextureFormat::Rgba32Uint,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        Self {
            texture: world.add_asset(image),
        }
    }
}
