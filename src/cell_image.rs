//! Handles coordinates of images divided into cells.

use crate::coords::{self, CellPos, CellRect, PixelPoint, SizeInCells, WithinBounds};

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
        PixelPoint::new(
            cell.x * Self::CELL_WIDTH as i32,
            cell.y * Self::CELL_HEIGHT as i32,
        )
    }

    /// Get a rectangle in pixel coordinates from a rectangle in character cells.
    /// Returns the top left, and bottom right (exclusive) of the rectangle in image pixels.
    /// Accepts coordinates outside the image.
    fn cell_rectangle(&self, rect: &CellRect) -> (PixelPoint, PixelPoint) {
        (
            self.cell_coordinates_unclipped(&rect.origin),
            self.cell_coordinates_unclipped(&(rect.origin + rect.size)),
        )
    }

    /// Given pixel coordinates, return which cell that is, and x and y inside the cell.
    /// May return coordinates outside the image.
    fn cell_unclipped(&self, point: PixelPoint) -> (CellPos, i32, i32) {
        let column = point.x.div_euclid(Self::CELL_WIDTH as i32);
        let cx = point.x.rem_euclid(Self::CELL_WIDTH as i32);
        let row = point.y.div_euclid(Self::CELL_HEIGHT as i32);
        let cy = point.y.rem_euclid(Self::CELL_HEIGHT as i32);
        (CellPos::new(column, row), cx, cy)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// Returns None if the coordinates are outside the image.
    fn cell(&self, point: PixelPoint) -> Option<(WithinBounds<CellPos>, i32, i32)> {
        let (cell, cx, cy) = self.cell_unclipped(point);
        let cell = coords::within_bounds(cell, self.size_in_cells())?;
        Some((cell, cx, cy))
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// If the arguments are outside the image, they are clamped to be inside it.
    fn cell_clamped(&self, point: PixelPoint) -> (WithinBounds<CellPos>, i32, i32) {
        let (width, height) = self.size_in_pixels(); // TODO: Should be Size2D<i32, PixelCoordType>
        let corner = PixelPoint::new(width as i32 - 1, height as i32 - 1);
        let (cell, cx, cy) = self.cell_unclipped(point.clamp(PixelPoint::zero(), corner));
        (
            coords::within_bounds(cell, self.size_in_cells()).unwrap(),
            cx,
            cy,
        )
    }

    /// Return the top-left edge of the character that is closest to the given point.
    /// If the arguments are outside the image, they are clamped to be inside it.
    fn cell_rounded(&self, point: PixelPoint) -> (WithinBounds<CellPos>, i32, i32) {
        self.cell_clamped(PixelPoint::new(
            point.x + Self::CELL_WIDTH as i32 / 2,
            point.y + Self::CELL_HEIGHT as i32 / 2,
        ))
    }

    /// Return the top-left edge of the character that is closest to the given point.
    /// May return coordinates outside the image
    fn cell_rounded_unclipped(&self, point: PixelPoint) -> (CellPos, i32, i32) {
        self.cell_unclipped(PixelPoint::new(
            point.x + Self::CELL_WIDTH as i32 / 2,
            point.y + Self::CELL_HEIGHT as i32 / 2,
        ))
    }

    fn cell_selection(&self, p0: PixelPoint, p1: PixelPoint) -> WithinBounds<CellRect> {
        let (c0, _, _) = self.cell_rounded_unclipped(p0);
        let (c1, _, _) = self.cell_rounded_unclipped(p1);
        let (column, width) = if c1.x >= c0.x {
            (c0.x, c1.x - c0.x)
        } else {
            (c1.x, c0.x - c1.x)
        };
        let (row, height) = if c1.y >= c0.y {
            (c0.y, c1.y - c0.y)
        } else {
            (c1.y, c0.y - c1.y)
        };
        coords::clamp_rect_to_bounds(
            CellRect::new(CellPos::new(column, row), SizeInCells::new(width, height)),
            self.size_in_cells(),
        )
    }
}
