/// Data for the "document" the user is working on.
/// The document is what is saved to file.
use crate::{mutation_monitor::MutationMonitor, vic::VicImage};

/// A "document" the user is working on.
pub struct Document {
    pub paint_color: usize,
    pub image: MutationMonitor<VicImage>,
}
