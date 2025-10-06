use bevy::{
    image::Image,
    math::{UVec2, Vec2},
};
use image::{DynamicImage, ImageBuffer, Pixel, Rgba};

use crate::{HeightMap, asset_loader::HeightMapLoaderError};

pub struct ImageBufferHeightMap<P: Pixel, Container> {
    pub buffer: ImageBuffer<P, Container>,
    pub pixel_scale: Vec2,
    pub bounds: UVec2,
}
impl<P: Pixel> ImageBufferHeightMap<P, Vec<P::Subpixel>>
where
    P::Subpixel: bytemuck::Pod + Into<f32>,
{
    pub fn from_bytes(size: UVec2, buffer: &[u8]) -> Self {
        let bounds = size - UVec2::ONE;
        let pixel_scale = bounds.as_vec2();
        let buffer =
            ImageBuffer::from_raw(size.x, size.y, bytemuck::cast_slice(buffer).to_owned()).unwrap();
        Self {
            buffer,
            pixel_scale,
            bounds,
        }
    }
}
impl ImageBufferHeightMap<Rgba<u8>, Vec<u8>> {
    pub fn try_from_image(image: Image) -> Result<Self, HeightMapLoaderError> {
        let size = image.size();
        let bounds = size - UVec2::ONE;
        let pixel_scale = bounds.as_vec2();
        let DynamicImage::ImageRgba8(buffer) = image.try_into_dynamic()? else {
            return Err(HeightMapLoaderError::UnsupportedImageType);
        };
        Ok(Self {
            buffer,
            pixel_scale,
            bounds,
        })
    }
}
impl HeightMap for ImageBufferHeightMap<Rgba<f32>, Vec<f32>> {
    fn h(&self, p: Vec2) -> f32 {
        let xy = (self.pixel_scale * (p + Vec2::ONE / 2.)).as_uvec2();
        self.buffer.get_pixel(xy.x, self.bounds.y - xy.y).channels()[0]
    }
}

impl HeightMap for ImageBufferHeightMap<Rgba<u8>, Vec<u8>> {
    fn h(&self, p: Vec2) -> f32 {
        let xy = (self.pixel_scale * (p + Vec2::ONE / 2.)).as_uvec2();
        self.buffer.get_pixel(xy.x, self.bounds.y - xy.y).channels()[0] as f32 / 255.
    }
}
