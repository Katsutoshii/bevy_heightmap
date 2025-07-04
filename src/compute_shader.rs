use bevy_app::{App, Plugin};
use bevy_asset::{DirectAssetAccessExt, Handle, RenderAssetUsages};
use bevy_ecs::{
    change_detection::DetectChanges,
    resource::Resource,
    system::{Commands, Res, ResMut},
    world::{FromWorld, World},
};
use bevy_image::Image;
use bevy_log::info;
use bevy_math::UVec3;
use bevy_render::{
    ExtractSchedule, MainWorld, RenderApp,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{
        AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, CachedPipelineState,
        ComputePassDescriptor, ComputePipelineDescriptor, Extent3d, PipelineCache,
        TextureDimension, TextureFormat, TextureUsages,
    },
    renderer::{RenderContext, RenderDevice},
};
use bevy_state::{
    app::AppExtStates,
    state::{NextState, States},
};
use std::{fmt::Debug, marker::PhantomData};

pub trait ComputeShader: AsBindGroup + Resource + Clone + Debug {
    fn shader_path() -> &'static str;
    fn workgroup_size() -> UVec3;
    fn prepare(commands: Commands, image: Res<ReadbackImage<Self>>);
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
}

// We need a plugin to organize all the systems and render node required for this example
#[derive(Default)]
pub struct ComputeShaderPlugin<S: ComputeShader> {
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Plugin for ComputeShaderPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReadbackImage<S>>()
            .add_plugins(ExtractResourcePlugin::<ReadbackImage<S>>::default())
            .init_state::<ComputeNodeState>();
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputePipeline<S>>()
            .init_resource::<ComputeNodeState>()
            .add_systems(ExtractSchedule, ComputeNodeState::extract_to_main);

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::<S>::default());
    }
}

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackImage<S: ComputeShader> {
    pub handle: Handle<Image>,
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> FromWorld for ReadbackImage<S> {
    fn from_world(world: &mut World) -> Self {
        let workgroup_size = S::workgroup_size();
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
            handle: world.add_asset(image),
            _marker: PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct ComptueShaderBindGroup<S: ComputeShader> {
    pub bind_group: BindGroup,
    pub _marker: PhantomData<S>,
}

#[derive(Resource)]
pub struct ComputePipeline<S: ComputeShader> {
    pub layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> FromWorld for ComputePipeline<S> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = S::bind_group_layout(render_device);
        let shader = world.load_asset(S::shader_path());
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
        Self {
            layout,
            pipeline,
            _marker: PhantomData,
        }
    }
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

/// The node that will execute the compute shader
struct ComputeNode<S: ComputeShader> {
    state: ComputeNodeState,
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeNode<S> {
    fn default() -> Self {
        Self {
            state: ComputeNodeState::default(),
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> render_graph::Node for ComputeNode<S> {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<ComputePipeline<S>>();
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
                        panic!("Initializing assets/{}:\n{}", S::shader_path(), err)
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
        let pipeline = world.resource::<ComputePipeline<S>>();
        let bind_group = &world.resource::<ComptueShaderBindGroup<S>>().bind_group;

        if self.state == ComputeNodeState::Ready {
            if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
                let workgroup_size = S::workgroup_size();
                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("GPU readback compute pass"),
                            ..Default::default()
                        });
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(workgroup_size.x, workgroup_size.y, workgroup_size.z);
            }
        }
        Ok(())
    }
}
