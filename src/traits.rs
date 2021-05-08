use image::{DynamicImage, ImageResult};

pub trait GlobalScreenshotBackend {
    fn get_global_screenshot(&self) -> ImageResult<DynamicImage>;
}

pub trait PerWindowScreenshotBackend {
    fn get_screenshot(&self, window_id: &str) -> ImageResult<DynamicImage>;
}
