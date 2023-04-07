//! A 2-dimensional grid.
// FIXME: make it generic again and/or merge with BitGrid.
// FIXME: pick either tuple or Position and use that everywhere.

#![allow(dead_code)] // FIXME

use std::fmt::{Display, Write};
use std::iter;
use std::str::FromStr;

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

    /// Returns the value in cell `(i,j)`, or `None` if `(i,j)` is not in bounds.
    pub fn get(&self, i: i16, j: i16) -> Option<bool> {
        let index = self.index(i, j)?;
        self.raw.get(index).copied()
    }

    /// Sets cell `(i,j)` to `value`.
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

    /// Count cells set to `true` in the Moore's neighborhood of `(i,j)`.
    ///
    /// The grid does *not* wrap around the edges, and `(i,j)` must point to a cell within the
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

    /// Rotate once, clockwise.
    #[must_use]
    pub fn rotate(&self) -> Self {
        let mut grid = Grid {
            height: self.width,
            width: self.height,
            raw: vec![false; self.raw.len()],
        };

        for (i, j, cell) in self.cells() {
            grid.set(j, self.height - i - 1, cell);
        }

        grid
    }

    /// Flip once, horizontally.
    ///
    /// To flip vertically, rotate twice then flip horizontally.
    #[must_use]
    pub fn flip(&self) -> Self {
        let mut grid = Grid {
            raw: vec![false; self.raw.len()],
            ..*self
        };

        for (i, j, cell) in self.cells() {
            grid.set(i, self.width - j - 1, cell);
        }

        grid
    }

    /// Invert all cells.
    #[must_use]
    pub fn invert(&self) -> Self {
        Grid {
            raw: self.raw.iter().map(|&x| !x).collect(),
            ..*self
        }
    }

    /// Overwrite part of `self`, starting at offset `(i, j)`, with cells from `other`.
    pub fn overwrite(&mut self, other: &Self, i: i16, j: i16) {
        for (x, y, cell) in other.cells() {
            self.set(i + x, j + y, cell);
        }
    }

    /// Extract part of `self`.
    #[must_use]
    pub fn extract(&self, i: i16, j: i16, height: i16, width: i16) -> Self {
        let mut grid = Grid::new(height, width);
        for x in 0..height {
            for y in 0..width {
                grid.set(x, y, self.get(i + x, j + y).unwrap());
            }
        }
        grid
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
            if j != 0 {
                f.write_char(' ')?;
            } else if i != 0 {
                f.write_char('\n')?;
            }
            f.write_char(if cell { '1' } else { '0' })?;
        }
        Ok(())
    }
}

impl FromStr for Grid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tmp: Result<Vec<Vec<_>>, _> = s
            .lines()
            .enumerate()
            .map(|(i, line)| {
                line.split(' ')
                    .enumerate()
                    .map(|(j, cell)| match cell {
                        "0" => Ok(false),
                        "1" => Ok(true),
                        _ => Err(format!("could not parse cell `{cell}` at ({i}, {j})")),
                    })
                    .collect()
            })
            .collect();
        Ok(Grid::from_nested_vecs(tmp?))
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

    #[test]
    fn parse_and_display_back() {
        const INITIAL: &str = "\
            0 1 1\n\
            1 1 0\n\
            1 0 0";
        let grid: Grid = INITIAL.parse().unwrap();
        assert_eq!(grid.to_string(), INITIAL);
    }

    #[test]
    fn rotate_once() {
        const INITIAL: &str = "\
            0 1 1 0\n\
            1 1 0 0\n\
            0 0 1 0";
        const ROTATED: &str = "\
            0 1 0\n\
            0 1 1\n\
            1 0 1\n\
            0 0 0";
        let grid: Grid = INITIAL.parse().unwrap();
        assert_eq!(grid.rotate().to_string(), ROTATED);
    }

    #[test]
    fn flip_horizontally_once() {
        const INITIAL: &str = "\
            0 1 1 0\n\
            1 1 0 0\n\
            0 0 1 0";
        const FLIPPED: &str = "\
            0 1 1 0\n\
            0 0 1 1\n\
            0 1 0 0";
        let grid: Grid = INITIAL.parse().unwrap();
        assert_eq!(grid.flip().to_string(), FLIPPED);
    }

    #[test]
    fn invert() {
        const INITIAL: &str = "\
            0 1 1 0\n\
            1 1 0 0\n\
            0 0 1 0";
        const INVERTED: &str = "\
            1 0 0 1\n\
            0 0 1 1\n\
            1 1 0 1";
        let grid: Grid = INITIAL.parse().unwrap();
        assert_eq!(grid.invert().to_string(), INVERTED);
    }

    #[test]
    fn overwrite() {
        const INITIAL: &str = "\
            0 1 1 0\n\
            1 1 0 0\n\
            0 0 1 0";
        const OTHER: &str = "\
            0 1\n\
            1 1";
        const CHANGED: &str = "\
            0 1 1 0\n\
            1 1 0 1\n\
            0 0 1 1";
        let mut grid: Grid = INITIAL.parse().unwrap();
        grid.overwrite(&OTHER.parse().unwrap(), 1, 2);
        assert_eq!(grid.to_string(), CHANGED);
    }

    #[test]
    fn extract() {
        const INITIAL: &str = "\
            0 1 1 0\n\
            1 1 0 0\n\
            0 0 1 0";
        const SUB: &str = "\
            1 1 0\n\
            0 0 1";
        let grid: Grid = INITIAL.parse().unwrap();
        assert_eq!(grid.extract(1, 0, 2, 3).to_string(), SUB);
    }
}
