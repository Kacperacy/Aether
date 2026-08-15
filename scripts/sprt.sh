#!/usr/bin/env bash
# Sequential probability ratio test — the accept/reject gate for a strength patch.
#
#   scripts/sprt.sh <baseline-binary> <candidate-binary> [elo0] [elo1]
#
# Defaults test H0: elo <= 0 against H1: elo >= 5, alpha = beta = 0.05.
# Use elo0=-5 elo1=0 for a "does not regress" (simplification) test instead.
set -euo pipefail
cd "$(dirname "$0")/.."

BASE="${1:?usage: sprt.sh <baseline> <candidate> [elo0] [elo1]}"
CAND="${2:?usage: sprt.sh <baseline> <candidate> [elo0] [elo1]}"
ELO0="${3:-0}"
ELO1="${4:-5}"
CONCURRENCY="${CONCURRENCY:-$(( $(getconf _NPROCESSORS_ONLN) - 2 ))}"

command -v fastchess >/dev/null || { echo "fastchess not found in PATH" >&2; exit 1; }

exec fastchess \
  -engine "cmd=$BASE" name=Baseline \
  -engine "cmd=$CAND" name=Candidate \
  -each proto=uci "tc=${TC:-8+0.08}" \
  -openings file=scripts/openings.epd format=epd order=random \
  -rounds 25000 -games 2 -repeat \
  -sprt elo0="$ELO0" elo1="$ELO1" alpha=0.05 beta=0.05 model=normalized \
  -concurrency "$CONCURRENCY" -recover \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=5 score=600 twosided=true \
  -maxmoves 200 -ratinginterval 50
