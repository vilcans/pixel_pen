//! Command-line interface

use pixel_pen::{Application, Document};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Pixel Pen", about = "Actual 8 bit graphics editor")]
struct Opts {
    /// File to load
    #[structopt(parse(from_os_str))]
    filename: Option<PathBuf>,
}

/// Parses command-line arguments and prints any errors, returns Application ready to start.
/// If app shouldn't start, returns None.
/// On error, returns the exit code for `process::exit`.
pub fn main() -> Result<Option<Application>, i32> {
    let opts = Opts::from_args();
    let app = if let Some(filename) = opts.filename {
        let doc = Document::load(&filename).map_err(|err| {
            eprintln!(
                "Could not load file {}: {}",
                filename.to_string_lossy(),
                err
            );
            1
        })?;
        Application::with_doc(doc)
    } else {
        Application::default()
    };
    Ok(Some(app))
}
