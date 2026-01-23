mod eval;
pub mod search;

use crate::search::{SearchInfo, SearchLimits, SearchResult, Searcher, SearcherType, create_searcher};
use aether_core::{Move, Score};
use board::Board;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

pub use search::SearcherType as AlgorithmType;

pub struct Engine {
    searcher: Box<dyn Searcher>,
    current_type: SearcherType,
    tt_size_mb: usize,
}

impl Engine {
    #[must_use]
    pub fn new(hash_size_mb: usize) -> Self {
        Self::with_searcher_type(SearcherType::default(), hash_size_mb)
    }

    #[must_use]
    pub fn with_searcher_type(searcher_type: SearcherType, hash_size_mb: usize) -> Self {
        let searcher = create_searcher(searcher_type, hash_size_mb);

        Self {
            searcher,
            current_type: searcher_type,
            tt_size_mb: hash_size_mb,
        }
    }

    /// Change the search algorithm
    pub fn set_searcher_type(&mut self, searcher_type: SearcherType) {
        if self.current_type != searcher_type {
            self.searcher = create_searcher(searcher_type, self.tt_size_mb);
            self.current_type = searcher_type;
        }
    }

    /// Get the current search algorithm type
    #[must_use]
    pub fn searcher_type(&self) -> SearcherType {
        self.current_type
    }

    /// Get the algorithm name
    #[must_use]
    pub fn algorithm_name(&self) -> &'static str {
        self.searcher.algorithm_name()
    }

    #[must_use]
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.searcher.stop_flag()
    }

    pub fn stop(&mut self) {
        self.searcher.stop();
    }

    pub fn new_game(&mut self) {
        self.searcher.new_game();
    }

    pub fn resize_tt(&mut self, size_mb: usize) {
        self.tt_size_mb = size_mb;
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
        mut on_info: impl FnMut(&SearchInfo, Option<Move>, Score),
    ) -> SearchResult {
        let limits = self.create_search_limits(depth, time_limit, hard_limit, nodes, infinite);
        self.searcher.search(board, &limits, &mut on_info)
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

    #[must_use]
    pub fn perft(&self, board: &mut Board, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut moves = Vec::new();
        movegen::legal(board, &mut moves);

        if depth == 1 {
            return moves.len() as u64;
        }

        let mut nodes = 0u64;
        for mv in moves {
            board.make_move(&mv).expect("legal move should not fail");
            nodes += self.perft(board, depth - 1);
            board.unmake_move(&mv).expect("unmake should not fail");
        }

        nodes
    }

    #[must_use]
    pub fn perft_divide(&self, board: &mut Board, depth: u8) -> Vec<(Move, u64)> {
        if depth == 0 {
            return Vec::new();
        }

        let mut moves = Vec::new();
        movegen::legal(board, &mut moves);

        let mut results = Vec::new();

        for mv in moves {
            board.make_move(&mv).expect("legal move should not fail");
            let nodes = self.perft(board, depth - 1);
            board.unmake_move(&mv).expect("unmake should not fail");
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
    use std::sync::atomic::Ordering;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(16);
        assert_eq!(engine.hashfull(), 0);
        assert_eq!(engine.searcher_type(), SearcherType::FullAlphaBeta);
    }

    #[test]
    fn test_engine_with_searcher_type() {
        let engine = Engine::with_searcher_type(SearcherType::PureAlphaBeta, 16);
        assert_eq!(engine.searcher_type(), SearcherType::PureAlphaBeta);
        assert_eq!(engine.algorithm_name(), "Pure Alpha-Beta");
    }

    #[test]
    fn test_change_searcher_type() {
        let mut engine = Engine::new(16);
        assert_eq!(engine.searcher_type(), SearcherType::FullAlphaBeta);

        engine.set_searcher_type(SearcherType::NegaScout);
        assert_eq!(engine.searcher_type(), SearcherType::NegaScout);
        assert_eq!(engine.algorithm_name(), "NegaScout");
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
    fn test_search_with_different_algorithms() {
        let mut board = Board::starting_position().unwrap();

        for searcher_type in SearcherType::all() {
            let mut engine = Engine::with_searcher_type(*searcher_type, 16);

            let limits = if *searcher_type == SearcherType::Mcts
                || *searcher_type == SearcherType::ClassicMcts
            {
                // MCTS variants use nodes instead of depth
                (None, Some(1000u64))
            } else {
                (Some(2u8), None)
            };

            let result = engine.search(
                &mut board,
                limits.0,
                None,
                None,
                limits.1,
                false,
                |_, _, _| {},
            );

            assert!(
                result.best_move.is_some(),
                "Algorithm {:?} should find a move",
                searcher_type
            );
        }
    }

    #[test]
    fn test_perft_starting_position() {
        let engine = Engine::new(16);
        let mut board = Board::starting_position().unwrap();

        assert_eq!(engine.perft(&mut board, 1), 20);
        assert_eq!(engine.perft(&mut board, 2), 400);
        assert_eq!(engine.perft(&mut board, 3), 8902);
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
