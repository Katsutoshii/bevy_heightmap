use bevy_math::{UVec2, Vec2};
use image::{ImageBuffer, Pixel};

/// Creates a heightmap from an image buffer.
pub trait ImageBufferHeightMap {
    /// Constructs the image buffer from raw bytes.
    fn from_bytes(size: UVec2, buffer: &[u8]) -> Self;
    /// Returns the height value at a given position.
    fn get_height_value(&self, p: Vec2) -> f32;
}

impl<P: Pixel> ImageBufferHeightMap for ImageBuffer<P, Vec<P::Subpixel>>
where
    P::Subpixel: bytemuck::Pod + Into<f32>,
{
    fn from_bytes(size: UVec2, buffer: &[u8]) -> Self {
        Self::from_raw(size.x, size.y, bytemuck::cast_slice(buffer).to_owned()).unwrap()
    }
    fn get_height_value(&self, p: Vec2) -> f32 {
        let bounds = UVec2 {
            x: self.width() - 1,
            y: self.height() - 1,
        };
        let pixel_scale = bounds.as_vec2();
        let xy = (pixel_scale * (p + Vec2::ONE / 2.)).as_uvec2();
        self.get_pixel(xy.x, bounds.y - xy.y).channels()[0].into()
    }
}
