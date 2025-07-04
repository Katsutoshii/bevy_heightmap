//! Simple example demonstrating the use of the [`Readback`] component to read back data from the GPU
//! using both a storage buffer and texture.

use std::marker::PhantomData;

use bevy_heightmap::compute_shader::{
    ComptueShaderBindGroup, ComputeNodeState, ComputePipeline, ComputeShader, ComputeShaderPlugin,
    ReadbackImage,
};

use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        gpu_readback::{Readback, ReadbackComplete},
        render_asset::RenderAssets,
        render_resource::*,
        renderer::RenderDevice,
        texture::GpuImage,
    },
};
use bevy_render::{storage::GpuShaderStorageBuffer, texture::FallbackImage};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ComputeShaderPlugin::<CustomComputeShader>::default(),
    ))
    .register_type::<ReadbackOnce>()
    .add_systems(Update, ReadbackOnce::update);
    app.sub_app_mut(RenderApp).add_systems(
        Render,
        (CustomComputeShader::prepare, prepare_buffer)
            .chain()
            .in_set(RenderSet::PrepareBindGroups)
            .run_if(not(resource_exists::<
                ComptueShaderBindGroup<CustomComputeShader>,
            >)),
    );
    app.insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(ComputeNodeState::Ready), on_ready)
        .run();
}

fn prepare_buffer(
    mut commands: Commands,
    pipeline: Res<ComputePipeline<CustomComputeShader>>,
    render_device: Res<RenderDevice>,
    input: Res<CustomComputeShader>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_images: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let bind_group = input
        .as_bind_group(
            &pipeline.layout,
            &render_device,
            &mut (images, fallback_images, buffers),
        )
        .unwrap();
    commands.insert_resource(ComptueShaderBindGroup::<CustomComputeShader> {
        bind_group: bind_group.bind_group,
        _marker: PhantomData,
    });
}

fn on_ready(mut commands: Commands, image: Res<ReadbackImage<CustomComputeShader>>) {
    // Textures can also be read back from the GPU. Pay careful attention to the format of the
    // texture, as it will affect how the data is interpreted.
    commands
        .spawn((
            Readback::texture(image.handle.clone()),
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
#[derive(AsBindGroup, Resource, Clone, Debug, Default)]
pub struct CustomComputeShader {
    #[storage_texture(0, image_format=Rgba32Uint, access=WriteOnly)]
    texture: Handle<Image>,
}
impl ComputeShader for CustomComputeShader {
    fn shader_path() -> &'static str {
        "shaders/gpu_readback.wgsl"
    }
    fn workgroup_size() -> UVec3 {
        UVec3::new(16, 16, 1)
    }
    fn prepare(mut commands: Commands, image: Res<ReadbackImage<Self>>) {
        commands.insert_resource(Self {
            texture: image.handle.clone(),
        });
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
