use crate::eval::Evaluator;
use crate::search::move_ordering::MoveOrderer;
use crate::search::{
    MAX_PLY, NodeType, SearchInfo, SearchLimits, SearchResult, TTEntry, TranspositionTable,
};
use aether_core::{MATE_SCORE, Move, NEG_MATE_SCORE, Piece, QUEEN_VALUE, Score, mated_in};
use board::{BoardOps, BoardQuery};
use movegen::{Generator, MoveGen};
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
// --- Search Constants ---

/// Number of nodes between time/stop checks
/// Balances search responsiveness with checking overhead
const NODE_CHECK_INTERVAL: u64 = 4096;

/// Safety margin for delta pruning in quiescence search (centipawns)
/// Accounts for positional factors beyond pure material count
const DELTA_PRUNING_MARGIN: Score = 200;

/// Depth reduction for null move pruning (R value)
/// Standard value balancing pruning aggressiveness with tactical safety
const NULL_MOVE_REDUCTION: u8 = 3;

/// Minimum depth required to apply null move pruning
/// Below this depth, null move overhead exceeds potential savings
const NULL_MOVE_MIN_DEPTH: u8 = 3;

/// Number of moves searched at full depth before applying LMR
/// First moves are likely best (due to move ordering) and deserve full analysis
const LMR_FULL_DEPTH_MOVES: usize = 4;

/// Minimum depth required to apply late move reductions
/// At shallow depths, LMR provides minimal time savings
const LMR_MIN_DEPTH: u8 = 3;

/// Minimum depth for aspiration window search
/// At shallow depths, aspiration windows cause more re-searches than savings
const ASPIRATION_DEPTH: u8 = 5;

/// Initial aspiration window size (centipawns)
/// Narrow enough to save time, wide enough to avoid excessive re-searches
const ASPIRATION_WINDOW: Score = 50;

/// Maximum aspiration delta before falling back to full window search
/// Beyond this value, re-searches waste more time than full-width search
const ASPIRATION_MAX_DELTA: Score = 400;

pub struct AlphaBetaSearcher<E: Evaluator> {
    evaluator: E,
    generator: Generator,
    tt: TranspositionTable,
    move_orderer: MoveOrderer,
    info: SearchInfo,
    stop_flag: Arc<AtomicBool>,
    start_time: Option<Instant>,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
}

