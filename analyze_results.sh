#!/bin/bash
# Skrypt do analizy wyników z plików PGN

if [ $# -eq 0 ]; then
    echo "Użycie: $0 <plik.pgn> [plik2.pgn ...]"
    exit 1
fi

for pgn_file in "$@"; do
    if [ ! -f "$pgn_file" ]; then
        echo "BŁĄD: Plik nie istnieje: $pgn_file"
        continue
    fi
    
    echo "========================================="
    echo "  Analiza: $pgn_file"
    echo "========================================="
    
    # Liczenie wyników
    total_games=$(grep -c "^\[Result" "$pgn_file" || echo 0)
    
    # Aether jako białe
    aether_white_wins=$(grep -B10 '^\[Result "1-0"\]' "$pgn_file" | grep -c '^\[White "Aether"\]' || echo 0)
    aether_white_draws=$(grep -B10 '^\[Result "1/2-1/2"\]' "$pgn_file" | grep -c '^\[White "Aether"\]' || echo 0)
    aether_white_losses=$(grep -B10 '^\[Result "0-1"\]' "$pgn_file" | grep -c '^\[White "Aether"\]' || echo 0)
    
    # Aether jako czarne
    aether_black_wins=$(grep -B10 '^\[Result "0-1"\]' "$pgn_file" | grep -c '^\[Black "Aether"\]' || echo 0)
    aether_black_draws=$(grep -B10 '^\[Result "1/2-1/2"\]' "$pgn_file" | grep -c '^\[Black "Aether"\]' || echo 0)
    aether_black_losses=$(grep -B10 '^\[Result "1-0"\]' "$pgn_file" | grep -c '^\[Black "Aether"\]' || echo 0)
    
    # Sumy
    total_wins=$((aether_white_wins + aether_black_wins))
    total_draws=$((aether_white_draws + aether_black_draws))
    total_losses=$((aether_white_losses + aether_black_losses))
    
    echo ""
    echo "Całkowite gry: $total_games"
    echo ""
    echo "Wyniki Aether:"
    echo "  Wygrane:  $total_wins (białe: $aether_white_wins, czarne: $aether_black_wins)"
    echo "  Remisy:   $total_draws (białe: $aether_white_draws, czarne: $aether_black_draws)"
    echo "  Przegrane: $total_losses (białe: $aether_white_losses, czarne: $aether_black_losses)"
    echo ""
    
    # Procent punktów
    if [ $total_games -gt 0 ]; then
        points=$(echo "scale=1; ($total_wins + $total_draws * 0.5)" | bc)
        percentage=$(echo "scale=1; $points / $total_games * 100" | bc)
        echo "Wynik: $points / $total_games ($percentage%)"
        
        # Oszacowanie ELO (jeśli znamy ELO przeciwnika)
        # Formuła: Expected = 1 / (1 + 10^((OpponentElo - PlayerElo)/400))
        # Dla Stockfish level 5 ≈ 1500-1600 ELO
        stockfish_elo=1550
        if (( $(echo "$percentage > 75" | bc -l) )); then
            echo "Szacunkowe ELO: >1700 (dominacja)"
        elif (( $(echo "$percentage > 60" | bc -l) )); then
            echo "Szacunkowe ELO: ~1650-1700 (przewaga)"
        elif (( $(echo "$percentage > 45" | bc -l) )); then
            echo "Szacunkowe ELO: ~1500-1600 (równy)"
        else
            echo "Szacunkowe ELO: <1500 (słabszy)"
        fi
    fi
    
    echo ""
    
    # Średnia długość gry
    if command -v awk &> /dev/null; then
        avg_moves=$(grep "^\[PlyCount" "$pgn_file" | awk -F'"' '{sum+=$2; count++} END {if(count>0) print sum/(count*2)}')
        if [ -n "$avg_moves" ]; then
            echo "Średnia długość gry: $avg_moves ruchów"
        fi
    fi
    
    # Przyczyny końca gry
    echo ""
    echo "Przyczyny zakończenia gry:"
    mate_wins=$(grep -i "checkmate" "$pgn_file" | grep -c "Aether" || echo 0)
    timeout_losses=$(grep -i "time" "$pgn_file" | grep -c "forfeit" || echo 0)
    echo "  Maty Aether: $mate_wins"
    echo "  Przegrane na czas: $timeout_losses"
    
    echo ""
done
