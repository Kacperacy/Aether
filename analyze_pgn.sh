#!/bin/bash
# Dokładna analiza wyników PGN - parsuje plik blok po bloku

if [ $# -eq 0 ]; then
    echo "Użycie: $0 <plik.pgn>"
    exit 1
fi

PGN_FILE="$1"

if [ ! -f "$PGN_FILE" ]; then
    echo "BŁĄD: Plik nie istnieje: $PGN_FILE"
    exit 1
fi

echo "========================================="
echo "  Szczegółowa analiza: $PGN_FILE"
echo "========================================="
echo ""

# Liczniki
AETHER_WHITE_WINS=0
AETHER_WHITE_DRAWS=0
AETHER_WHITE_LOSSES=0
AETHER_BLACK_WINS=0
AETHER_BLACK_DRAWS=0
AETHER_BLACK_LOSSES=0
ILLEGAL_MOVES=0
TOTAL_GAMES=0

# Parsowanie PGN blok po bloku
WHITE=""
BLACK=""
RESULT=""

while IFS= read -r line; do
    if [[ $line =~ ^\[White\ \"([^\"]+)\"\] ]]; then
        WHITE="${BASH_REMATCH[1]}"
    elif [[ $line =~ ^\[Black\ \"([^\"]+)\"\] ]]; then
        BLACK="${BASH_REMATCH[1]}"
    elif [[ $line =~ ^\[Result\ \"([^\"]+)\"\] ]]; then
        RESULT="${BASH_REMATCH[1]}"

        # Mamy komplet danych, zliczamy
        if [ -n "$WHITE" ] && [ -n "$BLACK" ] && [ -n "$RESULT" ]; then
            ((TOTAL_GAMES++))

            # Sprawdź czy Aether grał białymi
            if [[ "$WHITE" == "Aether" ]]; then
                case "$RESULT" in
                    "1-0")
                        ((AETHER_WHITE_WINS++))
                        ;;
                    "1/2-1/2")
                        ((AETHER_WHITE_DRAWS++))
                        ;;
                    "0-1")
                        ((AETHER_WHITE_LOSSES++))
                        ;;
                esac
            fi

            # Sprawdź czy Aether grał czarnymi
            if [[ "$BLACK" == "Aether" ]]; then
                case "$RESULT" in
                    "0-1")
                        ((AETHER_BLACK_WINS++))
                        ;;
                    "1/2-1/2")
                        ((AETHER_BLACK_DRAWS++))
                        ;;
                    "1-0")
                        ((AETHER_BLACK_LOSSES++))
                        ;;
                esac
            fi

            # Reset dla następnej gry
            WHITE=""
            BLACK=""
            RESULT=""
        fi
    fi

    # Zlicz nielegalne ruchy
    if [[ $line =~ "illegal move" ]] && [[ $line =~ "Aether" ]]; then
        ((ILLEGAL_MOVES++))
    fi
done < "$PGN_FILE"

# Obliczenia
TOTAL_WINS=$((AETHER_WHITE_WINS + AETHER_BLACK_WINS))
TOTAL_DRAWS=$((AETHER_WHITE_DRAWS + AETHER_BLACK_DRAWS))
TOTAL_LOSSES=$((AETHER_WHITE_LOSSES + AETHER_BLACK_LOSSES))
AETHER_GAMES=$((TOTAL_WINS + TOTAL_DRAWS + TOTAL_LOSSES))

echo "Całkowite gry w pliku: $TOTAL_GAMES"
echo "Gry Aethera: $AETHER_GAMES"
echo ""

echo "╔════════════════════════════════════════╗"
echo "║        WYNIKI AETHER                   ║"
echo "╠════════════════════════════════════════╣"
printf "║ Wygrane:    %-3d (W:%2d B:%2d)          ║\n" $TOTAL_WINS $AETHER_WHITE_WINS $AETHER_BLACK_WINS
printf "║ Remisy:     %-3d (W:%2d B:%2d)          ║\n" $TOTAL_DRAWS $AETHER_WHITE_DRAWS $AETHER_BLACK_DRAWS
printf "║ Przegrane:  %-3d (W:%2d B:%2d)          ║\n" $TOTAL_LOSSES $AETHER_WHITE_LOSSES $AETHER_BLACK_LOSSES
echo "╚════════════════════════════════════════╝"
echo ""

# Procent punktów
if [ $AETHER_GAMES -gt 0 ]; then
    POINTS=$(echo "scale=1; $TOTAL_WINS + $TOTAL_DRAWS * 0.5" | bc)
    PERCENTAGE=$(echo "scale=1; $POINTS / $AETHER_GAMES * 100" | bc)

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  WYNIK: $POINTS / $AETHER_GAMES pkt ($PERCENTAGE%)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Oszacowanie siły (dla Stockfish level 5 ≈ 1550 ELO)
    echo "Ocena siły gry:"
    if (( $(echo "$PERCENTAGE >= 75" | bc -l) )); then
        echo "  ★★★★★ Dominacja ($PERCENTAGE%)"
        echo "  Szacunkowe ELO: ~1750+ (znacznie silniejszy od SF5)"
    elif (( $(echo "$PERCENTAGE >= 60" | bc -l) )); then
        echo "  ★★★★☆ Silny ($PERCENTAGE%)"
        echo "  Szacunkowe ELO: ~1650-1750 (silniejszy od SF5)"
    elif (( $(echo "$PERCENTAGE >= 55" | bc -l) )); then
        echo "  ★★★☆☆ Dobry ($PERCENTAGE%)"
        echo "  Szacunkowe ELO: ~1600-1650 (lekka przewaga nad SF5)"
    elif (( $(echo "$PERCENTAGE >= 45" | bc -l) )); then
        echo "  ★★★☆☆ Równy ($PERCENTAGE%)"
        echo "  Szacunkowe ELO: ~1500-1600 (podobny do SF5)"
    elif (( $(echo "$PERCENTAGE >= 30" | bc -l) )); then
        echo "  ★★☆☆☆ Słabszy ($PERCENTAGE%)"
        echo "  Szacunkowe ELO: ~1350-1500 (słabszy od SF5)"
    else
        echo "  ★☆☆☆☆ Bardzo słaby ($PERCENTAGE%)"
        echo "  Szacunkowe ELO: <1350 (znacznie słabszy od SF5)"
    fi
    echo ""
fi

# Statystyki dodatkowe
if [ $ILLEGAL_MOVES -gt 0 ]; then
    echo "⚠️  UWAGA: Znaleziono $ILLEGAL_MOVES nielegalnych ruchów Aethera"
    echo ""
fi

# Analiza per kolor
if [ $AETHER_GAMES -gt 0 ]; then
    WHITE_GAMES=$((AETHER_WHITE_WINS + AETHER_WHITE_DRAWS + AETHER_WHITE_LOSSES))
    BLACK_GAMES=$((AETHER_BLACK_WINS + AETHER_BLACK_DRAWS + AETHER_BLACK_LOSSES))

    if [ $WHITE_GAMES -gt 0 ]; then
        WHITE_SCORE=$(echo "scale=1; ($AETHER_WHITE_WINS + $AETHER_WHITE_DRAWS * 0.5) / $WHITE_GAMES * 100" | bc)
        echo "Wynik białymi: $WHITE_SCORE% ($AETHER_WHITE_WINS-$AETHER_WHITE_DRAWS-$AETHER_WHITE_LOSSES)"
    fi

    if [ $BLACK_GAMES -gt 0 ]; then
        BLACK_SCORE=$(echo "scale=1; ($AETHER_BLACK_WINS + $AETHER_BLACK_DRAWS * 0.5) / $BLACK_GAMES * 100" | bc)
        echo "Wynik czarnymi: $BLACK_SCORE% ($AETHER_BLACK_WINS-$AETHER_BLACK_DRAWS-$AETHER_BLACK_LOSSES)"
    fi
    echo ""
fi

echo "Aby zobaczyć szczegóły gier, otwórz $PGN_FILE w analizatorze PGN."
