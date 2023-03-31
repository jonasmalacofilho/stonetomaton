//! A 2-dimensional grid.

/// A 2-dimensional grid of `T` values.
#[derive(Debug, Clone)]
pub struct Grid<T> {
    height: i8,
    width: i8,
    raw: Vec<T>,
}

impl<T> Grid<T> {
    pub fn from_nested_vecs(vecs: Vec<Vec<T>>) -> Self {
        let (h, w) = (vecs.len(), vecs[0].len());
        let (height, width): (i8, i8) = (h.try_into().unwrap(), w.try_into().unwrap());

        let mut raw = Vec::with_capacity(h * w);
        for row in vecs {
            assert_eq!(row.len(), w);
            raw.extend(row);
        }

        Self { height, width, raw }
    }

    pub fn height(&self) -> i8 {
        self.height
    }

    pub fn width(&self) -> i8 {
        self.width
    }

    pub fn get(&self, i: i8, j: i8) -> Option<&T> {
        self.raw.get(self.index(i, j)?)
    }

    pub fn get_mut(&mut self, i: i8, j: i8) -> Option<&mut T> {
        let index = self.index(i, j)?;
        self.raw.get_mut(index)
    }

    pub fn cells(&self) -> impl Iterator<Item = (i8, i8, &T)> {
        let w: usize = self.width as _;

        self.raw
            .iter()
            .enumerate()
            .map(move |(index, value)| ((index / w) as i8, (index % w) as i8, value))
    }

    pub fn moore_neighborhood(&self, i: i8, j: i8) -> impl Iterator<Item = &T> {
        // TODO: return an empty iterator instead... without penalizing performance.
        assert!(self.index(i, j).is_some(), "`(i, j)` should be in bounds");

        ((i - 1)..=(i + 1)).flat_map(move |x| {
            ((j - 1)..=(j + 1)).filter_map(move |y| {
                if (x, y) == (i, j) {
                    None
                } else {
                    self.get(x, y)
                }
            })
        })
    }

    fn index(&self, i: i8, j: i8) -> Option<usize> {
        if i < 0 || i >= self.height || j < 0 || j >= self.width {
            return None;
        }
        let (i, j, w): (usize, usize, usize) = (i as _, j as _, self.width as _);
        Some(i * w + j)
    }
}

impl<T: Default + Clone> Grid<T> {
    pub fn from_default(height: i8, width: i8) -> Self {
        let (h, w): (usize, usize) = (height.try_into().unwrap(), width.try_into().unwrap());
        let raw = vec![Default::default(); h * w];
        Self { height, width, raw }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_shared_or_mut_references() {
        //     Grid
        // +-----------+
        // | 0 1 2 3 4 |
        // | 5 6 7 8 9 |
        // +-----------+

        let vecs = vec![vec![0, 1, 2, 3, 4], vec![5, 6, 7, 8, 9]];
        let mut grid = Grid::from_nested_vecs(vecs);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.width(), 5);

        assert_eq!(grid.get(1, 3), Some(&8));
        assert_eq!(grid.get(2, 3), None);
        assert_eq!(grid.get(-1, 3), None);
        assert_eq!(grid.get(1, 5), None);
        assert_eq!(grid.get(1, -1), None);

        assert_eq!(grid.get_mut(1, 3), Some(&mut 8));
        assert_eq!(grid.get_mut(2, 3), None);
        assert_eq!(grid.get_mut(-1, 3), None);
        assert_eq!(grid.get_mut(1, 5), None);
        assert_eq!(grid.get_mut(1, -1), None);
    }

    #[test]
    fn iterate_through_all_cells() {
        //   Grid
        // +-------+
        // | 0 1 2 |
        // | 3 4 5 |
        // +-------+

        let vecs = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let grid = Grid::from_nested_vecs(vecs);

        assert_eq!(
            grid.cells().collect::<Vec<_>>(),
            vec![
                (0, 0, &0),
                (0, 1, &1),
                (0, 2, &2),
                (1, 0, &3),
                (1, 1, &4),
                (1, 2, &5)
            ]
        );
    }

    #[test]
    fn iterate_through_the_moore_neighborhood() {
        //    Grid           Moore neighborhoods
        //  +-------+      +-------+      +-------+
        //  | 0 1 2 |      | 0 1 2 |      |       |
        //  | 3 4 5 |      | 3 * 5 |      |   4 5 |
        //  | 6 7 8 |      | 6 7 8 |      |   7 * |
        //  +-------+      +-------+      +-------+
        //                (i,j)=(1,1)    (i,j)=(2,2)

        let vecs = vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
        let grid = Grid::from_nested_vecs(vecs);

        assert_eq!(
            grid.moore_neighborhood(1, 1).collect::<Vec<_>>(),
            vec![&0, &1, &2, &3, &5, &6, &7, &8]
        );

        assert_eq!(
            grid.moore_neighborhood(2, 2).collect::<Vec<_>>(),
            vec![(&4), (&5), (&7),]
        );
    }

    #[test]
    #[should_panic]
    fn neighborhood_requires_cell_to_be_in_bounds() {
        //    Grid
        // +-------+
        // | 0 1 2 |
        // | 3 4 5 |
        // +-------+
        //           *
        //      (i,j)=(2,3)

        let vecs = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let grid = Grid::from_nested_vecs(vecs);

        let _should_panic = grid.moore_neighborhood(2, 3);
    }
}
