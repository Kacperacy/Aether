use crate::{BoardQuery, Move, Score};
use std::time::Duration;

/// Searcher for the best move in a given position.
pub trait Searcher {
    /// Performs a search on the given board with the specified limits.
    fn search<T: BoardQuery + Clone + 'static>(
        &mut self,
        board: &T,
        limits: &SearchLimits,
    ) -> SearchResult;

    /// Returns information about the current search.
    fn get_info(&self) -> &SearchInfo;

    /// Stops the current search.
    fn stop(&mut self);
}

#[derive(Debug, Clone)]
pub struct SearchLimits {
    /// Maximum search depth.
    pub depth: Option<u8>,

    /// Maximum number of nodes to search.
    pub nodes: Option<u64>,

    /// Maximum time to search.
    pub time: Option<Duration>,

    /// Whether to search indefinitely until stopped.
    pub infinite: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    /// Current depth of the search.
    pub depth: u8,

    /// Selective search depth (maximum depth reached including extensions)
    pub selective_depth: u8,

    /// Time elapsed
    pub time_elapsed: Duration,

    /// Principal variation (best line found so far)
    pub pv: Vec<Move>,

    /// Current best score
    pub score: Score,

    /// Nodes pers second
    pub nps: u64,

    /// Hash table usage (percentage, 0-1000 for permille)
    pub hash_full: u16,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Best move found.
    pub best_move: Option<Move>,

    /// Score of the best move.
    pub score: Score,

    /// Principal variation (best line of moves)
    pub pv: Vec<Move>,

    /// Search statistics
    pub info: SearchInfo,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            depth: Some(3), // Default depth of 3
            nodes: None,
            time: None,
            infinite: false,
        }
    }
}

impl SearchLimits {
    /// Creates new search limits with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates search limits with a specified depth.
    pub fn depth(depth: u8) -> Self {
        Self {
            depth: Some(depth),
            ..Self::default()
        }
    }

    /// Creates search limits with a specified number of nodes.
    pub fn nodes(nodes: u64) -> Self {
        Self {
            nodes: Some(nodes),
            ..Self::default()
        }
    }

    /// Creates search limits with a specified time duration.
    pub fn time(time: Duration) -> Self {
        Self {
            time: Some(time),
            ..Self::default()
        }
    }

    /// Creates search limits for an infinite search.
    pub fn infinite() -> Self {
        Self {
            infinite: true,
            ..Self::default()
        }
    }
}

impl SearchInfo {
    /// Creates new search info with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates nodes per second based on time elapsed.
    pub fn calculate_nps(&mut self) {
        let seconds = self.time_elapsed.as_secs_f64();
        if seconds > 0.0 {
            self.nps = (self.nps as f64 / seconds) as u64;
        }
    }
}

impl SearchResult {
    /// Creates new search result with default values.
    pub fn new(best_move: Option<Move>, score: Score) -> Self {
        Self {
            best_move,
            score,
            pv: Vec::new(),
            info: SearchInfo::new(),
        }
    }

    /// Create a search result with full information
    pub fn with_info(
        best_move: Option<Move>,
        score: Score,
        pv: Vec<Move>,
        info: SearchInfo,
    ) -> Self {
        Self {
            best_move,
            score,
            pv,
            info,
        }
    }
}
