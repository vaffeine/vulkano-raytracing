use clap;
use cgmath;
use vulkano;

use std;

pub struct Args {
    pub model: String,
    pub resolution: Vec<u32>,
    pub position: cgmath::Vector3<f32>,
    pub fov: [f32; 2],
    pub sensitivity: f32,
    pub fps_update_interval: i64,
    pub log_level: vulkano::instance::debug::MessageTypes,
}

fn is_supported_model_format(val: String) -> Result<(), String> {
    const SUPPORTED_FORMATS: &[&str] = &["obj"];
    let extension = val.rsplit('.')
        .next()
        .ok_or("can't determinate extension of file")?;
    if !std::path::Path::new(&val).is_file() {
        return Err(String::from("file doesn't exist"));
    }
    if SUPPORTED_FORMATS.contains(&extension) {
        Ok(())
    } else {
        Err(format!(
            "model format is not supported. Supported formats: {:?}",
            SUPPORTED_FORMATS
        ))
    }
}

const LOG_LEVELS: &[&str] = &["none", "error", "warning", "perf", "info", "debug"];

fn log_level_from_str(val: &str) -> vulkano::instance::debug::MessageTypes {
    let idx = LOG_LEVELS
        .iter()
        .enumerate()
        .find(|&(_, v)| v == &val)
        .unwrap()
        .0;
    vulkano::instance::debug::MessageTypes {
        error: idx > 0,
        warning: idx > 1,
        performance_warning: idx > 2,
        information: idx > 3,
        debug: idx > 4,
    }
}

impl Args {
    pub fn get_matches() -> Args {
        let matches = clap::App::new("tracer")
            .version(crate_version!())
            .author(crate_authors!())
            .about("Interactive raytracer, that renders triangulated models")
            .arg(
                clap::Arg::with_name("model")
                    .help("Sets the path to file with model to render")
                    .required(true)
                    .index(1)
                    .validator(is_supported_model_format),
            )
            .arg(
                clap::Arg::with_name("resolution")
                    .short("r")
                    .long("resolution")
                    .number_of_values(2)
                    .value_names(&["width", "height"])
                    .display_order(1)
                    .help("Sets the resolution of the image [default: 640 480]"),
            )
            .arg(
                clap::Arg::with_name("position")
                    .short("p")
                    .long("position")
                    .number_of_values(3)
                    .value_names(&["x", "y", "z"])
                    .display_order(2)
                    .help("Sets the position of camera [default: 0.0 0.0 5.0]"),
            )
            .arg(
                clap::Arg::with_name("fov")
                    .long("fov")
                    .number_of_values(2)
                    .value_names(&["horizontal", "vertical"])
                    .display_order(3)
                    .help("Sets the field of view [default: 40.0 40.0]"),
            )
            .arg(
                clap::Arg::with_name("sensitivity")
                    .long("sensitivity")
                    .takes_value(true)
                    .display_order(4)
                    .help("Sets the sensitivity of the controls (camera movement) [default: 1.0]"),
            )
            .arg(
                clap::Arg::with_name("fps-update-interval")
                    .long("fps-update-interval")
                    .takes_value(true)
                    .display_order(5)
                    .help(
                        "Sets the interval (in milliseconds) of FPS update. \
                         Displayed FPS is the average in the last interval [default: 100]",
                    ),
            )
            .arg(
                clap::Arg::with_name("log-level")
                    .long("log-level")
                    .takes_value(true)
                    .possible_values(LOG_LEVELS)
                    .display_order(6)
                    .help(
                        "Sets the log messages amount [default: perf]",
                    ),
            )
            .get_matches();
        let model = matches.value_of("model").unwrap().to_string();
        // kbknapp promisses `default_values` method in clap v3. But for now...
        let resolution = if matches.is_present("resolution") {
            values_t!(matches, "resolution", u32).unwrap_or_else(|e| e.exit())
        } else {
            vec![640, 480]
        };
        let position = if matches.is_present("position") {
            values_t!(matches, "position", f32).unwrap_or_else(|e| e.exit())
        } else {
            vec![0.0, 0.0, 5.0]
        };
        let fov = if matches.is_present("fov") {
            values_t!(matches, "fov", f32).unwrap_or_else(|e| e.exit())
        } else {
            vec![40.0, 40.0]
        };
        // ...and if I use `default_value` for this one, it will always dispaly it
        // in the help message if no model is passed.
        // which is not the end of the world but just pisses me off
        let sensitivity = if matches.is_present("sensitivity") {
            value_t!(matches, "sensitivity", f32).unwrap_or_else(|e| e.exit())
        } else {
            1.0
        };
        let fps_update_interval = if matches.is_present("fps-update-interval") {
            value_t!(matches, "fps-update-interval", i64).unwrap_or_else(|e| e.exit())
        } else {
            100
        };
        let log_level = if matches.is_present("log-level") {
            log_level_from_str(matches.value_of("log-level").unwrap())
        } else {
            vulkano::instance::debug::MessageTypes::errors_and_warnings()
        };
        Args {
            model,
            resolution,
            position: cgmath::Vector3::new(position[0], position[1], position[2]),
            fov: [fov[0], fov[1]],
            sensitivity,
            fps_update_interval,
            log_level,
        }
    }
}
