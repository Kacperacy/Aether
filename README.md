# Aether

A modular chess engine written in Rust.

## Features

Search:

- Alpha-beta with iterative deepening and aspiration windows
- Transposition table with 16-byte packed entries, probed in quiescence as well as the main search
- Lazy move selection — the best remaining move is extracted on demand rather than the whole list sorted
- Move ordering: TT move, SEE, MVV-LVA, killer moves, per-side history
- Null move pruning, late move reductions, futility and reverse futility pruning
- Quiescence search with delta pruning

Move generation:

- Magic bitboards
- Target-masked generation, so captures are produced without building the quiet moves first
- Fixed-capacity move lists, stack allocated

Evaluation:

- Material and piece-square tables, maintained incrementally alongside make/unmake
- The accumulator type is owned by the evaluator, so an NNUE evaluator can replace the
  piece-square one without changes to `board` or `engine::search`

Interface:

- UCI protocol
- Optional `tune` build exposing search parameters as UCI options

## Project Structure

```
aether/
├── core/       - Core types: BitBoard, Move, Piece, Square, etc.
├── attacks/    - Magic bitboards and attack lookup tables
├── board/      - Board representation, make/unmake, FEN parsing
├── movegen/    - Move generation, legality, perft
├── engine/     - Search, evaluation, tunable parameters, bench
├── interface/  - UCI protocol implementation
└── scripts/    - Benchmark and engine-vs-engine match wrappers
```

## Building

Requires Rust stable, edition 2024. The toolchain is pinned in `rust-toolchain.toml`;
the minimum supported version is 1.88 (let-chains).

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

The engine communicates via UCI protocol. Use with any UCI-compatible GUI (Arena, Cutechess, etc.).

## Development

Two numbers gate every change. Both are exact, and both must be accounted for:

```bash
# Move generation. Must stay bit-identical unless generation itself changed.
printf 'position startpos\nperft 6\nquit\n' | ./target/release/aether   # 119060324

# Search. Deterministic for a given binary.
./target/release/aether bench                                          # 3864760
```

`bench` searches a fixed set of positions to a fixed depth. It is depth-limited,
single-threaded and integer-only, so nothing consults the clock and the node count is the
same on every machine — a faster CPU changes NPS, not which nodes are searched. It changes
only when search behaviour changes, so an unexplained change means something moved that you
did not intend to move. CI pins the value.

The NPS printed alongside is the speed signal. Node count unchanged plus NPS up is an
unambiguous win; NPS alone is not, because it depends on machine load. Compare two binaries
by interleaving their runs rather than against a figure recorded earlier.

## Testing

```bash
cargo test --workspace
```

Deep perft is a separate tier, ignored by default because it searches billions of nodes:

```bash
cargo test -p movegen --test perft_deep --release -- --ignored
```

## Perft

Run the fast move-generation correctness suite:

```bash
cargo test -p movegen --test perft --release
```

`perft` is also available inside a UCI session, and prints a per-move breakdown at depth 1:

```bash
printf 'position startpos\nperft 6\nquit\n' | cargo run --release
```

## Matches

Engine-versus-engine testing, via [fastchess](https://github.com/Disservin/fastchess):

```bash
scripts/bench.sh [depth]                                  # search regression + NPS
scripts/match.sh <baseline-binary> <candidate-binary> 400 # quick A/B
scripts/sprt.sh  <baseline-binary> <candidate-binary>     # accept/reject a strength patch
```

`match.sh` takes `NODES=`, `TC=` and `CONCURRENCY=` from the environment. Prefer a fixed
node count for pure search changes, which removes clock jitter.

A fixed-node search is deterministic, so one opening always produces the same game. If the
book has fewer openings than rounds it cycles, and the repeated games are identical while
the reported error bars still assume every game is independent — which makes noise look
decisive. `scripts/match.sh` refuses that configuration rather than reporting it.

## Tuning

Search parameters live in `engine/src/params.rs`. The default build folds them to compile-time
constants; the `tune` build exposes each as a UCI spin option with its permitted range, so an
external tuner can drive them without recompiling:

```bash
cargo build --release --features tune
```

## Magic Bitboards

Generate magic constants (only needed after modifying move generation):

```bash
cargo run -p attacks --features codegen --bin gen_magics
```

Output: `attacks/src/magic_constants.rs`
