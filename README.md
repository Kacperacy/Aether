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
├── attacks/    - Magic bitboards and attack lookup tables
├── board/      - Board representation, make/unmake, FEN parsing
├── movegen/    - Move generation, legality, perft
├── engine/     - Search and evaluation
└── interface/  - UCI protocol implementation
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

Run the move-generation correctness suite:

```bash
cargo test -p movegen --test perft --release
```

`perft` is also available inside a UCI session:

```bash
printf 'position startpos\nperft 6\nquit\n' | cargo run --release
```

## Magic Bitboards

Generate magic constants (only needed after modifying move generation):

```bash
cargo run -p attacks --features codegen --bin gen_magics
```

Output: `attacks/src/magic_constants.rs`