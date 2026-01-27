#!/bin/bash
# Run benchfile command on all algorithms and compare NPS by game phase
# Usage: ./run_benchfile.sh [epd_file] [depth] [limit]
#   epd_file - Path to EPD file (default: positions/opening/noob_3moves.epd)
#   depth    - Search depth (default: 10)
#   limit    - Max positions to test (default: 100)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_DIR/results"

# Default parameters
EPD_FILE="${1:-$PROJECT_DIR/positions/opening/noob_3moves.epd}"
DEPTH="${2:-10}"
LIMIT="${3:-100}"

# Build engine
echo "Building engine..."
cargo build --release --package aether 2>/dev/null

AETHER_BIN="$PROJECT_DIR/target/release/aether"
if [ ! -f "$AETHER_BIN" ]; then
    echo "Error: Engine binary not found"
    exit 1
fi

if [ ! -f "$EPD_FILE" ]; then
    echo "Error: EPD file not found: $EPD_FILE"
    echo "Run ./scripts/download_positions.sh first"
    exit 1
fi

mkdir -p "$RESULTS_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo ""
echo "=== Benchfile Comparison ==="
echo "File: $EPD_FILE"
echo "Depth: $DEPTH"
echo "Limit: $LIMIT positions"
echo ""

# Algorithms to test
ALGORITHMS=("FullAlphaBeta" "PureAlphaBeta" "Mtdf" "NegaScout" "MCTS")

for algo in "${ALGORITHMS[@]}"; do
    echo ""
    echo "====================================================="
    echo "Algorithm: $algo"
    echo "====================================================="

    OUTPUT_FILE="$RESULTS_DIR/benchfile_${algo}_${TIMESTAMP}.txt"

    # Run benchfile command via UCI
    echo -e "setoption name Algorithm value $algo\nbenchfile $EPD_FILE $DEPTH $LIMIT\nquit" | \
        "$AETHER_BIN" | tee "$OUTPUT_FILE"

    echo ""
    echo "Results saved to: $OUTPUT_FILE"
done

echo ""
echo "=== Comparison Complete ==="
echo "Results saved to: $RESULTS_DIR/"
