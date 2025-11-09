#!/bin/bash
# Pełny test - 100 gier vs różne poziomy Stockfish

echo "=== Pełny test Aether vs Stockfish (100 gier per poziom) ==="
echo ""

for level in 3 5 7; do
    echo ""
    echo "====================================="
    echo "  Testowanie przeciwko poziomowi $level"
    echo "====================================="
    ./run_tournament.sh \
        --level $level \
        --games 100 \
        --tc "40/60+0.6" \
        --concurrency 8 \
        --output "full_test_level${level}.pgn"
    
    echo ""
    echo "Zakończono testy dla poziomu $level"
    echo ""
done

echo ""
echo "====================================="
echo "  Wszystkie testy zakończone!"
echo "====================================="
echo "Wyniki w plikach: full_test_level*.pgn"
