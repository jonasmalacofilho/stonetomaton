use std::ops::RangeInclusive;

use clap::Parser;

/// Stone automaton maze challenges solver.
#[derive(Parser, Debug)]
pub struct Options {
    /// Only solve challenge `NUMBER` (default is to solve all).
    #[arg(short, long, value_name = "NUMBER")]
    challenge: Option<u8>,

    /// Stop before `LIMIT` generations.
    #[arg(short = 'G', long, value_name = "LIMIT", default_value_t = 50_000)]
    pub max_generations: usize,

    /// Ignore arcs `LIMIT` moves worse than current best estimate.
    #[arg(short = 'P', long, value_name = "LIMIT", default_value_t = 50)]
    pub max_pessimism: u16,

    /// Check the resulting path(s).
    #[arg(long)]
    pub check: bool,
}

impl Options {
    pub fn challenges(&self) -> RangeInclusive<u8> {
        if let Some(only) = self.challenge {
            only..=only
        } else {
            0..=5
        }
    }
}
