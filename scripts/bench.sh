#!/usr/bin/env bash
# Search regression check.
#
# The bench node count is deterministic for a given binary. Compare it against
# the value recorded in docs/benchmarks.md: an unexplained change means search
# behaviour moved when you did not intend it to.
#
#   scripts/bench.sh [depth]
set -euo pipefail
cd "$(dirname "$0")/.."

DEPTH="${1:-}"
cargo build --release --bin aether >/dev/null
exec ./target/release/aether bench ${DEPTH:+"$DEPTH"}
