//! Find a path in a 2-D automaton.
//!
//! # High-level overview
//!
//! Finds a path from a source to a destination in an 2-dimensional automaton, moving one cell at a
//! time, either horizontally or vertically, and only passing through cells that are at that time
//! white.
//!
//! The problem is conceptually equivalent to finding a path in a 3-dimensional lattice, that
//! infinitely grows in a generation axis, and where each layer is a generation of the automaton.
//! The agent moving from the source to the destination must move in the generation axis in unit
//! (1) steps.
//!
//! # Implementation notes
//!
//! While the rules of the challenge don't require that the path be optimal, computing the shortest
//! path using a breadth-first search is desirable because it bounds the generations that must be
//! computed to a finite number, as long any path exists. If a simpler depth-first search were
//! used, it could infinitely recurse even when a finite path existed.
//!
//! Additionally, the 3-dimensional graph resulting from the problem, as described in the
//! "High-level overview", has a few interesting properties:
//!
//! 1. Only cells in the subsequent generation can be reached from the cells in the current
//! generation;
//!
//! 2. The length of any path from the source to some `(generation, position)` pair is always
//! equal to that `generation`;
//!
//! 3. If all cells in one generation are visited before any cells in the subsequent generation,
//! then the first path found between `source` and `position` will be optimal, and the generation
//! when that happens is the first when `position` is reachable from `source`.
//!
//! While it's sufficient for correctness to ensure that all cells in one generation are visited
//! before any cells in the subsequent generation (property 3), a FIFO queue can result in  a
//! performance improvement over scanning all cells in each generation to find those that have been
//! reached so far. On the other hand, a min-heap, as used in Dijkstra's algorithm, isn't
//! necessary: the restrictions in movement through the graph (property 1) already ensure that the
//! simple FIFO order results in a priority queue by generation and, consequently, path length
//! (property 2).
//!
//! Finally, this program has a very narrow use case---it's a code challenge---and error messages
//! will only be seen by the author---_Me. Hello there!_---For simplicity, panicking and
//! `unwrap`/`expect` are liberally used, instead of the more complex error modeling and handling
//! mechanisms that would be typically be found in production code.
//!
//! # Build, test and execute
//!
//! This program is written in Rust, and a Rust toolchain in necessary to build it. It has been
//! tested with (stable) Rust 1.68.1.
//!
//! - Run the unit tests: `cargo test`
//! - Build and execute the program: `cargo run --release`
//! - View this documentation in the browser: `cargo doc --open`
//! - For more options, consult the Cargo documentation.
//!
//! ---
//!
//! Copyright 2023 [Jonas Malaco].
//!
//! [Jonas Malaco]: https://github.com/jonasmalacofilho

mod grid;
mod position;

use std::collections::VecDeque;
use std::fmt::{Display, Write};

use crate::grid::Grid;
use crate::position::Movement::{self, *};
use crate::position::Position;

const MAX_GENERATIONS: usize = 100_000;

fn main() {
    const INPUT: &str = include_str!("../input.txt");
    let path = find_path(INPUT);
    println!("{}", path_to_string(&path));
    eprintln!("{} movements", path.len());
}

/// Reads the input and finds a path from  source to destination.
// See "Implementation notes" in the top-level module documentation for more information.
fn find_path(input: &str) -> Vec<Movement> {
    let (initial_state, Some(src), Some(dst)) = parse(input) else {
        panic!("missing source and/or destination")
    };
    let height = initial_state.grid.height();
    let width = initial_state.grid.width();

    let mut automaton = vec![initial_state];
    let mut history = vec![Grid::<Option<Movement>>::new(height, width)];
    let mut to_visit = VecDeque::new();

    to_visit.push_back((0, src));

    while let Some((gen, pos)) = to_visit.pop_front() {
        if pos == dst {
            let (mut gen, mut pos) = (gen, pos);
            let mut path = vec![];

            while gen > 0 {
                assert!(
                    !automaton[gen].green(pos).unwrap(),
                    "path goes through green cell: {pos:?}, generation: {gen}"
                );

                let movement = history[gen].get(pos.i, pos.j).unwrap().unwrap();
                path.push(movement);

                pos = pos.previous(movement);
                gen -= 1;
            }

            path.reverse();
            return path;
        }

        assert!(gen <= MAX_GENERATIONS);

        if gen + 1 >= automaton.len() {
            automaton.push(automaton[gen].next_generation());
            history.push(Grid::new(height, width));
        }

        for movement in [Up, Down, Left, Right] {
            let next = pos.next(movement);
            if let Some(false) = automaton[gen + 1].green(next) {
                let parent = history[gen + 1].get_mut(next.i, next.j).unwrap();
                if parent.is_none() {
                    to_visit.push_back((gen + 1, next));
                    *parent = Some(movement);
                }
            }
        }
    }

    unreachable!();
}

