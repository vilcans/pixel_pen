use std::collections::HashMap;

use bit_vec::BitVec;

/// Pixels or cells that are affected by an update
pub struct UpdateArea {
    pixels: Vec<(i32, i32)>,
}

impl UpdateArea {
    pub fn from_pixel(x: i32, y: i32) -> Self {
        Self {
            pixels: vec![(x, y)],
        }
    }
    pub fn iter_pixels(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.pixels.iter().copied()
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
        columns: u32,
        rows: u32,
    ) -> HashMap<(u32, u32), BitVec> {
        let x_range = 0..(columns * cell_width) as i32;
        let y_range = 0..(rows * cell_height) as i32;
        let mut cells = HashMap::new();
        for (x, y) in self.pixels.iter().copied() {
            if x_range.contains(&x) && y_range.contains(&y) {
                let (x, y) = (x as u32, y as u32);
                let col = x / cell_width;
                let row = y / cell_height;
                let cx = x % cell_width;
                let cy = y % cell_height;
                cells
                    .entry((col, row))
                    .or_insert_with(|| {
                        BitVec::from_elem(cell_width as usize * cell_height as usize, false)
                    })
                    .set((cx + cy * cell_width) as usize, true)
            }
        }
        cells
    }
}
