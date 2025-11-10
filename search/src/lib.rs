//! Search crate
//!
//! Responsibilities:
//! - Implement game-tree search (e.g., minimax, alpha-beta, iterative deepening, quiescence).
//! - Consume move generation (`movegen`) and evaluation (`eval`) without embedding policy into lower layers.
//! - Expose a clean API for the engine to request best moves and principal variations.
//!
//! This crate should avoid direct UI/CLI dependencies and remain compute-focused.

mod alpha_beta;
mod move_ordering;
mod search_info;
mod transposition_table;

pub use alpha_beta::AlphaBetaSearcher;
pub use move_ordering::{AdvancedMoveOrderer, MoveOrderer, SimpleMoveOrderer};
pub use search_info::{SearchInfo, SearchLimits, SearchResult};
pub use transposition_table::{EntryType, TTEntry, TranspositionTable};

use aether_types::{BoardQuery, Move};
use eval::Evaluator;

/// Trait for chess position search algorithms.
///
/// This trait allows different search implementations (alpha-beta, MCTS, etc.)
/// to be used interchangeably by the engine.
///
/// # Example
///
/// ```ignore
/// use search::{AlphaBetaSearcher, Searcher, SearchLimits};
/// use board::Board;
///
/// let mut searcher = AlphaBetaSearcher::new();
/// let board = Board::starting_position().unwrap();
/// let limits = SearchLimits::depth(6);
///
/// let result = searcher.search(&board, &limits);
/// println!("Best move: {}", result.best_move.unwrap());
/// println!("Score: {}", result.score);
/// ```
pub trait Searcher {
    /// Search for the best move in the current position
    ///
    /// # Arguments
    /// * `board` - The current board position (must support BoardQuery + BoardOps)
    /// * `limits` - Search constraints (depth, time, nodes)
    ///
    /// # Returns
    /// SearchResult containing the best move, score, and search statistics
    fn search<T>(&mut self, board: &T, limits: &SearchLimits) -> SearchResult
    where
        T: BoardQuery + Clone + 'static;

    /// Get the current search information (for real-time updates)
    fn get_info(&self) -> &SearchInfo;

    /// Stop the search early (for time management)
    fn stop(&mut self);
}

/// Helper trait for boards that can make/unmake moves
///
/// This is separate from BoardQuery to allow search to work with
/// any board implementation that supports move execution
pub trait SearchableBoard: BoardQuery + Clone {
    /// Make a move and return a Result indicating success
    fn make_move(&mut self, mv: Move) -> Result<(), String>;

    /// Unmake the last move
    fn unmake_move(&mut self, mv: Move) -> Result<(), String>;
}

/// Generic search function that works with any Searcher implementation.
///
/// This allows the engine to be agnostic about the search algorithm used.
///
/// # Example
///
/// ```ignore
/// let result = find_best_move(&board, &mut searcher, &evaluator, &limits);
/// if let Some(mv) = result.best_move {
///     println!("Playing: {}", mv);
/// }
/// ```
pub fn find_best_move<T, S, E>(
    board: &T,
    searcher: &mut S,
    _evaluator: &E,
    limits: &SearchLimits,
) -> SearchResult
where
    T: BoardQuery + Clone + 'static,
    S: Searcher,
    E: Evaluator,
{
    searcher.search(board, limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_limits_default() {
        let limits = SearchLimits::default();
        assert!(limits.depth.is_some());
    }
}
