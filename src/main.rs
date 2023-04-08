//! Find paths in 2-D-automaton mazes.
//!
//! _Partial solution of the [Stone Automata Maze Challenge]._
//!
//! # High-level overview
//!
//! This program finds paths in 2-dimensional automata, moving horizontally or vertically one cell
//! at a time, and only passing through dead (white) cells.
//!
//! The problem is conceptually equivalent to finding paths in 3-dimensional lattices that grow
//! infinitely in the generation axis, and where each layer is one generation of the corresponding
//! automaton.
//!
//! The agent (particle) going from the source to the destination must move in the generation
//! axis in unit (1) steps, and must do so at every generation.
//!
//! # Implementation notes
//!
//! In short, it's quite messy, full of incomplete and inconsistent APIs, and with a bunch of hacks
//! (or non-general heuristics, if you prefer to think of them like that).
//!
//! That said, the 3-dimensional lattice described in the "High-level overview" has a few
//! interesting properties, which are leveraged in this program in several places.
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
//! As error messages will only be seen by the author---_me... hello there!_---panicking is
//! liberally used, instead of the more complex error modeling and handling mechanisms that would
//! be typically be found in production code.
//!
//! # Challenge notes
//!
//! This program will by default (try to) solve all challenges. See `--help` for options that,
//! among other things, allow solving only one specific challenge.
//!
//! ## Challenge 1
//!
//! Finds an optimal (shortest) path from source to destination.
//!
//! ## Challenges 2 and 3
//!
//! Finds a path from source to destination. Extra lives and individuality are currently *not*
//! used. The path found is therefore expected to be suboptimal, but, in practice, it's within ~50%
//! of the orthogonal distance (and, therefore, from the best possible path).
//!
//! ## Challenge 4
//!
//! As the grid is larger, the algorithm used in the previous challenges had to be changed to
//! optimize for memory usage, instead of CPU time. And somewhere along the way I must also have
//! done something wrong, since it can't find any paths (at least at the time of writing this)...
//!
//! The puzzle was another issue: while it was reasonably simple to solve using a general *natural*
//! intelligence (me), an algorithmic solution for it is still to-do.
//!
//! ## Challenge 5
//!
//! I have some ideas, but none are particular advanced, and I wont have time to implement them. So
//! just compute one path and get one particle on the grid.
//!
//! ## Challenge 0 (extra)
//!
//! This is kept from the previous level (1), question 2. In practice, it's a lot faster and,
//! therefore, very useful to quickly check that changes don't regress behavior or performance.
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
//! [Stone Automata Maze Challenge]: https://sigmageek.com/challenge_results/stone-automata-maze-challenge

mod bitgrid;
mod grid;
mod options;
mod position;
mod puzzle;

use std::fmt::Display;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;

use crate::bitgrid::BitGrid;
use crate::grid::Grid;
use crate::options::{Algorithm::*, Options};
use crate::position::Movement::{self, *};
use crate::position::Position;

type OptHashMap<K, V> = ahash::AHashMap<K, V>;

const CHAL: &str = "=> ";
const STEP: &str = "   ";
const INFO: &str = "    - ";
const WARN: &str = "(W) - ";

