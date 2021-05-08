use clap::{App, Arg};
use fb::FrameBufferBackend;
use image::{DynamicImage, ImageError, ImageResult};
use log::info;

use crate::{
    traits::{GlobalScreenshotBackend, PerWindowScreenshotBackend},
    xlib::XLibState,
};

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
    let image = match matches.value_of("backend") {
        Some("fb") => fb_screenshot(),
        Some("xlib") => xlib_screenshot(window),
        _ => xlib_screenshot(window),
    }?;

    let image_name = matches.value_of("filename").unwrap();
    info!("saving as {}", image_name);
    image.save(image_name)?;

    Ok(())
}

fn xlib_screenshot(window: Option<&str>) -> ImageResult<DynamicImage> {
    let x = XLibState::new().expect("failed to init xlib state.");

    match window {
        Some(id) => {
            info!("window: {}", id);
            x.get_screenshot(id)
        }
        None => x.get_global_screenshot(),
    }
}

fn fb_screenshot() -> ImageResult<DynamicImage> {
    let fb = FrameBufferBackend::new()
        .map_err(|err| ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, err)))?;

    fb.get_global_screenshot()
}
