use std::ops::{BitOr, Shl};

use rayon::prelude::*;

use crate::position::Movement;
use crate::Grid;
use crate::{find_path, Automaton};

pub fn find_paths(
    automaton: &Automaton,
    with_inner_code: Option<u128>,
    max_generations: usize,
    max_pessimism: u16,
) -> Vec<(u128, Grid, Vec<Movement>)> {
    assert!(automaton.immutable_endpoints);

    let mut inner_grids = generate_inner_grids();
    if let Some(only) = with_inner_code {
        inner_grids.retain(|&(code, _)| code == only);
    }

    inner_grids
        .par_drain(0..inner_grids.len())
        .map(|(code, inner)| {
            let mut automaton = automaton.clone();
            automaton.grid.overwrite(&inner, 2300, 2300);
            let path = find_path(automaton.clone(), max_generations, max_pessimism).unwrap();
            (code, inner, path)
        })
        .collect()
}

fn generate_inner_grids() -> Vec<(u128, Grid)> {
    fn code(grid: &Grid) -> u128 {
        assert_eq!(grid.raw().len(), 100);
        let mut code: u128 = 0;
        for &v in grid.raw().iter() {
            code = code.shl(1u128).bitor(v as u128);
        }
        dbg!(code);
        code
    }

    let mut patterns = Vec::new();
    for grid in [BASE1, BASE2, BASE3] {
        let mut grid: Grid = grid.parse().unwrap();
        for rotations in 0..4 {
            if rotations > 0 {
                grid = grid.rotate();
            }
            let flip = grid.flip();
            for grid in [grid.clone(), grid.invert(), flip.invert(), flip] {
                patterns.push((code(&grid), grid));
            }
        }
    }
    patterns
}

const BASE1: &str = "\
    0 0 0 0 0 0 0 0 0 0\n\
    1 1 1 1 1 1 1 1 1 0\n\
    1 0 0 0 0 0 0 0 0 0\n\
    1 1 1 1 1 1 1 1 1 0\n\
    1 0 0 0 0 0 0 0 0 0\n\
    1 1 1 1 1 1 1 1 1 0\n\
    1 0 0 0 0 0 0 0 0 0\n\
    1 1 1 1 1 1 1 1 1 0\n\
    1 0 0 0 0 0 0 0 0 0\n\
    1 1 1 1 1 1 1 1 1 1";

const BASE2: &str = "\
    0 0 0 0 0 0 0 0 0 0\n\
    0 1 0 1 1 1 0 1 1 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 1 1 0 1 1 1 0 1\n\
    0 0 0 0 0 0 0 0 0 1";

const BASE3: &str = "\
    1 1 1 1 1 1 1 1 1 1\n\
    0 0 0 0 0 0 0 0 0 1\n\
    0 1 1 1 1 1 1 1 0 1\n\
    0 1 0 0 0 0 0 1 0 1\n\
    0 1 0 1 1 1 0 1 0 1\n\
    0 1 0 1 0 1 0 1 0 1\n\
    0 1 0 1 0 0 0 1 0 1\n\
    0 1 0 1 1 1 1 1 0 1\n\
    0 1 0 0 0 0 0 0 0 1\n\
    0 1 1 1 1 1 1 1 1 1";
