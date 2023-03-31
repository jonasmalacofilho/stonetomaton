//! Find a path in a 2-D automaton.
//!
//! # High-level overview
//!
//! Finds a path from a source to a destination in an 2-dimensional automaton, moving horizontally
//! or vertically one cell at a time, and only through white cells (also commonly known as "dead"
//! cells).
//!
//! The problem is conceptually equivalent to finding a path in a 3-dimensional lattice that grows
//! in the generation (or time) axis, and where each layer is the new generation of the automaton.
//!
//! The agent moving from the source to the destination is only allowed to step in white/dead
//! cells, and they must move in the generation/time axis in unit (1) steps.
//!
//! # Implementation notes
//!
//! This implementation balances simplicity, optimality of the solution, and space/time efficiency.
//!
//! While the rules of the challenge don't require that the computed path be optimal, computing the
//! shortest path (using a breadth-first search) is desirable because it bounds the generations
//! that must be computed to a finite number (as long a path exists). If instead a simpler
//! depth-first search was used, it could infinitely recurse even if a finite path exists.
//!
//! Additionally, the graph resulting from the 3-dimensional lattice and agent restrictions
//! described in the "High-level overview" has some interesting properties:
//!
//! - the length of any path from the source to some `(generation, position)` cell is always equal
//! to that `generation` (and, thus, it's the shortest path from `(0, source)` to `(generation,
//! position)`);
//! - only cells with `generation == g + 1` can be reached from cells with `generation == g`.
//!
//! Because of these properties, it's sufficient (for correctness and efficiency) to ensure that
//! cells within some `generation` are visited before any cells of the subsequent `generation + 1`
//! generation. In practice, a simple FIFO queue can be used, instead of the priority queue
//! typically used in Dijkstra's algorithm.
//!
//! Finally, this program has a very narrow use case---it's a code challenge---and the only
//! reasonable behavior when handling errors is to print some diagnostic information and terminate
//! the program. So, for simplicity, panicking and `unwrap`/`expect` are liberally used, instead of
//! the more complex error modeling and handling mechanisms that would be typically be found in
//! production code.
//!
//! # Build, test and execute
//!
//! - Run the unit tests: `cargo test`
//! - Build and execute the program: `cargo run --release`
//! - View this (internal) documentation: `cargo doc --open`
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
use crate::position::{Movement, Position};

fn main() {
    const INPUT: &str = include_str!("../input.txt");

    println!("{}", run(INPUT));
}

fn run(input: &str) -> String {
    let (auto, Some(src), Some(dst)) = from_str(input) else { panic!("missing source and/or destination") };

    let height = auto.grid.height();
    let width = auto.grid.width();

    let mut generations = vec![auto];

    // See "Implementation notes" for why a simple FIFO queue is sufficient.
    let mut queue = VecDeque::new();
    let mut parents: Vec<Grid<Option<Movement>>> = vec![Grid::from_default(height, width)];

    queue.push_back((0, src));

    let mut iterations: usize = 0;

    while let Some((gen, pos)) = queue.pop_front() {
        if pos == dst {
            dbg!(gen);

            let (mut gen, mut pos) = (gen, pos);
            let mut movements = vec![];

            while gen > 0 {
                assert!(
                    !generations[gen].green(pos).unwrap(),
                    "path goes through green cell: {pos:?}, generation: {gen}"
                );
                let movement = parents[gen].get(pos.i, pos.j).unwrap().unwrap();
                movements.push(movement);
                pos = pos.previous(movement);
                gen -= 1;
            }

            movements.reverse();
            dbg!(movements.len());

            let mut buf = String::new();

            let mut first = true;
            for m in movements {
                if first {
                    first = false;
                } else {
                    buf.push(' ');
                }
                buf.push(m.into());
            }

            dbg!(iterations);
            return buf;
        }

        if gen + 1 >= generations.len() {
            generations.push(generations[gen].next_generation());
            parents.push(Grid::from_default(height, width));
        }

        let grid = &generations[gen + 1];

        use crate::position::Movement::*;

        for movement in [Up, Down, Left, Right] {
            let next = pos.next(movement);
            if let Some(false) = grid.green(next) {
                let parent = parents[gen + 1].get_mut(next.i, next.j).unwrap();
                if parent.is_none() {
                    queue.push_back((gen + 1, next));
                    *parent = Some(movement);
                }
            }
        }

        iterations += 1;
    }

    unreachable!();
}

#[derive(Debug)]
struct Automaton {
    grid: Grid<bool>,
}

impl Automaton {
    fn green(&self, pos: Position) -> Option<bool> {
        self.grid.get(pos.i, pos.j).copied()
    }

    fn next_generation(&self) -> Self {
        let mut new_gen = Grid::from_default(self.grid.height(), self.grid.width());

        for (i, j, &green) in self.grid.cells() {
            let green_neighbors = self
                .grid
                .moore_neighborhood(i, j)
                .filter(|green| **green)
                .count();

            *new_gen.get_mut(i, j).unwrap() = (green && (4..6).contains(&green_neighbors))
                || (!green && (2..5).contains(&green_neighbors));
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
fn from_str(s: &str) -> (Automaton, Option<Position>, Option<Position>) {
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

        let (initial, _, _) = from_str(INPUT);
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

        let (initial, _, _) = from_str(INPUT);
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

        const GOLDEN: &str = "D U D U D D R R R D R L R R";
        assert_eq!(run(INPUT), GOLDEN);
    }

    #[test]
    fn dont_regress() {
        const INPUT: &str = include_str!("../input.txt");
        // const GOLDEN: &str = "D D R D R D D R R R D R R R R R R D R R R D D R L U R R R R L R D R D R D D D R D D R R D D R R L D L D R D D R D D R U D L D U R R D U R D R R R R R D R D R R U R D R D R D L R D R L D D D D L D D R R R R D U R R R U L D R D R D R D D L D R R R R R L U R U R R U R D D R R L D D D D D D L D D R D D D R D L R R R R R R R R D R L U D R D D U R U R L R L R R R D D R R R R R U R R U U R D D D R R R R R D L L D R R R R D D L R D D D D R D D";
        const GOLDEN: &str = "D U D U D U D U D U D U R R R R R R R R R R R R R R R R R R D D D D D D R D D D R R L D D D D D U D R D R D D R R D D D D L R D D R U R U U D R R D L R R R R D D R D R D R D D U D L U R R R R D R D R R U D L R R D U U D U D R R D R R D D U D R D D L R R R D D R U D R L R R D U R D D R R D D R R R R R L R D D R L D R D D D D D D D R R U R R D D D R U D R R D D R R R R R L R U D R U R R R D R R R D D L L R R R R R R D D U D D D D D R D D";
        let output = run(INPUT);
        assert_eq!(output.len(), GOLDEN.len());
        assert_eq!(output, GOLDEN);
    }
}
