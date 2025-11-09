#!/bin/bash
# Skrypt do testowania Aether vs Stockfish przy użyciu cutechess-cli
#
# Użycie:
#   ./run_tournament.sh [opcje]
#
# Opcje:
#   --level N        Poziom Stockfish (0-20, domyślnie 5)
#   --games N        Liczba gier (domyślnie 100)
#   --tc TIME        Time control (domyślnie "40/60+0.6" = 60s na 40 ruchów + 0.6s increment)
#   --concurrency N  Liczba równoległych gier (domyślnie 8)
#   --output FILE    Plik wyjściowy PGN (domyślnie results_TIMESTAMP.pgn)

set -e

# Domyślne wartości
STOCKFISH_LEVEL=5
GAMES=100
TIME_CONTROL="40/60+0.6"  # 60s na 40 ruchów + 0.6s increment (symuluje 5+3 na Lichess)
CONCURRENCY=8
OUTPUT_FILE=""
STOCKFISH_PATH=$(which stockfish 2>/dev/null || echo "stockfish")
AETHER_PATH="$(pwd)/target/release/aether"

# Parsowanie argumentów
while [[ $# -gt 0 ]]; do
    case $1 in
        --level)
            STOCKFISH_LEVEL="$2"
            shift 2
            ;;
        --games)
            GAMES="$2"
            shift 2
            ;;
        --tc)
            TIME_CONTROL="$2"
            shift 2
            ;;
        --concurrency)
            CONCURRENCY="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        *)
            echo "Nieznana opcja: $1"
            echo "Użycie: $0 [--level N] [--games N] [--tc TIME] [--concurrency N] [--output FILE]"
            exit 1
            ;;
    esac
done

# Utwórz nazwę pliku wyjściowego jeśli nie podano
if [ -z "$OUTPUT_FILE" ]; then
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    OUTPUT_FILE="results_sf${STOCKFISH_LEVEL}_${TIMESTAMP}.pgn"
fi

# Sprawdź czy silniki istnieją
if [ ! -f "$AETHER_PATH" ]; then
    echo "BŁĄD: Nie znaleziono Aether UCI w: $AETHER_PATH"
    echo "Zbuduj silnik: cargo build --release"
    exit 1
fi

if ! command -v cutechess-cli &> /dev/null; then
    echo "BŁĄD: cutechess-cli nie jest zainstalowany"
    echo "Zainstaluj: sudo apt-get install cutechess  (lub zobacz setup_chess_testing.sh)"
    exit 1
fi

if ! command -v $STOCKFISH_PATH &> /dev/null; then
    echo "BŁĄD: Stockfish nie jest zainstalowany"
    echo "Zainstaluj: sudo apt-get install stockfish"
    exit 1
fi

# Wyświetl konfigurację
echo "========================================="
echo "  Turniej: Aether vs Stockfish"
echo "========================================="
echo "Aether:           $AETHER_PATH"
echo "Stockfish:        $STOCKFISH_PATH (poziom $STOCKFISH_LEVEL)"
echo "Liczba gier:      $GAMES"
echo "Time control:     $TIME_CONTROL"
echo "Równoległość:     $CONCURRENCY"
echo "Plik wynikowy:    $OUTPUT_FILE"
echo "========================================="
echo ""
echo "Rozpoczynanie turnieju..."
echo ""

# Oblicz liczbę rund (każda runda = 2 gry z zamianą kolorów)
ROUNDS=$((GAMES / 2))

# Uruchom turniej
cutechess-cli \
    -engine cmd="$AETHER_PATH" name="Aether" proto=uci \
        option."Move Overhead"=100 \
    -engine cmd="$STOCKFISH_PATH" name="Stockfish-L${STOCKFISH_LEVEL}" proto=uci \
        option."Skill Level"=$STOCKFISH_LEVEL \
    -each tc="$TIME_CONTROL" \
    -openings file="$(pwd)/openings.epd" format=epd order=random \
    -rounds $ROUNDS \
    -games 2 \
    -repeat \
    -concurrency $CONCURRENCY \
    -pgnout "$OUTPUT_FILE" \
    -ratinginterval 10 \
    -recover

echo ""
echo "========================================="
echo "Turniej zakończony!"
echo "========================================="
echo "Wyniki zapisane w: $OUTPUT_FILE"
echo ""
echo "Analiza wyników:"
if [ -f "$OUTPUT_FILE" ]; then
    # Count all games that finished
    TOTAL_GAMES=$(grep -c '^\[Result' "$OUTPUT_FILE" 2>/dev/null || echo 0)

    # Count 1-0 results (White wins)
    WHITE_WINS=$(grep -c '\[Result "1-0"\]' "$OUTPUT_FILE" 2>/dev/null || echo 0)

    # Count 0-1 results (Black wins)
    BLACK_WINS=$(grep -c '\[Result "0-1"\]' "$OUTPUT_FILE" 2>/dev/null || echo 0)

    # Count draws
    DRAWS=$(grep -c '\[Result "1/2-1/2"\]' "$OUTPUT_FILE" 2>/dev/null || echo 0)

    echo "Rozegrane gry:      $TOTAL_GAMES"
    echo "Wygrane białe:      $WHITE_WINS"
    echo "Wygrane czarne:     $BLACK_WINS"
    echo "Remisy:             $DRAWS"
    echo ""

    if [ $TOTAL_GAMES -gt 0 ]; then
        # Approximate 50-50 split of colors between engines
        AETHER_SCORE=$(echo "scale=1; (($WHITE_WINS + $BLACK_WINS) / 2 + $DRAWS * 0.5) / $TOTAL_GAMES * 100" | bc 2>/dev/null || echo "N/A")
        echo "Przybliżony wynik Aethera: ~${AETHER_SCORE}%"
        echo "(Dokładny wynik wymaga analizy PGN - użyj analyze_results.sh)"
    fi
fi
echo ""
echo "Aby zobaczyć szczegóły, otwórz plik PGN w programie do analizy szachowej."
