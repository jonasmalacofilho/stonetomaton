//! A 2-dimensional grid.

use std::fmt::{Display, Write};
use std::iter;

/// A 2-dimensional grid of bool values.
///
/// The heigh, width, and all coordinates are signed integers, making it easier to deal with
/// movements around `0`, which can result in negative coordinates.
#[derive(Debug, Clone)]
pub struct Grid {
    height: i16,
    width: i16,
    raw: Vec<bool>,
}

impl Grid {
    /// Creates a grid from a 2-dimensional nested vecs.
    pub fn from_nested_vecs(vecs: Vec<Vec<bool>>) -> Self {
        let (h, w) = (vecs.len(), vecs[0].len());
        let (height, width): (i16, i16) = (h.try_into().unwrap(), w.try_into().unwrap());

        let mut raw = Vec::with_capacity(h * w);
        for row in vecs {
            assert_eq!(row.len(), w);
            raw.extend(row);
        }

        Self { height, width, raw }
    }

    pub fn height(&self) -> i16 {
        self.height
    }

    pub fn width(&self) -> i16 {
        self.width
    }

    /// Returns a reference to the value in cell `(i, j)`, or `None` if `(i, j)` is not in bounds.
    pub fn get(&self, i: i16, j: i16) -> Option<bool> {
        let index = self.index(i, j)?;
        self.raw.get(index).copied()
    }

    /// Returns a mutable reference to the value in cell `(i, j)`, or `None` if `(i, j)` is not in
    /// bounds.
    pub fn set(&mut self, i: i16, j: i16, value: bool) {
        let index = self.index(i, j).unwrap();
        self.raw[index] = value;
    }

    /// Iterate through all cells in the grid.
    pub fn cells(&self) -> impl Iterator<Item = (i16, i16, bool)> + '_ {
        let (mut i, mut j) = (0, 0);
        iter::from_fn(move || {
            if let Some(index) = self.index(i, j) {
                // SAFETY: `index` is in bounds.
                let cur = (i, j, unsafe { *self.raw.get_unchecked(index) });
                j += 1;
                if j >= self.width {
                    j = 0;
                    i += 1;
                }
                Some(cur)
            } else {
                None
            }
        })
    }

    /// Count cells set to `true` in the Moore's neighborhood of `(i, j)`.
    ///
    /// The grid does *not* wrap around the edges, and `(i, j)` must point to a cell within the
    /// grid (in other words, it must in bounds).
    pub fn count_neighbors(&self, i: i16, j: i16) -> u8 {
        assert!(self.index(i, j).is_some());

        fn helper(grid: &Grid, i: i16, j: i16, addi: i16, addj: i16) -> u8 {
            if let Some(index) = grid.index(i + addi, j + addj) {
                // SAFETY: `index` is in bounds.
                unsafe { *grid.raw.get_unchecked(index) as u8 }
            } else {
                0
            }
        }

        helper(self, i, j, -1, -1)
            + helper(self, i, j, -1, 0)
            + helper(self, i, j, -1, 1)
            + helper(self, i, j, 0, -1)
            + helper(self, i, j, 0, 1)
            + helper(self, i, j, 1, -1)
            + helper(self, i, j, 1, 0)
            + helper(self, i, j, 1, 1)
    }

    pub fn raw(&self) -> &[bool] {
        &self.raw
    }

    // FIXME: add tests since safety now depends on its correctness.
    fn index(&self, i: i16, j: i16) -> Option<usize> {
        if !(0..self.height).contains(&i) || !(0..self.width).contains(&j) {
            return None;
        }
        let (i, j, w): (usize, usize, usize) = (i as _, j as _, self.width as _);
        Some(i * w + j)
    }

    /// Creates a grid of `height` and `width` with all values set to `false`.
    pub fn new(height: i16, width: i16) -> Self {
        let (h, w): (usize, usize) = (height.try_into().unwrap(), width.try_into().unwrap());
        let raw = vec![Default::default(); h * w];
        Self { height, width, raw }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, j, cell) in self.cells() {
            if j == 0 && i != 0 {
                f.write_char('\n')?;
            }
            f.write_str(if cell { "██" } else { "░░" })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_shared_or_mut_references() {
        //     Grid
        // +-----------+
        // | 0 1 1 1 1 |
        // | 1 1 1 1 1 |
        // +-----------+

        let vecs = vec![vec![false, true, true, true, true], vec![true; 5]];
        let mut grid = Grid::from_nested_vecs(vecs);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.width(), 5);

        assert_eq!(grid.get(1, 3), Some(true));
        assert_eq!(grid.get(2, 3), None);
        assert_eq!(grid.get(-1, 3), None);
        assert_eq!(grid.get(1, 5), None);
        assert_eq!(grid.get(1, -1), None);

        grid.set(1, 3, false);
        assert_eq!(grid.get(1, 3), Some(false));
    }

    #[test]
    fn iterate_through_all_cells() {
        //   Grid
        // +-------+
        // | 0 1 1 |
        // | 1 1 1 |
        // +-------+

        let vecs = vec![vec![false, true, true], vec![true; 3]];
        let grid = Grid::from_nested_vecs(vecs);

        assert_eq!(
            grid.cells().collect::<Vec<_>>(),
            vec![
                (0, 0, false),
                (0, 1, true),
                (0, 2, true),
                (1, 0, true),
                (1, 1, true),
                (1, 2, true)
            ]
        );
    }

    #[test]
    fn count_neighbors() {
        //    Grid           Moore neighborhoods
        //  +-------+      +-------+      +-------+
        //  | 0 1 1 |      | 0 1 1 |      |       |
        //  | 1 1 1 |      | 1 * 1 |      |   1 1 |
        //  | 1 1 1 |      | 1 1 1 |      |   1 * |
        //  +-------+      +-------+      +-------+
        //                (i,j)=(1,1)    (i,j)=(2,2)

        let vecs = vec![vec![false, true, true], vec![true; 3], vec![true; 3]];
        let grid = Grid::from_nested_vecs(vecs);

        assert_eq!(grid.count_neighbors(1, 1), 7);
        assert_eq!(grid.count_neighbors(2, 2), 3);
    }

    #[test]
    #[should_panic]
    fn neighborhood_requires_cell_to_be_in_bounds() {
        //   Grid
        // +-------+
        // | 0 1 1 |
        // | 1 1 1 |
        // +-------+
        //           *
        //      (i,j)=(2,3)

        let vecs = vec![vec![false, true, true], vec![true; 3]];
        let grid = Grid::from_nested_vecs(vecs);

        let _should_panic = grid.count_neighbors(2, 3);
    }
}
