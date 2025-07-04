//! Simple example demonstrating the use of the [`Readback`] component to read back data from the GPU
//! using both a storage buffer and texture.

use bevy_app::{App, Plugin};
use bevy_asset::{DirectAssetAccessExt, Handle};
use bevy_ecs::{
    resource::Resource,
    schedule::{
        IntoScheduleConfigs,
        common_conditions::{not, resource_exists},
    },
    system::{Commands, Res},
    world::{FromWorld, World},
};
use bevy_image::Image;
use bevy_math::UVec3;
use bevy_render::{
    Render, RenderApp, RenderSet,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::RenderAssets,
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{
        BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries,
        CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, IntoBinding,
        PipelineCache, ShaderStages, StorageTextureAccess, TextureFormat,
        binding_types::texture_storage_2d,
    },
    renderer::{RenderContext, RenderDevice},
    texture::GpuImage,
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

// We need a plugin to organize all the systems and render node required for this example
pub struct GpuReadbackPlugin {
    pub workgroup_size: UVec3,
}

impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<ReadbackImage>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputePipeline>().add_systems(
            Render,
            prepare_bind_group
                .in_set(RenderSet::PrepareBindGroups)
                // We don't need to recreate the bind group every frame
                .run_if(not(resource_exists::<GpuBufferBindGroup>)),
        );

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(
                ComputeNodeLabel,
                ComputeNode {
                    workgroup_size: self.workgroup_size,
                },
            );
    }
}

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackImage(pub Handle<Image>);

#[derive(Resource)]
pub struct GpuBufferBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    image: Res<ReadbackImage>,
    images: Res<RenderAssets<GpuImage>>,
) {
    let image = images.get(&image.0).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((image.texture_view.into_binding(),)),
    );
    commands.insert_resource(GpuBufferBindGroup(bind_group));
}

#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (texture_storage_2d(
                    TextureFormat::R32Uint,
                    StorageTextureAccess::WriteOnly,
                ),),
            ),
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("GPU readback compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
        ComputePipeline { layout, pipeline }
    }
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

/// The node that will execute the compute shader
#[derive(Default)]
pub struct ComputeNode {
    pub workgroup_size: UVec3,
}

impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..Default::default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(
                self.workgroup_size.x,
                self.workgroup_size.y,
                self.workgroup_size.z,
            );
        }
        Ok(())
    }
}
