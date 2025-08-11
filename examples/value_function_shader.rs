use std::f32::consts::PI;

use bevy_compute_readback::{ComputeShader, ComputeShaderPlugin, ReadbackLimit};
use bevy_ecs::world::DeferredWorld;

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::GRAY,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bevy_heightmap::{HeightMap, HeightMapPlugin, image::ImageBufferHeightMap};
use bevy_image::TextureFormatPixelInfo;
use image::Rgba;

pub const SCALE: f32 = 1024.;
pub const HEIGHT: f32 = 32.;
pub const THETA: f32 = PI / 8.;
pub const FOV: f32 = PI / 4.;
pub fn y_offset(z: f32) -> f32 {
    THETA.tan() * z
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        HeightMapPlugin,
        ComputeShaderPlugin::<NoiseComputeShader> {
            limit: ReadbackLimit::Finite(1),
            remove_on_complete: false,
            ..default()
        },
    ))
    .insert_resource(ClearColor(Color::BLACK))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    noise_shader: ResMut<NoiseComputeShader>,
) {
    let texture: Handle<Image> = asset_server.load("textures/uv.png");
    commands.spawn((
        Name::new("Terrain"),
        Mesh3d(noise_shader.generated_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(texture),
            ..default()
        })),
        Transform {
            scale: Vec2::splat(SCALE).extend(HEIGHT),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("Origin"),
        Mesh3d(meshes.add(Cuboid {
            half_size: Vec3::ONE * 0.5,
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        })),
        Transform {
            scale: 10. * Vec3::ONE,
            ..default()
        },
    ));
    commands.spawn((
        Name::new("One"),
        Mesh3d(meshes.add(Cuboid {
            half_size: Vec3::ONE * 0.5,
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY.into(),
            ..default()
        })),
        Transform {
            scale: 10. * Vec3::ONE,
            translation: (Vec2::ONE * SCALE / 2.).extend(0.),
            ..default()
        },
    ));
    let default_height = 1500.;
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV,
            near: 0.1,
            far: 2000.,
            ..default()
        }),
        Transform::from_xyz(0.0, -y_offset(default_height), default_height)
            .with_rotation(Quat::from_axis_angle(Vec3::X, THETA)),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, default_height)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
    ));
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Resource, Clone, Debug, ExtractResource)]
pub struct NoiseComputeShader {
    #[storage_texture(0, image_format=Rgba32Float, access=WriteOnly)]
    texture: Handle<Image>,

    // Mesh computed from the readback texture.
    generated_mesh: Handle<Mesh>,
}
impl ComputeShader for NoiseComputeShader {
    fn compute_shader() -> ShaderRef {
        "shaders/value_function_readback.wgsl".into()
    }
    fn workgroup_size() -> UVec3 {
        UVec3::new(1024, 1024, 1)
    }
    fn readback(&self) -> Option<Readback> {
        Some(Readback::texture(self.texture.clone()))
    }
    fn on_readback(trigger: Trigger<ReadbackComplete>, mut world: DeferredWorld) {
        let size = Self::workgroup_size().xy();
        let heightmap =
            ImageBufferHeightMap::<Rgba<f32>, Vec<f32>>::from_bytes(size, &trigger.event().0);
        let mesh_handle = world.resource::<Self>().generated_mesh.clone();
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let Some(mesh) = meshes.get_mut(&mesh_handle) else {
            return;
        };
        *mesh = heightmap.build_mesh(size);
    }
}
impl FromWorld for NoiseComputeShader {
    fn from_world(world: &mut World) -> Self {
        let size = Self::workgroup_size();
        let mut image = Image::new(
            Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: size.z,
            },
            TextureDimension::D2,
            vec![0; TextureFormat::Rgba32Float.pixel_size() * size.x as usize * size.y as usize],
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        Self {
            texture: world.add_asset(image),
            generated_mesh: world.add_asset(Cuboid::default()),
        }
    }
}
