pub mod alpha_beta;
mod move_ordering;
mod move_picker;
pub mod see;
pub mod tt;

pub use tt::{NodeType, TTEntry, TranspositionTable};

use crate::eval::Score;
use aether_core::Move;
use std::time::Duration;

pub(crate) const MAX_PLY: usize = 128;
const MAX_PV_LENGTH: usize = MAX_PLY;

/// Bounds on a search. **Every field applies simultaneously** — the search stops
/// at whichever fires first. A field left `None` imposes no bound, so an
/// all-`None` value means "search until told to stop".
#[derive(Debug, Clone, Default)]
pub struct SearchLimits {
    /// Maximum iterative-deepening depth.
    pub depth: Option<u8>,
    /// Maximum nodes to visit.
    pub nodes: Option<u64>,
    /// Soft limit: do not *start* a new iteration past this.
    pub time: Option<Duration>,
    /// Hard limit: abandon the search in progress past this.
    pub hard_time: Option<Duration>,
}

impl SearchLimits {
    /// Search to a fixed depth and nothing else. Used by `bench` and by tests,
    /// where reproducibility matters more than wall-clock.
    #[must_use]
    pub fn depth(depth: u8) -> Self {
        Self {
            depth: Some(depth),
            ..Self::default()
        }
    }

    /// True when no bound at all is set, i.e. the search would only end when
    /// the stop flag is raised.
    #[must_use]
    pub const fn is_unbounded(&self) -> bool {
        self.depth.is_none()
            && self.nodes.is_none()
            && self.time.is_none()
            && self.hard_time.is_none()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    pub depth: u8,
    pub selective_depth: u8,
    pub nodes: u64,
    pub time_elapsed: Duration,
    pub pv: Vec<Move>,
    pub nps: u64,
    pub hash_full: u16,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_nps(&mut self) {
        if self.time_elapsed.as_millis() > 0 {
            self.nps = (self.nodes as u128 * 1000 / self.time_elapsed.as_millis()) as u64;
        }
    }
}

/// Outcome of a search. The principal variation, node counts and timing all live
/// on `info` — there is deliberately no second copy of them here, because the two
/// used to be updated separately and drift apart.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: Score,
    pub info: SearchInfo,
}

impl SearchResult {
    /// The principal variation of the last completed iteration.
    #[must_use]
    pub fn pv(&self) -> &[Move] {
        &self.info.pv
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use board::Board;

    /// Parse a test position. Fixtures carry distant kings so the board is
    /// legal without the kings taking part in whatever is under test.
    pub(crate) fn pos(fen: &str) -> Board {
        fen.parse().expect("valid FEN")
    }
}
