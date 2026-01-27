#!/bin/bash
# Run a tournament between all chess engine algorithms using fastchess
# Usage: ./run_tournament.sh [tc] [rounds]
#   tc     - Time control in seconds+increment format (default: 10+0.1)
#   rounds - Number of rounds per pairing (default: 50)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_DIR/results"

# Default parameters
TC="${1:-10+0.1}"
ROUNDS="${2:-50}"
CONCURRENCY="${3:-4}"

# Check if fastchess is available
if ! command -v fastchess &> /dev/null; then
    echo "Error: fastchess is not installed or not in PATH"
    echo "Install from: https://github.com/Disservin/fastchess"
    exit 1
fi

# Build all binaries in release mode
echo "Building engine binaries..."
cargo build --release --package aether 2>/dev/null

# Check if binaries exist
AETHER_BIN="$PROJECT_DIR/target/release/aether"
if [ ! -f "$AETHER_BIN" ]; then
    echo "Error: Engine binary not found at $AETHER_BIN"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

# Generate timestamp for results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
PGN_FILE="$RESULTS_DIR/tournament_${TIMESTAMP}.pgn"

echo ""
echo "=== Aether Chess Engine Tournament ==="
echo "Time control: $TC"
echo "Rounds per pairing: $ROUNDS"
echo "Concurrency: $CONCURRENCY"
echo "Results: $PGN_FILE"
echo ""

# Run tournament with all algorithms
# Each algorithm is selected via UCI setoption command
fastchess \
    -engine cmd="$AETHER_BIN" name=FullAlphaBeta option.Algorithm=FullAlphaBeta \
    -engine cmd="$AETHER_BIN" name=PureAlphaBeta option.Algorithm=PureAlphaBeta \
    -engine cmd="$AETHER_BIN" name=MTDf option.Algorithm=Mtdf \
    -engine cmd="$AETHER_BIN" name=NegaScout option.Algorithm=NegaScout \
    -engine cmd="$AETHER_BIN" name=MCTS option.Algorithm=MCTS \
    -each tc="$TC" \
    -rounds "$ROUNDS" \
    -games 2 \
    -repeat \
    -recover \
    -concurrency "$CONCURRENCY" \
    -pgnout "$PGN_FILE"

echo ""
echo "Tournament complete!"
echo "Results saved to: $PGN_FILE"
