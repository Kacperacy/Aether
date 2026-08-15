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

# A fixed-node search is fully deterministic, so one opening always produces the
# exact same game. If the book has fewer openings than rounds it cycles, and the
# repeats are replayed verbatim — while fastchess still computes its confidence
# interval as though every game were independent. A pure-noise result then looks
# decisive: 10 openings over 100 rounds once reported "52 +/- 35 Elo, LOS 99.9%"
# off what were really 10 distinct games.
#
# Under a clock this does not bite nearly as hard, because timing jitter makes
# repeated openings diverge into genuinely different games.
OPENINGS=$(grep -cve '^[[:space:]]*$' scripts/openings.epd)
if [ "$ROUNDS" -gt "$OPENINGS" ]; then
  if [ -n "${NODES:-}" ]; then
    echo "ERROR: $ROUNDS rounds but only $OPENINGS openings, at a fixed node count." >&2
    echo "Each opening would replay identically $(( ROUNDS / OPENINGS ))x, so the" >&2
    echo "reported error bars would badly understate the true noise." >&2
    echo "Use at most $(( OPENINGS * 2 )) games, or enlarge scripts/openings.epd." >&2
    exit 1
  fi
  echo "WARNING: $ROUNDS rounds but only $OPENINGS openings; openings will repeat." >&2
fi

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
