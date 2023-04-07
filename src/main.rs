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
mod options;
mod position;
mod puzzle;

use std::fmt::Display;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use clap::Parser;

use crate::grid::Grid;
use crate::options::Options;
use crate::position::Movement::{self, *};
use crate::position::Position;

type OptHashMap<K, V> = ahash::AHashMap<K, V>;

const PCH: &str = "=> ";
const PST: &str = "   ";
const PPG: &str = "    - ";
const PAL: &str = "(W) - ";

fn main() {
    let options = Options::parse();

    for challenge in options.challenges() {
        eprintln!("{PCH}Solving challenge {challenge}");

        let (input, output) = challenge_paths(challenge);

        eprintln!("{PST}Reading input from {input:?}");
        let input = fs::read_to_string(input).unwrap();
        let mut automaton = parse_allow_indeterminate(&input, (2300..2310, 2300..2310));
        if challenge != 0 {
            automaton.immutable_endpoints = true;
        }

        if challenge == 4 {
            eprintln!("{PST}Applying puzzle solution");
            let inner: Grid = puzzle::SOLUTION.parse().unwrap();
            automaton.grid.overwrite(&inner, 2300, 2300);
            fs::write("/tmp/tmp.txt", format!("{automaton}\n")).unwrap();
        }

        eprintln!("{PST}Path finding");
        let path = match find_path(
            automaton.clone(),
            options.max_generations,
            options.max_pessimism,
        ) {
            Ok(path) => {
                eprintln!("{PST}Path found: {} movements", path.len());
                if options.check {
                    eprintln!("{PST}Checking");
                    let lives_lost = lives_lost(&path, automaton);
                    assert_eq!(lives_lost, 0); // FIXME
                    eprintln!("{PST}Passed: loses {} lives", lives_lost);
                }
                path
            }
            Err(best_attempt) => {
                eprintln!("{PST}Best attempt found: {} movements", best_attempt.len());
                best_attempt
            }
        };

        eprintln!("{PST}Saving output to {output:?}");
        let mut path = path_to_string(&path);
        if challenge == 5 {
            // FIXME: HACK.
            path.insert_str(0, "0 ");
        }
        path.push('\n');
        fs::write(output, path).unwrap();

        eprintln!();
    }
}

fn challenge_paths(number: u8) -> (PathBuf, PathBuf) {
    let suffix = if number == 0 {
        "".into()
    } else {
        number.to_string()
    };
    let input = format!("input{suffix}.txt");
    let output = format!("output{suffix}.txt");
    (input.into(), output.into())
}

/// Reads the input and finds a path from  source to destination.
// See "Implementation notes" in the top-level module documentation for more information.
fn find_path(
    mut automaton: Automaton,
    max_generations: usize,
    max_pessimism: u16,
) -> Result<Vec<Movement>, Vec<Movement>> {
    let source = automaton.source;
    let destination = automaton.destination;

    let mut history = vec![OptHashMap::<Position, Movement>::default()];

    // HACK: avoids special casing gen 0.
    history[0].insert(source, Down);

    let mut best_pos = automaton.source;
    let mut best_dist = best_pos.distance(&automaton.destination);
    let mut best_gen = 0;

    for gen in 0..max_generations {
        if gen != 0 && gen % 100 == 0 {
            eprintln!("{PPG}gen={gen} best_pos={best_pos:?} best_dist={best_dist}");
        }

        let next_generation = automaton.next_generation();
        let mut next_history =
            OptHashMap::with_capacity_and_hasher(history[gen].capacity(), Default::default());

        for &pos in history[gen].keys() {
            if pos == destination {
                return Ok(assemble_path(&history, gen, pos));
            }

            for movement in [Up, Down, Left, Right] {
                let next = pos.next(movement);
                if let Some(false) = next_generation.green(next) {
                    let dist = next.distance(&automaton.destination);
                    if dist < best_dist {
                        best_pos = next;
                        best_dist = dist;
                        best_gen = gen + 1;
                    } else if dist > best_dist + max_pessimism {
                        // HACK: Bound how much history we keep, and consequently how much memory
                        // we need, by only considering moves near our current best position.
                        continue;
                    }
                    next_history.entry(next).or_insert(movement);
                }
            }
        }

        if next_history.is_empty() {
            eprintln!("{PAL}No movement in gen={gen}");
            break;
        }

        automaton = next_generation;
        history.push(next_history);
    }

    eprintln!("{PAL}Failed to reach the destination (distance={best_dist})");
    assert!(best_gen > 0);
    Err(assemble_path(&history, best_gen, best_pos))
}

