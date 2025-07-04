use bevy::prelude::*;
use bevy_asset::RenderAssetUsages;
use bevy_heightmap::{gpu_readback::ReadbackImage, *};

use bevy::render::{
    gpu_readback::{Readback, ReadbackComplete},
    render_resource::*,
};

const TEXTURE_SIZE: UVec3 = UVec3::new(4, 1, 1);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create a storage texture with some data
    let size = Extent3d {
        width: TEXTURE_SIZE.x,
        height: TEXTURE_SIZE.y,
        depth_or_array_layers: TEXTURE_SIZE.z,
    };
    // let mut image = Image::new_fill(
    //     size,
    //     TextureDimension::D2,
    //     &[0; 32],
    //     TextureFormat::Rgba32Float,
    //     RenderAssetUsages::RENDER_WORLD,
    // );
    let mut image = Image::new_uninit(
        size,
        TextureDimension::D2,
        TextureFormat::R32Uint,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC
        | TextureUsages::COPY_DST
        | TextureUsages::STORAGE_BINDING
        | TextureUsages::TEXTURE_BINDING;
    dbg!(size);
    if let Some(buffer) = &image.data {
        dbg!(buffer.len());
    }
    let image = images.add(image);

    // Textures can also be read back from the GPU. Pay careful attention to the format of the
    // texture, as it will affect how the data is interpreted.
    commands.spawn(Readback::texture(image.clone())).observe(
        |trigger: Trigger<ReadbackComplete>, mut commands: Commands| {
            dbg!(trigger.event().0.len());
            let data: Vec<u32> = trigger.event().to_shader_type();
            info!("Image {:?}", data);
            commands.entity(trigger.target()).remove::<Readback>();
        },
    );
    commands.insert_resource(ReadbackImage(image));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            gpu_readback::GpuReadbackPlugin {
                workgroup_size: TEXTURE_SIZE,
            },
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}
