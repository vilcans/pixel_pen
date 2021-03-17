//! File I/O

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::{error::Error, image_io, mutation_monitor::MutationMonitor, vic::VicImage, Document};

pub fn load_any_file(filename: &Path) -> Result<Document, Error> {
    let ext = filename
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());
    if ext == Some("flf".to_string()) {
        let image = image_io::load_file(filename)?;
        Ok(Document {
            image: MutationMonitor::<VicImage>::new_dirty(image),
            ..Default::default()
        })
    } else {
        load_own(filename)
    }
}

/// Load a file in our own format
pub fn load_own(filename: &Path) -> Result<Document, Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let doc = serde_json::from_reader(reader)?;
    Ok(doc)
}

pub fn save(document: &Document, filename: &Path) -> Result<(), Error> {
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, document)?;
    Ok(())
}
