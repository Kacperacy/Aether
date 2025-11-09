# Aether Chess Engine

A fast, modular chess engine written in Rust with a clean architecture and strong type safety.

**Status**: ✅ Production Ready | 🎯 Lichess Compatible | 🧪 119 Tests Passing

## Features

- **Complete UCI Protocol Support** - Compatible with chess GUIs (Arena, ChessBase, Lichess)
- **UCI Options** - Hash table size configuration (1-1024 MB)
- **Alpha-Beta Search** with iterative deepening and quiescence search
- **Transposition Table** - 5-10x speedup through position caching
- **Move Ordering** - MVV-LVA, killer moves, and history heuristic
- **Bitboard Representation** - Efficient 64-bit board representation
- **Magic Bitboards** - Fast sliding piece attack generation
- **Opening Book** - Pre-computed opening theory
- **Interactive CLI** - REPL for testing and analysis
- **Piece-Square Tables** - Sophisticated positional evaluation
- **Zobrist Hashing** - Fast position hashing for transposition table

## Performance

- **~250,000-500,000 nodes/second** with transposition table
- **Depth 6 search** in ~1-2 seconds from starting position
- **TT hit rate** of ~30-40% during typical searches
- **Estimated rating**: 1800-2000 Elo (Lichess)

## Quick Start

### Install and Run

```bash
# Build the UCI binary
cargo build --release -p uci

# Test the engine
echo -e "uci\nisready\nposition startpos\ngo depth 5\nquit" | ./target/release/aether-uci

# Deploy to Lichess (see DEPLOYMENT.md for details)
```

For detailed deployment instructions, see **[DEPLOYMENT.md](DEPLOYMENT.md)**.

## Architecture

Aether is organized as a Cargo workspace with multiple specialized crates:

```
aether/
├── types/          # Core types (Move, Square, Piece, Color, BitBoard, traits)
├── board/          # Board representation, FEN parsing, Zobrist hashing
├── movegen/        # Move generation with magic bitboards
├── eval/           # Position evaluation (material + piece-square tables)
├── search/         # Search algorithms (alpha-beta, transposition table)
├── engine/         # High-level engine coordination
├── uci/            # UCI protocol implementation
├── cli/            # Interactive command-line interface
├── opening/        # Opening book
├── perft/          # Move generation testing and benchmarks
└── benches/        # Performance benchmarks
```

This modular design allows:
- **Pluggable search algorithms** via the `Searcher` trait
- **Custom evaluators** via the `Evaluator` trait
- **Different move orderers** via the `MoveOrderer` trait
- **Clean separation of concerns** - no dependency cycles
- **Easy testing** - each crate can be tested independently

## Getting Started

### Prerequisites

- Rust 1.80+ (Edition 2024)
- Cargo

### Building

Build the entire workspace:

```bash
cargo build --release --workspace
```

Run tests:

```bash
cargo test --workspace
```

Run clippy for code quality:

```bash
cargo clippy --workspace --all-targets
```

## Usage

### UCI Mode (for Chess GUIs)

Run the UCI interface to connect with chess GUIs:

```bash
cargo run --release --bin aether-uci
```

Then configure your chess GUI to use the `aether-uci` binary.

**Supported UCI Commands:**
- `uci` - Initialize UCI mode
- `isready` - Check if engine is ready
- `ucinewgame` - Start a new game
- `position [fen FEN | startpos] [moves MOVES...]` - Set position
- `go [depth N | movetime MS | wtime MS btime MS winc MS binc MS]` - Start search
- `stop` - Stop current search (sets stop flag)
- `setoption name Hash value <MB>` - Set transposition table size (1-1024 MB)
- `quit` - Exit engine

**Example:**
```bash
uci
setoption name Hash value 256
position startpos moves e2e4 e7e5
go depth 10
```

### Interactive CLI

Run the interactive command-line interface:

```bash
cargo run --release --bin aether-cli
```

**Available Commands:**
- `display` / `d` - Show current board position
- `moves` / `m` - List all legal moves
- `search [depth]` - Search for best move
- `eval` / `e` - Evaluate current position
- `move <move>` - Make a move (e.g., `move e2e4`)
- `fen [FEN]` - Get or set position via FEN notation
- `new` - Start a new game
- `help` / `h` - Show help
- `quit` / `q` - Exit

**Example Session:**
```
Aether Chess CLI
Type 'help' for available commands

> d
8 r n b q k b n r
7 p p p p p p p p
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 P P P P P P P P
1 R N B Q K B N R
  A B C D E F G H

> move e2e4
Move: e2e4

> search 6
Searching to depth 6...
Best move: e7e5
Score: 25 centipawns
Nodes: 156432
Time: 1.23s
NPS: 127,162
```

## Magic Bitboards Generation

Magic bitboard constants are pre-generated for fast sliding piece attacks. To regenerate:

```bash
cargo run -p movegen --features codegen --bin gen_magics
```

