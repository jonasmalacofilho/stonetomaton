//! Position and movement in a 2-dimensional grid.

/// A position in a 2-dimensional grid.
///
/// The `i` and `j` coordinates are signed integers, making it easier to deal with movements around
/// `0`, which can result in negative coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// Row index.
    pub i: i8,
    /// Column index.
    pub j: i8,
}

impl Position {
    /// Returns the new position after a `movement` from `self`.
    #[must_use]
    pub fn next(self, movement: Movement) -> Self {
        let inc = movement.to_tuple();
        Position {
            i: self.i + inc.0,
            j: self.j + inc.1,
        }
    }

    /// The position from which a `movement` would land on `self`.
    #[must_use]
    pub fn previous(self, movement: Movement) -> Self {
        let inc = movement.to_tuple();
        Position {
            i: self.i - inc.0,
            j: self.j - inc.1,
        }
    }
}

/// An individual movement of the agent.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
}

impl Movement {
    fn to_tuple(self) -> (i8, i8) {
        // The origin is at the top left, and row indices grown down.
        match self {
            Movement::Up => (-1, 0),
            Movement::Down => (1, 0),
            Movement::Left => (0, -1),
            Movement::Right => (0, 1),
        }
    }
}

impl From<Movement> for char {
    fn from(value: Movement) -> Self {
        match value {
            Movement::Up => 'U',
            Movement::Down => 'D',
            Movement::Left => 'L',
            Movement::Right => 'R',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_down_and_left() {
        let p0 = Position { i: 3, j: 5 };
        let p1 = p0.next(Movement::Down);
        let p2 = p1.next(Movement::Left);
        assert_eq!(p2, Position { i: 4, j: 4 });
    }

    #[test]
    fn position_before_movement() {
        let p0 = Position { i: 3, j: 5 };
        let p1 = p0.next(Movement::Down);
        let p2 = p1.next(Movement::Left);
        assert_eq!(p2.previous(Movement::Left), p1);
        assert_eq!(p1.previous(Movement::Down), p0);
    }
}
