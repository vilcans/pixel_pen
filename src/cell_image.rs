//! Handles coordinates of images divided into cells.

use crate::coords::{CellPos, CellRect, PixelPoint, SizeInCells, WithinBounds};

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
    fn cell_coordinates_unclipped(&self, cell: &CellPos) -> PixelPoint {
        PixelPoint {
            x: cell.column * Self::CELL_WIDTH as i32,
            y: cell.row * Self::CELL_HEIGHT as i32,
        }
    }

    /// Get a rectangle in pixel coordinates from a rectangle in character cells.
    /// Returns the top left, and bottom right (exclusive) of the rectangle in image pixels.
    /// Accepts coordinates outside the image.
    fn cell_rectangle(&self, rect: &CellRect) -> (PixelPoint, PixelPoint) {
        (
            self.cell_coordinates_unclipped(&rect.top_left),
            self.cell_coordinates_unclipped(&(rect.top_left + rect.size)),
        )
    }

    /// Given pixel coordinates, return which cell that is, and x and y inside the cell.
    /// May return coordinates outside the image.
    fn cell_unclipped(&self, point: PixelPoint) -> (CellPos, i32, i32) {
        let column = point.x.div_euclid(Self::CELL_WIDTH as i32);
        let cx = point.x.rem_euclid(Self::CELL_WIDTH as i32);
        let row = point.y.div_euclid(Self::CELL_HEIGHT as i32);
        let cy = point.y.rem_euclid(Self::CELL_HEIGHT as i32);
        (CellPos { column, row }, cx, cy)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// Returns None if the coordinates are outside the image.
    fn cell(&self, point: PixelPoint) -> Option<(WithinBounds<CellPos>, i32, i32)> {
        let (cell, cx, cy) = self.cell_unclipped(point);
        let cell = cell.within_bounds(&self.size_in_cells())?;
        Some((cell, cx, cy))
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// If the arguments are outside the image, they are clamped to be inside it.
    fn cell_clamped(&self, point: PixelPoint) -> (WithinBounds<CellPos>, i32, i32) {
        let (width, height) = self.size_in_pixels();
        let (cell, cx, cy) =
            self.cell_unclipped(point.clamped(width as i32 - 1, height as i32 - 1));
        (cell.within_bounds(&self.size_in_cells()).unwrap(), cx, cy)
    }

    /// Return the top-left edge of the character that is closest to the given point.
    /// If the arguments are outside the image, they are clamped to be inside it.
    fn cell_rounded(&self, point: PixelPoint) -> (WithinBounds<CellPos>, i32, i32) {
        self.cell_clamped(PixelPoint {
            x: point.x + Self::CELL_WIDTH as i32 / 2,
            y: point.y + Self::CELL_HEIGHT as i32 / 2,
        })
    }

    /// Return the top-left edge of the character that is closest to the given point.
    /// May return coordinates outside the image
    fn cell_rounded_unclipped(&self, point: PixelPoint) -> (CellPos, i32, i32) {
        self.cell_unclipped(PixelPoint {
            x: point.x - Self::CELL_WIDTH as i32 / 2,
            y: point.y - Self::CELL_HEIGHT as i32 / 2,
        })
    }

    fn cell_selection(&self, p0: PixelPoint, p1: PixelPoint) -> WithinBounds<CellRect> {
        let (c0, _, _) = self.cell_rounded_unclipped(p0);
        let (c1, _, _) = self.cell_rounded_unclipped(p1);
        let (column, width) = if c1.column >= c0.column {
            (c0.column, c1.column - c0.column)
        } else {
            (c1.column, c0.column - c1.column)
        };
        let (row, height) = if c1.row >= c0.row {
            (c0.row, c1.row - c0.row)
        } else {
            (c1.row, c0.row - c1.row)
        };
        CellRect::from_cell_width_height(CellPos { column, row }, width as u32, height as u32)
            .clamp_to_bounds(self.size_in_cells())
    }
}
