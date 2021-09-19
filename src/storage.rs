//! File I/O

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::{
    error::Error,
    image_io::{self, FileFormat},
    mutation_monitor::MutationMonitor,
    vic::VicImage,
    Document,
};

pub fn load_any_file(filename: &Path) -> Result<Document, Error> {
    match image_io::identify_file(filename)? {
        FileFormat::Unknown => load_own(filename),
        format => {
            //println!("Loading \"{}\" in format {:?}", filename.display(), format);
            let image = image_io::load_file(filename, format)?;
            Ok(Document {
                image: MutationMonitor::<VicImage>::new_dirty(image),
                ..Default::default()
            })
        }
    }
}

/// Load a file in our own format
pub fn load_own(filename: &Path) -> Result<Document, Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut doc: Document = serde_json::from_reader(reader)?;
    doc.filename = Some(filename.to_owned());
    Ok(doc)
}

pub fn save(document: &Document, filename: &Path) -> Result<(), Error> {
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, document)?;
    Ok(())
}
