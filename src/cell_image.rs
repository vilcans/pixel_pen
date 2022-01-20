//! Handles coordinates of images divided into cells.

use crate::coords::{CellPos, CellRect, Point, SizeInCells, WithinBounds};

pub trait CellImageSize {
    /// Get the size of the image in cells.
    fn size_in_cells(&self) -> SizeInCells;

    /// Get the width and height of the image in pixels.
    fn size_in_pixels(&self) -> (usize, usize);
}

pub trait CellCoordinates: CellImageSize {
    const CELL_WIDTH: usize;
    const CELL_HEIGHT: usize;

    /// Get the pixel coordinates of the top-left corner of a character cell.
    /// Accepts coordinates outside the image.
    fn cell_coordinates_unclipped(&self, cell: &CellPos) -> Point {
        Point {
            x: cell.column * Self::CELL_WIDTH as i32,
            y: cell.row * Self::CELL_HEIGHT as i32,
        }
    }

    /// Get a rectangle in pixel coordinates from a rectangle in character cells.
    /// Returns the top left, and bottom right (exclusive) of the rectangle in image pixels.
    /// Accepts coordinates outside the image.
    fn cell_rectangle(&self, rect: &CellRect) -> (Point, Point) {
        (
            self.cell_coordinates_unclipped(&rect.top_left),
            self.cell_coordinates_unclipped(&(rect.top_left + rect.size)),
        )
    }

    /// Given pixel coordinates, return which cell that is, and x and y inside the cell.
    /// May return coordinates outside the image.
    fn cell_unclipped(&self, point: Point) -> (CellPos, i32, i32) {
        let column = point.x.div_euclid(Self::CELL_WIDTH as i32);
        let cx = point.x.rem_euclid(Self::CELL_WIDTH as i32);
        let row = point.y.div_euclid(Self::CELL_HEIGHT as i32);
        let cy = point.y.rem_euclid(Self::CELL_HEIGHT as i32);
        (CellPos { column, row }, cx, cy)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// Returns None if the coordinates are outside the image.
    fn cell(&self, point: Point) -> Option<(WithinBounds<CellPos>, i32, i32)> {
        let (cell, cx, cy) = self.cell_unclipped(point);
        let cell = cell.within_bounds(&self.size_in_cells())?;
        Some((cell, cx, cy))
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// If the arguments are outside the image, they are clamped to be inside it.
    fn cell_clamped(&self, point: Point) -> (WithinBounds<CellPos>, i32, i32) {
        let (width, height) = self.size_in_pixels();
        let (cell, cx, cy) =
            self.cell_unclipped(point.clamped(width as i32 - 1, height as i32 - 1));
        (cell.within_bounds(&self.size_in_cells()).unwrap(), cx, cy)
    }

    /// Return the top-left edge of the character that is closest to the given point.
    /// If the arguments are outside the image, they are clamped to be inside it.
    fn cell_rounded(&self, point: Point) -> (WithinBounds<CellPos>, i32, i32) {
        self.cell_clamped(Point {
            x: point.x + Self::CELL_WIDTH as i32 / 2,
            y: point.y + Self::CELL_HEIGHT as i32 / 2,
        })
    }
}
