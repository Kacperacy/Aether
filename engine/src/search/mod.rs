//! Search crate

pub mod alpha_beta;
mod move_ordering;
pub mod tt;

pub use tt::{NodeType, TTEntry, TranspositionTable};

use aether_core::{Move, Score};
use board::BoardQuery;
use std::time::Duration;

const MAX_PLY: usize = 128;

/// Searcher for the best move in a given position
pub trait Searcher {
    /// Performs a search on the given board with the specified limits
    fn search<T: BoardQuery + Clone + 'static>(
        &mut self,
        board: &T,
        limits: &SearchLimits,
    ) -> SearchResult;

    /// Returns information about the current search
    fn get_info(&self) -> &SearchInfo;

    /// Stops the current search
    fn stop(&mut self);
}

#[derive(Debug, Clone)]
pub struct SearchLimits {
    /// Maximum search depth
    pub depth: Option<u8>,

    /// Maximum number of nodes to search
    pub nodes: Option<u64>,

    /// Maximum time to search (soft limit - finish current iteration)
    pub time: Option<Duration>,

    /// Hard time limit for the search (terminate immediately when reached)
    pub hard_time: Option<Duration>,

    /// Whether to search indefinitely until stopped
    pub infinite: bool,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            depth: Some(3), // Default depth of 3
            nodes: None,
            time: None,
            hard_time: None,
            infinite: false,
        }
    }
}

impl SearchLimits {
    /// Creates new search limits with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates search limits with a specified depth
    pub fn depth(depth: u8) -> Self {
        Self {
            depth: Some(depth),
            ..Self::default()
        }
    }

    /// Creates search limits with a specified number of nodes
    pub fn nodes(nodes: u64) -> Self {
        Self {
            depth: None, // No depth limit - iterate until node count reached
            nodes: Some(nodes),
            time: None,
            hard_time: None,
            infinite: false,
        }
    }

    /// Creates search limits with a specified time duration
    pub fn time(time: Duration) -> Self {
        Self {
            depth: None, // No depth limit - iterate until time runs out
            nodes: None,
            time: Some(time),
            hard_time: None,
            infinite: false,
        }
    }

    /// Creates search limits with specified time and hard time limits
    pub fn time_with_hard_limit(time: Duration, hard_time: Duration) -> Self {
        Self {
            depth: None, // No depth limit - iterate until time runs out
            nodes: None,
            time: Some(time),
            hard_time: Some(hard_time),
            infinite: false,
        }
    }

    /// Creates search limits for an infinite search
    pub fn infinite() -> Self {
        Self {
            depth: Some(128), // Large depth limit for infinite search
            nodes: None,
            time: None,
            hard_time: None,
            infinite: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    /// Current depth of the search
    pub depth: u8,

    /// Selective search depth (maximum depth reached including extensions)
    pub selective_depth: u8,

    /// Number of nodes searched so far
    pub nodes: u64,

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

impl SearchInfo {
    /// Creates new search info with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates nodes per second based on time elapsed
    pub fn calculate_nps(&mut self) {
        if self.time_elapsed.as_millis() > 0 {
            self.nps = (self.nodes as u128 * 1000 / self.time_elapsed.as_millis()) as u64;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Best move found
    pub best_move: Option<Move>,

    /// Score of the best move
    pub score: Score,

    /// Principal variation (best line of moves)
    pub pv: Vec<Move>,

    /// Search statistics
    pub info: SearchInfo,
}

impl SearchResult {
    /// Creates new search result with default values
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
