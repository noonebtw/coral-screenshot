use std::io::{Error, ErrorKind};

use clap::{App, Arg};
use fb::FrameBufferBackend;
use image::{DynamicImage, ImageError, ImageResult};
use log::info;
use traits::Backend;

use crate::{traits::ScreenshotBackend, xlib::XLibState};

mod fb;
mod traits;
mod xlib;

fn main() -> Result<(), ImageError> {
    simple_logger::SimpleLogger::new().init().unwrap();

    let matches = App::new("Coral")
        .version(std::env!("CARGO_PKG_VERSION"))
        .author("No One")
        .about("Coral takes screenshots")
        .arg(
            Arg::with_name("backend")
                .short("b")
                .long("backend")
                .value_name("backend")
                .help("backend thats used. Currently supports `fb` and `xlib`")
                .default_value("xlib"),
        )
        .arg(
            Arg::with_name("window")
                .short("w")
                .long("window")
                .value_name("window")
                .help("The hexadecimal id of the X window to take a screenshot of.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("filename")
                .short("o")
                .long("output")
                .value_name("filename")
                .help("The file name the screenshot is saved to.")
                .default_value("screenshot.png"),
        )
        .get_matches();

    let window = matches.value_of("window");

    let backend = matches.value_of("backend");

    info!("getting screenshot.. [backend = \"{}\"]", backend.unwrap());
    let image = screenshot(backend.into(), window)?;

    let image_name = matches.value_of("filename").unwrap();
    info!("saving as {}", image_name);
    image.save(image_name)?;

    Ok(())
}

fn screenshot(backend: Backend, window: Option<&str>) -> ImageResult<DynamicImage> {
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
