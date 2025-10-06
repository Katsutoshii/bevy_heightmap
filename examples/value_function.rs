use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{GRAY, WHITE},
    prelude::*,
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

fn setup(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let texture: Handle<Image> = asset_server.load("textures/uv.png");
    let mesh: Handle<Mesh> = meshes.add(
        ValueFunctionHeightMap(|p: Vec2| ((20. * p.x).sin() + (20. * p.y).sin()) / 2.)
            .build_mesh(UVec2::new(128, 128)),
    );
    commands.spawn((
        Name::new("Terrain"),
        Terrain::default(),
        Mesh3d(mesh),
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
        EguiPlugin::default(),
        WorldInspectorPlugin::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, Terrain::update)
    .run();
}
