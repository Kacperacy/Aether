use crate::eval::Evaluator;
use crate::search::alpha_beta::common::{
    AVG_LEGAL_MOVES, MoveListStack, NODE_CHECK_MASK, PvTable, order_moves_mvv_lva,
};
use crate::search::searcher::{SearchCallback, Searcher};
use crate::search::{MAX_PLY, SearchInfo, SearchLimits, SearchResult};
use aether_core::{MATE_SCORE, Move, NEG_MATE_SCORE, Score, mated_in};
use board::Board;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const PV_COLLECTION_LIMIT: usize = 32;

/// Pure NegaScout searcher - baseline implementation without optimizations
///
/// NegaScout (Principal Variation Search) assumes the first move is best.
/// It searches the first move with a full window, then uses null-window
/// searches for remaining moves. If a move fails high, it re-searches
/// with a full window.
///
/// Features:
/// - Iterative deepening
/// - Quiescence search (captures only)
/// - MVV-LVA move ordering
/// - Draw detection (repetition, 50-move, insufficient material)
/// - Null-window search for non-PV moves
///
/// Explicitly NO:
/// - Transposition table
/// - Null move pruning
/// - Late move reductions (LMR)
/// - Futility pruning
/// - Aspiration windows
/// - Killer moves / History heuristic
/// - SEE-based ordering
pub struct PureNegaScoutSearcher<E: Evaluator> {
    evaluator: E,
    info: SearchInfo,
    stop_flag: Arc<AtomicBool>,
    start_time: Option<Instant>,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
    nodes_limit: Option<u64>,
    pv_table: PvTable,
    move_lists: MoveListStack,
}

impl<E: Evaluator> PureNegaScoutSearcher<E> {
    pub fn new(evaluator: E) -> Self {
        Self {
            evaluator,
            info: SearchInfo::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            start_time: None,
            soft_limit: None,
            hard_limit: None,
            nodes_limit: None,
            pv_table: PvTable::new(),
            move_lists: MoveListStack::new(),
        }
    }

    /// NegaScout search with null-window technique
    fn negascout(
        &mut self,
        board: &mut Board,
        depth: u8,
        ply: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.info.nodes += 1;

        if ply as u8 > self.info.selective_depth {
            self.info.selective_depth = ply as u8;
        }

        self.pv_table.clear_ply(ply);

        if self.should_abort_search() {
            return 0;
        }

        // Quiescence at depth 0
        if depth == 0 {
            return self.quiescence(board, ply, alpha, beta);
        }

        // Max ply reached
        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        // Draw detection (skip at root)
        if ply > 0 {
            if board.is_fifty_move_draw() {
                return 0;
            }

            if board.is_threefold_repetition() {
                return 0;
            }

            if board.is_insufficient_material() {
                return 0;
            }
        }

        let in_check = board.is_in_check(board.side_to_move());

        // Generate legal moves
        let mut moves = self.move_lists.take(ply);
        moves.clear();
        movegen::legal(board, &mut moves);

        if moves.is_empty() {
            self.move_lists.put_back(ply, moves);
            return if in_check { mated_in(ply as u32) } else { 0 };
        }

        // Simple MVV-LVA ordering
        order_moves_mvv_lva(&mut moves);

        let mut best_score = NEG_MATE_SCORE;
        let mut first_move = true;

        for mv in moves.iter() {
            if board.make_move(mv).is_err() {
                continue;
            }

            // Check extension
            let gives_check = board.is_in_check(board.side_to_move());
            let extension: u8 = if gives_check && ply < MAX_PLY - 10 { 1 } else { 0 };

            let score;

            if first_move {
                // First move: full window search (principal variation)
                score = -self.negascout(board, depth - 1 + extension, ply + 1, -beta, -alpha);
                first_move = false;
            } else {
                // Subsequent moves: null window search
                let null_window_score =
                    -self.negascout(board, depth - 1 + extension, ply + 1, -alpha - 1, -alpha);

                // If it fails high and there's room, re-search with full window
                if null_window_score > alpha && null_window_score < beta {
                    score =
                        -self.negascout(board, depth - 1 + extension, ply + 1, -beta, -alpha);
                } else {
                    score = null_window_score;
                }
            }

            let _ = board.unmake_move(mv);

            if score > best_score {
                best_score = score;

                if ply < PV_COLLECTION_LIMIT {
                    self.pv_table.update(ply, *mv, PV_COLLECTION_LIMIT);
                }
            }

            if score >= beta {
                self.move_lists.put_back(ply, moves);
                return best_score;
            }

            if score > alpha {
                alpha = score;
            }
        }

        self.move_lists.put_back(ply, moves);
        best_score
    }

