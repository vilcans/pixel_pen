//! Command-line interface

use pixel_pen::Application;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Pixel Pen", about = "Actual 8 bit graphics editor")]
struct Opts {
    /// File to load
    #[structopt(parse(from_os_str))]
    filename: Option<PathBuf>,
}

/// Main entry point. Parses command-line arguments and prints any errors.
/// On error, returns the exit code for `process::exit`.
pub fn main() -> Result<(), i32> {
    let opts = Opts::from_args();
    let app = if let Some(filename) = opts.filename {
        Application::load(&filename).map_err(|err| {
            eprintln!(
                "Could not load file {}: {}",
                filename.to_string_lossy(),
                err
            );
            1
        })?
    } else {
        Application::default()
    };
    eframe::run_native(Box::new(app));
    // run_native never returns
}
