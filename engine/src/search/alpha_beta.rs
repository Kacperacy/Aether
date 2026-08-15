use crate::eval::{EvalState, Evaluator};
use crate::eval::{MATE_SCORE, NEG_MATE_SCORE, Score, mated_in, material};
use crate::search::move_ordering::MoveOrderer;
use crate::search::move_picker::MovePicker;
use crate::search::{
    MAX_PLY, MAX_PV_LENGTH, NodeType, SearchInfo, SearchLimits, SearchResult, TTEntry,
    TranspositionTable,
};
use aether_core::{Move, Piece};
use board::Board;
use movegen::MoveList;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const NODE_CHECK_MASK: u64 = 0xFFF;
const DELTA_PRUNING_MARGIN: Score = 200;
const DELTA_MAX_GAIN: Score = material::QUEEN_VALUE * 2 - material::PAWN_VALUE;
const NULL_MOVE_REDUCTION: u8 = 3;
const NULL_MOVE_MIN_DEPTH: u8 = 3;
const LMR_FULL_DEPTH_MOVES: usize = 4;
const LMR_MIN_DEPTH: u8 = 3;
const ASPIRATION_DEPTH: u8 = 5;
const ASPIRATION_WINDOW: Score = 25;
const ASPIRATION_MAX_DELTA: Score = 400;
const FUTILITY_MARGIN: [Score; 4] = [0, 100, 200, 300];

/// Futility margin for `depth`, clamped to the table rather than indexed by a
/// separate bound constant that could drift out of sync and panic.
#[inline]
fn futility_margin(depth: u8) -> Score {
    let idx = (depth as usize).min(FUTILITY_MARGIN.len() - 1);
    FUTILITY_MARGIN[idx]
}
const FUTILITY_MAX_DEPTH: u8 = 3;
const RFP_MARGIN: Score = 120;
const RFP_MAX_DEPTH: u8 = 3;
const PV_COLLECTION_LIMIT: usize = 32;

pub struct AlphaBetaSearcher<E: Evaluator> {
    evaluator: E,
    tt: TranspositionTable,
    move_orderer: MoveOrderer,
    info: SearchInfo,
    stop_flag: Arc<AtomicBool>,
    start_time: Option<Instant>,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
    nodes_limit: Option<u64>,
    pv_table: [[Move; MAX_PV_LENGTH]; MAX_PV_LENGTH],
    pv_length: [usize; MAX_PV_LENGTH],
    eval_state: EvalState,
}

impl<E: Evaluator> AlphaBetaSearcher<E> {
    pub fn new(evaluator: E, tt_size_mb: usize) -> Self {
        Self {
            evaluator,
            tt: TranspositionTable::new(tt_size_mb),
            move_orderer: MoveOrderer::new(),
            info: SearchInfo::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            start_time: None,
            soft_limit: None,
            hard_limit: None,
            nodes_limit: None,
            pv_table: [[Move::NULL; MAX_PV_LENGTH]; MAX_PV_LENGTH],
            pv_length: [0; MAX_PV_LENGTH],
            eval_state: EvalState::empty(),
        }
    }

