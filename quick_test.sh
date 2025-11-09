#!/bin/bash
# Szybki test - 10 gier vs Stockfish poziom 5 w szybkim time control

echo "=== Szybki test Aether vs Stockfish (10 gier) ==="
echo ""

./run_tournament.sh \
    --level 5 \
    --games 10 \
    --tc "10+0.1" \
    --concurrency 2 \
    --output quick_test.pgn