impl<E: Evaluator> AlphaBetaSearcher<E> {
    /// Creates a new AlphaBetaSearcher with the given evaluator
    pub fn new(evaluator: E, tt_size_mb: usize) -> Self {
        Self {
            evaluator,
            generator: Generator::new(),
            tt: TranspositionTable::new(tt_size_mb),
            move_orderer: MoveOrderer::new(),
            info: SearchInfo::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            start_time: None,
            soft_limit: None,
            hard_limit: None,
        }
    }

    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_flag)
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
    }

    pub fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    pub fn clear_tt(&mut self) {
        self.tt.clear();
    }

    pub fn clear_move_ordering(&mut self) {
        self.move_orderer.clear();
    }

    pub fn resize_tt(&mut self, size_mb: usize) {
        self.tt.resize(size_mb);
    }

    pub fn hashfull(&self) -> u16 {
        self.tt.hashfull()
    }

    pub fn search<T: BoardOps + BoardQuery>(
        &mut self,
        board: &mut T,
        limits: &SearchLimits,
        mut on_info: impl FnMut(&SearchInfo, Option<Move>, Score),
    ) -> SearchResult {
        self.stop_flag.store(false, Ordering::Release);
        self.info = SearchInfo::new();
        self.start_time = Some(Instant::now());
        self.soft_limit = limits.time;
        self.hard_limit = limits.hard_time;

        let start_time = self.start_time.unwrap();
        let max_depth = limits.depth.unwrap_or(MAX_PLY as u8).min(MAX_PLY as u8);

        let mut best_move: Option<Move> = None;
        let mut best_score: Score = NEG_MATE_SCORE;
        let mut pv = Vec::new();

        let mut legal_moves = Vec::new();
        self.generator.legal(board, &mut legal_moves);

        if legal_moves.is_empty() {
            return SearchResult {
                best_move: None,
                score: if board.is_in_check(board.side_to_move()) {
                    NEG_MATE_SCORE
                } else {
                    0 // Stalemate
                },
                pv: Vec::new(),
                info: self.info.clone(),
            };
        }

        if legal_moves.len() == 1 {
            return SearchResult {
                best_move: Some(legal_moves[0]),
                score: 0,
                pv: vec![legal_moves[0]],
                info: self.info.clone(),
            };
        }

        let mut prev_score: Score = 0;

        // Iterative deepening
        for depth in 1..=max_depth {
            if let Some(limit) = self.soft_limit {
                if start_time.elapsed() >= limit {
                    break;
                }
            }

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            self.info.depth = depth;
            let mut current_pv = Vec::new();

            let score;

            if depth >= ASPIRATION_DEPTH {
                let mut delta = ASPIRATION_WINDOW;
                let mut alpha = (prev_score - delta).max(NEG_MATE_SCORE);
                let mut beta = (prev_score + delta).min(MATE_SCORE);
                let mut best_pv = Vec::new();

                loop {
                    current_pv.clear();
                    let result =
                        self.alpha_beta(board, depth, 0, alpha, beta, &mut current_pv, true);

                    if self.stop_flag.load(Ordering::Acquire) {
                        score = prev_score;
                        current_pv = best_pv;
                        break;
                    }

                    if result <= alpha {
                        // Failed low, widen window downwards
                        alpha = (alpha - delta).max(NEG_MATE_SCORE);
                        delta *= 2;

                        if delta > ASPIRATION_MAX_DELTA {
                            alpha = NEG_MATE_SCORE;
                            beta = MATE_SCORE;
                        }
                    } else if result >= beta {
                        // Failed high - save PV as it might be good
                        if !current_pv.is_empty() {
                            best_pv = current_pv.clone();
                        }
                        beta = (beta + delta).min(MATE_SCORE);
                        delta *= 2;

                        if delta > ASPIRATION_MAX_DELTA {
                            alpha = NEG_MATE_SCORE;
                            beta = MATE_SCORE;
                        }
                    } else {
                        // Successful search
                        score = result;
                        break;
                    }
                }
            } else {
                score = self.alpha_beta(
                    board,
                    depth,
                    0,
                    NEG_MATE_SCORE,
                    MATE_SCORE,
                    &mut current_pv,
                    true,
                );
            }

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            prev_score = score;
            best_score = score;
            if !current_pv.is_empty() {
                best_move = Some(current_pv[0]);
                pv = current_pv;
            }

            self.info.score = score;
            self.info.time_elapsed = start_time.elapsed();
            mem::swap(&mut self.info.pv, &mut pv);
            self.info.hash_full = self.tt.hashfull();
            self.info.calculate_nps();

            on_info(&self.info, best_move, score);

            if score.abs() > MATE_SCORE - (max_depth as Score) {
                // Found a mate, stop searching deeper
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

    /// Main alpha-beta search function (monolithic, Stockfish-style)
    fn alpha_beta<T: BoardOps + BoardQuery>(
        &mut self,
        board: &mut T,
        depth: u8,
        ply: usize,
        mut alpha: Score,
        beta: Score,
        pv: &mut Vec<Move>,
        is_pv_node: bool,
    ) -> Score {
        // =========================================================================
        // Step 1. Initialize node
        // =========================================================================
        self.info.nodes += 1;
        pv.clear();

        if self.should_abort_search() {
            return 0;
        }

        // =========================================================================
        // Step 2. Check for quiescence
        // =========================================================================
        if depth == 0 {
            return self.quiescence(board, ply, 0, alpha, beta);
        }

        // =========================================================================
        // Step 3. Check terminal conditions (max ply, repetition, 50-move, insufficient material)
        // =========================================================================
        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        if ply > 0
            && (board.is_twofold_repetition()
                || board.is_fifty_move_draw()
                || board.is_insufficient_material())
        {
            return 0;
        }

        // =========================================================================
        // Step 4. Transposition table lookup
        // =========================================================================
        let zobrist_key = board.zobrist_hash_raw();
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = self.tt.probe(zobrist_key) {
            tt_move = entry.best_move;

            // Use TT score for cutoffs in non-PV nodes with sufficient depth
            if entry.depth >= depth && !is_pv_node {
                let tt_score = TTEntry::score_from_tt(entry.score, ply);

                match entry.node_type {
                    NodeType::Exact => return tt_score,
                    NodeType::LowerBound if tt_score >= beta => return beta,
                    NodeType::UpperBound if tt_score <= alpha => return alpha,
                    _ => {}
                }
            }
        }

        let in_check = board.is_in_check(board.side_to_move());

        // =========================================================================
        // Step 5. Null move pruning
        // =========================================================================
        if !is_pv_node
            && !in_check
            && depth >= NULL_MOVE_MIN_DEPTH
            && self.has_non_pawn_material(board)
        {
            board.make_null_move();

            let null_score = -self.alpha_beta(
                board,
                depth.saturating_sub(NULL_MOVE_REDUCTION + 1),
                ply + 1,
                -beta,
                -beta + 1,
                &mut Vec::new(),
                false,
            );

            board.unmake_null_move();

            if null_score >= beta {
                return beta;
            }
        }

        // =========================================================================
        // Step 6. Generate and order moves
        // =========================================================================
        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        if moves.is_empty() {
            return if in_check {
                mated_in(ply as u32)
            } else {
                0 // Stalemate
            };
        }

        self.move_orderer.order_moves_full(&mut moves, tt_move, ply);

        // =========================================================================
        // Step 7. Main move loop
        // =========================================================================
        let mut best_score = NEG_MATE_SCORE;
        let mut best_move: Option<Move> = None;
        let mut node_type = NodeType::UpperBound;

        for (move_index, mv) in moves.iter().enumerate() {
            // --- Step 7a. Make move ---
            if board.make_move(mv).is_err() {
                continue;
            }

            let mut child_pv: Vec<Move> = Vec::new();
            let gives_check = board.is_in_check(board.side_to_move());
            let extension: u8 = if gives_check { 1 } else { 0 };

            let score;

            if move_index == 0 {
                // --- Step 7b. First move: full window search ---
                score = -self.alpha_beta(
                    board,
                    depth - 1 + extension,
                    ply + 1,
                    -beta,
                    -alpha,
                    &mut child_pv,
                    is_pv_node,
                );
            } else {
                // --- Step 7c. Late Move Reductions (LMR) ---
                let can_reduce = move_index >= LMR_FULL_DEPTH_MOVES
                    && depth >= LMR_MIN_DEPTH
                    && mv.capture.is_none()
                    && mv.promotion.is_none()
                    && !in_check
                    && !gives_check;

                let mut lmr_score;

                if can_reduce {
                    // LMR: reduced depth + null window
                    let reduction = 1 + (move_index as u8 / 6);
                    lmr_score = -self.alpha_beta(
                        board,
                        depth.saturating_sub(reduction + 1) + extension,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        &mut Vec::new(),
                        false,
                    );
                } else {
                    // PVS: null window at full depth
                    lmr_score = -self.alpha_beta(
                        board,
                        depth - 1 + extension,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        &mut Vec::new(),
                        false,
                    );
                }

                // --- Step 7d. Re-search with full window if needed ---
                if lmr_score > alpha && lmr_score < beta {
                    lmr_score = -self.alpha_beta(
                        board,
                        depth - 1 + extension,
                        ply + 1,
                        -beta,
                        -alpha,
                        &mut child_pv,
                        true,
                    );
                }

                score = lmr_score;
            }

            // --- Step 7e. Unmake move ---
            let _ = board.unmake_move(mv);

            // --- Step 7f. Update best score ---
            if score > best_score {
                best_score = score;
                best_move = Some(*mv);

                // Update PV: current move + child PV
                pv.clear();
                pv.push(*mv);
                pv.extend_from_slice(&child_pv);
            }

            // --- Step 7g. Beta cutoff ---
            if score >= beta {
                // Update move ordering heuristics for quiet moves
                if mv.capture.is_none() && mv.promotion.is_none() {
                    self.move_orderer.store_killer(*mv, ply);
                    self.move_orderer.update_history(*mv, depth as usize);
                }

                // Store in TT
                let tt_score = TTEntry::score_to_tt(beta, ply);
                let entry = TTEntry::new(
                    zobrist_key,
                    best_move,
                    tt_score,
                    depth,
                    NodeType::LowerBound,
                    self.tt.generation(),
                );
                self.tt.store(entry);

                return beta;
            }

            // --- Step 7h. Update alpha ---
            if score > alpha {
                alpha = score;
                node_type = NodeType::Exact;
            }
        }

        // =========================================================================
        // Step 8. Store result in transposition table
        // =========================================================================
        let tt_score = TTEntry::score_to_tt(best_score, ply);
        let entry = TTEntry::new(
            zobrist_key,
            best_move,
            tt_score,
            depth,
            node_type,
            self.tt.generation(),
        );
        self.tt.store(entry);

        best_score
    }

    /// Quiescence search - search captures until a quiet position is reached
    /// depth: quiescence depth (0 = first call, -1, -2, ... for deeper levels)
    fn quiescence<T: BoardOps + BoardQuery>(
        &mut self,
        board: &mut T,
        ply: usize,
        depth: i32,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.info.nodes += 1;

        if self.should_abort_search() {
            return 0;
        }

        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        let in_check = board.is_in_check(board.side_to_move());

        if !in_check {
            let stand_pat = self.evaluator.evaluate(board);

            if stand_pat >= beta {
                return beta;
            }

            if stand_pat > alpha {
                alpha = stand_pat;
            }

            // Delta pruning: if even the best possible capture (queen)
            // can't improve alpha, skip search
            if stand_pat + QUEEN_VALUE + DELTA_PRUNING_MARGIN < alpha {
                return alpha;
            }
        }

        let mut moves = Vec::new();
        if in_check {
            self.generator.legal(board, &mut moves);

            if moves.is_empty() {
                return mated_in(ply as u32);
            }
        } else {
            // Generate captures
            self.generator.captures(board, &mut moves);

            // Generate checks at root quiescence
            if depth == 0 {
                self.generator.checks(board, &mut moves);
            }
        }

        self.move_orderer.order_moves(&mut moves);

        for mv in moves {
            board
                .make_move(&mv)
                .expect("make_move failed in quiescence");
            let score = -self.quiescence(board, ply + 1, depth - 1, -beta, -alpha);
            board
                .unmake_move(&mv)
                .expect("unmake_move failed in quiescence");

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    #[inline]
    fn should_abort_search(&self) -> bool {
        // Use Relaxed for quick check - we don't need strict ordering here
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        if self.info.nodes % NODE_CHECK_INTERVAL == 0 && self.should_stop() {
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
        }

        false
    }

    #[inline]
    fn has_non_pawn_material<T: BoardQuery>(&self, board: &T) -> bool {
        let side = board.side_to_move();
        board.piece_count(Piece::Knight, side) > 0
            || board.piece_count(Piece::Bishop, side) > 0
            || board.piece_count(Piece::Rook, side) > 0
            || board.piece_count(Piece::Queen, side) > 0
    }
}

impl<E: Evaluator> Default for AlphaBetaSearcher<E>
where
    E: Default,
{
    fn default() -> Self {
        Self::new(E::default(), 16) // Default TT size of 16 MB
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::SimpleEvaluator;
    use board::{Board, FenOps};

    #[test]
    fn test_search_basic() {
        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let mut board = Board::starting_position().unwrap();

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(!result.pv.is_empty());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_mate_in_one() {
        // Position with mate in 1 for white
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        assert!(result.best_move.is_some());
        // Should find Ra8# (mate)
        let best = result.best_move.unwrap();
        assert_eq!(best.to.to_string(), "a8");
    }

    #[test]
    fn test_search_detects_threefold_repetition() {
        // Setup a position where repetition can occur
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut board = Board::from_fen(fen).unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        // Make moves that repeat position
        // (This is a conceptual test - full implementation needs move setup)

        let limits = SearchLimits::depth(6);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        // Search should complete without hanging on repetitions
        assert!(result.best_move.is_some());
    }

    #[test]
    fn test_search_avoids_immediate_repetition() {
        // Position where bot could repeat immediately
        let fen = "4k3/8/8/8/8/8/8/4K2R w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 16);

        // First move
        let limits = SearchLimits::depth(6);
        let result1 = searcher.search(&mut board, &limits, |_, _, _| {});
        let best_move1 = result1.best_move.unwrap();

        board.make_move(&best_move1).unwrap();

        // Opponent moves (simulate)
        let mut opponent_moves = Vec::new();
        searcher.generator.legal(&mut board, &mut opponent_moves);
        board.make_move(&opponent_moves[0]).unwrap();

        // Second move - should NOT repeat position
        let result2 = searcher.search(&mut board, &limits, |_, _, _| {});
        let best_move2 = result2.best_move.unwrap();

        board.make_move(&best_move2).unwrap();

        // Check that position didn't repeat
        assert!(!board.is_threefold_repetition());
    }

    #[test]
    fn test_search_recognizes_insufficient_material_draw() {
        // K+B vs K - insufficient material
        let fen = "8/8/8/4k3/8/8/2B5/4K3 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let limits = SearchLimits::depth(6);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        // Score should be close to 0 (draw)
        assert!(
            result.score.abs() < 50,
            "Insufficient material should evaluate near 0"
        );
    }

    #[test]
    fn test_fifty_move_rule_in_search() {
        // Position with halfmove clock near 100
        let fen = "4k3/8/8/8/8/8/8/4K3 w - - 100 1";
        let mut board = Board::from_fen(fen).unwrap();

        assert!(board.is_fifty_move_draw());

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        // Search should immediately return draw score
        let limits = SearchLimits::depth(1);
        let mut pv = Vec::new();
        let score = searcher.alpha_beta(
            &mut board, 1, 1, // ply > 0 to trigger draw detection
            -1000, 1000, &mut pv, true,
        );

        assert_eq!(score, 0, "Fifty-move rule should return 0 (draw)");
    }
}
