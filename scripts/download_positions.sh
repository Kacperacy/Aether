#!/bin/bash
# Download test positions from Stockfish Books repository
# https://github.com/official-stockfish/books

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
POSITIONS_DIR="$PROJECT_DIR/positions"

STOCKFISH_BOOKS_RAW="https://raw.githubusercontent.com/official-stockfish/books/master"

echo "Downloading test positions from Stockfish Books..."
echo "Target directory: $POSITIONS_DIR"
echo ""

mkdir -p "$POSITIONS_DIR"/{opening,middlegame,endgame}

# Function to download with progress
download_file() {
    local url="$1"
    local output="$2"
    local name="$3"

    echo "Downloading $name..."
    if command -v curl &> /dev/null; then
        curl -L --progress-bar "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget --show-progress -q "$url" -O "$output"
    else
        echo "Error: curl or wget required"
        exit 1
    fi
    echo "  -> Saved to $output"
}

# Opening positions (after 3 moves from start)
echo ""
echo "=== Opening Positions ==="
download_file \
    "$STOCKFISH_BOOKS_RAW/noob_3moves.epd.zip" \
    "/tmp/noob_3moves.epd.zip" \
    "noob_3moves.epd"

if command -v unzip &> /dev/null; then
    unzip -o -q /tmp/noob_3moves.epd.zip -d "$POSITIONS_DIR/opening/"
    rm /tmp/noob_3moves.epd.zip
else
    echo "Warning: unzip not found, keeping zip file"
    mv /tmp/noob_3moves.epd.zip "$POSITIONS_DIR/opening/"
fi

# Alternative: smaller opening book
download_file \
    "$STOCKFISH_BOOKS_RAW/2moves_v1.epd.zip" \
    "/tmp/2moves_v1.epd.zip" \
    "2moves_v1.epd (alternative)"

if command -v unzip &> /dev/null; then
    unzip -o -q /tmp/2moves_v1.epd.zip -d "$POSITIONS_DIR/opening/"
    rm /tmp/2moves_v1.epd.zip
fi

# Endgame positions
echo ""
echo "=== Endgame Positions ==="
download_file \
    "$STOCKFISH_BOOKS_RAW/endgames.epd.zip" \
    "/tmp/endgames.epd.zip" \
    "endgames.epd"

if command -v unzip &> /dev/null; then
    unzip -o -q /tmp/endgames.epd.zip -d "$POSITIONS_DIR/endgame/"
    rm /tmp/endgames.epd.zip
fi

# Middlegame positions - these are larger, download selectively
echo ""
echo "=== Middlegame Positions ==="
echo "Note: Full middlegame books are very large (100MB+)."
echo "Downloading smaller balanced set..."

download_file \
    "$STOCKFISH_BOOKS_RAW/Drawkiller_balanced_big.epd.zip" \
    "/tmp/drawkiller.epd.zip" \
    "Drawkiller_balanced_big.epd"

if command -v unzip &> /dev/null; then
    unzip -o -q /tmp/drawkiller.epd.zip -d "$POSITIONS_DIR/middlegame/"
    rm /tmp/drawkiller.epd.zip
fi

# Summary
echo ""
echo "=== Download Complete ==="
echo ""
echo "Files downloaded:"
find "$POSITIONS_DIR" -name "*.epd" -exec wc -l {} \; 2>/dev/null | while read count file; do
    echo "  $file: $count positions"
done

echo ""
echo "Usage with fastchess:"
echo "  fastchess -openings file=positions/opening/noob_3moves.epd format=epd ..."
echo ""
echo "Usage with benchfile command:"
echo "  ./target/release/aether"
echo "  > benchfile positions/opening/noob_3moves.epd 10"