fn assemble_path(
    history: &[OptHashMap<Position, Movement>],
    mut gen: usize,
    mut pos: Position,
) -> Vec<Movement> {
    let mut path = vec![];
    while gen > 0 {
        let movement = history[gen][&pos];
        path.push(movement);
        pos = pos.previous(movement);
        gen -= 1;
    }
    path.reverse();
    path
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
#[derive(Debug, Clone)]
pub struct Automaton {
    grid: Grid,
    source: Position,
    destination: Position,

    pub immutable_endpoints: bool,
}

impl Automaton {
    fn green(&self, pos: Position) -> Option<bool> {
        self.grid.get(pos.i, pos.j)
    }

    fn next_generation(&self) -> Self {
        let mut new_gen = Grid::new(self.grid.height(), self.grid.width());

        for (i, j, green) in self.grid.cells() {
            let green_neighbors = self.grid.count_neighbors(i, j);

            let new_cell = if green {
                (4..=5).contains(&green_neighbors)
            } else {
                (2..=4).contains(&green_neighbors)
            };

            new_gen.set(i, j, new_cell);
        }

        if self.immutable_endpoints {
            new_gen.set(self.source.i, self.source.j, false);
            new_gen.set(self.destination.i, self.destination.j, false);
        }

        Automaton {
            grid: new_gen,
            ..*self
        }
    }
}

impl Display for Automaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.grid.fmt(f)
    }
}

fn parse_allow_indeterminate(s: &str, (xi, xj): (Range<i16>, Range<i16>)) -> Automaton {
    let mut source = None;
    let mut destination = None;

    let tmp: Vec<Vec<_>> = s
        .lines()
        .enumerate()
        .map(|(i, line)| {
            line.split(' ')
                .enumerate()
                .map(|(j, cell)| {
                    let pos = Position {
                        i: i.try_into()
                            .unwrap_or_else(|_| panic!("unexpectedly high row index {i}")),
                        j: j.try_into()
                            .unwrap_or_else(|_| panic!("unexpectedly high column index {j}")),
                    };

                    match cell {
                        "0" => false,
                        "1" => true,
                        "3" => {
                            source = Some(pos);
                            false
                        }
                        "4" => {
                            destination = Some(pos);
                            false
                        }
                        "x" => {
                            assert!(xi.contains(&pos.i) && xj.contains(&pos.j));
                            false
                        }
                        _ => panic!("could not parse cell `{cell}` at ({i}, {j})"),
                    }
                })
                .collect()
        })
        .collect();
    let grid = Grid::from_nested_vecs(tmp);

    Automaton {
        grid,
        source: source.expect("missing source"),
        destination: destination.expect("missing destination"),
        immutable_endpoints: false,
    }
}

fn lives_lost(path: &[Movement], mut automaton: Automaton) -> usize {
    let mut current = automaton.source;
    let mut lost = 0;

    for (gen, movement) in path.iter().enumerate() {
        if automaton.green(current).unwrap() {
            lost += 1;

            // Leave this here in case we need to debug a bug.
            // dbg!(lost, gen, movement, current);

            // However: initial and final positions cannot be green.
            assert_ne!(gen, 0);
            assert_ne!(gen, path.len() - 1);
        }

        automaton = automaton.next_generation();
        current = current.next(*movement);
    }

    // Check that the destination is actually correct.
    assert_eq!(current, automaton.destination);

    lost
}

#[cfg(test)]
mod main_tests {
    use std::fs;

    use super::*;

    const MAX_GENERATIONS: usize = 50_000;
    const MAX_PESSIMISM: u16 = 50;

    fn parse(s: &str) -> Automaton {
        parse_allow_indeterminate(s, (0..0, 0..0))
    }

    fn validade_path_format(path: &str) {
        for (gen, movement) in path.split(' ').enumerate() {
            // Leave this here in case we need to debug a bug.
            dbg!(gen, movement);

            assert!(["D", "U", "R", "L"].contains(&movement));
        }
    }

    #[test]
    fn parse_grid_and_display_back() {
        const INPUT: &str = "\
            0 0 0 0 0\n\
            0 1 0 1 4\n\
            3 0 1 0 0\n\
            0 0 0 0 0";

        let initial = parse(INPUT);
        assert_eq!(initial.grid.height(), 4);
        assert_eq!(initial.grid.width(), 5);
        assert_eq!(initial.source, Position { i: 2, j: 0 });
        assert_eq!(initial.destination, Position { i: 1, j: 4 });
        assert_eq!(initial.to_string(), INPUT.replace(['3', '4'], "0"));
    }

