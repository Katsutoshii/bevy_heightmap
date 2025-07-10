use bevy_app::{App, Plugin, Update};
use bevy_asset::DirectAssetAccessExt;
use bevy_ecs::{
    bundle::Bundle,
    change_detection::DetectChanges,
    component::{Component, HookContext},
    entity::Entity,
    observer::Trigger,
    query::With,
    resource::Resource,
    schedule::{
        IntoScheduleConfigs,
        common_conditions::{not, resource_exists},
    },
    system::{Commands, Query, Res, ResMut, StaticSystemParam},
    world::{DeferredWorld, FromWorld, World},
};
use bevy_math::UVec3;
use bevy_render::{
    ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    gpu_readback::{Readback, ReadbackComplete},
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{
        AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, CachedPipelineState,
        ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderRef,
    },
    renderer::{RenderContext, RenderDevice},
};
use bevy_state::{
    app::AppExtStates,
    state::{NextState, OnEnter, States},
};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// Plugin to create all the required systems for using a custom compute shader.
pub struct ComputeShaderPlugin<S: ComputeShader> {
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeShaderPlugin<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> Plugin for ComputeShaderPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<S>()
            .add_plugins(ExtractResourcePlugin::<S>::default())
            .init_state::<ComputeNodeState>()
            .add_systems(Update, ComputeShaderReadback::<S>::update)
            .add_systems(
                OnEnter(ComputeNodeState::Ready),
                ComputeShaderReadback::<S>::on_shader_ready,
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputePipeline<S>>()
            .init_resource::<ComputeNodeState>()
            .add_systems(ExtractSchedule, ComputeNodeState::extract_to_main)
            .add_systems(
                Render,
                (S::prepare_bind_group)
                    .chain()
                    .in_set(RenderSet::PrepareBindGroups)
                    .run_if(not(resource_exists::<ComputeShaderBindGroup<S>>)),
            );

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(
                ComputeNodeLabel::<S>::default(),
                ComputeNode::<S>::default(),
            );
    }
}

#[derive(Default)]
pub enum ReadbackLimit {
    #[default]
    Infinite,
    Finite(usize),
}
#[derive(Component)]
#[component(on_add = ComputeShaderReadback::<S>::on_add )]
pub struct ComputeShaderReadback<S: ComputeShader> {
    pub limit: ReadbackLimit,
    pub count: usize,
    pub marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeShaderReadback<S> {
    fn default() -> Self {
        Self {
            limit: ReadbackLimit::default(),
            count: 0,
            marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> ComputeShaderReadback<S> {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(S::on_readback);
    }
    fn update(
        mut commands: Commands,
        mut compute_shader_readbacks: Query<(Entity, &mut Self), With<Readback>>,
    ) {
        for (entity, mut compute_shader_readback) in compute_shader_readbacks.iter_mut() {
            match compute_shader_readback.limit {
                ReadbackLimit::Finite(limit) => {
                    compute_shader_readback.count += 1;
                    if compute_shader_readback.count > limit {
                        commands.entity(entity).remove::<Readback>();
                    }
                }
                ReadbackLimit::Infinite => {}
            }
        }
    }
    fn on_shader_ready(
        mut commands: Commands,
        compute_shader: Res<S>,
        mut compute_shader_readbacks: Query<(Entity, &mut Self)>,
    ) {
        for (entity, mut compute_shader_readback) in compute_shader_readbacks.iter_mut() {
            compute_shader_readback.count = 0;
            commands.entity(entity).insert(compute_shader.readbacks());
        }
    }
}

/// Trait to implement for your custom shader.
pub trait ComputeShader: AsBindGroup + Clone + Debug + FromWorld + ExtractResource {
    /// Asset path or handle to the shader.
    fn compute_shader() -> ShaderRef;
    /// Workgroup size.
    fn workgroup_size() -> UVec3;
    /// Optional bind group preparation.
    fn prepare_bind_group(
        mut commands: Commands,
        pipeline: Res<ComputePipeline<Self>>,
        render_device: Res<RenderDevice>,
        input: Res<Self>,
        param: StaticSystemParam<<Self as AsBindGroup>::Param>,
    ) {
        let bind_group = input
            .as_bind_group(&pipeline.layout, &render_device, &mut param.into_inner())
            .unwrap();
        commands.insert_resource(ComputeShaderBindGroup::<Self> {
            bind_group: bind_group.bind_group,
            _marker: PhantomData,
        });
    }
    /// Optional readbacks.
    fn readbacks(&self) -> impl Bundle {}
    /// Optional processing on readback. Could write back to the CPU buffer, etc.
    fn on_readback(_trigger: Trigger<ReadbackComplete>, mut _world: DeferredWorld) {}
}

/// Stores prepared bind group data for the compute shader.
#[derive(Resource)]
pub struct ComputeShaderBindGroup<S: ComputeShader> {
    pub bind_group: BindGroup,
    pub _marker: PhantomData<S>,
}

/// Track compute node state.
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

/// Defines the pipeline for the compute shader.
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
        let shader = match S::compute_shader() {
            ShaderRef::Default => panic!("Must define compute_shader."),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => world.load_asset(path),
        };
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
#[derive(Debug, Clone, RenderLabel)]
struct ComputeNodeLabel<S: ComputeShader> {
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeNodeLabel<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> PartialEq for ComputeNodeLabel<S> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<S: ComputeShader> Eq for ComputeNodeLabel<S> {}
impl<S: ComputeShader> Hash for ComputeNodeLabel<S> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

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

        match self.state {
            ComputeNodeState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = ComputeNodeState::Init;
                        *world.resource_mut::<ComputeNodeState>() = self.state;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Error loading compute shader: \n{err}")
                    }
                    _ => {}
                }
            }
            ComputeNodeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.pipeline)
                {
                    self.state = ComputeNodeState::Ready;
                    *world.resource_mut::<ComputeNodeState>() = self.state;
                }
            }
            ComputeNodeState::Ready => {
                if let CachedPipelineState::Creating(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.pipeline)
                {
                    self.state = ComputeNodeState::Loading;
                    *world.resource_mut::<ComputeNodeState>() = self.state;
                }
            }
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
        let bind_group = &world.resource::<ComputeShaderBindGroup<S>>().bind_group;

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
