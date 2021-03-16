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

/// Main entry point. Parses command-line arguments and prints any errors.
/// On error, returns the exit code for `process::exit`.
pub fn main() -> Result<(), i32> {
    let opts = Opts::from_args();
    let mut app = if let Some(filename) = opts.filename {
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
    app.file_dialog = Some(Box::new(file_dialog));
    eframe::run_native(Box::new(app));
    // run_native never returns
}

fn file_dialog() -> Option<String> {
    use nfd::Response;
    let result = nfd::open_file_dialog(Some("png,flf"), None).ok()?;

    match result {
        Response::Okay(file_path) => Some(file_path),
        Response::OkayMultiple(files) => files.first().cloned(),
        Response::Cancel => None,
    }
}
