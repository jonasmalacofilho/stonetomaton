use std::fmt::{Display, Write};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
}

impl Movement {
    fn to_inc(self) -> (i8, i8) {
        // The automaton origin is at the top left, and the row indices grown *down.*
        match self {
            Movement::Up => (-1, 0),
            Movement::Down => (1, 0),
            Movement::Left => (0, -1),
            Movement::Right => (0, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub i: i8,
    pub j: i8,
}

impl Position {
    pub fn previous(self, movement: Movement) -> Self {
        let inc = movement.to_inc();
        Position {
            i: self.i - inc.0,
            j: self.j - inc.1,
        }
    }

    pub fn next(self, movement: Movement) -> Self {
        let inc = movement.to_inc();
        Position {
            i: self.i + inc.0,
            j: self.j + inc.1,
        }
    }
}

impl Display for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char((*self).into())
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
