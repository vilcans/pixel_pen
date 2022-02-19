//! Command-line interface

use pixel_pen::{error::Error, storage, Application, Document};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Pixel Pen", about = "Actual 8 bit graphics editor")]
struct Opts {
    /// Open the given file in import mode
    #[structopt(long = "--import")]
    import_file: Option<PathBuf>,
    /// File to load
    #[structopt(parse(from_os_str))]
    filename: Option<PathBuf>,
    /// Save the image to the given file and quit.
    /// File may be in pixelpen format or the image may be exported as a standard image file.
    #[structopt(long = "--save")]
    save_file: Option<PathBuf>,
}

/// Parses command-line arguments and prints any errors, returns Application ready to start.
/// If app shouldn't start, returns None.
/// On error, returns the exit code for `process::exit`.
pub fn main() -> Result<Option<Application>, i32> {
    let opts = Opts::from_args();
    let doc = if let Some(filename) = &opts.filename {
        Some(storage::load_any_file(filename).map_err(|err| {
            eprintln!(
                "Could not load file {}: {}",
                filename.to_string_lossy(),
                err
            );
            1
        })?)
    } else {
        None
    };
    match execute_commands(&opts, doc.as_ref()) {
        Err(err) => {
            eprintln!("Command failed: {}", err);
            Err(2)
        }
        Ok(true) => Ok(None),
        Ok(false) => {
            let mut app = Application::new();
            let mut doc = doc.unwrap_or_else(Document::new);
            doc.index_number = 1;
            let editor_index = app.add_editor(doc);
            if let Some(filename) = opts.import_file {
                match app
                    .editor_mut(editor_index)
                    .unwrap()
                    .start_import_mode(&filename)
                {
                    Ok(_) => Ok(Some(app)),
                    Err(e) => {
                        eprintln!("Failed to open {:?} for import: {}", filename, e);
                        Err(1)
                    }
                }
            } else {
                Ok(Some(app))
            }
        }
    }
}

/// Returns Ok(true) if a command was executed and the app should quit.
/// Returns Ok(false) if the app should start the GUI.
fn execute_commands(opts: &Opts, doc: Option<&Document>) -> Result<bool, Error> {
    let mut executed = false;
    if let Some(filename) = &opts.save_file {
        match storage::save_any_file(doc.unwrap(), filename) {
            Ok(()) => executed = true,
            Err(e) => {
                eprintln!("Failed to save: {:?}", e);
                return Err(e);
            }
        }
    }
    Ok(executed)
}
