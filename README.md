# Aether

A modular chess engine written in Rust. The repository is a Cargo workspace composed of multiple crates (core, board,
movegen, eval, search, engine, uci, cli, perft).

- Minimum supported Rust: latest stable with Edition 2024

## Getting Started

Build the entire workspace:

```
cargo build --workspace
```

Run basic checks and tests:

```
cargo check --workspace
cargo test --workspace
```

## Magic Bitboards Generation

Magic bitboard constants are generated inside the `movegen` crate to keep paths stable.

- Generate/re-generate magics:

```
cargo run -p movegen --features codegen --bin gen_magics
```

- Output file: `movegen/src/magic_constants.rs`

(Note: The generator is behind the optional `codegen` feature to avoid pulling `rand` into normal builds. The previous
root-level flag is deprecated; use the command above.)

## Perft

Run correctness/performance tests for move generation:

```
cargo test -p perft
# or run the Criterion benchmark
cargo bench -p perft
```