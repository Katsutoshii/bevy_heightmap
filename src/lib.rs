/// Simple crate for loading a heightmap .png image as a mesh.
use asset_loader::HeightMapLoader;
use bevy::prelude::*;

pub mod asset_loader;
pub mod mesh_builder;

use mesh_builder::MeshBuilder;

/// Minimal representation of a rectangular height map.
/// ```
/// use bevy::prelude::*;
/// use bevy_heightmap::*;
/// let heightmap = HeightMap {
///   size: UVec2::new(10, 10),
///   h: |p: Vec2| ((20. * p.x).sin() + (20. * p.y).sin()) / 2.
/// };
/// let mesh: Mesh = heightmap.into();
/// assert_eq!(mesh.count_vertices(), 4 * 10 * 10);
/// ```
pub struct HeightMap<H: Fn(Vec2) -> f32> {
    pub size: UVec2,
    pub h: H,
}
impl<H: Fn(Vec2) -> f32> From<HeightMap<H>> for Mesh {
    fn from(HeightMap { size, h }: HeightMap<H>) -> Self {
        MeshBuilder::grid(size, &h).build()
    }
}

/// Enables loading Meshes from images with `.hmp.png` extension.
/// ```
/// use bevy::prelude::*;
/// use bevy_heightmap::*;
/// fn setup(asset_server: Res<AssetServer>) {
///     let mesh: Handle<Mesh> = asset_server.load("textures/terrain.hmp.png");
/// }
/// ```
pub struct HeightMapPlugin;
impl Plugin for HeightMapPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<HeightMapLoader>(HeightMapLoader::EXTENSIONS);
    }
    fn finish(&self, app: &mut App) {
        app.init_asset_loader::<asset_loader::HeightMapLoader>();
    }
}
