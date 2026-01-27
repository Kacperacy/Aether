#!/bin/bash
# Run benchmark comparison across all algorithms on test positions
# Usage: ./scripts/run_comparison.sh [time_ms] [limit]
#   time_ms - Time limit in milliseconds (default: 1000)
#   limit - Max positions per file (default: all)

set -e

TIME_MS=${1:-1000}
LIMIT=${2:-}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_DIR/results"
POSITIONS_DIR="$PROJECT_DIR/positions"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================="
echo "Aether Benchmark Comparison Suite"
echo "=================================="
echo ""
echo "Settings:"
echo "  Time limit: ${TIME_MS}ms"
if [ -n "$LIMIT" ]; then
    echo "  Limit: $LIMIT positions per file"
else
    echo "  Limit: All positions"
fi
echo ""

# Build the engine
echo -e "${YELLOW}Building engine in release mode...${NC}"
cd "$PROJECT_DIR"
cargo build --release 2>/dev/null

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}Build successful!${NC}"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Find all EPD files
if [ ! -d "$POSITIONS_DIR" ]; then
    echo -e "${RED}Positions directory not found: $POSITIONS_DIR${NC}"
    echo "Please create test positions or run ./scripts/download_positions.sh"
    exit 1
fi

EPD_FILES=$(find "$POSITIONS_DIR" -name "*.epd" -type f 2>/dev/null | sort)
if [ -z "$EPD_FILES" ]; then
    echo -e "${RED}No EPD files found in $POSITIONS_DIR${NC}"
    exit 1
fi

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
COMBINED_CSV="$RESULTS_DIR/comparison_${TIMESTAMP}.csv"

echo "Found EPD files:"
echo "$EPD_FILES" | while read -r f; do echo "  - $(basename "$f")"; done
echo ""

# Process each EPD file
FIRST_FILE=1
for EPD_FILE in $EPD_FILES; do
    BASENAME=$(basename "$EPD_FILE" .epd)
    OUTPUT_CSV="$RESULTS_DIR/${BASENAME}_${TIMESTAMP}.csv"

    echo -e "${YELLOW}Processing: $BASENAME${NC}"

    # Build the command
    if [ -n "$LIMIT" ]; then
        CMD="benchexport $EPD_FILE $OUTPUT_CSV $TIME_MS"
    else
        CMD="benchexport $EPD_FILE $OUTPUT_CSV $TIME_MS"
    fi

    # Run the benchmark
    echo "$CMD" | "$PROJECT_DIR/target/release/aether" 2>&1 | tail -n 20

    if [ -f "$OUTPUT_CSV" ]; then
        echo -e "${GREEN}Results saved to: $OUTPUT_CSV${NC}"

        # Append to combined CSV (skip header for subsequent files)
        if [ $FIRST_FILE -eq 1 ]; then
            cat "$OUTPUT_CSV" > "$COMBINED_CSV"
            FIRST_FILE=0
        else
            tail -n +2 "$OUTPUT_CSV" >> "$COMBINED_CSV"
        fi
    else
        echo -e "${RED}Failed to generate results for $BASENAME${NC}"
    fi

    echo ""
done

# Generate summary from combined CSV
if [ -f "$COMBINED_CSV" ]; then
    echo "=================================="
    echo "Combined Results Summary"
    echo "=================================="
    echo ""
    echo "Output file: $COMBINED_CSV"
    echo ""

    # Count records per algorithm
    echo "Records per algorithm:"
    tail -n +2 "$COMBINED_CSV" | cut -d',' -f1 | sort | uniq -c | sort -rn
    echo ""

    # Average NPS per algorithm
    echo "Average NPS per algorithm:"
    awk -F',' 'NR>1 {sum[$1]+=$7; count[$1]++} END {for(a in sum) printf "  %-20s %12.0f\n", a, sum[a]/count[a]}' "$COMBINED_CSV" | sort -k2 -rn
    echo ""

    # Average nodes per algorithm
    echo "Average nodes per algorithm:"
    awk -F',' 'NR>1 {sum[$1]+=$5; count[$1]++} END {for(a in sum) printf "  %-20s %12.0f\n", a, sum[a]/count[a]}' "$COMBINED_CSV" | sort -k2 -n
    echo ""

    # Results by phase
    echo "Records by game phase:"
    tail -n +2 "$COMBINED_CSV" | cut -d',' -f3 | sort | uniq -c
    echo ""

    echo -e "${GREEN}Benchmark comparison complete!${NC}"
else
    echo -e "${RED}No combined results generated${NC}"
fi
