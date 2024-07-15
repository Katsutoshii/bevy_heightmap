use std::f32::consts::PI;

use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use image::{Rgb, RgbImage};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const STRIDE: f32 = WIDTH as f32 / 2.;
const MIN_HEIGHT: f32 = 1. / 50.;

fn mesh_to_world(mesh_position: Vec3) -> Vec3 {
    Vec3::new(mesh_position.x, -mesh_position.z, mesh_position.y)
}
fn world2d_to_index(world_position: Vec2) -> UVec2 {
    (world_position * STRIDE + STRIDE).as_uvec2()
}
fn height_to_rgb(height: f32) -> Rgb<u8> {
    let z = ((height + MIN_HEIGHT) / MIN_HEIGHT * 255.) as u8;
    Rgb([z, z, z])
}
fn mesh_to_image(mesh: &Mesh) -> RgbImage {
    info!("Vertices: {}", mesh.count_vertices());

    let mut image = RgbImage::new(WIDTH + 1, HEIGHT + 1);
    let vertex_positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    for position in vertex_positions.iter() {
        let world_position = mesh_to_world(Vec3::from(*position));
        let index = world2d_to_index(world_position.xy());
        *image.get_pixel_mut(index.x, index.y) = height_to_rgb(world_position.z);
    }
    image
}

#[derive(Component, Default)]
struct Terrain {
    loaded: bool,
}
impl Terrain {
    fn update(mut mesh_handles: Query<(&Handle<Mesh>, &mut Terrain)>, meshes: Res<Assets<Mesh>>) {
        for (handle, mut terrain) in mesh_handles.iter_mut() {
            if terrain.loaded {
                continue;
            }
            if let Some(mesh) = meshes.get(handle) {
                terrain.loaded = true;
                let image = mesh_to_image(mesh);
                // This can't work in WASM as there is no filesystem access
                #[cfg(not(target_arch = "wasm32"))]
                IoTaskPool::get()
                    .spawn(async move { image.save("assets/textures/output_heightmap.png") })
                    .detach();
            }
        }
    }
}

pub const SCALE: f32 = 512.;
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
    let mesh: Handle<Mesh> = asset_server.load("models/terrain.glb#Mesh0/Primitive0");
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
                rotation: Quat::from_axis_angle(Vec3::X, PI / 2.),
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