fn main() {
    let options = Options::parse();

    for challenge in options.challenges() {
        eprintln!("{CHAL}Solving challenge {challenge}");

        let (input, output) = challenge_paths(challenge);

        eprintln!("{STEP}Reading input from {input:?}");
        let input = fs::read_to_string(input).unwrap();
        let mut automaton = parse_allow_indeterminate(&input, (2300..2310, 2300..2310));
        if challenge != 0 {
            automaton.immutable_endpoints = true;
        }
        eprintln!(
            "{INFO}immutable_endpoints={}",
            automaton.immutable_endpoints
        );

        if challenge == 4 {
            eprintln!("{STEP}Applying puzzle solution");
            let inner: Grid = puzzle::SOLUTION.parse().unwrap();
            automaton.grid.overwrite(&inner, 2300, 2300);
        }

        eprintln!("{STEP}Path finding");
        let instant = Instant::now();
        let path = match (&options.algorithm, challenge) {
            (Some(Heuristic), _) | (None, 0..=3 | 5) => {
                eprintln!("{INFO}Using heuristic algorithm");
                find_path(
                    automaton.clone(),
                    options.max_generations,
                    options.max_pessimism,
                )
            }
            (Some(Robust), _) | (None, _) => {
                eprintln!("{INFO}Using robust algorithm");
                find_path_robust(automaton.clone(), options.max_generations)
            }
        };
        eprintln!("{INFO}elapsed={:?}", instant.elapsed());

        let path = match path {
            Ok(path) => {
                eprintln!("{STEP}Path found: {} movements", path.len());
                if options.check {
                    eprintln!("{STEP}Checking");
                    let instant = Instant::now();
                    let lives_lost = lives_lost(&path, automaton);
                    assert_eq!(lives_lost, 0); // FIXME
                    eprintln!("{INFO}elapsed={:?}", instant.elapsed());
                    eprintln!("{STEP}Passed: loses {} lives", lives_lost);
                }
                path
            }
            Err(best_attempt) => {
                eprintln!("{STEP}Best attempt found: {} movements", best_attempt.len());
                best_attempt
            }
        };

        eprintln!("{STEP}Saving output to {output:?}");
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

/// Finds a path from a source to a destination.
///
/// The `max_pessimism` (roughly equivalent to how much backtracking is allowed) can result in huge
/// savings in space and time, in comparison to [`find_path_robust`].
///
/// On the other hand, the path returned may be suboptimal or, in some cases, no path might be
/// found unless `max_pessimism` is set to really high values, which in turn results in excessive
/// (instead of reduced) memory usage.
pub fn find_path(
    mut automaton: Automaton,
    max_generations: usize,
    max_pessimism: u16,
) -> Result<Vec<Movement>, Vec<Movement>> {
    eprintln!("{INFO}max_generations={max_generations} max_pessimism={max_pessimism}");

    let source = automaton.source;
    let destination = automaton.destination;

    let mut reached = vec![OptHashMap::<Position, Movement>::default()];

    // HACK: avoids special casing gen 0.
    reached[0].insert(source, Down);

    let mut best_pos = automaton.source;
    let mut best_dist = best_pos.distance(&automaton.destination);
    let mut best_gen = 0;

    for gen in 0..max_generations {
        if gen != 0 && gen % 100 == 0 {
            eprintln!("{INFO}gen={gen} best_pos={best_pos:?} best_dist={best_dist}");
        }

        let next_generation = automaton.next_generation();
        let mut to_visit =
            OptHashMap::with_capacity_and_hasher(reached[gen].capacity(), Default::default());

        for &pos in reached[gen].keys() {
            debug_assert_eq!(automaton.alive(pos), Some(false));

            if pos == destination {
                return Ok(assemble_path(&reached, gen, pos));
            }

            for movement in [Up, Down, Left, Right] {
                let next = pos.next(movement);
                if let Some(false) = next_generation.alive(next) {
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
                    to_visit.entry(next).or_insert(movement);
                }
            }
        }

        if to_visit.is_empty() {
            eprintln!("{WARN}No movement in gen={gen}");
            break;
        }

        automaton = next_generation;
        reached.push(to_visit);
    }

    eprintln!("{WARN}Failed to reach the destination (distance={best_dist})");
    assert!(best_gen > 0);
    Err(assemble_path(&reached, best_gen, best_pos))
}

/// Finds a path from a source to a destination, optimized for the worst case.
///
/// Generally slower than [`find_path`], but the memory usage only depend on the number of
/// generations required to reach the destination, which can be benefetial if a lot of backtracking
/// is required to find the (best or only) path.
pub fn find_path_robust(
    mut automaton: Automaton,
    max_generations: usize,
) -> Result<Vec<Movement>, Vec<Movement>> {
    eprintln!("{INFO}max_generations={max_generations} max_pessimism=N/A");

    let source = automaton.source;
    let destination = automaton.destination;

    let mut reached = vec![BitGrid::new(
        automaton.grid.height(),
        automaton.grid.width(),
    )];

    // HACK: avoids special casing gen 0.
    reached[0].insert(source);

    let mut best_pos = automaton.source;
    let mut best_dist = best_pos.distance(&automaton.destination);
    let mut best_gen = 0;

    for gen in 0..max_generations {
        if gen != 0 && gen % 100 == 0 {
            eprintln!("{INFO}gen={gen} best_pos={best_pos:?} best_dist={best_dist}");
        }

        let next_generation = automaton.next_generation();
        let mut to_visit = BitGrid::with_dim_from(&reached[gen]);

        for pos in reached[gen].iter() {
            debug_assert_eq!(automaton.alive(pos), Some(false));

            if pos == destination {
                return Ok(assemble_path_from_sets(&reached, gen, pos));
            }

            for movement in [Up, Down, Left, Right] {
                let next = pos.next(movement);
                if let Some(false) = next_generation.alive(next) {
                    let dist = next.distance(&automaton.destination);
                    if dist < best_dist {
                        best_pos = next;
                        best_dist = dist;
                        best_gen = gen + 1;
                    }
                    to_visit.insert(next);
                }
            }
        }

        if to_visit.is_empty() {
            eprintln!("{WARN}No movement in gen={gen}");
            break;
        }

        automaton = next_generation;
        reached.push(to_visit);
    }

    eprintln!("{WARN}Failed to reach the destination (distance={best_dist})");
    assert!(best_gen > 0);
    Err(assemble_path_from_sets(&reached, best_gen, best_pos))
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

fn assemble_path_from_sets(
    history: &[BitGrid],
    mut gen: usize,
    mut pos: Position,
) -> Vec<Movement> {
    let mut path = vec![];
    'outer: while gen > 0 {
        for movement in [Up, Down, Left, Right] {
            let prev = pos.previous(movement);
            if history[gen - 1].contains(prev) {
                path.push(movement);
                gen -= 1;
                pos = prev;
                continue 'outer;
            }
        }
        panic!("missing movement"); // FIXME: improve panic message.
    }
    path.reverse();
    path
}

/// Returns the space-separated list of movements as a string.
pub fn path_to_string(path: &[Movement]) -> String {
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
    fn alive(&self, pos: Position) -> Option<bool> {
        self.grid.get(pos.i, pos.j)
    }

    #[must_use]
    fn next_generation(&self) -> Self {
        let mut new_gen = Grid::new(self.grid.height(), self.grid.width());

        for (i, j, alive) in self.grid.cells() {
            let alive_neighbors = self.grid.neighbors(i, j);

            let new_cell = if alive {
                (4..=5).contains(&alive_neighbors)
            } else {
                (2..=4).contains(&alive_neighbors)
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

pub fn lives_lost(path: &[Movement], mut automaton: Automaton) -> usize {
    let mut current = automaton.source;
    let mut lost = 0;

    for (gen, movement) in path.iter().enumerate() {
        if automaton.alive(current).unwrap() {
            lost += 1;

            // Leave this here in case we need to debug a bug.
            // dbg!(lost, gen, movement, current);

            // However: initial and final positions cannot be alive.
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
    fn immutable_endpoints() {
        const INPUT: &str = "\
            1 1 1 0\n\
            3 1 4 1\n\
            0 1 1 0";
        const EXPECTED: &str = "\
            0 0 0 1\n\
            0 1 0 0\n\
            1 0 0 1";

        let mut automaton = parse(INPUT);
        automaton.immutable_endpoints = true;

        automaton = automaton.next_generation();
        assert_eq!(automaton.to_string(), EXPECTED);

        for i in 0..10 {
            automaton = automaton.next_generation();
            dbg!(i, &automaton);
            assert_eq!(automaton.alive(Position { i: 1, j: 0 }), Some(false));
            assert_eq!(automaton.alive(Position { i: 1, j: 2 }), Some(false));
        }
    }

    #[test]
    fn shortest_path() {
        const INPUT: &str = "\
            3 0 0 1 0 0\n\
            0 1 1 0 1 1\n\
            0 0 1 1 0 0\n\
            0 0 0 0 0 4";
        const GOLDEN_LENGTH: usize = 14;

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

                let input = fs::read_to_string("input.txt").unwrap();
                let automaton = parse(&input);

                let path = find_path(automaton.clone(), MAX_GENERATIONS, MAX_PESSIMISM).unwrap();
                assert_eq!(lives_lost(&path, automaton), 0);
                assert_eq!(path.len(), GOLDEN_LENGTH);

                validade_path_format(&path_to_string(&path));
            }

            #[test]
            fn question2_alt_solver() {
                const GOLDEN_LENGTH: usize = 220;

                let input = fs::read_to_string("input.txt").unwrap();
                let automaton = parse(&input);

                let path = find_path_robust(automaton.clone(), MAX_GENERATIONS).unwrap();
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
                const GOLDEN_LENGTH: usize = 6264; // FIXME: suboptimal

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
                const GOLDEN_LENGTH: usize = 6200; // FIXME: suboptimal

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
