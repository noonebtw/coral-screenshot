use framebuffer::{Framebuffer, FramebufferError};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgb};

use crate::traits::GlobalScreenshotBackend;

/// Framebuffer backend.
#[derive(Debug)]
pub struct FrameBufferBackend(Framebuffer);

impl FrameBufferBackend {
    pub fn new() -> Result<Self, FramebufferError> {
        Ok(Self(Framebuffer::new("/dev/fb0")?))
    }

    fn width(&self) -> u32 {
        self.0.var_screen_info.xres
    }

    fn height(&self) -> u32 {
        self.0.var_screen_info.yres
    }
}

impl GenericImageView for FrameBufferBackend {
    type Pixel = Rgb<u8>;

    type InnerImageView = Self;

    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width(), self.height())
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let bpp = self.0.var_screen_info.bits_per_pixel / 8;
        let line = self.0.fix_screen_info.line_length;
        let x_offset = self.0.var_screen_info.xoffset;

        let frame = self.0.read_frame();

        let px_start = (y * line + x * bpp + x_offset) as usize;

        let r = frame[px_start + 0];
        let b = frame[px_start + 1];
        let g = frame[px_start + 2];

        Self::Pixel::from_channels(r as u8, g as u8, b as u8, 0u8)
    }

    fn inner(&self) -> &Self::InnerImageView {
        self
    }
}

impl GenericImage for FrameBufferBackend {
    type InnerImage = Self;

    fn get_pixel_mut(&mut self, _x: u32, _y: u32) -> &mut Self::Pixel {
        todo!()
    }

    fn put_pixel(&mut self, _x: u32, _y: u32, _pixel: Self::Pixel) {
        todo!()
    }

    fn blend_pixel(&mut self, _x: u32, _y: u32, _pixel: Self::Pixel) {
        todo!()
    }

    fn inner_mut(&mut self) -> &mut Self::InnerImage {
        self
    }
}

impl GlobalScreenshotBackend for FrameBufferBackend {
    fn get_global_screenshot(&self) -> image::ImageResult<image::DynamicImage> {
        Ok(DynamicImage::ImageRgb8(
            self.view(0, 0, self.width(), self.height()).to_image(),
        ))
    }
}
