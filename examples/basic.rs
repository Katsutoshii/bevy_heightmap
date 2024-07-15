use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_heightmap::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Component, Default)]
struct Terrain {}
impl Terrain {
    fn update() {}
}

pub const SCALE: f32 = 1024.;
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
    let h = |p: Vec2| ((p.x * 10.).sin() + (p.y * 10.).sin()) / 50.;
    let mesh = meshes.add(MeshBuilder::grid_mesh(UVec2 { x: 32, y: 32 }, h).build());
    commands.spawn((
        Name::new("Terrain"),
        Terrain::default(),
        PbrBundle {
            mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture),
                ..default()
            }),
            transform: Transform {
                scale: SCALE * Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));
    commands.spawn((
        Name::new("Origin"),
        PbrBundle {
            mesh: meshes.add(Cuboid {
                half_size: Vec3::ONE * 0.5,
            }),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform {
                scale: 10. * Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));
    commands.spawn((
        Name::new("One"),
        PbrBundle {
            mesh: meshes.add(Cuboid {
                half_size: Vec3::ONE * 0.5,
            }),
            material: materials.add(StandardMaterial {
                base_color: Color::DARK_GRAY,
                ..default()
            }),
            transform: Transform {
                scale: 10. * Vec3::ONE,
                translation: (Vec2::ONE * SCALE / 2.).extend(0.),
                ..default()
            },
            ..default()
        },
    ));
    let default_height = 1500.;
    commands.spawn(Camera3dBundle {
        projection: PerspectiveProjection {
            fov: FOV,
            near: 0.1,
            far: 2000.,
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(0.0, -y_offset(default_height), default_height)
            .with_rotation(Quat::from_axis_angle(Vec3::X, THETA)),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, default_height)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        directional_light: DirectionalLight {
            color: Color::ANTIQUE_WHITE,
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, WorldInspectorPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, Terrain::update)
        .run();
}
