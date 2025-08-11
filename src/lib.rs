/// Simple crate for loading a heightmap .png image as a mesh.
use asset_loader::HeightMapLoader;

pub mod asset_loader;
pub mod image;
pub mod mesh_builder;

use bevy_app::{App, Plugin};
use bevy_asset::AssetApp;
use bevy_math::{UVec2, Vec2};
use bevy_render::mesh::Mesh;
use mesh_builder::MeshBuilder;

pub use image::ImageBufferHeightMap;

/// A Heightmap is anything that provides a 2d value function `h()`.
pub trait HeightMap: Sized {
    /// Compute the height value at a given point `p``.
    fn h(&self, p: Vec2) -> f32;

    /// Builds a mesh from the heightmap.
    fn build_mesh(&self, size: UVec2) -> Mesh {
        let mut builder = MeshBuilder::grid(size);
        builder.update_z_positions(self);
        builder.build()
    }
}

/// Height map from value function;
/// ```
/// use bevy::prelude::*;
/// use bevy_heightmap::*;
/// let heightmap = HeightMap {
///   size: UVec2::new(10, 10),
///   h: |p: Vec2| ((20. * p.x).sin() + (20. * p.y).sin()) / 2.
/// };
/// let mesh: Mesh = heightmap.into();
/// assert_eq!(mesh.count_vertices(), 10 * 10);
/// ```
pub struct ValueFunctionHeightMap<H: Fn(Vec2) -> f32>(pub H);
impl<H: Fn(Vec2) -> f32> HeightMap for ValueFunctionHeightMap<H> {
    fn h(&self, p: Vec2) -> f32 {
        self.0(p)
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
