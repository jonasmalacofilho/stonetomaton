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

use std::fmt::Display;
use std::hint::black_box;
use std::io::{self, Read};
use std::time::{Duration, Instant};

use clap::Parser;

use crate::grid::Grid;
use crate::options::Options;
use crate::position::Movement::{self, *};
use crate::position::Position;

type OptHashMap<K, V> = ahash::AHashMap<K, V>;

fn main() {
    let options = Options::parse();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut automaton = parse(&input);
    if options.immutable_endpoints {
        automaton.immutable_endpoints = true;
    }

    if options.bench_automaton {
        const ITERATIONS: usize = 150;
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            automaton = automaton.next_generation();
        }
        black_box(&automaton);
        let throughput_value = start.elapsed().div_f64(ITERATIONS as f64);
        let throughput = 1. / throughput_value.as_secs_f64();
        let estimate_6200 = Duration::from_secs_f64(6200. / throughput);
        dbg!(throughput_value, throughput, estimate_6200);
        return;
    }

    let path = find_path(
        automaton.clone(),
        options.max_generations,
        options.max_pessimism,
    )
    .unwrap();

    println!("{}", path_to_string(&path));
    eprintln!("{} movements", path.len());

    if options.check {
        let lives_lost = lives_lost(&path, automaton);
        eprintln!("{} lives lost", lives_lost);
        assert_eq!(lives_lost, 0);
    }
}

/// Reads the input and finds a path from  source to destination.
// See "Implementation notes" in the top-level module documentation for more information.
fn find_path(
    mut automaton: Automaton,
    max_generations: usize,
    max_pessimism: u16,
) -> Option<Vec<Movement>> {
    let source = automaton.source;
    let destination = automaton.destination;

    // Potential optimizations:
    // - A* might find the shortest path sooner and (by changing `history` to be some sparse map)
    // with a smaller history requirement.
    //
    // Performed optimizations (assuming they are implemented correctly):
    // - only the next generation of the automaton is strickly necessary;
    // - `history` can be stored sparsely;
    // - instead of a queue, it's possible to just scan the current generation (but the queue size
    // is bounded by `2 * R * C`);

    let mut history = vec![OptHashMap::<Position, Movement>::default()];

    let mut best_pos = automaton.source;

    for gen in 0..max_generations {
        if gen > 0 && gen % 100 == 0 {
            dbg!(gen, best_pos, history[gen].len(), history[gen].capacity());
        }

        let next_generation = automaton.next_generation();
        history.push(OptHashMap::with_capacity_and_hasher(
            history[gen].capacity(),
            Default::default(),
        ));

        for (i, j, cell) in automaton.grid.cells() {
            let pos = Position { i, j };

            if cell
                || (gen > 0 && history[gen].get(&pos).is_none())
                || (gen == 0 && Position { i, j } != source)
            {
                continue;
            }

            let pos_dist = pos.distance(&automaton.destination);
            let best_dist = best_pos.distance(&automaton.destination);
            if pos_dist < best_dist {
                best_pos = pos;
            } else if pos_dist > best_dist + max_pessimism {
                // HACK: Bound the how much history we keep and, consequently, how much memory we
                // use, by only considering moves closer to the destination. In a way this is a
                // poor program's version of an A* algorithm.
                // FIXME: should compare against last gen's best dist, not the best dist being
                // updated in this generation.
                // FIXME: replace with an actual A*.
                continue;
            }

            // eprintln!("forward: {gen} {i} {j}");
            // dbg!(&history[gen]);
            // dbg!(cell);
            // dbg!(gen > 0 && history[gen].get(i, j).is_none());
            // dbg!(gen == 0 && Position { i, j } != source);

            if pos == destination {
                let (mut gen, mut pos) = (gen, pos);
                let mut path = vec![];

                while gen > 0 {
                    // eprintln!("backward: {gen} {} {}", pos.i, pos.j);

                    let movement = history[gen][&pos];
                    path.push(movement);

                    pos = pos.previous(movement);
                    gen -= 1;
                }

                path.reverse();
                return Some(path);
            }

            for movement in [Up, Down, Left, Right] {
                let next = pos.next(movement);
                if let Some(false) = next_generation.green(next) {
                    history[gen + 1].entry(next).or_insert(movement);
                }
            }
        }

        automaton = next_generation;
    }

    None
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
struct Automaton {
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

/// Parses the input.
fn parse(s: &str) -> Automaton {
    let mut source = None;
    let mut destination = None;

    let grid: Vec<Vec<_>> = s
        .lines()
        .enumerate()
        .map(|(i, line)| {
            line.split_whitespace()
                .enumerate()
                .map(|(j, cell)| {
                    let cell = cell
                        .parse::<u8>()
                        .unwrap_or_else(|_| panic!("could not parse cell `{cell}` at ({i}, {j})"));

                    if cell == 1 {
                        return true;
                    }

                    let pos = Some(Position {
                        i: i.try_into()
                            .unwrap_or_else(|_| panic!("unexpectedly high row index {i}")),
                        j: j.try_into()
                            .unwrap_or_else(|_| panic!("unexpectedly high column index {j}")),
                    });

                    if cell == 3 {
                        source = pos;
                    } else if cell == 4 {
                        destination = pos;
                    }

                    false
                })
                .collect()
        })
        .collect();

    let grid = Grid::from_nested_vecs(grid);

    Automaton {
        grid,
        source: source.unwrap(),
        destination: destination.unwrap(),
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
3 0 0 1 0 0
0 1 1 0 1 1
0 0 1 1 0 0
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
                fs::write("output1.txt.candidate", path).unwrap();
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
                fs::write("output2.txt.candidate", path).unwrap();
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
                fs::write("output3.txt.candidate", path).unwrap();
            }
        }
    }
}
