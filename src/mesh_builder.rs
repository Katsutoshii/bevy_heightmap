use bevy::{
    prelude::*,
    render::{
        mesh::Indices, render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology::TriangleList,
    },
};

/// Utility struct for building a mesh.
#[derive(Default)]
pub struct MeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

const FLIP_Y: Vec2 = Vec2 { x: 1., y: -1. };

impl MeshBuilder {
    /// Allocate memory for all attribute vectors.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            uvs: Vec::with_capacity(capacity),
            normals: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
        }
    }

    /// Compute a grid mesh of quads according to size.
    pub fn grid<H: Fn(Vec2) -> f32>(size: UVec2, h: &H) -> Self {
        let capacity = 4 * size.x as usize * size.y as usize;
        let mut builder = Self::with_capacity(capacity);

        let cell_size = size.as_vec2().recip();
        for y in 0..size.y {
            for x in 0..size.x {
                let i = y * size.x + x;

                let position = UVec2::new(x, y).as_vec2();
                let offset = position * FLIP_Y * cell_size + MeshQuad::global_offset(cell_size);
                let min = -cell_size / 2. + offset;
                let max = cell_size / 2. + offset;
                let quad = MeshQuad::new(min, max).apply_height_fn(h);
                builder.add_quad(quad, i);
            }
        }
        builder
    }

    /// Adds a quad to the mesh.
    pub fn add_quad(&mut self, quad: MeshQuad, i: u32) {
        self.positions.extend(quad.positions());
        self.uvs.extend(quad.uvs());
        self.normals.extend(quad.normals());
        self.indices.extend(quad.indices(4 * i));
    }

    /// Produce a mesh from the accumulated attributes.
    pub fn build(self) -> Mesh {
        Mesh::new(TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// Wrapper for positions of a mesh quad with functions for producing all primitives
/// required for building a mesh.
pub struct MeshQuad([[f32; 3]; 4]);
impl MeshQuad {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self([
            [min.x, min.y, 0.],
            [max.x, min.y, 0.],
            [max.x, max.y, 0.],
            [min.x, max.y, 0.],
        ])
    }

    /// Compute the height for all points in the quad by
    /// applying the function `h`.
    pub fn apply_height_fn<H: Fn(Vec2) -> f32>(mut self, h: H) -> Self {
        for [x, y, z] in self.0.iter_mut() {
            *z = h(Vec2::new(*x, *y));
        }
        self
    }

    /// Returns the positions of this quad.
    pub fn positions(&self) -> [[f32; 3]; 4] {
        self.0
    }

    /// Compute the indices for the quad.
    pub fn indices(&self, offset: u32) -> [u32; 6] {
        [
            // Triangle 1
            offset,
            1 + offset,
            3 + offset,
            // Triangle 2
            1 + offset,
            2 + offset,
            3 + offset,
        ]
    }

    /// Returns the minimum point in the quad.
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.0[0][0], self.0[0][1])
    }

    /// Return the maximum point in the quad.
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.0[2][0], self.0[2][1])
    }

    /// Compute the offset required for transforming the mesh coordinates to normalized
    /// world coordinates.
    pub fn global_offset(size: Vec2) -> Vec2 {
        -FLIP_Y / 2. + size * FLIP_Y / 2.
    }

    /// Compute UVs for all triangles.
    pub fn uvs(&self) -> [[f32; 2]; 4] {
        let min = self.min();
        let max = self.max();
        let size = max - min;
        let offset = (max - size / 2.) - Self::global_offset(size);
        let uv_offset = offset * FLIP_Y;

        let uv_min = uv_offset;
        let uv_max = size + uv_offset;
        [
            [uv_min.x, uv_max.y],
            [uv_max.x, uv_max.y],
            [uv_max.x, uv_min.y],
            [uv_min.x, uv_min.y],
        ]
    }

    /// Computes the normal of a triangle.
    fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
        let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
        (b - a).cross(c - a).normalize().into()
    }

    /// Compute normals for all triangles.
    pub fn normals(&self) -> [[f32; 3]; 4] {
        [
            // Triangle 1
            Self::face_normal(self.0[0], self.0[1], self.0[3]),
            Self::face_normal(self.0[0], self.0[1], self.0[3]),
            // Triangle 2
            Self::face_normal(self.0[1], self.0[2], self.0[3]),
            Self::face_normal(self.0[1], self.0[2], self.0[3]),
        ]
    }
}
