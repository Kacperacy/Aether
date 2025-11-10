//! Search information and result types.
//!
//! This module defines the data structures used to:
//! - Configure search limits (`SearchLimits`)
//! - Track real-time search statistics (`SearchInfo`)
//! - Return search results (`SearchResult`)

use aether_types::Move;
use eval::Score;
use std::time::Duration;

/// Search constraints and limits.
///
/// Allows configuration of search depth, time, or node limits.
///
/// # Examples
///
/// ```
/// use search::SearchLimits;
/// use std::time::Duration;
///
/// // Search to depth 6
/// let limits = SearchLimits::depth(6);
///
/// // Search for 5 seconds
/// let limits = SearchLimits::time(Duration::from_secs(5));
///
/// // Search 1 million nodes
/// let limits = SearchLimits::nodes(1_000_000);
///
/// // Infinite search (until stopped)
/// let limits = SearchLimits::infinite();
/// ```
#[derive(Debug, Clone)]
pub struct SearchLimits {
    /// Maximum search depth in plies (half-moves)
    pub depth: Option<u8>,

    /// Maximum number of nodes to search
    pub nodes: Option<u64>,

    /// Maximum time to search
    pub time: Option<Duration>,

    /// Search infinitely until stopped
    pub infinite: bool,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            depth: Some(5), // Default to depth 5
            nodes: None,
            time: None,
            infinite: false,
        }
    }
}

impl SearchLimits {
    /// Create search limits with only depth constraint.
    ///
    /// # Example
    ///
    /// ```
    /// use search::SearchLimits;
    ///
    /// let limits = SearchLimits::depth(8);
    /// assert_eq!(limits.depth, Some(8));
    /// ```
    pub fn depth(depth: u8) -> Self {
        Self {
            depth: Some(depth),
            nodes: None,
            time: None,
            infinite: false,
        }
    }

    /// Create search limits with only time constraint
    pub fn time(duration: Duration) -> Self {
        Self {
            depth: None,
            nodes: None,
            time: Some(duration),
            infinite: false,
        }
    }

    /// Create search limits with only node constraint
    pub fn nodes(nodes: u64) -> Self {
        Self {
            depth: None,
            nodes: Some(nodes),
            time: None,
            infinite: false,
        }
    }

    /// Create infinite search limits (search until stopped)
    pub fn infinite() -> Self {
        Self {
            depth: None,
            nodes: None,
            time: None,
            infinite: true,
        }
    }
}

/// Real-time search statistics.
///
/// Tracks information during search for progress updates and UCI output.
#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    /// Current search depth
    pub depth: u8,

    /// Selective search depth (maximum depth reached including extensions)
    pub selective_depth: u8,

    /// Number of nodes searched
    pub nodes: u64,

    /// Time elapsed
    pub time_elapsed: Duration,

    /// Principal variation (best line found)
    pub pv: Vec<Move>,

    /// Current best score
    pub score: Score,

    /// Nodes per second
    pub nps: u64,

    /// Hash table usage (percentage, 0-1000 for per-mille)
    pub hash_full: u16,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate nodes per second
    pub fn calculate_nps(&mut self) {
        let seconds = self.time_elapsed.as_secs_f64();
        if seconds > 0.0 {
            self.nps = (self.nodes as f64 / seconds) as u64;
        }
    }
}

/// Result of a search.
///
/// Contains the best move found and associated statistics.
///
/// # Example
///
/// ```ignore
/// use search::{AlphaBetaSearcher, Searcher, SearchLimits};
///
/// let mut searcher = AlphaBetaSearcher::new();
/// let result = searcher.search(&board, &SearchLimits::depth(6));
///
/// if let Some(best_move) = result.best_move {
///     println!("Best move: {} with score {}", best_move, result.score);
///     println!("Nodes searched: {}", result.info.nodes);
///     println!("NPS: {}", result.info.nps);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Best move found
    pub best_move: Option<Move>,

    /// Score of the best move (from side-to-move perspective)
    pub score: Score,

    /// Principal variation (best line)
    pub pv: Vec<Move>,

    /// Search statistics
    pub info: SearchInfo,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(best_move: Option<Move>, score: Score) -> Self {
        Self {
            best_move,
            score,
            pv: Vec::new(),
            info: SearchInfo::new(),
        }
    }

    /// Create a search result with full information
    pub fn with_info(best_move: Option<Move>, score: Score, pv: Vec<Move>, info: SearchInfo) -> Self {
        Self {
            best_move,
            score,
            pv,
            info,
        }
    }
}
