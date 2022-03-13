use std::collections::HashMap;

use bit_vec::BitVec;

use crate::{
    coords::{self, CellPos, PixelPoint, SizeInCells, WithinBounds},
    line,
};

/// Pixels or cells that are affected by an update
pub struct UpdateArea {
    pixels: Vec<PixelPoint>,
}

impl UpdateArea {
    pub fn from_pixel(p: PixelPoint) -> Self {
        Self { pixels: vec![p] }
    }

    /// Create an UpdateArea from a line between two pixels.
    /// To avoid overdrawing the ending point of a previous line,
    /// the starting pixel `p0` is not included in the line.
    pub fn pixel_line(p0: PixelPoint, p1: PixelPoint) -> Self {
        UpdateArea {
            pixels: line::line(p0, p1).skip(1).collect(),
        }
    }

    /// Get the character cells affected by this area.
    /// `cell_width` and `cell_height` is the size of the cells (often 8 by 8 pixels).
    /// `columns` and `rows` are the image width and height in cells,
    /// and constrain the result to exclude cells outside the image bounds.
    /// Returns a mapping between `(column, row)` and a mask of the pixels in the cell.
    pub fn cells_and_pixels(
        &self,
        cell_width: u32,
        cell_height: u32,
        size_in_cells: SizeInCells,
    ) -> HashMap<WithinBounds<CellPos>, BitVec> {
        let mut cells = HashMap::new();
        for PixelPoint { x, y, .. } in self.pixels.iter().copied() {
            if let Some(cell) = coords::cell_within_bounds(
                CellPos::new(
                    x.div_euclid(cell_width as i32),
                    y.div_euclid(cell_height as i32),
                ),
                size_in_cells,
            ) {
                let (x, y) = (x as u32, y as u32);
                let cx = x.rem_euclid(cell_width);
                let cy = y.rem_euclid(cell_height);
                cells
                    .entry(cell)
                    .or_insert_with(|| {
                        BitVec::from_elem(cell_width as usize * cell_height as usize, false)
                    })
                    .set((cx + cy * cell_width) as usize, true)
            }
        }
        cells
    }
}
