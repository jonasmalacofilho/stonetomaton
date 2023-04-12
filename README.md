# Find paths in 2-D-automaton mazes

_Partial solution of the [Stone Automata Maze Challenge]._

For a high-level overview, some implementation notes, and other information, see the module-level
documentation in [`src/main.rs`].

## Build, test and execute

This program is written in Rust, and a Rust toolchain in necessary to build it. It has been tested
with (stable) Rust 1.68.1.

- Run the unit tests: `cargo test`
- Build and execute the program: `cargo run --release`
- View the documentation in the browser: `cargo doc --open`
- For more options, consult the Cargo documentation.

## Submissions

- Level 1: [code][level1-tag] | [outputs][level1-output] | --- | 17.8 ms | 2.4 MiB heap peak
- Level 2: [code][level2-tag] | [outputs][level2-output] | [run log][level2-log] | 17.7 min[^1] | 3.4 GiB heap peak[^1]

The submissions were prepared on a i7-8700K with 32 GiB of RAM.

[^1]: Resources needed for all challenges, running sequentially.


[Stone Automata Maze Challenge]: https://sigmageek.com/stone_results/stone-automata-maze-challenge
[`src/main.rs`]: ./src/main.rs
[level1-output]: ./submissions/level1
[level1-tag]: https://github.com/jonasmalacofilho/stonetomaton/releases/tag/submission-level-1-question-2
[level2-log]: ./submissions/level2/run.log
[level2-output]: ./submissions/level2
[level2-tag]: https://github.com/jonasmalacofilho/stonetomaton/releases/tag/submission-level-2

<!-- Original challenge URL: https://sigmageek.com/solution/stone-automata-maze-challenge -->
