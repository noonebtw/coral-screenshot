use std::io::{Error, ErrorKind};

use framebuffer::{Framebuffer, FramebufferError};
use image::{DynamicImage, GenericImage, GenericImageView, ImageError, Pixel, Rgb};

use crate::traits::ScreenshotBackend;

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
        let var_info = &self.0.var_screen_info;
        let fix_info = &self.0.fix_screen_info;

        let bytes_per_pixel = var_info.bits_per_pixel / 8;
        let line = fix_info.line_length;
        let x_offset = var_info.xoffset;

        let frame = self.0.read_frame();

        let px_idx = (y * line + x * bytes_per_pixel + x_offset) as usize;

        let px = unsafe { (frame.as_ptr().add(px_idx) as *const usize).read() };

        let r = (px >> var_info.red.offset) & ((1 << var_info.red.length) - 1);
        let g = (px >> var_info.green.offset) & ((1 << var_info.green.length) - 1);
        let b = (px >> var_info.blue.offset) & ((1 << var_info.blue.length) - 1);
        let a = (px >> var_info.transp.offset) & ((1 << var_info.transp.length) - 1);

        Self::Pixel::from_channels(r as u8, g as u8, b as u8, a as u8)
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

impl ScreenshotBackend for FrameBufferBackend {
    fn global_screenshot(&self) -> image::ImageResult<image::DynamicImage> {
        Ok(DynamicImage::ImageRgb8(
            self.view(0, 0, self.width(), self.height()).to_image(),
        ))
    }

    fn window_screenshot(&self, _: &str) -> image::ImageResult<DynamicImage> {
        Err(ImageError::IoError(Error::new(
            ErrorKind::Other,
            "framebuffer backend doesn\'t support per window screencapture.",
        )))
    }
}
