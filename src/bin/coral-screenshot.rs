use std::path::PathBuf;

use clap::{App, Arg};
use coral::screenshot::{self, Backend};
use image::{DynamicImage, ImageError, ImageOutputFormat, ImageResult};
use log::{info, warn};

struct Config {
    output_file: Option<PathBuf>,
    window: Option<String>,
    backend: Backend,
    silent: bool,
}

impl Config {
    fn parse() -> Self {
        let matches = App::new(std::env!("CARGO_BIN_NAME"))
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
                clap::Arg::with_name("quiet")
                    .short("q")
                    .long("quiet")
                    .takes_value(false)
                    .help("silent execution"),
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
                    .help("The file name the screenshot is saved to."),
            )
            .get_matches();

        Self {
            output_file: matches.value_of("filename").map(|s| s.into()),
            backend: matches.value_of("backend").into(),
            window: matches.value_of("window").map(str::to_owned),
            silent: matches.is_present("quiet"),
        }
    }

    fn save_image(&self, image: DynamicImage) -> ImageResult<()> {
        match &self.output_file {
            Some(path) => {
                info!("saving as {}", path.to_string_lossy());
                image.save(path)?;
            }
            None => {
                if !atty::is(atty::Stream::Stdout) {
                    let stdout = std::io::stdout();

                    image.write_to(&mut stdout.lock(), ImageOutputFormat::Png)?;
                } else {
                    warn!("stdout is a tty, aborting printing binary..");
                }
            }
        };

        Ok(())
    }
}

fn main() -> Result<(), ImageError> {
    let config = Config::parse();

    if !config.silent {
        simple_logger::SimpleLogger::new().init().unwrap();
    }

    info!("getting screenshot.. [backend = \"{:?}\"]", config.backend);
    let image = screenshot::get_screenshot(&config.backend, config.window.as_deref())?;

    config.save_image(image).unwrap();

    Ok(())
}
