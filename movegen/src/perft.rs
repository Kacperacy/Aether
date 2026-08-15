//! Perft: count the leaf nodes reachable in `depth` plies.
//!
//! Pure move generation over board state — no search, no evaluation — so it
//! belongs here rather than behind the engine facade.

use crate::legal;
use aether_core::Move;
use board::Board;
use std::time::Duration;

/// Number of leaf nodes at `depth`.
#[must_use]
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    legal(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for mv in moves {
        board.make_move(&mv).expect("legal move should not fail");
        nodes += perft(board, depth - 1);
        board.unmake_move(&mv).expect("unmake should not fail");
    }

    nodes
}

/// Per-root-move node counts, the standard "divide" breakdown.
#[must_use]
pub fn perft_divide(board: &mut Board, depth: u32) -> Vec<(Move, u64)> {
    if depth == 0 {
        return Vec::new();
    }

    let mut moves = Vec::new();
    legal(board, &mut moves);

    let mut results = Vec::with_capacity(moves.len());
    for mv in moves {
        board.make_move(&mv).expect("legal move should not fail");
        let nodes = perft(board, depth - 1);
        board.unmake_move(&mv).expect("unmake should not fail");
        results.push((mv, nodes));
    }

    results
}

/// A divide plus the timing figures a UCI front-end wants to report.
pub struct PerftReport {
    pub moves: Vec<(Move, u64)>,
    pub nodes: u64,
    pub elapsed: Duration,
}

impl PerftReport {
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

/// Run a timed divide.
#[must_use]
pub fn perft_report(board: &mut Board, depth: u32) -> PerftReport {
    let start = std::time::Instant::now();
    let moves = perft_divide(board, depth);
    let elapsed = start.elapsed();
    let nodes = moves.iter().map(|(_, n)| n).sum();

    PerftReport {
        moves,
        nodes,
        elapsed,
    }
}
