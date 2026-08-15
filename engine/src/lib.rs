pub mod eval;
pub mod search;

use crate::eval::Score;
use crate::eval::SimpleEvaluator;
use crate::search::alpha_beta::AlphaBetaSearcher;
use crate::search::{SearchInfo, SearchLimits, SearchResult};
use aether_core::Move;
use board::Board;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// Default transposition-table size in MiB, and the UCI `Hash` option default.
pub const DEFAULT_HASH_MB: usize = 16;
/// Bounds accepted for the UCI `Hash` option.
pub const MIN_HASH_MB: usize = 1;
pub const MAX_HASH_MB: usize = 1024;

pub struct Engine {
    searcher: AlphaBetaSearcher<SimpleEvaluator>,
}

impl Engine {
    #[must_use]
    pub fn new(hash_size_mb: usize) -> Self {
        let evaluator = SimpleEvaluator::new();
        let searcher = AlphaBetaSearcher::new(evaluator, hash_size_mb);

        Self { searcher }
    }

    #[must_use]
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.searcher.stop_flag()
    }

    pub fn stop(&mut self) {
        self.searcher.stop();
    }

    pub fn new_game(&mut self) {
        self.searcher.clear_tt();
        self.searcher.clear_move_ordering();
    }

    pub fn resize_tt(&mut self, size_mb: usize) {
        self.searcher.resize_tt(size_mb);
    }

    #[must_use]
    pub fn hashfull(&self) -> u16 {
        self.searcher.hashfull()
    }

    #[must_use]
    pub fn legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        movegen::legal(board, &mut moves);
        moves
    }

    pub fn search(
        &mut self,
        board: &mut Board,
        depth: Option<u8>,
        time_limit: Option<Duration>,
        hard_limit: Option<Duration>,
        nodes: Option<u64>,
        infinite: bool,
        on_info: impl FnMut(&SearchInfo, Option<Move>, Score),
    ) -> SearchResult {
        let limits = self.create_search_limits(depth, time_limit, hard_limit, nodes, infinite);
        self.searcher.search(board, &limits, on_info)
    }

    fn create_search_limits(
        &self,
        depth: Option<u8>,
        time_limit: Option<Duration>,
        hard_limit: Option<Duration>,
        nodes: Option<u64>,
        infinite: bool,
    ) -> SearchLimits {
        if infinite {
            return SearchLimits::infinite();
        }

        if let Some(n) = nodes {
            return SearchLimits::nodes(n);
        }

        if let (Some(soft), Some(hard)) = (time_limit, hard_limit) {
            SearchLimits::time_with_hard_limit(soft, hard)
        } else if let Some(t) = time_limit {
            SearchLimits::time(t)
        } else if let Some(d) = depth {
            SearchLimits::depth(d)
        } else {
            SearchLimits::default()
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(DEFAULT_HASH_MB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(16);
        assert_eq!(engine.hashfull(), 0);
    }

    #[test]
    fn test_legal_moves() {
        let engine = Engine::new(16);
        let board = Board::starting_position().unwrap();
        let moves = engine.legal_moves(&board);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_search_starting_position() {
        let mut engine = Engine::new(16);
        let mut board = Board::starting_position().unwrap();

        let result = engine.search(&mut board, Some(3), None, None, None, false, |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(!result.pv.is_empty());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_stop_search() {
        let mut engine = Engine::new(16);
        let stop_flag = engine.stop_flag();

        engine.stop();
        assert!(stop_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_new_game_clears_tt() {
        let mut engine = Engine::new(16);
        let mut board = Board::starting_position().unwrap();

        engine.search(&mut board, Some(6), None, None, None, false, |_, _, _| {});
        assert!(engine.hashfull() > 0);

        engine.new_game();
        assert_eq!(engine.hashfull(), 0);
    }
}
