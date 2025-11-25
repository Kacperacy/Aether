mod eval;
pub mod search;

pub use eval::{Evaluator, SimpleEvaluator};
pub use search::{
    NodeType, SearchInfo, SearchLimits, SearchResult, Searcher, TTEntry, TranspositionTable,
};

use aether_core::{Color, MATE_SCORE, Move, NEG_MATE_SCORE, Score};
use board::{Board, BoardOps, BoardQuery};
use movegen::{Generator, MoveGen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const NODE_CHECK_INTERVAL: u64 = 512;

/// Main chess engine facade
pub struct Engine {
    /// Move generator
    generator: Generator,
    /// Position evaluator
    evaluator: SimpleEvaluator,
    /// Transposition table
    tt: TranspositionTable,
    /// Stop flag for interrupting search
    stop_flag: Arc<AtomicBool>,
    /// Search statistics
    info: SearchInfo,
}

impl Engine {
    /// Create a new engine with specified hash size in MB
    pub fn new(hash_size_mb: usize) -> Self {
        Self {
            generator: Generator::new(),
            evaluator: SimpleEvaluator::new(),
            tt: TranspositionTable::new(hash_size_mb),
            stop_flag: Arc::new(AtomicBool::new(false)),
            info: SearchInfo::new(),
        }
    }

    /// Get a clone of the stop flag for external control
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_flag)
    }

    /// Stop the current search
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Clear transposition table (call on ucinewgame)
    pub fn new_game(&mut self) {
        self.tt.clear();
    }

    /// Resize transposition table
    pub fn resize_tt(&mut self, size_mb: usize) {
        self.tt.resize(size_mb);
    }

    /// Get hash table usage in permille
    pub fn hashfull(&self) -> u16 {
        self.tt.hashfull()
    }

    /// Generate all legal moves for a position
    pub fn legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);
        moves
    }

    /// Search for the best move
    /// soft_limit: target time, will finish current iteration then stop
    /// hard_limit: absolute max, will abort search immediately
    pub fn search(
        &mut self,
        board: &mut Board,
        depth: Option<u8>,
        time_limit: Option<Duration>,
        mut on_info: impl FnMut(&SearchInfo, Option<Move>, Score),
    ) -> SearchResult {
        self.stop_flag.store(false, Ordering::SeqCst);
        self.info = SearchInfo::new();
        self.tt.new_search();

        let start_time = Instant::now();
        let max_depth = depth.unwrap_or(64);

        // Calculate soft and hard limits
        // Soft limit: when to stop starting new iterations
        // Hard limit: absolute max time - should interrupt search if exceeded
        let soft_limit = time_limit;
        let hard_limit = time_limit.map(|t| {
            // Hard limit = soft limit + 10%, capped at +100ms
            let soft_ms = t.as_millis() as u64;
            let extra = (soft_ms / 10).min(100);
            Duration::from_millis(soft_ms + extra)
        });

        let mut best_move: Option<Move> = None;
        let mut best_score = NEG_MATE_SCORE;
        let mut pv = Vec::new();

        // Generate legal moves
        let mut legal_moves = Vec::new();
        self.generator.legal(board, &mut legal_moves);

        if legal_moves.is_empty() {
            return SearchResult {
                best_move: None,
                score: 0,
                pv: Vec::new(),
                info: self.info.clone(),
            };
        }

        // Single move - return immediately
        if legal_moves.len() == 1 {
            return SearchResult {
                best_move: Some(legal_moves[0]),
                score: 0,
                pv: vec![legal_moves[0]],
                info: self.info.clone(),
            };
        }

        // Iterative deepening
        for d in 1..=max_depth {
            // Check soft limit before starting new iteration
            if let Some(limit) = soft_limit {
                if start_time.elapsed() >= limit {
                    break;
                }
            }

            if self.stop_flag.load(Ordering::SeqCst) {
                break;
            }

            self.info.depth = d;
            let mut current_pv = Vec::new();

            let score = self.alpha_beta(
                board,
                d,
                0,
                NEG_MATE_SCORE,
                MATE_SCORE,
                &mut current_pv,
                start_time,
                hard_limit,
            );

            // If we were stopped mid-search, don't update best move
            // (the results might be incomplete)
            if self.stop_flag.load(Ordering::SeqCst) {
                break;
            }

            best_score = score;
            if !current_pv.is_empty() {
                best_move = Some(current_pv[0]);
                pv = current_pv;
            }

            // Update info
            self.info.score = score;
            self.info.time_elapsed = start_time.elapsed();
            self.info.pv = pv.clone();
            self.info.hash_full = self.tt.hashfull();

            if self.info.time_elapsed.as_millis() > 0 {
                self.info.nps =
                    (self.info.nodes as u128 * 1000 / self.info.time_elapsed.as_millis()) as u64;
            }

            // Callback with current search info
            on_info(&self.info, best_move, score);

            // Stop if we found a mate
            if score.abs() > 90000 {
                break;
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            pv,
            info: self.info.clone(),
        }
    }

    /// Alpha-beta search with negamax
    fn alpha_beta(
        &mut self,
        board: &mut Board,
        depth: u8,
        ply: usize,
        mut alpha: Score,
        beta: Score,
        pv: &mut Vec<Move>,
        start_time: Instant,
        time_limit: Option<Duration>,
    ) -> Score {
        self.info.nodes += 1;

        // Periodic time check
        if self.info.nodes % NODE_CHECK_INTERVAL == 0 {
            if self.should_stop(start_time, time_limit) {
                self.stop_flag.store(true, Ordering::SeqCst);
                return 0;
            }
        }

        // Check if already stopped
        if self.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        // Leaf node - evaluate
        if depth == 0 {
            return self.quiescence(board, alpha, beta, start_time, time_limit);
        }

        // Generate moves
        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        // No moves - checkmate or stalemate
        if moves.is_empty() {
            if board.is_in_check(board.side_to_move()) {
                return -MATE_SCORE + ply as Score;
            }
            return 0; // Stalemate
        }

        // Order moves (captures first - simple MVV-LVA)
        self.order_moves(&mut moves);

        let mut local_pv = Vec::new();

        for mv in moves {
            board.make_move(&mv).ok();

            let mut child_pv = Vec::new();
            let score = -self.alpha_beta(
                board,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                &mut child_pv,
                start_time,
                time_limit,
            );

            board.unmake_move(&mv).ok();

            if score >= beta {
                return beta; // Beta cutoff
            }

            if score > alpha {
                alpha = score;
                local_pv.clear();
                local_pv.push(mv);
                local_pv.extend_from_slice(&child_pv);
            }
        }

        *pv = local_pv;
        alpha
    }

    /// Quiescence search - search captures until position is quiet
    fn quiescence(
        &mut self,
        board: &mut Board,
        mut alpha: Score,
        beta: Score,
        start_time: Instant,
        time_limit: Option<Duration>,
    ) -> Score {
        self.info.nodes += 1;

        // Periodic time check in qsearch too
        if self.info.nodes % NODE_CHECK_INTERVAL == 0 {
            if self.should_stop(start_time, time_limit) {
                return 0;
            }
        }

        let stand_pat = self.evaluator.evaluate(board);

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Generate captures only
        let mut moves = Vec::new();
        self.generator.captures(board, &mut moves);

        self.order_moves(&mut moves);

        for mv in moves {
            board.make_move(&mv).ok();
            let score = -self.quiescence(board, -beta, -alpha, start_time, time_limit);
            board.unmake_move(&mv).ok();

            // Check if we were stopped
            if self.stop_flag.load(Ordering::SeqCst) {
                return 0;
            }

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Simple move ordering: captures first, ordered by MVV-LVA
    fn order_moves(&self, moves: &mut [Move]) {
        moves.sort_by(|a, b| {
            let a_score = self.move_score(a);
            let b_score = self.move_score(b);
            b_score.cmp(&a_score)
        });
    }

    /// Score a move for ordering (higher = search first)
    fn move_score(&self, mv: &Move) -> i32 {
        let mut score = 0;

        // Captures: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        if let Some(captured) = mv.capture {
            score += 10 * captured.value() as i32 - mv.piece.value() as i32;
        }

        // Promotions
        if let Some(promo) = mv.promotion {
            score += promo.value() as i32;
        }

        score
    }

    /// Check if search should stop
    fn should_stop(&self, start_time: Instant, time_limit: Option<Duration>) -> bool {
        if self.stop_flag.load(Ordering::SeqCst) {
            return true;
        }

        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                return true;
            }
        }

        false
    }

    /// Perft - count nodes at given depth
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

    /// Perft divide - count nodes per move at given depth
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