    fn quiescence(
        &mut self,
        board: &mut Board,
        ply: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.info.nodes += 1;

        if ply as u8 > self.info.selective_depth {
            self.info.selective_depth = ply as u8;
        }

        if self.should_abort_search() {
            return 0;
        }

        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        let in_check = board.is_in_check(board.side_to_move());

        // Stand-pat score (not when in check)
        if !in_check {
            let stand_pat = self.evaluator.evaluate(board);

            if stand_pat >= beta {
                return stand_pat;
            }

            if stand_pat > alpha {
                alpha = stand_pat;
            }
        }

        // Generate moves
        let mut moves = Vec::with_capacity(if in_check { AVG_LEGAL_MOVES } else { 16 });

        if in_check {
            movegen::legal(board, &mut moves);

            if moves.is_empty() {
                return mated_in(ply as u32);
            }
        } else {
            movegen::captures(board, &mut moves);
        }

        // Simple MVV-LVA ordering
        order_moves_mvv_lva(&mut moves);

        for mv in moves {
            if board.make_move(&mv).is_err() {
                continue;
            }

            let score = -self.quiescence(board, ply + 1, -beta, -alpha);
            let _ = board.unmake_move(&mv);

            if score >= beta {
                return score;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    #[inline]
    fn should_abort_search(&self) -> bool {
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        if (self.info.nodes & NODE_CHECK_MASK) == 0 && self.should_stop() {
            self.stop_flag.store(true, Ordering::Release);
            return true;
        }

        false
    }

    #[inline]
    fn should_stop(&self) -> bool {
        if let Some(start) = self.start_time {
            if let Some(limit) = self.hard_limit {
                if start.elapsed() >= limit {
                    return true;
                }
            }
            if let Some(limit) = self.soft_limit {
                if start.elapsed() >= limit {
                    return true;
                }
            }
        }

        if let Some(limit) = self.nodes_limit {
            if self.info.nodes >= limit {
                return true;
            }
        }

        false
    }
}

impl<E: Evaluator + Send> Searcher for PureNegaScoutSearcher<E> {
    fn search(
        &mut self,
        board: &mut Board,
        limits: &SearchLimits,
        on_info: SearchCallback<'_>,
    ) -> SearchResult {
        self.stop_flag.store(false, Ordering::Release);
        self.info = SearchInfo::new();
        self.start_time = Some(Instant::now());
        self.soft_limit = limits.time;
        self.hard_limit = limits.hard_time;
        self.nodes_limit = limits.nodes;

        let start_time = self.start_time.unwrap();
        let max_depth = limits.depth.unwrap_or(MAX_PLY as u8).min(MAX_PLY as u8);

        let mut best_move: Option<Move> = None;
        let mut best_score: Score = NEG_MATE_SCORE;
        let mut pv = Vec::with_capacity(max_depth as usize);

        // Check for no legal moves
        let mut legal_moves = Vec::with_capacity(AVG_LEGAL_MOVES);
        movegen::legal(board, &mut legal_moves);

        if legal_moves.is_empty() {
            return SearchResult {
                best_move: None,
                score: if board.is_in_check(board.side_to_move()) {
                    NEG_MATE_SCORE
                } else {
                    0
                },
                pv: Vec::new(),
                info: self.info.clone(),
            };
        }

        // Single move - just return it
        if legal_moves.len() == 1 {
            let only_move = legal_moves[0];
            board
                .make_move(&only_move)
                .expect("legal move should not fail");
            let score = -self.quiescence(board, 1, NEG_MATE_SCORE, MATE_SCORE);
            board
                .unmake_move(&only_move)
                .expect("unmake should not fail");

            self.info.depth = 1;
            self.info.time_elapsed = start_time.elapsed();
            self.info.calculate_nps();

            return SearchResult {
                best_move: Some(only_move),
                score,
                pv: vec![only_move],
                info: self.info.clone(),
            };
        }

        // Iterative deepening (no aspiration windows)
        for depth in 1..=max_depth {
            // Check soft time limit
            if let Some(limit) = self.soft_limit {
                if start_time.elapsed() >= limit {
                    break;
                }
            }

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            self.info.depth = depth;

            // Full-window search
            let score = self.negascout(board, depth, 0, NEG_MATE_SCORE, MATE_SCORE);

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            best_score = score;

            // Extract PV
            let new_pv = self.pv_table.get_pv();
            if !new_pv.is_empty() {
                best_move = Some(new_pv[0]);
                pv = new_pv;
            }

            self.info.score = score;
            self.info.time_elapsed = start_time.elapsed();
            self.info.pv = pv.clone();
            self.info.calculate_nps();

            on_info(&self.info, best_move, score);

            // Stop if mate found
            if score.abs() > MATE_SCORE - (max_depth as Score) {
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

    fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_flag)
    }

    fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
    }

    fn new_game(&mut self) {
        // No TT to clear in pure implementation
    }

    fn resize_tt(&mut self, _size_mb: usize) {
        // No TT in pure implementation
    }

    fn hashfull(&self) -> u16 {
        0 // No TT
    }

    fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    fn algorithm_name(&self) -> &'static str {
        "NegaScout"
    }
}

impl<E: Evaluator + Default> Default for PureNegaScoutSearcher<E> {
    fn default() -> Self {
        Self::new(E::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::SimpleEvaluator;

    #[test]
    fn test_pure_negascout_basic() {
        let evaluator = SimpleEvaluator::new();
        let mut searcher = PureNegaScoutSearcher::new(evaluator);

        let mut board = Board::starting_position().unwrap();

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(!result.pv.is_empty());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_pure_negascout_mate_in_one() {
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = PureNegaScoutSearcher::new(evaluator);

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        let best = result.best_move.unwrap();
        assert_eq!(best.to.to_string(), "a8");
    }

    #[test]
    fn test_pure_negascout_same_result_as_alpha_beta() {
        // Both algorithms should find the same best move
        use crate::search::alpha_beta::PureAlphaBetaSearcher;

        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut negascout = PureNegaScoutSearcher::new(evaluator.clone());
        let mut alphabeta = PureAlphaBetaSearcher::new(evaluator);

        let limits = SearchLimits::depth(4);

        let ns_result = negascout.search(&mut board, &limits, &mut |_, _, _| {});
        let ab_result = alphabeta.search(&mut board, &limits, &mut |_, _, _| {});

        // Same best move and score
        assert_eq!(ns_result.best_move, ab_result.best_move);
        assert_eq!(ns_result.score, ab_result.score);
    }
}