/// Returns the space-separated list of movements as a string.
fn path_to_string(path: &[Movement]) -> String {
    if path.is_empty() {
        return String::new();
    }
    let mut path = path.iter();
    let mut buf = String::new();
    buf.push((*path.next().unwrap()).into());
    for &m in path {
        buf.push(' ');
        buf.push(m.into());
    }
    buf
}

/// The automaton.
#[derive(Debug)]
struct Automaton {
    grid: Grid<bool>,
}

impl Automaton {
    fn green(&self, pos: Position) -> Option<bool> {
        self.grid.get(pos.i, pos.j).copied()
    }

    fn next_generation(&self) -> Self {
        let mut new_gen = Grid::new(self.grid.height(), self.grid.width());

        for (i, j, &green) in self.grid.cells() {
            let green_neighbors = self
                .grid
                .moore_neighborhood(i, j)
                .filter(|green| **green)
                .count();

            *new_gen.get_mut(i, j).unwrap() = (green && (4..=5).contains(&green_neighbors))
                || (!green && (2..=4).contains(&green_neighbors));
        }

        Automaton { grid: new_gen }
    }
}

impl Display for Automaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, j, &green) in self.grid.cells() {
            if j != 0 {
                f.write_char(' ')?;
            } else if i != 0 {
                f.write_char('\n')?;
            }
            f.write_char(if green { '1' } else { '0' })?;
        }
        Ok(())
    }
}

/// Parses the input.
fn parse(s: &str) -> (Automaton, Option<Position>, Option<Position>) {
    let mut src = None;
    let mut dst = None;

    let grid: Vec<Vec<_>> = s
        .lines()
        .enumerate()
        .map(|(i, line)| {
            line.split_whitespace()
                .enumerate()
                .map(|(j, cell)| {
                    let cell = cell
                        .parse::<u8>()
                        .expect("could not parse cell at ({i}, {j})");

                    if cell == 1 {
                        return true;
                    }

                    let pos = Some(Position {
                        i: i.try_into().expect("unexpectedly high row index {i}"),
                        j: j.try_into().expect("unexpectedly high column index {j}"),
                    });

                    if cell == 3 {
                        src = pos;
                    } else if cell == 4 {
                        dst = pos;
                    }

                    false
                })
                .collect()
        })
        .collect();

    let grid = Grid::from_nested_vecs(grid);

    (Automaton { grid }, src, dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_grid_and_display_back() {
        const INPUT: &str = "\
0 0 0 0 0
0 1 0 1 0
0 0 1 0 0
0 0 0 0 0";

        let (initial, _, _) = parse(INPUT);
        assert_eq!(initial.grid.height(), 4);
        assert_eq!(initial.grid.width(), 5);
        assert_eq!(initial.to_string(), INPUT);
    }

    #[test]
    fn one_generation() {
        const INPUT: &str = "\
1 1 1 0
0 1 0 1
0 1 1 0";
        const EXPECTED: &str = "\
0 0 0 1
1 1 0 0
1 0 0 1";

        let (initial, _, _) = parse(INPUT);
        let second = initial.next_generation();
        assert_eq!(second.to_string(), EXPECTED);
    }

    #[test]
    fn shortest_path() {
        const INPUT: &str = "\
3 0 0 1 0 0
0 1 1 0 1 1
0 0 1 1 0 0
0 0 0 0 0 4";
        const GOLDEN_LENGTH: usize = 14;
        const GOLDEN_OUTPUT: &str = "D U D U D D R R R D R L R R";

        let path = find_path(INPUT);
        assert_eq!(path.len(), GOLDEN_LENGTH);

        let output = path_to_string(&path);
        assert_eq!(output, GOLDEN_OUTPUT);
    }

    #[test]
    fn dont_regress_with_challenge_input() {
        const INPUT: &str = include_str!("../input.txt");
        const GOLDEN_LENGTH: usize = 220;
        const GOLDEN_OUTPUT: &str = "D U D U D U D U D U D U R R R R R R R R R R R R R R R R R R D D D D D D R D D D R R L D D D D D U D R D R D D R R D D D D L R D D R U R U U D R R D L R R R R D D R D R D R D D U D L U R R R R D R D R R U D L R R D U U D U D R R D R R D D U D R D D L R R R D D R U D R L R R D U R D D R R D D R R R R R L R D D R L D R D D D D D D D R R U R R D D D R U D R R D D R R R R R L R U D R U R R R D R R R D D L L R R R R R R D D U D D D D D R D D";

        let path = find_path(INPUT);
        assert_eq!(path.len(), GOLDEN_LENGTH);

        let output = path_to_string(&path);
        assert_eq!(output, GOLDEN_OUTPUT);
    }
}
