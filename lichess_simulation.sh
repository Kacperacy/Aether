#!/bin/bash
# Symulacja warunków Lichess 5+3

echo "=== Symulacja Lichess 5+3 ==="
echo "Stockfish poziom 5 (odpowiednik ~1500-1600 ELO)"
echo ""

./run_tournament.sh \
    --level 5 \
    --games 50 \
    --tc "5+3" \
    --concurrency 1 \
    --output lichess_5plus3.pgn
