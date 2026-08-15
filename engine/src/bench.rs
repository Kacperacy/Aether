//! Fixed-workload search benchmark.
//!
//! `bench` searches a fixed set of positions to a fixed depth and reports the
//! total node count. That number is the project's regression signal: it is
//! deterministic for a given binary, so it must stay **bit-identical** unless a
//! change was deliberately meant to alter which nodes get searched. A silent
//! change means something moved that you did not intend to move.
//!
//! The NPS figure alongside it is the speed signal — nodes unchanged plus NPS up
//! is an unambiguous optimisation win.

use crate::search::SearchLimits;
use crate::{DEFAULT_HASH_MB, Engine};
use board::Board;
use std::time::{Duration, Instant};

/// Default search depth for `bench`: ~6.4M nodes in ~1.3s on a release build.
/// Deep enough that the node count is a sensitive signal, fast enough to run on
/// every commit.
pub const DEFAULT_BENCH_DEPTH: u8 = 10;

/// Positions covering openings, tactical middlegames, endgames, and the
/// awkward cases (promotions, en passant, stalemate-adjacent, zugzwang).
/// Kept deliberately fixed — changing this list invalidates every recorded
/// node count, so treat it as append-only if it must change at all.
pub const BENCH_POSITIONS: &[&str] = &[
    // Opening / early middlegame
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
    "r1bqkb1r/pp3ppp/2n1pn2/2pp4/3P4/2NBPN2/PPP2PPP/R1BQK2R w KQkq - 0 7",
    // Kiwipete — dense tactics, all castling rights
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    // Sharp middlegames
    "r3r1k1/pp3pbp/1qp1b1p1/2B5/2BP4/Q1n2N2/P4PPP/3R1K1R w - - 4 18",
    "4rrk1/pp1n1pp1/3bp1p1/3p4/3P1P2/2PB1N2/PP4PP/R4RK1 w - - 2 18",
    "2r3k1/1p3pp1/p2p3p/P1nPr3/2P1P3/2N1K2P/1R4P1/3R4 b - - 3 30",
    "r2q1rk1/1b2bppp/p2ppn2/1p6/3NP3/1BN1B3/PPP2PPP/R2Q1RK1 w - - 0 12",
    // Pawn structure / passed pawns
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "8/1p3pp1/7p/5P1P/2k3P1/8/2K2P2/8 w - - 0 1",
    "8/pp2r1k1/2p1p3/3pP2p/1P1P1P1P/P5KR/8/8 w - - 0 1",
    // Promotion-heavy
    "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
    "8/2P5/8/8/8/5k2/6p1/6K1 w - - 0 1",
    // Endgames
    "8/8/8/4k3/8/8/4P3/4K3 w - - 0 1",
    "8/8/1p6/p1p5/P1P5/1P6/8/K1k5 w - - 0 1",
    "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1",
    "8/8/4k3/8/8/4K3/4P3/8 w - - 0 1",
    // Zugzwang-prone (null move must not be trusted here)
    "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1",
    "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1",
    // Positions with unusual legality (pins, en passant)
    "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
    "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQ1RK1 b kq - 0 5",
    "4k3/8/8/8/8/8/1r6/K7 w - - 0 1",
];

/// Result of a bench run.
pub struct BenchResult {
    pub nodes: u64,
    pub elapsed: Duration,
    pub positions: usize,
    pub depth: u8,
}

impl BenchResult {
    /// Nodes per second, or 0 when the run was too fast to time.
    #[must_use]
    pub fn nps(&self) -> u64 {
        let millis = self.elapsed.as_millis();
        if millis == 0 {
            0
        } else {
            (u128::from(self.nodes) * 1000 / millis) as u64
        }
    }
}

/// Search every bench position to `depth`, summing nodes.
///
/// A fresh [`Engine`] is used and `new_game()` is called between positions, so
/// the result does not depend on carry-over transposition-table or history
/// state — that is what makes the node count reproducible.
#[must_use]
pub fn run(depth: u8) -> BenchResult {
    let mut engine = Engine::new(DEFAULT_HASH_MB);
    let mut nodes = 0u64;
    let start = Instant::now();

    for fen in BENCH_POSITIONS {
        let Ok(mut board) = fen.parse::<Board>() else {
            debug_assert!(false, "bench position is not valid FEN: {fen}");
            continue;
        };

        engine.new_game();
        let result = engine.search(&mut board, &SearchLimits::depth(depth), |_, _, _| {});
        nodes += result.info.nodes;
    }

    BenchResult {
        nodes,
        elapsed: start.elapsed(),
        positions: BENCH_POSITIONS.len(),
        depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_bench_positions_are_valid_fen() {
        for fen in BENCH_POSITIONS {
            assert!(
                fen.parse::<Board>().is_ok(),
                "bench position is not valid FEN: {fen}"
            );
        }
    }

    /// The whole point of bench is reproducibility: two runs of the same binary
    /// at the same depth must agree exactly. If this ever fails, the search has
    /// picked up a dependency on state that survives `new_game()`.
    #[test]
    fn test_bench_node_count_is_deterministic() {
        let a = run(4);
        let b = run(4);
        assert_eq!(
            a.nodes, b.nodes,
            "bench must be reproducible across runs of the same binary"
        );
        assert!(a.nodes > 0);
    }
}