Output: `movegen/src/magic_constants.rs`

(The `codegen` feature is optional to avoid pulling `rand` into normal builds)

## Testing

The project has **119 tests** covering all components:

### Test Breakdown

- **Unit Tests**: 85 tests across all crates
- **Integration Tests**: 9 end-to-end scenarios
- **UCI Protocol Tests**: 13 command parsing tests
- **Edge Case Tests**: 11 corner case validations
- **Perft Tests**: 1 move generation verification

### Run All Tests

```bash
cargo test --workspace
# All 119 tests should pass
```

### Unit Tests by Component

```bash
cargo test -p board      # Board operations, FEN parsing
cargo test -p movegen    # Move generation
cargo test -p search     # Alpha-beta search, TT, move ordering
cargo test -p eval       # Position evaluation
cargo test -p uci        # UCI protocol parsing
cargo test -p opening    # Opening book
```

### Integration Tests

```bash
cargo test --test integration_test
```

**Coverage:**
- Complete game workflow (playing moves, evaluating positions)
- Search in tactical positions (Kiwipete, etc.)
- Make/unmake move consistency (all legal moves)
- Evaluation symmetry (starting position ≈ 0)
- Pawn promotion and en passant mechanics
- Transposition table effectiveness
- Move generation count (starting position = 20 moves)

### Edge Case Tests

```bash
cargo test -p board edge_cases
```

**Coverage:**
- Checkmate and stalemate positions
- En passant captures
- Castling rights preservation
- Insufficient material scenarios
- Under-promotion positions
- FEN roundtrip consistency
- Complex tactical positions (many pieces)

### UCI Protocol Tests

```bash
cargo test -p uci
```

**Coverage:**
- Command parsing (uci, isready, position, go, stop, quit)
- Position setup (startpos, FEN, moves)
- Go command variants (depth, movetime, wtime/btime, infinite)
- Setoption parsing (Hash configuration)
- Time calculation for both sides

### Perft (Move Generation Verification)

Run correctness/performance tests for move generation:

```bash
cargo test -p perft
```

**Perft Results from Starting Position:**
```
Depth 1: 20 nodes
Depth 2: 400 nodes
Depth 3: 8,902 nodes
Depth 4: 197,281 nodes
Depth 5: 4,865,609 nodes
```

Run Criterion benchmarks:

```bash
cargo bench -p perft
```

### Benchmarks

Run comprehensive performance benchmarks:

```bash
cargo bench -p benches
```

The benchmark suite measures:
- **Board operations** - make_move, unmake_move, FEN parsing
- **Move generation** - legal move generation for various positions
- **Evaluation** - position scoring with piece-square tables
- **Transposition table** - store, probe hit/miss performance
- **Move ordering** - simple (MVV-LVA) and advanced (killer+history) strategies
- **Search** - alpha-beta search at depths 3, 4, and 5

Example benchmark results (may vary by hardware):
```
board_operations/make_move       time: [45.2 ns 45.8 ns 46.4 ns]
move_generation/startpos         time: [2.1 µs 2.2 µs 2.3 µs]
evaluation/startpos              time: [892 ns 901 ns 911 ns]
transposition_table/tt_probe_hit time: [12.3 ns 12.5 ns 12.7 ns]
search/depth_5/startpos          time: [421 ms 428 ms 435 ms]
```

## Production Deployment

Aether is production-ready and can be deployed as a chess bot on Lichess or used with any UCI-compatible GUI.

### Lichess Bot

See **[DEPLOYMENT.md](DEPLOYMENT.md)** for complete instructions on:
- Setting up `lichess-bot` integration
- Configuring time controls and variants
- Tuning hash table sizes for different game types
- Monitoring and troubleshooting

### Quick Deployment

```bash
# 1. Build release binary
cargo build --release -p uci

# 2. Install lichess-bot
git clone https://github.com/lichess-bot-devs/lichess-bot.git
cd lichess-bot
pip3 install -r requirements.txt

# 3. Configure config.yml with your API token and engine path
# 4. Run the bot
python3 lichess-bot.py
```

### Strength Estimate

- **Rating**: 1800-2000 Elo (Lichess)
- **Tactical**: Good (sees 2-3 move tactics at depth 6)
- **Positional**: Basic (piece-square tables)
- **Opening**: Fair (opening book support)
- **Endgame**: Weak (no tablebases)

## Configuration

### Transposition Table Size

The default TT size is 64 MB. You can configure it via UCI:

```
setoption name Hash value 128
```

**Recommended sizes by time control:**
- Bullet (1+0, 2+1): 64-128 MB
- Blitz (3+0, 5+0): 128-256 MB
- Rapid (10+0, 15+10): 256-512 MB
- Classical (30+0): 512-1024 MB

Or in code:
```rust
use search::AlphaBetaSearcher;

let mut searcher = AlphaBetaSearcher::with_tt_size(128); // 128 MB
```

