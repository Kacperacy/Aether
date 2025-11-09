# Aether Chess Engine - Production Deployment Guide

This guide explains how to deploy Aether as a bot on Lichess or other chess platforms.

## Prerequisites

- Rust 1.80+ with Edition 2024 support
- Git
- A Lichess account (for Lichess deployment)
- Lichess Bot API token (https://lichess.org/account/oauth/token)

## Building for Production

### 1. Build Release Binary

```bash
# Build the UCI binary in release mode (optimized)
cargo build --release -p uci

# The binary will be at: target/release/aether
```

### 2. Verify UCI Protocol

Test that the UCI protocol works correctly:

```bash
# Start the engine
./target/release/aether

# Type these commands (one per line):
uci
isready
position startpos
go depth 5
quit
```

You should see:
- `id name Aether 0.1.0`
- `id author Kacper Maciołek`
- `uciok`
- `readyok`
- Search info and `bestmove`

## Deploying to Lichess

### Method 1: Using lichess-bot (Recommended)

**lichess-bot** is the official Lichess bot client.

#### 1. Install lichess-bot

```bash
git clone https://github.com/lichess-bot-devs/lichess-bot.git
cd lichess-bot
pip3 install -r requirements.txt
```

#### 2. Configure lichess-bot

Create `config.yml`:

```yaml
token: "YOUR_LICHESS_API_TOKEN"    # Get from https://lichess.org/account/oauth/token
url: "https://lichess.org/"
engine:
  dir: "../Aether/target/release"   # Path to UCI binary directory
  name: "aether"                 # Binary name
  protocol: "uci"
  ponder: false
  uci_options:
    Hash: 256                        # MB of memory for transposition table
    Move Overhead: 200               # ms to reserve for network latency (recommended: 100-300 for Lichess)
    # Threads: 1                     # Not yet implemented

challenge:
  concurrency: 1
  min_increment: 0
  max_increment: 180
  min_initial: 0
  max_initial: 315360000
  variants:
    - standard
  time_controls:
    - bullet
    - blitz
    - rapid
    - classical
  modes:
    - casual
    - rated

quit_after_all_games_finish: false
```

#### 3. Run the Bot

```bash
python3 lichess-bot.py
```

The bot will:
- Connect to Lichess
- Accept challenges based on your configuration
- Play games using Aether engine

### Method 2: Manual UCI Integration

If you want to integrate with a custom GUI or platform:

#### UCI Commands Supported

| Command | Description | Example |
|---------|-------------|---------|
| `uci` | Initialize UCI mode | `uci` |
| `isready` | Check readiness | `isready` |
| `ucinewgame` | Start new game | `ucinewgame` |
| `position` | Set position | `position startpos moves e2e4 e7e5` |
| `go` | Start search | `go depth 10` |
| `stop` | Stop search (sets stop flag) | `stop` |
| `setoption` | Set engine option | `setoption name Hash value 256`<br>`setoption name Move Overhead value 200` |
| `quit` | Exit engine | `quit` |

#### Position Command

```bash
# Set starting position
position startpos

# Set position with moves
position startpos moves e2e4 e7e5 g1f3

# Set from FEN
position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
```

#### Go Command Options

```bash
# Search to fixed depth
go depth 10

# Search for fixed time (ms)
go movetime 5000

# Search with time controls
go wtime 300000 btime 300000 winc 5000 binc 5000

# Search infinitely (until stop)
go infinite

# Search with node limit
go nodes 1000000
```

#### Setoption Command

```bash
# Set hash table size (1-1024 MB)
setoption name Hash value 256

# Set move overhead for network latency (0-5000 ms)
setoption name Move Overhead value 200
```

## Performance Tuning

### Hash Table Size

The hash table (transposition table) significantly improves search speed:

- **Bullet (1+0, 2+1)**: 64-128 MB
- **Blitz (3+0, 5+0)**: 128-256 MB
- **Rapid (10+0, 15+10)**: 256-512 MB
- **Classical (30+0)**: 512-1024 MB

Configure via:
```bash
setoption name Hash value 256
```

### Move Overhead

Move Overhead reserves time before each move to compensate for network/GUI latency. This is **critical for online play** to avoid time losses.

**Recommended settings:**

- **Local GUIs (Arena, ChessBase)**: 10-30 ms (default: 10 ms)
- **Lichess (good connection)**: 100-150 ms
- **Lichess (unstable connection)**: 200-300 ms
- **High latency platforms**: 300-500 ms

Configure via:
```bash
setoption name Move Overhead value 200
```

**How it works:**
- If the engine calculates 5000ms for a move with 200ms overhead, it will only search for 4800ms
- This ensures the move is sent before time expires, accounting for network delays
- Without proper overhead, the engine may lose on time even when winning

### Expected Performance

- **Nodes per second**: 250,000 - 500,000 NPS
- **Depth 6 search**: ~1-2 seconds (starting position)
- **Transposition table hit rate**: 30-40%

### Time Management

The engine uses an **conservative adaptive time management algorithm** that prevents time losses in online play:

**With Increment (e.g., 5+3, 3+2):**

Adapts time allocation based on remaining clock (very conservative to prevent time losses):
- **>180s (early game)**: Uses 1/40 of time + full increment (~10.3s for 5+3, ~7.3s net)
- **60-180s (mid game)**: Uses 1/30 of time + full increment (~8.0s for 5+3, ~5.0s net)
- **<60s (time trouble)**: Uses 1/20 of time + full increment (~6.0s for 1min, ~3.0s net)

**Without Increment (e.g., 5+0, 3+0):**
- Uses 1/30 of remaining time (~10s for 5+0)
- Conservative approach for safety

**Algorithm:**
```rust
divisor = if time > 180s { 40 } else if time > 60s { 30 } else { 20 }
time_per_move = (remaining_time / divisor) + increment - move_overhead
```

**Targeting:** ~3-7s net time usage per move (conservative to prevent time losses on Lichess).

**Example for 5+3:**
- After 32 moves: ~179s remaining (safe margin)
- Net usage: ~7s early game, ~5s mid game, ~3s time trouble

This conservative approach prioritizes avoiding time losses over maximum search depth. For exact control, override with `go movetime`.

## Monitoring and Debugging

### Enable Logging

The engine sends info via UCI `info string` commands:

```
info string Starting search with limits: depth=Some(6), time=None, nodes=None
info depth 6 seldepth 6 score cp 50 nodes 12345 nps 450000 time 27 pv e2e4 e7e5
info hashfull 123
bestmove e2e4
```

### Common Issues

**Issue**: Engine doesn't respond
- **Solution**: Check that stdin/stdout are properly connected
- **Debug**: Run manually and type commands

**Issue**: Engine plays weak moves
- **Solution**: Increase search depth or hash size
- **Debug**: Check `info` output for depth reached

**Issue**: Engine times out
- **Solution**: Reduce hash size or check CPU usage
- **Debug**: Monitor search time in `info` output

**Issue**: Engine crashes
- **Solution**: Check logs, reduce hash size, verify installation
- **Debug**: Run with `RUST_BACKTRACE=1`

## Platform-Specific Deployment

### Linux

```bash
# Build
cargo build --release -p uci

# Run
./target/release/aether
```

### macOS

```bash
# Build
cargo build --release -p uci

# Run
./target/release/aether
```

### Windows

```powershell
# Build
cargo build --release -p uci

# Run
.\target\release\aether.exe
```

## Production Checklist

Before deploying:

- [ ] Build in release mode (`--release`)
- [ ] Test UCI protocol manually
- [ ] Configure appropriate hash size for time control
- [ ] Test against known positions
- [ ] Monitor first few games for issues
- [ ] Set up automatic restart on crash (optional)
- [ ] Configure challenge filters appropriately

## Limitations and Future Work

### Current Limitations

1. **Stop Command**: Currently sets a stop flag but cannot interrupt synchronous search mid-execution. For production use with strict time controls, consider implementing async search.

2. **Single-Threaded**: Engine uses only one CPU core. Multi-threading is planned for future versions.

3. **No Pondering**: Engine does not think during opponent's time.

4. **Basic Time Management**: Uses simple time allocation formula. More sophisticated time management planned.

### Planned Features

- **Async Search**: True stop command support
- **Multi-Threading**: Parallel search for higher NPS
- **Pondering**: Think on opponent's time
- **Endgame Tablebases**: Perfect play in endgames
- **Better Time Management**: More sophisticated algorithms
- **Neural Network Evaluation**: ML-based evaluation (NNUE)

## Strength Estimates

Based on current implementation:

- **Tactical Awareness**: Good (sees 2-3 move tactics at depth 6)
- **Positional Understanding**: Basic (piece-square tables only)
- **Opening Play**: Fair (has opening book)
- **Endgame**: Weak (no tablebases, basic evaluation)

**Estimated Rating**: 1800-2000 Elo (Lichess)

This can be improved significantly with:
- Deeper search (more time)
- Better evaluation (tuned weights, neural networks)
- Endgame tablebases
- More sophisticated search extensions

## Support

For issues, feature requests, or questions:
- GitHub Issues: https://github.com/Kacperacy/Aether/issues
- Email: [your email]

## License

MIT License - see LICENSE file for details
