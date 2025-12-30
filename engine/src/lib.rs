mod eval;
pub mod search;

use crate::eval::SimpleEvaluator;
use crate::search::alpha_beta::AlphaBetaSearcher;
use crate::search::{SearchInfo, SearchLimits, SearchResult};
use aether_core::{Move, Score};
use board::{Board, BoardOps};
use movegen::{Generator, MoveGen};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

pub struct Engine {
    generator: Generator,
    searcher: AlphaBetaSearcher<SimpleEvaluator>,
}

impl Engine {
    #[must_use]
    pub fn new(hash_size_mb: usize) -> Self {
        let evaluator = SimpleEvaluator::new();
        let searcher = AlphaBetaSearcher::new(evaluator, hash_size_mb);

        Self {
            generator: Generator::new(),
            searcher,
        }
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
        self.generator.legal(board, &mut moves);
        moves
    }

    pub fn search(
        &mut self,
        board: &mut Board,
        depth: Option<u8>,
        time_limit: Option<Duration>,
        on_info: impl FnMut(&SearchInfo, Option<Move>, Score),
    ) -> SearchResult {
        // Convert parameters to SearchLimits
        let limits = self.create_search_limits(depth, time_limit);

        // Delegate to AlphaBetaSearcher
        self.searcher.search(board, &limits, on_info)
    }

    fn create_search_limits(
        &self,
        depth: Option<u8>,
        time_limit: Option<Duration>,
    ) -> SearchLimits {
        let soft_limit = time_limit;
        let hard_limit = time_limit.map(|t| {
            // Hard limit = soft limit + 10%, capped at +100ms
            let soft_ms = t.as_millis() as u64;
            let extra = (soft_ms / 10).min(100);
            Duration::from_millis(soft_ms + extra)
        });

        if let (Some(soft), Some(hard)) = (soft_limit, hard_limit) {
            SearchLimits::time_with_hard_limit(soft, hard)
        } else if let Some(d) = depth {
            SearchLimits::depth(d)
        } else {
            SearchLimits::default()
        }
    }

    #[must_use]
    pub fn perft(&self, board: &mut Board, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        if depth == 1 {
            return moves.len() as u64;
        }

        let mut nodes = 0u64;
        for mv in moves {
            board.make_move(&mv).ok();
            nodes += self.perft(board, depth - 1);
            board.unmake_move(&mv).ok();
        }

        nodes
    }

    #[must_use]
    pub fn perft_divide(&self, board: &mut Board, depth: u8) -> Vec<(Move, u64)> {
        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        let mut results = Vec::new();

        for mv in moves {
            board.make_move(&mv).ok();
            let nodes = self.perft(board, depth - 1);
            board.unmake_move(&mv).ok();
            results.push((mv, nodes));
        }

        results
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::Board;
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
        assert_eq!(moves.len(), 20); // 20 legal moves from starting position
    }

    #[test]
    fn test_search_starting_position() {
        let mut engine = Engine::new(16);
        let mut board = Board::starting_position().unwrap();

        let result = engine.search(&mut board, Some(3), None, |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(!result.pv.is_empty());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_perft_starting_position() {
        let engine = Engine::new(16);
        let mut board = Board::starting_position().unwrap();

        // Known perft values for starting position
        assert_eq!(engine.perft(&mut board, 1), 20);
        assert_eq!(engine.perft(&mut board, 2), 400);
        assert_eq!(engine.perft(&mut board, 3), 8902);
    }

    #[test]
    fn test_stop_search() {
        let mut engine = Engine::new(16);
        let stop_flag = engine.stop_flag();

        // Start search in background would go here
        // For now, just test that stop works
        engine.stop();
        assert!(stop_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_new_game_clears_tt() {
        let mut engine = Engine::new(16);
        let mut board = Board::starting_position().unwrap();

        // Do a search to populate TT (deeper search to ensure TT gets populated)
        engine.search(&mut board, Some(6), None, |_, _, _| {});
        assert!(engine.hashfull() > 0);

        // New game should clear TT
        engine.new_game();
        assert_eq!(engine.hashfull(), 0);
    }
}