    /// Apply `mv`, keeping the evaluation accumulator in step. The accumulator
    /// delta is computed from the pre-move position, so it must be pushed
    /// before the board mutates — and rolled back if the move is rejected.
    #[inline(always)]
    fn do_move(&mut self, board: &mut Board, mv: &Move) -> board::Result<()> {
        self.eval_state.push(board, mv);
        match board.make_move(mv) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.eval_state.pop();
                Err(e)
            }
        }
    }

    #[inline(always)]
    fn undo_move(&mut self, board: &mut Board, mv: &Move) {
        let _ = board.unmake_move(mv);
        self.eval_state.pop();
    }

    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_flag)
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
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

    pub fn search(
        &mut self,
        board: &mut Board,
        limits: &SearchLimits,
        mut on_info: impl FnMut(&SearchInfo, Option<Move>, Score),
    ) -> SearchResult {
        self.stop_flag.store(false, Ordering::Release);
        self.info = SearchInfo::new();
        self.start_time = Some(Instant::now());
        self.soft_limit = limits.time;
        self.hard_limit = limits.hard_time;
        self.nodes_limit = limits.nodes;
        self.move_orderer.clear_repetitions();
        self.tt.new_search();
        self.eval_state.reset(board);

        let start_time = self.start_time.unwrap();
        let max_depth = limits.depth.unwrap_or(MAX_PLY as u8).min(MAX_PLY as u8);

        let mut best_move: Option<Move> = None;
        let mut best_score: Score = NEG_MATE_SCORE;
        self.info.pv.reserve(max_depth as usize);

        let mut legal_moves = MoveList::new();
        movegen::legal(board, &mut legal_moves);

        if legal_moves.is_empty() {
            return SearchResult {
                best_move: None,
                score: if board.is_in_check(board.side_to_move()) {
                    NEG_MATE_SCORE
                } else {
                    0
                },
                info: self.info.clone(),
            };
        }

        if legal_moves.len() == 1 {
            let only_move = legal_moves[0];
            self.do_move(board, &only_move)
                .expect("legal move should not fail");
            let score = -self.quiescence(board, 1, 0, NEG_MATE_SCORE, MATE_SCORE);
            self.undo_move(board, &only_move);

            self.info.depth = 1;
            self.info.time_elapsed = start_time.elapsed();
            self.info.pv = vec![only_move];
            self.info.calculate_nps();

            return SearchResult {
                best_move: Some(only_move),
                score,
                info: self.info.clone(),
            };
        }

        let mut prev_score: Score = 0;

        for depth in 1..=max_depth {
            if let Some(limit) = self.soft_limit
                && start_time.elapsed() >= limit
            {
                break;
            }

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            self.info.depth = depth;

            let score;

            if depth >= ASPIRATION_DEPTH {
                let mut delta = ASPIRATION_WINDOW;
                let mut alpha = (prev_score - delta).max(NEG_MATE_SCORE);
                let mut beta = (prev_score + delta).min(MATE_SCORE);
                let mut best_pv_len = 0;
                let mut best_pv_backup = [Move::NULL; MAX_PV_LENGTH];

                loop {
                    let result = self.alpha_beta(board, depth, 0, alpha, beta, true);

                    if self.stop_flag.load(Ordering::Acquire) {
                        score = prev_score;
                        self.pv_table[0][..best_pv_len]
                            .copy_from_slice(&best_pv_backup[..best_pv_len]);
                        self.pv_length[0] = best_pv_len;
                        break;
                    }

                    if result <= alpha {
                        alpha = (alpha - delta).max(NEG_MATE_SCORE);
                        delta *= 2;

                        if delta > ASPIRATION_MAX_DELTA {
                            alpha = NEG_MATE_SCORE;
                            beta = MATE_SCORE;
                        }
                    } else if result >= beta {
                        let len = self.pv_length[0];
                        best_pv_backup[..len].copy_from_slice(&self.pv_table[0][..len]);
                        best_pv_len = len;

                        beta = (beta + delta).min(MATE_SCORE);
                        delta *= 2;

                        if delta > ASPIRATION_MAX_DELTA {
                            alpha = NEG_MATE_SCORE;
                            beta = MATE_SCORE;
                        }
                    } else {
                        score = result;
                        break;
                    }
                }
            } else {
                score = self.alpha_beta(board, depth, 0, NEG_MATE_SCORE, MATE_SCORE, true);
            }

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            prev_score = score;
            best_score = score;

            // Publish this iteration's PV directly. This used to `mem::swap` into
            // a caller-held Vec, which left the returned result holding the
            // *previous* depth's PV.
            let pv_len = self.pv_length[0];
            if pv_len > 0 {
                best_move = Some(self.pv_table[0][0]);
                self.info.pv.clear();
                for i in 0..pv_len {
                    self.info.pv.push(self.pv_table[0][i]);
                }
            }

            self.info.time_elapsed = start_time.elapsed();
            self.info.hash_full = self.tt.hashfull();
            self.info.calculate_nps();

            on_info(&self.info, best_move, score);

            if score.abs() > MATE_SCORE - (max_depth as Score) {
                break;
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            info: self.info.clone(),
        }
    }

    fn alpha_beta(
        &mut self,
        board: &mut Board,
        depth: u8,
        ply: usize,
        mut alpha: Score,
        mut beta: Score,
        is_pv_node: bool,
    ) -> Score {
        self.info.nodes += 1;

        if ply as u8 > self.info.selective_depth {
            self.info.selective_depth = ply as u8;
        }

        if ply < PV_COLLECTION_LIMIT {
            self.pv_length[ply] = 0;
        }

        if self.should_abort_search() {
            return 0;
        }

        // ===== Quiescence =====
        if depth == 0 {
            return self.quiescence(board, ply, 0, alpha, beta);
        }

        // ===== Terminal conditions =====
        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board, &self.eval_state);
        }

        if ply > 0 {
            if board.is_fifty_move_draw() {
                return 0;
            }

            let dominated_repetition = if ply <= 2 {
                board.is_twofold_repetition()
            } else {
                board.is_threefold_repetition()
            };
            if dominated_repetition {
                return 0;
            }

            if board.is_insufficient_material() {
                return 0;
            }
        }

        // ===== Transposition table probe =====
        // ===== Mate-distance pruning =====
        // A mate found deeper than one already proven at this ply cannot improve
        // the result, and this keeps reported mate distances honest.
        if ply > 0 {
            alpha = alpha.max(mated_in(ply as u32));
            beta = beta.min(-mated_in(ply as u32 + 1));
            if alpha >= beta {
                return alpha;
            }
        }

        let zobrist_key = board.zobrist_hash_raw();
        self.tt.prefetch(zobrist_key);
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = self.tt.probe(zobrist_key) {
            tt_move = entry.best_move();

            if entry.depth >= depth && !is_pv_node {
                let tt_score = TTEntry::score_from_tt(entry.score(), ply);

                // Return the stored score rather than the window edge: the
                // search is fail-soft, and `beta`/`alpha` would throw away the
                // mate distance the entry carries.
                match entry.node_type() {
                    NodeType::Exact => return tt_score,
                    NodeType::LowerBound if tt_score >= beta => return tt_score,
                    NodeType::UpperBound if tt_score <= alpha => return tt_score,
                    _ => {}
                }
            }
        }

        let in_check = board.is_in_check(board.side_to_move());
        let static_eval = if in_check {
            NEG_MATE_SCORE
        } else {
            self.evaluator.evaluate(board, &self.eval_state)
        };

        // ===== Reverse Futility Pruning (Static Null Move) =====
        if !is_pv_node
            && !in_check
            && depth <= RFP_MAX_DEPTH
            && static_eval - RFP_MARGIN * (depth as Score) >= beta
        {
            return beta;
        }

        // ===== Null move pruning =====
        if !is_pv_node
            && !in_check
            && depth >= NULL_MOVE_MIN_DEPTH
            && self.has_non_pawn_material(board)
        {
            board.make_null_move();
            self.eval_state.push_null();

            let null_score = -self.alpha_beta(
                board,
                depth.saturating_sub(NULL_MOVE_REDUCTION + 1),
                ply + 1,
                -beta,
                -beta + 1,
                false,
            );

            board.unmake_null_move();
            self.eval_state.pop();

            if null_score >= beta {
                return beta;
            }
        }

        // ===== Futility pruning flag =====
        let can_futility_prune = !is_pv_node
            && !in_check
            && depth <= FUTILITY_MAX_DEPTH
            && static_eval + futility_margin(depth) <= alpha;

        // ===== Generate and order moves =====
        let mut picker = MovePicker::new(board, &self.move_orderer, tt_move, ply);

        if picker.is_empty() {
            return if in_check { mated_in(ply as u32) } else { 0 };
        }

        // ===== Main move loop =====
        let mut best_score = NEG_MATE_SCORE;
        let mut best_move: Option<Move> = None;
        let mut node_type = NodeType::UpperBound;

        let mut moves_searched = 0;

        while let Some(mv) = picker.next() {
            if self.do_move(board, &mv).is_err() {
                continue;
            }

            let dominated =
                can_futility_prune && moves_searched > 0 && !mv.is_capture() && !mv.is_promotion();

            let gives_check = board.is_in_check(board.side_to_move());

            if dominated && !gives_check {
                self.undo_move(board, &mv);
                continue;
            }

            let is_first_move = moves_searched == 0;
            moves_searched += 1;

            if board.is_twofold_repetition() {
                self.move_orderer.mark_repetition_move(&mv);
            }

            let extension: u8 = if gives_check && ply < MAX_PLY - 10 {
                1
            } else {
                0
            };

            let score;

            if is_first_move {
                score = -self.alpha_beta(
                    board,
                    depth - 1 + extension,
                    ply + 1,
                    -beta,
                    -alpha,
                    is_pv_node,
                );
            } else {
                let can_reduce = moves_searched >= LMR_FULL_DEPTH_MOVES
                    && depth >= LMR_MIN_DEPTH
                    && !mv.is_capture()
                    && !mv.is_promotion()
                    && !in_check
                    && !gives_check;

                let mut lmr_score;

                if can_reduce {
                    let reduction = 1 + (moves_searched as u8 / 6);
                    lmr_score = -self.alpha_beta(
                        board,
                        depth.saturating_sub(reduction + 1) + extension,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        false,
                    );
                } else {
                    lmr_score = -self.alpha_beta(
                        board,
                        depth - 1 + extension,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        false,
                    );
                }

                if lmr_score > alpha && lmr_score < beta {
                    lmr_score = -self.alpha_beta(
                        board,
                        depth - 1 + extension,
                        ply + 1,
                        -beta,
                        -alpha,
                        true,
                    );
                }

                score = lmr_score;
            }

            self.undo_move(board, &mv);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);

                if ply < PV_COLLECTION_LIMIT {
                    self.pv_table[ply][0] = mv;
                    let child_len = self.pv_length[ply + 1].min(MAX_PV_LENGTH - ply - 2);
                    for i in 0..child_len {
                        self.pv_table[ply][i + 1] = self.pv_table[ply + 1][i];
                    }
                    self.pv_length[ply] = child_len + 1;
                }
            }

            if score >= beta {
                if !mv.is_capture() && !mv.is_promotion() {
                    self.move_orderer.store_killer(mv, ply);
                    if let Some((piece, side)) = board.piece_at(mv.from_sq()) {
                        self.move_orderer
                            .update_history(mv, piece, side, depth as usize);
                    }
                }

                if !self.aborted() {
                    let tt_score = TTEntry::score_to_tt(best_score, ply);
                    self.tt.store(TTEntry::new(
                        zobrist_key,
                        best_move,
                        tt_score,
                        depth,
                        NodeType::LowerBound,
                        self.tt.generation(),
                    ));
                }

                return best_score;
            }

            if score > alpha {
                alpha = score;
                node_type = NodeType::Exact;
            }
        }

        // ===== Store in transposition table =====
        // An aborted search returns a fabricated 0 from wherever it stopped, so
        // every score on the unwinding path is meaningless. Writing those as
        // real bounds poisons the table for the next `go`.
        if !self.aborted() {
            let tt_score = TTEntry::score_to_tt(best_score, ply);
            self.tt.store(TTEntry::new(
                zobrist_key,
                best_move,
                tt_score,
                depth,
                node_type,
                self.tt.generation(),
            ));
        }

        best_score
    }

    fn quiescence(
        &mut self,
        board: &mut Board,
        ply: usize,
        depth: i32,
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
            return self.evaluator.evaluate(board, &self.eval_state);
        }

        // Quiescence shares the table with the main search. Every stored entry
        // is at least as deep as this node — quiescence stores at depth 0 and
        // nothing is shallower — so any hit is usable without a depth test.
        let zobrist_key = board.zobrist_hash_raw();
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = self.tt.probe(zobrist_key) {
            tt_move = entry.best_move();
            let tt_score = TTEntry::score_from_tt(entry.score(), ply);

            match entry.node_type() {
                NodeType::Exact => return tt_score,
                NodeType::LowerBound if tt_score >= beta => return tt_score,
                NodeType::UpperBound if tt_score <= alpha => return tt_score,
                _ => {}
            }
        }

        let in_check = board.is_in_check(board.side_to_move());

        if !in_check {
            let stand_pat = self.evaluator.evaluate(board, &self.eval_state);

            if stand_pat >= beta {
                return stand_pat;
            }

            if stand_pat > alpha {
                alpha = stand_pat;
            }

            if stand_pat + DELTA_MAX_GAIN + DELTA_PRUNING_MARGIN < alpha {
                return alpha;
            }
        }

        let mut picker = MovePicker::quiescence(
            board,
            &self.move_orderer,
            tt_move,
            ply,
            in_check,
            depth == 0,
        );

        if in_check && picker.is_empty() {
            return mated_in(ply as u32);
        }

        let original_alpha = alpha;
        let mut best_move: Option<Move> = None;

        while let Some(mv) = picker.next() {
            if self.do_move(board, &mv).is_err() {
                continue;
            }
            let score = -self.quiescence(board, ply + 1, depth - 1, -beta, -alpha);
            self.undo_move(board, &mv);

            if score >= beta {
                if !self.aborted() {
                    self.tt.store(TTEntry::new(
                        zobrist_key,
                        Some(mv),
                        TTEntry::score_to_tt(score, ply),
                        0,
                        NodeType::LowerBound,
                        self.tt.generation(),
                    ));
                }
                return score;
            }

            if score > alpha {
                alpha = score;
                best_move = Some(mv);
            }
        }

        if !self.aborted() {
            let node_type = if alpha > original_alpha {
                NodeType::Exact
            } else {
                NodeType::UpperBound
            };

            self.tt.store(TTEntry::new(
                zobrist_key,
                best_move,
                TTEntry::score_to_tt(alpha, ply),
                0,
                node_type,
                self.tt.generation(),
            ));
        }

        alpha
    }

    /// True once the search has been told to stop, by the caller or by its own
    /// time/node limit. Scores produced after this point are not real.
    #[inline(always)]
    fn aborted(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
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
        if let Some(start) = self.start_time
            && let Some(limit) = self.hard_limit
            && start.elapsed() >= limit
        {
            return true;
        }

        if let Some(limit) = self.nodes_limit
            && self.info.nodes >= limit
        {
            return true;
        }

        false
    }

    #[inline]
    fn has_non_pawn_material(&self, board: &Board) -> bool {
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
        Self::new(E::default(), 16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::SimpleEvaluator;

    #[test]
    fn test_search_basic() {
        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let mut board = Board::starting_position().unwrap();

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(!result.pv().is_empty());
        assert!(result.info.nodes > 0);
    }

    /// The reported PV must belong to the same iteration as `best_move`.
    /// A previous `mem::swap` published the *previous* depth's PV, so the two
    /// could disagree about the very first move.
    #[test]
    fn test_reported_pv_matches_best_move() {
        for depth in 1..=5u8 {
            let mut searcher = AlphaBetaSearcher::new(SimpleEvaluator::new(), 1);
            let mut board = Board::starting_position().unwrap();

            let result = searcher.search(&mut board, &SearchLimits::depth(depth), |_, _, _| {});

            let best = result.best_move.expect("a best move at every depth");
            assert_eq!(
                result.pv().first().copied(),
                Some(best),
                "depth {depth}: PV head {:?} disagrees with best_move {best}",
                result.pv().first()
            );
            assert_eq!(
                result.pv(),
                result.info.pv.as_slice(),
                "depth {depth}: result and info must expose the same PV"
            );
        }
    }

    #[test]
    fn test_mate_in_one() {
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        assert!(result.best_move.is_some());
        let best = result.best_move.unwrap();
        assert_eq!(best.to_sq().to_string(), "a8");
    }

    #[test]
    fn test_search_detects_threefold_repetition() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let limits = SearchLimits::depth(6);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        assert!(result.best_move.is_some());
    }

    #[test]
    fn test_search_avoids_immediate_repetition() {
        let fen = "4k3/8/8/8/8/8/8/4K2R w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 16);

        let limits = SearchLimits::depth(6);
        let result1 = searcher.search(&mut board, &limits, |_, _, _| {});
        let best_move1 = result1.best_move.unwrap();

        board.make_move(&best_move1).unwrap();

        let mut opponent_moves = MoveList::new();
        movegen::legal(&board, &mut opponent_moves);
        board.make_move(&opponent_moves[0]).unwrap();

        let result2 = searcher.search(&mut board, &limits, |_, _, _| {});
        let best_move2 = result2.best_move.unwrap();

        board.make_move(&best_move2).unwrap();

        assert!(!board.is_threefold_repetition());
    }

    #[test]
    fn test_search_recognizes_insufficient_material_draw() {
        let fen = "8/8/8/4k3/8/8/2B5/4K3 w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let limits = SearchLimits::depth(6);
        let result = searcher.search(&mut board, &limits, |_, _, _| {});

        assert!(
            result.score.abs() < 50,
            "Insufficient material should evaluate near 0"
        );
    }

    #[test]
    fn test_fifty_move_rule_in_search() {
        let fen = "4k3/8/8/8/8/8/8/4K3 w - - 100 1";
        let mut board: Board = fen.parse().unwrap();

        assert!(board.is_fifty_move_draw());

        let evaluator = SimpleEvaluator::new();
        let mut searcher = AlphaBetaSearcher::new(evaluator, 1);

        let score = searcher.alpha_beta(&mut board, 1, 1, -1000, 1000, true);

        assert_eq!(score, 0, "Fifty-move rule should return 0 (draw)");
    }
}
