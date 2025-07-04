use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{GRAY, WHITE},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_egui::EguiPlugin;
use bevy_heightmap::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Component, Default)]
struct Terrain {}
impl Terrain {
    fn update() {}
}

pub const SCALE: f32 = 1024.;
pub const HEIGHT: f32 = 32.;
pub const THETA: f32 = PI / 8.;
pub const FOV: f32 = PI / 4.;
pub fn y_offset(z: f32) -> f32 {
    THETA.tan() * z
}

#[derive(Resource)]
pub struct ShaderMaterialHandle(pub Handle<ShaderMaterial>);
impl FromWorld for ShaderMaterialHandle {
    fn from_world(world: &mut World) -> Self {
        Self(world.add_asset(ShaderMaterial::default()))
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ShaderMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}
impl Default for ShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE.into(),
        }
    }
}
impl Material for ShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/value_function_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

fn setup(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    shader_material: Res<ShaderMaterialHandle>,
) {
    let h = |_p: Vec2| 0.0;
    let mesh: Handle<Mesh> = meshes.add(HeightMap {
        size: UVec2::new(128, 128),
        h,
    });
    commands.spawn((
        Name::new("Terrain"),
        Terrain::default(),
        Mesh3d(mesh),
        MeshMaterial3d(shader_material.0.clone()),
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
        Transform::from_xyz(0.0, 0.0, default_height)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
    ));
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        HeightMapPlugin,
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::default(),
        MaterialPlugin::<ShaderMaterial>::default(),
    ))
    .init_resource::<ShaderMaterialHandle>()
    .add_systems(Startup, setup)
    .add_systems(Update, Terrain::update)
    .run();
}
