use std::iter;
use std::ops::Index;

use bit_vec::BitVec;

use crate::position::Position;

#[derive(Debug, Clone)]
pub struct BitGrid {
    height: i16,
    width: i16,
    raw: BitVec,
}

impl BitGrid {
    pub fn new(height: i16, width: i16) -> Self {
        let (h, w): (usize, usize) = (height.try_into().unwrap(), width.try_into().unwrap());
        let raw = BitVec::from_elem(h * w, false);
        Self { height, width, raw }
    }

    #[inline]
    pub fn with_dim_from(other: &BitGrid) -> Self {
        let raw = BitVec::from_elem(other.raw.len(), false);
        Self { raw, ..*other }
    }

    #[inline]
    pub fn insert(&mut self, position: Position) {
        let offset = self.offset(position).unwrap();
        self.raw.set(offset, true);
    }

    #[inline]
    pub fn contains(&self, position: Position) -> bool {
        if let Some(offset) = self.offset(position) {
            self.raw[offset]
        } else {
            false
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.none()
    }

    pub fn iter(&self) -> impl Iterator<Item = Position> + '_ {
        let mut pos = Position { i: 0, j: 0 };
        iter::from_fn(move || {
            while let Some(offset) = self.offset(pos) {
                let cur = pos;
                pos.j += 1;
                if pos.j >= self.width {
                    pos.j = 0;
                    pos.i += 1;
                }
                if self.raw[offset] {
                    return Some(cur);
                }
            }
            None
        })
    }

    #[inline]
    fn offset(&self, position: Position) -> Option<usize> {
        if !(0..self.height).contains(&position.i) || !(0..self.width).contains(&position.j) {
            return None;
        }
        let (i, j, w): (usize, usize, usize) = (position.i as _, position.j as _, self.width as _);
        Some(i * w + j)
    }
}

impl Index<Position> for BitGrid {
    type Output = bool;

    #[inline]
    fn index(&self, index: Position) -> &Self::Output {
        &self.raw[self.offset(index).unwrap()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut grid = BitGrid::new(5, 10);

        assert_eq!(grid.raw, BitGrid::with_dim_from(&grid).raw);

        assert_eq!(grid.iter().collect::<Vec<_>>(), vec![]);
        assert!(grid.raw.none());

        grid.insert(Position { i: 4, j: 9 });
        assert!(grid.contains(Position { i: 4, j: 9 }));
        assert_eq!(
            grid.iter().collect::<Vec<_>>(),
            vec![Position { i: 4, j: 9 }]
        );
        assert!(grid.raw.any());
        assert!(!grid.raw.all());
    }
}