    #[test]
    fn one_generation() {
        const INPUT: &str = "\
            1 1 1 0\n\
            3 1 4 1\n\
            0 1 1 0";
        const EXPECTED: &str = "\
            0 0 0 1\n\
            1 1 0 0\n\
            1 0 0 1";

        let initial = parse(INPUT);
        let second = initial.next_generation();
        assert_eq!(second.to_string(), EXPECTED);
    }

    #[test]
    fn shortest_path() {
        const INPUT: &str = "\
            3 0 0 1 0 0\n\
            0 1 1 0 1 1\n\
            0 0 1 1 0 0\n\
            0 0 0 0 0 4";
        const GOLDEN_LENGTH: usize = 14;
        // const GOLDEN_OUTPUT1: &str = "D U D U D D R R R D R L R R";
        // const GOLDEN_OUTPUT2: &str = "D U D U D D R R R R D L R R";

        let automaton = parse(INPUT);
        let path = find_path(automaton.clone(), MAX_GENERATIONS, MAX_PESSIMISM).unwrap();

        assert_eq!(lives_lost(&path, automaton), 0);
        assert_eq!(path.len(), GOLDEN_LENGTH);

        validade_path_format(&path_to_string(&path));
    }

    mod dont_regress {
        use super::*;

        mod level1 {
            use super::*;

            #[test]
            fn question2() {
                const GOLDEN_LENGTH: usize = 220;
                // const GOLDEN_OUTPUT: &str = "D U D U D U D U D U D U R R R R R R R R R R R R R R R R R R D D D D D D R D D D R R L D D D D D U D R D R D D R R D D D D L R D D R U R U U D R R D L R R R R D D R D R D R D D U D L U R R R R D R D R R U D L R R D U U D U D R R D R R D D U D R D D L R R R D D R U D R L R R D U R D D R R D D R R R R R L R D D R L D R D D D D D D D R R U R R D D D R U D R R D D R R R R R L R U D R U R R R D R R R D D L L R R R R R R D D U D D D D D R D D";

                let input = fs::read_to_string("input.txt").unwrap();
                let automaton = parse(&input);

                let path = find_path(automaton.clone(), MAX_GENERATIONS, MAX_PESSIMISM).unwrap();
                assert_eq!(lives_lost(&path, automaton), 0);
                assert_eq!(path.len(), GOLDEN_LENGTH);

                validade_path_format(&path_to_string(&path));
            }
        }

        mod level2 {
            use super::*;

            #[test]
            #[ignore]
            fn challenge1() {
                const GOLDEN_LENGTH: usize = 6176; // score ==  1000

                let input = fs::read_to_string("input1.txt").unwrap();
                let mut automaton = parse(&input);
                automaton.immutable_endpoints = true;

                let path = find_path(automaton.clone(), MAX_GENERATIONS, MAX_PESSIMISM).unwrap();
                assert_eq!(lives_lost(&path, automaton), 0);
                assert_eq!(path.len(), GOLDEN_LENGTH);

                let mut path = path_to_string(&path);
                validade_path_format(&path);

                path.push('\n');
                fs::write("output1.txt.test", path).unwrap();
            }

            #[test]
            #[ignore]
            fn challenge2() {
                const GOLDEN_LENGTH: usize = 6264; // FIXME: suboptimal, unchecked, ~810 s, ~21 GiB.
                                                   // score >= 957

                let input = fs::read_to_string("input2.txt").unwrap();
                let mut automaton = parse(&input);
                automaton.immutable_endpoints = true;

                let path = find_path(automaton.clone(), MAX_GENERATIONS, MAX_PESSIMISM).unwrap();
                assert!(dbg!(lives_lost(&path, automaton)) <= 5);
                assert!(dbg!(path.len()) <= GOLDEN_LENGTH);

                let mut path = path_to_string(&path);
                validade_path_format(&path);

                path.push('\n');
                fs::write("output2.txt.test", path).unwrap();
            }

            #[test]
            #[ignore]
            fn challenge3() {
                const GOLDEN_LENGTH: usize = 6200; // FIXME: suboptimal, unchecked, ~820 s, ~17? GiB.
                                                   // score >= 967

                let input = fs::read_to_string("input3.txt").unwrap();
                let mut automaton = parse(&input);
                automaton.immutable_endpoints = true;

                let path = find_path(automaton.clone(), MAX_GENERATIONS, MAX_PESSIMISM).unwrap();
                assert_eq!(lives_lost(&path, automaton), 0);
                assert!(dbg!(path.len()) <= GOLDEN_LENGTH);

                let mut path = path_to_string(&path);
                validade_path_format(&path);

                path.push('\n');
                fs::write("output3.txt.test", path).unwrap();
            }
        }
    }
}
