// Uses GPU readback to get the results from a compute shader.
use bevy_heightmap::compute_shader::{ComputeNodeState, ComputeShader, ComputeShaderPlugin};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ComputeShaderPlugin::<CustomComputeShader>::default(),
    ))
    .register_type::<ReadbackOnce>()
    .add_systems(Update, ReadbackOnce::update)
    .insert_resource(ClearColor(Color::BLACK))
    .add_systems(OnEnter(ComputeNodeState::Ready), on_ready)
    .run();
}

fn on_ready(mut commands: Commands, image: Res<CustomComputeShader>) {
    // Textures can also be read back from the GPU. Pay careful attention to the format of the
    // texture, as it will affect how the data is interpreted.
    commands
        .spawn((
            Readback::texture(image.texture.clone()),
            ReadbackOnce::default(),
        ))
        .observe(|trigger: Trigger<ReadbackComplete>| {
            // You probably want to interpret the data as a color rather than a ShaderType,
            // but in this case we know the data is a single channel storage texture, so we can
            // interpret it as a Vec<u32>
            let data: Vec<u32> = trigger.event().to_shader_type();
            info!("Image len: {}", data.len());
            info!("Image {:?}", &data[0..128]);
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

#[derive(Component, Reflect, Debug, Default, Copy, Clone)]
#[reflect(Component)]
pub struct ReadbackOnce(pub usize);
impl ReadbackOnce {
    fn update(mut commands: Commands, mut query: Query<(Entity, &mut ReadbackOnce)>) {
        for (entity, mut readback_count) in query.iter_mut() {
            readback_count.0 += 1;
            if readback_count.0 > 1 {
                commands.entity(entity).remove::<(ReadbackOnce, Readback)>();
            }
        }
    }
}
