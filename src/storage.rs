//! File I/O

use std::{
    ffi::OsString,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
    str::FromStr,
};

use crate::{
    error::Error,
    image_io::{self, FileFormat},
    Document,
};

/// File name extension (without the ".") for our own file format.
pub const NATIVE_EXTENSION: &str = "pixelpen";

/// Load a file in any supported file format.
pub fn load_any_file(filename: &Path) -> Result<Document, Error> {
    match image_io::identify_file(filename)? {
        FileFormat::Unknown => load_own(filename),
        format => {
            //println!("Loading \"{}\" in format {:?}", filename.display(), format);
            let image = image_io::load_file(filename, format)?;
            Ok(Document::from_image(image))
        }
    }
}

/// Save or export the file to any supported file format.
pub fn save_any_file(document: &Document, filename: &Path) -> Result<(), Error> {
    let native_extension = OsString::from_str(NATIVE_EXTENSION).unwrap();
    if filename.extension() == Some(&native_extension) {
        save(document, filename)
    } else {
        let image = document.image.render();
        image.save(filename).map_err(Error::from)
    }
}

/// Load a file in our own (native) format
pub fn load_own(filename: &Path) -> Result<Document, Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut doc: Document = serde_json::from_reader(reader)?;
    doc.filename = Some(filename.to_owned());
    Ok(doc)
}

/// Save a file in our own (native) format
pub fn save(document: &Document, filename: &Path) -> Result<(), Error> {
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, document)?;
    Ok(())
}
