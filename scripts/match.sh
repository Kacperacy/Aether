#!/usr/bin/env bash
# Quick A/B match between two engine binaries.
#
#   scripts/match.sh <baseline-binary> <candidate-binary> [games]
#
# Deliberately fast: a short time control so the feedback loop stays in the
# tens of seconds. This is a smoke test — it catches breakage, it does not
# prove a small gain. Use scripts/sprt.sh for a verdict.
#
# Override via env:
#   TC=8+0.08          longer control (slower, less time-scaling noise)
#   NODES=50000        fixed nodes per move instead of a clock: removes all
#                      timing jitter, ideal for pure search changes
#   CONCURRENCY=6
set -euo pipefail
cd "$(dirname "$0")/.."

BASE="${1:?usage: match.sh <baseline> <candidate> [games]}"
CAND="${2:?usage: match.sh <baseline> <candidate> [games]}"
GAMES="${3:-200}"
ROUNDS=$(( GAMES / 2 ))
CONCURRENCY="${CONCURRENCY:-$(( $(getconf _NPROCESSORS_ONLN) - 2 ))}"
TC="${TC:-2+0.02}"

if [ -n "${NODES:-}" ]; then
  LIMIT=(nodes="$NODES")
else
  LIMIT=(tc="$TC")
fi

command -v fastchess >/dev/null || { echo "fastchess not found in PATH" >&2; exit 1; }

exec fastchess \
  -engine "cmd=$BASE" name=Baseline \
  -engine "cmd=$CAND" name=Candidate \
  -each proto=uci "${LIMIT[@]}" \
  -openings file=scripts/openings.epd format=epd order=random \
  -rounds "$ROUNDS" -games 2 -repeat \
  -concurrency "$CONCURRENCY" -recover \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=5 score=600 twosided=true \
  -maxmoves 200 -ratinginterval 20
