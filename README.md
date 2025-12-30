# Aether

A modular chess engine written in Rust.

## Features

- Magic bitboard-based move generation
- Alpha-beta search with iterative deepening
- Transposition tables with Zobrist hashing
- Move ordering: TT move, SEE, MVV-LVA, killer moves, history heuristic
- Null move pruning and late move reductions (LMR)
- Aspiration windows
- Quiescence search with delta pruning
- UCI protocol support

## Project Structure

```
aether/
├── core/       - Core types: BitBoard, Move, Piece, Square, etc.
├── board/      - Board representation and FEN parsing
├── movegen/    - Move generation with magic bitboards
├── engine/     - Search and evaluation
├── interface/  - UCI protocol implementation
└── benches/    - Performance benchmarks
```

## Building

Requires Rust stable (Edition 2024).

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

The engine communicates via UCI protocol. Use with any UCI-compatible GUI (Arena, Cutechess, etc.).

## Testing

```bash
cargo test --workspace
```

## Perft

Run move generation correctness tests:

```bash
cargo run --release -- perft 6
```

## Magic Bitboards

Generate magic constants (only needed after modifying move generation):

```bash
cargo run -p movegen --features codegen --bin gen_magics
```

Output: `movegen/src/magic_constants.rs`