//! Simple example demonstrating the use of the [`Readback`] component to read back data from the GPU
//! using both a storage buffer and texture.

use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        gpu_readback::{Readback, ReadbackComplete},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
    },
};
use bevy_render::MainWorld;

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 16;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            GpuReadbackPlugin,
            ExtractResourcePlugin::<ReadbackImage>::default(),
        ))
        .init_resource::<ReadbackImage>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(ComputeNodeState::Ready), ComputeNodeState::on_ready)
        .run();
}

#[derive(Default, States, Resource, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum ComputeNodeState {
    #[default]
    Loading,
    Init,
    Ready,
}
impl ComputeNodeState {
    fn extract_to_main(render_state: Res<ComputeNodeState>, mut world: ResMut<MainWorld>) {
        if render_state.is_changed() {
            world
                .resource_mut::<NextState<ComputeNodeState>>()
                .set(*render_state);
        }
    }
    fn on_ready(mut commands: Commands, image: Res<ReadbackImage>) {
        // Textures can also be read back from the GPU. Pay careful attention to the format of the
        // texture, as it will affect how the data is interpreted.
        commands.spawn(Readback::texture(image.0.clone())).observe(
            |trigger: Trigger<ReadbackComplete>| {
                // You probably want to interpret the data as a color rather than a ShaderType,
                // but in this case we know the data is a single channel storage texture, so we can
                // interpret it as a Vec<u32>
                let data: Vec<u32> = trigger.event().to_shader_type();
                info!("Image len: {}", data.len());
                info!("Image {:?}", &data[0..128]);
            },
        );
    }
}

// We need a plugin to organize all the systems and render node required for this example
struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ComputeNodeState>()
            .init_resource::<ComputeShaderHandle>();
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputeShaderHandle>()
            .init_resource::<ComputePipeline>()
            .init_resource::<ComputeNodeState>()
            .add_systems(
                Render,
                prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups)
                    // We don't need to recreate the bind group every frame
                    .run_if(not(resource_exists::<GpuBufferBindGroup>)),
            )
            .add_systems(ExtractSchedule, ComputeNodeState::extract_to_main);

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

#[derive(Resource, ExtractResource, Clone)]
struct ReadbackImage(Handle<Image>);
impl FromWorld for ReadbackImage {
    fn from_world(world: &mut World) -> Self {
        // Create a storage texture with some data
        let size = Extent3d {
            width: BUFFER_LEN as u32,
            height: BUFFER_LEN as u32,
            ..default()
        };
        // We create an uninitialized image since this texture will only be used for getting data out
        // of the compute shader, not getting data in, so there's no reason for it to exist on the CPU
        let mut image = Image::new_uninit(
            size,
            TextureDimension::D2,
            TextureFormat::Rgba32Uint,
            RenderAssetUsages::RENDER_WORLD,
        );
        // We also need to enable the COPY_SRC, as well as STORAGE_BINDING so we can use it in the
        // compute shader
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        let image = world.add_asset(image);
        Self(image)
    }
}

#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

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
struct ComputeShaderHandle(pub Handle<Shader>);
impl FromWorld for ComputeShaderHandle {
    fn from_world(world: &mut World) -> Self {
        Self(world.load_asset(SHADER_ASSET_PATH))
    }
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
                    TextureFormat::Rgba32Uint,
                    StorageTextureAccess::WriteOnly,
                ),),
            ),
        );
        let shader = world.resource::<ComputeShaderHandle>().0.clone();
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
struct ComputeNode {
    state: ComputeNodeState,
}
impl render_graph::Node for ComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<ComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            ComputeNodeState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = ComputeNodeState::Init;
                        *world.resource_mut::<ComputeNodeState>() = self.state;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            ComputeNodeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.pipeline)
                {
                    info!("Pipeline ready");
                    self.state = ComputeNodeState::Ready;
                    *world.resource_mut::<ComputeNodeState>() = self.state;
                }
            }
            ComputeNodeState::Ready => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();

        if self.state == ComputeNodeState::Ready {
            if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("GPU readback compute pass"),
                            ..default()
                        });
                pass.set_bind_group(0, &bind_group.0, &[]);
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(BUFFER_LEN as u32, BUFFER_LEN as u32, 1);
            }
        }
        Ok(())
    }
}
