use std::io::{Error, ErrorKind};

use image::{DynamicImage, ImageError, ImageResult};

use crate::{fb::FrameBufferBackend, traits::ScreenshotBackend, xlib::XLibState};

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    XLib,
    FrameBuffer,
    Default,
}

pub fn get_screenshot(backend: &Backend, window: Option<&str>) -> ImageResult<DynamicImage> {
    let backend: Box<dyn ScreenshotBackend> = match backend {
        Backend::FrameBuffer => Box::new(
            FrameBufferBackend::new()
                .map_err(|e| ImageError::IoError(Error::new(ErrorKind::Other, e)))?,
        ),
        Backend::XLib | Backend::Default => {
            Box::new(XLibState::new().ok_or(ImageError::IoError(Error::new(
                ErrorKind::Other,
                "Failed to connect to X server.",
            )))?)
        }
    };

    backend.screenshot(window)
}

impl From<Option<&str>> for Backend {
    fn from(name: Option<&str>) -> Self {
        match name {
            Some("fb") => Self::FrameBuffer,
            Some("xlib") => Self::XLib,
            _ => Self::Default,
        }
    }
}