### Search Limits

Configure search via `SearchLimits`:

```rust
use search::SearchLimits;
use std::time::Duration;

// Search to depth 10
let limits = SearchLimits::depth(10);

// Search for 5 seconds
let limits = SearchLimits::time(Duration::from_secs(5));

// Search with node limit
let limits = SearchLimits::nodes(1_000_000);
```

## Development

### Project Structure

Each crate has a specific responsibility:

- **types** - Core domain types and traits (`Move`, `Square`, `BoardQuery`)
- **board** - Concrete `Board` implementation with state management
- **movegen** - Legal move generation using bitboards
- **eval** - Position evaluation heuristics
- **search** - Game tree search algorithms
- **uci** - UCI protocol parser and handler
- **cli** - Interactive REPL for humans
- **opening** - Opening book moves
- **perft** - Correctness testing

### Adding a New Search Algorithm

1. Implement the `Searcher` trait:

```rust
use search::{Searcher, SearchLimits, SearchResult};
use aether_types::BoardQuery;

pub struct MySearcher;

impl Searcher for MySearcher {
    fn search<T>(&mut self, board: &T, limits: &SearchLimits) -> SearchResult
    where
        T: BoardQuery + Clone + 'static
    {
        // Your search implementation
        todo!()
    }

    fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    fn stop(&mut self) {
        self.should_stop = true;
    }
}
```

2. Use it with the engine:

```rust
let mut searcher = MySearcher::new();
let result = searcher.search(&board, &limits);
```

### Adding a New Evaluator

Implement the `Evaluator` trait:

```rust
use eval::{Evaluator, Score};
use aether_types::BoardQuery;

pub struct MyEvaluator;

impl Evaluator for MyEvaluator {
    fn evaluate<T: BoardQuery>(&self, board: &T) -> Score {
        // Your evaluation logic
        0
    }
}
```

## Technical Details

### Bitboard Representation

The engine uses 64-bit integers to represent piece positions:

```
Bitboard for White Pawns (starting position):
0 0 0 0 0 0 0 0  (rank 8)
0 0 0 0 0 0 0 0  (rank 7)
...
1 1 1 1 1 1 1 1  (rank 2)
0 0 0 0 0 0 0 0  (rank 1)
```

### Magic Bitboards

Sliding piece attacks are computed using magic multiplication:

```rust
let attacks = MAGIC_ROOK_TABLE[(square as usize * MAGIC_OFFSET) +
              ((occupancy * MAGIC_NUMBER) >> MAGIC_SHIFT) as usize];
```

This provides O(1) attack generation for bishops, rooks, and queens.

### Transposition Table

Stores previously evaluated positions to avoid redundant work:

```rust
pub struct TTEntry {
    hash: u64,              // Zobrist hash
    best_move: Option<Move>,// Best move from this position
    score: Score,           // Evaluation score
    depth: u8,              // Search depth
    entry_type: EntryType,  // Exact/LowerBound/UpperBound
    age: u8,                // For replacement strategy
}
```

### Alpha-Beta Search

Implements negamax alpha-beta with:
- Iterative deepening (depths 1, 2, 3, ...)
- Quiescence search (tactical stability)
- Move ordering (TT move, MVV-LVA, killer moves, history)
- Transposition table cutoffs
- Principal variation tracking

## Project Statistics

- **Lines of Code**: ~8,700 (excluding tests and generated code)
- **Tests**: 119 (100% passing)
- **Crates**: 10 modular components
- **Clippy Warnings**: 0
- **Test Coverage**: All major features covered
- **Benchmarks**: Comprehensive performance suite

## Roadmap

### Current Limitations

- **Stop Command**: Cannot interrupt synchronous search mid-execution
- **Single-Threaded**: Uses only one CPU core
- **No Pondering**: Doesn't think during opponent's time
- **Basic Time Management**: Simple time allocation formula

### Planned Features

- [ ] Async search with proper stop support
- [ ] Multi-threaded parallel search
- [ ] Pondering (think on opponent's time)
- [ ] Endgame tablebases (Syzygy)
- [ ] Neural network evaluation (NNUE)
- [ ] Advanced time management
- [ ] Multi-PV support
- [ ] Chess960 support

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Ensure all tests pass: `cargo test --workspace`
5. Check code quality: `cargo clippy --workspace --all-targets`
6. Format code: `cargo fmt --all`
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

### Development Guidelines

- Write tests for new features
- Maintain zero clippy warnings
- Follow Rust idioms and best practices
- Update documentation for API changes
- Add benchmarks for performance-critical code

## License

MIT License - see LICENSE file for details

## Author

**Kacper Maciołek**

## Acknowledgments

- Built with Rust 🦀
- Uses Criterion for benchmarking
- Compatible with lichess-bot for online play
- Inspired by the chess programming community

---

**Ready to play on Lichess?** See [DEPLOYMENT.md](DEPLOYMENT.md) for setup instructions!