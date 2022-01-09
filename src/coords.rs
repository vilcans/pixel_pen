//! Screen, pixel and character cell coordinate systems.

use std::{fmt::Display, ops::Deref};

mod transform;

pub use transform::PixelTransform;

/// Integer point, e.g. pixel coordinates.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Width and height in character cells.
#[derive(Debug, Clone, Copy)]
pub struct SizeInCells {
    pub width: u32,
    pub height: u32,
}

impl SizeInCells {
    /// The total number of cells (width * height).
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

/// Position of a cell; column and row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellPos {
    pub column: i32,
    pub row: i32,
}

impl CellPos {
    /// Checks whether this `CellPos` is within the given bounds.
    /// Returns `Some(WithinBounds<CellCoords>)` if it is, otherwise `None`.
    pub fn within_bounds(&self, bounds: &SizeInCells) -> Option<WithinBounds<CellPos>> {
        if self.column < 0
            || self.column as u32 >= bounds.width
            || self.row < 0
            || self.row as u32 >= bounds.height
        {
            None
        } else {
            Some(WithinBounds(*self))
        }
    }
}

/// Wraps a value that is known to be within bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithinBounds<T>(T);

impl<T> Deref for WithinBounds<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WithinBounds<CellPos> {
    /// Get the horizontal position (column) and vertical position (row) as a tuple.
    pub fn as_tuple(&self) -> (usize, usize) {
        (self.column as usize, self.row as usize)
    }
}

#[cfg(test)]
mod test {
    use crate::coords::SizeInCells;

    use super::CellPos;

    #[test]
    fn within_bounds() {
        let c = CellPos { column: 1, row: 2 };
        let v = c.within_bounds(&SizeInCells {
            width: 10,
            height: 20,
        });
        assert!(v.is_some());
    }

    #[test]
    fn within_bounds_successful() {
        let c = CellPos {
            column: 10,
            row: 21,
        };
        let v = c.within_bounds(&SizeInCells {
            width: 10,
            height: 20,
        });
        assert!(v.is_none());
    }
}
