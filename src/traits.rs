use image::{DynamicImage, ImageResult};

pub trait ScreenshotBackend {
    fn global_screenshot(&self) -> ImageResult<DynamicImage>;
    fn window_screenshot(&self, window_id: &str) -> ImageResult<DynamicImage>;

    fn screenshot(&self, window_id: Option<&str>) -> ImageResult<DynamicImage> {
        match window_id {
            Some(id) => self.window_screenshot(id),
            None => self.global_screenshot(),
        }
    }
}
