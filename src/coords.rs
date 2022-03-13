//! Screen, pixel and character cell coordinate systems.

pub use bounds::{cell_rect_within_size, cell_within_bounds, clamp_rect_to_bounds, WithinBounds};
pub use transform::PixelTransform;

mod bounds;
mod transform;

pub struct PixelUnit;
pub struct CellUnit;

/// Coordinates for a pixel
pub type PixelPoint = euclid::Point2D<i32, PixelUnit>;

/// Rectangle in pixel coordinates.
pub type PixelRect = euclid::Rect<i32, PixelUnit>;

/// Position of a cell; column (x) and row (y).
pub type CellPos = euclid::Point2D<i32, CellUnit>;

/// Width and height in character cells.
pub type SizeInCells = euclid::Size2D<i32, CellUnit>;

/// Rectangle of cells
pub type CellRect = euclid::Rect<i32, CellUnit>;
