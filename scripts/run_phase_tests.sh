#!/bin/bash
# Run tests by game phase (opening, middlegame, endgame) using fastchess
# Requires position files downloaded via download_positions.sh
#
# Usage: ./run_phase_tests.sh [algorithm1] [algorithm2] [tc] [rounds]
#   algorithm1 - First algorithm (default: FullAlphaBeta)
#   algorithm2 - Second algorithm (default: MCTS)
#   tc         - Time control (default: 10+0.1)
#   rounds     - Rounds per phase (default: 50)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
POSITIONS_DIR="$PROJECT_DIR/positions"
RESULTS_DIR="$PROJECT_DIR/results"

# Default parameters
ALGO1="${1:-FullAlphaBeta}"
ALGO2="${2:-MCTS}"
TC="${3:-10+0.1}"
ROUNDS="${4:-50}"
CONCURRENCY="${5:-4}"

# Check if fastchess is available
if ! command -v fastchess &> /dev/null; then
    echo "Error: fastchess is not installed or not in PATH"
    echo "Install from: https://github.com/Disservin/fastchess"
    exit 1
fi

# Build engine
echo "Building engine..."
cargo build --release --package aether 2>/dev/null

AETHER_BIN="$PROJECT_DIR/target/release/aether"
if [ ! -f "$AETHER_BIN" ]; then
    echo "Error: Engine binary not found"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

# Generate timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo ""
echo "=== Phase-based Testing: $ALGO1 vs $ALGO2 ==="
echo "Time control: $TC"
echo "Rounds per phase: $ROUNDS"
echo ""

# Function to run test for a phase
run_phase_test() {
    local phase=$1
    local epd_file=$2
    local tc_override=${3:-$TC}

    if [ ! -f "$epd_file" ]; then
        echo "Skipping $phase: Position file not found ($epd_file)"
        echo "Run ./scripts/download_positions.sh first"
        return 1
    fi

    local pgn_file="$RESULTS_DIR/${phase}_${ALGO1}_vs_${ALGO2}_${TIMESTAMP}.pgn"

    echo "--- $phase ---"
    echo "Positions: $epd_file"
    echo "Output: $pgn_file"
    echo ""

    fastchess \
        -engine cmd="$AETHER_BIN" name="$ALGO1" option.Algorithm="$ALGO1" \
        -engine cmd="$AETHER_BIN" name="$ALGO2" option.Algorithm="$ALGO2" \
        -openings file="$epd_file" format=epd order=random \
        -each tc="$tc_override" \
        -rounds "$ROUNDS" \
        -games 2 \
        -repeat \
        -recover \
        -concurrency "$CONCURRENCY" \
        -pgnout "$pgn_file"

    echo ""
}

# Opening test - shorter games from near-start positions
echo "=== OPENING POSITIONS ==="
OPENING_EPD="$POSITIONS_DIR/opening/noob_3moves.epd"
if [ ! -f "$OPENING_EPD" ]; then
    OPENING_EPD="$POSITIONS_DIR/opening/2moves_v1.epd"
fi
run_phase_test "opening" "$OPENING_EPD" || true

# Middlegame test
echo "=== MIDDLEGAME POSITIONS ==="
MIDDLEGAME_EPD="$POSITIONS_DIR/middlegame/Drawkiller_balanced_big.epd"
run_phase_test "middlegame" "$MIDDLEGAME_EPD" || true

# Endgame test - longer time control for complex endgames
echo "=== ENDGAME POSITIONS ==="
ENDGAME_EPD="$POSITIONS_DIR/endgame/endgames.epd"
run_phase_test "endgame" "$ENDGAME_EPD" "30+0.3" || true

echo ""
echo "=== Phase Testing Complete ==="
echo "Results saved to: $RESULTS_DIR/"
echo ""
echo "To analyze results:"
echo "  - Use fastchess output for Elo estimates"
echo "  - PGN files can be analyzed with pgn-extract or python-chess"
