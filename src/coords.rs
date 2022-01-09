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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SizeInCells {
    pub width: u32,
    pub height: u32,
}

impl SizeInCells {
    /// A 1 by 1 size.
    pub const ONE: SizeInCells = SizeInCells {
        width: 1,
        height: 1,
    };

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

impl std::ops::Add<SizeInCells> for CellPos {
    type Output = CellPos;

    fn add(self, rhs: SizeInCells) -> Self::Output {
        CellPos {
            column: self.column + rhs.width as i32,
            row: self.row + rhs.height as i32,
        }
    }
}

/// A rectangle of character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRect {
    pub top_left: CellPos,
    pub size: SizeInCells,
}

impl CellRect {
    pub fn from_cell_width_height(top_left: CellPos, width: u32, height: u32) -> CellRect {
        CellRect {
            top_left,
            size: SizeInCells { width, height },
        }
    }
    /// Checks that this rectangle fits inside a certain size.
    /// If it does, returns a `WithinBounds<CellRect>`, otherwise `None`.
    pub fn within_size(&self, bounds: SizeInCells) -> Option<WithinBounds<CellRect>> {
        if self.top_left.within_bounds(&bounds).is_none()
            || self.right() > bounds.width as i32
            || self.bottom() > bounds.height as i32
        {
            None
        } else {
            Some(WithinBounds(*self))
        }
    }

    /// Get the leftmost column.
    pub fn left(&self) -> i32 {
        self.top_left.column
    }
    /// Get the column to the right of the rightmost one.
    pub fn right(&self) -> i32 {
        self.top_left.column + self.size.width as i32
    }
    /// Get the topmost row.
    pub fn top(&self) -> i32 {
        self.top_left.row
    }
    /// Get the row below the bottom one.
    pub fn bottom(&self) -> i32 {
        self.top_left.row + self.size.height as i32
    }
    /// Get the total width.
    pub fn width(&self) -> u32 {
        self.size.width
    }
    /// Get the total height.
    pub fn height(&self) -> u32 {
        self.size.height
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

    #[test]
    fn add_pos_and_size() {
        let c = CellPos { column: 5, row: 7 };
        let s = SizeInCells {
            width: 10,
            height: 100,
        };
        assert_eq!(
            CellPos {
                column: 15,
                row: 107
            },
            c + s
        );
    }
}
