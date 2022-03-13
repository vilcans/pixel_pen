//! Screen, pixel and character cell coordinate systems.

use std::ops::Deref;
pub use transform::PixelTransform;

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

// TODO: Make within bounds and within_size a generic trait?

/// Checks whether this `CellPos` is within the given bounds.
/// Returns `Some(WithinBounds<CellCoords>)` if it is, otherwise `None`.
pub fn cell_within_bounds(
    candidate: CellPos,
    bounds: SizeInCells,
) -> Option<WithinBounds<CellPos>> {
    let bounds = CellRect::new(CellPos::zero(), bounds.cast());
    if bounds.contains(candidate) {
        Some(WithinBounds(candidate))
    } else {
        None
    }
}

/// Checks that this rectangle fits inside a certain size.
/// If it does, returns a `WithinBounds<CellRect>`, otherwise `None`.
pub fn cell_rect_within_size(
    candidate: CellRect,
    bounds: SizeInCells,
) -> Option<WithinBounds<CellRect>> {
    let bounds = CellRect::new(CellPos::zero(), bounds.cast());
    if candidate.contains_rect(&bounds) {
        Some(WithinBounds(candidate))
    } else {
        None
    }
}

pub fn clamp_rect_to_bounds(candidate: CellRect, bounds: SizeInCells) -> WithinBounds<CellRect> {
    let left = candidate.min_x().clamp(0, bounds.width as i32);
    let right = candidate.max_x().clamp(0, bounds.width as i32);
    let top = candidate.min_y().clamp(0, bounds.height as i32);
    let bottom = candidate.max_y().clamp(0, bounds.height as i32);
    WithinBounds::assume_within_bounds(CellRect::new(
        CellPos::new(left, top),
        SizeInCells::new(right - left, bottom - top),
    ))
}

/// Wraps a value that is known to be within bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithinBounds<T>(T);

impl<T> WithinBounds<T> {
    pub fn assume_within_bounds(t: T) -> WithinBounds<T> {
        Self(t)
    }
}

impl<T> Deref for WithinBounds<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WithinBounds<CellPos> {
    /// Get the horizontal position (column) and vertical position (row) as a tuple.
    pub fn as_tuple(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

#[cfg(test)]
mod test {
    use super::{cell_within_bounds, clamp_rect_to_bounds, CellPos, CellRect, SizeInCells};

    #[test]
    fn within_bounds() {
        let c = CellPos::new(1, 2);
        let v = cell_within_bounds(c, SizeInCells::new(10, 20));
        assert!(v.is_some());
    }

    #[test]
    fn within_bounds_successful() {
        let c = CellPos::new(10, 21);
        let v = cell_within_bounds(c, SizeInCells::new(10, 20));
        assert!(v.is_none());
    }

    #[test]
    fn add_pos_and_size() {
        let c = CellPos::new(5, 7);
        let s = SizeInCells::new(10, 100);
        assert_eq!(CellPos::new(15, 107), c + s);
    }

    #[test]
    fn rect_clamp_to_size() {
        let r = CellRect::new(CellPos::new(2, 10), SizeInCells::new(8, 4));
        let c = clamp_rect_to_bounds(r, SizeInCells::new(5, 12));
        assert_eq!(
            *c,
            CellRect::new(CellPos::new(2, 10), SizeInCells::new(3, 2))
        );
    }
}
