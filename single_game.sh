#!/bin/bash
# Uruchom pojedynczą grę Aether vs Stockfish (do debugowania)

STOCKFISH_LEVEL=${1:-5}
TIME_CONTROL=${2:-"5+3"}

echo "=== Pojedyncza gra: Aether vs Stockfish poziom $STOCKFISH_LEVEL ==="
echo "Time control: $TIME_CONTROL"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT="single_game_${TIMESTAMP}.pgn"

cutechess-cli \
    -engine cmd="$(pwd)/target/release/aether" name="Aether" \
        option."Move Overhead"=100 \
    -engine cmd="stockfish" name="Stockfish-L${STOCKFISH_LEVEL}" \
        option."Skill Level"=$STOCKFISH_LEVEL \
    -each tc="$TIME_CONTROL" \
    -rounds 1 \
    -pgnout "$OUTPUT" \
    -debug

echo ""
echo "Gra zapisana w: $OUTPUT"
echo ""
echo "Wynik:"
cat "$OUTPUT" | grep "^\[Result"
