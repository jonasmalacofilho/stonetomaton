use std::ops::RangeInclusive;

use clap::Parser;

/// Stone automaton maze challenges solver.
#[derive(Parser, Debug)]
pub struct Options {
    /// Only solve challenge `NUMBER` (default is to solve all).
    #[arg(short, long, value_name = "NUMBER")]
    challenge: Option<u8>,

    /// Only use inner grid with code `CODE` (default is to try several inner grids).
    #[arg(long, value_name = "CODE")]
    pub with_inner_code: Option<u128>,

    /// Stop before `LIMIT` generations.
    #[arg(long, value_name = "LIMIT", default_value_t = 50_000)]
    pub max_generations: usize,

    /// Ignore arcs `LIMIT` moves worse than current best estimate.
    #[arg(long, value_name = "LIMIT", default_value_t = 50)]
    pub max_pessimism: u16,

    /// Source and destination cells are immutable and always dead/white.
    // TODO: old/clean up.
    #[arg(long)]
    pub immutable_endpoints: bool,

    /// Check the resulting path(s).
    // TODO: old/clean up.
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
