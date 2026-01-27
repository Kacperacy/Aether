use crate::eval::Evaluator;
use crate::search::alpha_beta::common::{AVG_LEGAL_MOVES, NODE_CHECK_MASK};
use crate::search::move_ordering::MoveOrderer;
use crate::search::searcher::{SearchCallback, Searcher};
use crate::search::{
    MAX_PLY, MAX_PV_LENGTH, NodeType, SearchInfo, SearchLimits, SearchResult, TTEntry,
    TranspositionTable,
};
use aether_core::{MATE_SCORE, Move, NEG_MATE_SCORE, PAWN_VALUE, QUEEN_VALUE, Score, mated_in};
use board::Board;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const DELTA_PRUNING_MARGIN: Score = 200;
const DELTA_MAX_GAIN: Score = QUEEN_VALUE * 2 - PAWN_VALUE;
const PV_COLLECTION_LIMIT: usize = 32;

/// MTD(f) searcher - Memory-enhanced Test Driver
///
/// MTD(f) performs a series of zero-window alpha-beta searches,
/// converging on the true minimax value. It relies heavily on the
/// transposition table to avoid re-searching positions.
pub struct MtdfSearcher<E: Evaluator> {
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
    move_lists: Vec<Vec<Move>>,
}

impl<E: Evaluator> MtdfSearcher<E> {
    pub fn new(evaluator: E, tt_size_mb: usize) -> Self {
        let move_lists = (0..MAX_PLY)
            .map(|_| Vec::with_capacity(AVG_LEGAL_MOVES))
            .collect();

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
            pv_table: [[Move::default(); MAX_PV_LENGTH]; MAX_PV_LENGTH],
            pv_length: [0; MAX_PV_LENGTH],
            move_lists,
        }
    }

    /// MTD(f) algorithm - iteratively narrows the search window
    fn mtdf(&mut self, board: &mut Board, first_guess: Score, depth: u8) -> Score {
        let mut g = first_guess;
        let mut upper = MATE_SCORE;
        let mut lower = NEG_MATE_SCORE;

        while lower < upper {
            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            let beta = if g == lower { g + 1 } else { g };

            // Zero-window search
            g = self.alpha_beta_with_memory(board, depth, 0, beta - 1, beta);

            if g < beta {
                upper = g;
            } else {
                lower = g;
            }
        }

        g
    }

    /// Alpha-beta with TT (memory-enhanced)
    fn alpha_beta_with_memory(
        &mut self,
        board: &mut Board,
        depth: u8,
        ply: usize,
        mut alpha: Score,
        mut beta: Score,
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

        if depth == 0 {
            return self.quiescence(board, ply, alpha, beta);
        }

        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        // Draw detection
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

        // TT probe
        let zobrist_key = board.zobrist_hash_raw();
        self.tt.prefetch(zobrist_key);
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = self.tt.probe(zobrist_key) {
            tt_move = entry.best_move;

            if entry.depth >= depth {
                let tt_score = TTEntry::score_from_tt(entry.score, ply);

                match entry.node_type {
                    NodeType::Exact => return tt_score,
                    NodeType::LowerBound => {
                        if tt_score >= beta {
                            return tt_score;
                        }
                        alpha = alpha.max(tt_score);
                    }
                    NodeType::UpperBound => {
                        if tt_score <= alpha {
                            return tt_score;
                        }
                        beta = beta.min(tt_score);
                    }
                }
            }
        }

        let in_check = board.is_in_check(board.side_to_move());

        // Generate and order moves
        let mut moves = mem::take(&mut self.move_lists[ply]);
        moves.clear();
        movegen::legal(board, &mut moves);

        if moves.is_empty() {
            self.move_lists[ply] = moves;
            return if in_check { mated_in(ply as u32) } else { 0 };
        }

        let side = board.side_to_move();
        let occupied = board.occupied();
        let pieces = board.pieces();
        self.move_orderer
            .order_moves_with_see(&mut moves, tt_move, ply, side, occupied, pieces);

        let mut best_score = NEG_MATE_SCORE;
        let mut best_move: Option<Move> = None;
        let mut node_type = NodeType::UpperBound;

        for mv in moves.iter() {
            if board.make_move(mv).is_err() {
                continue;
            }

            let gives_check = board.is_in_check(board.side_to_move());
            let extension: u8 = if gives_check && ply < MAX_PLY - 10 { 1 } else { 0 };

            let score =
                -self.alpha_beta_with_memory(board, depth - 1 + extension, ply + 1, -beta, -alpha);

            let _ = board.unmake_move(mv);

            if score > best_score {
                best_score = score;
                best_move = Some(*mv);

                if ply < PV_COLLECTION_LIMIT {
                    self.pv_table[ply][0] = *mv;
                    let child_len = self.pv_length[ply + 1].min(MAX_PV_LENGTH - ply - 2);
                    for i in 0..child_len {
                        self.pv_table[ply][i + 1] = self.pv_table[ply + 1][i];
                    }
                    self.pv_length[ply] = child_len + 1;
                }
            }

            if score >= beta {
                if mv.capture.is_none() && mv.promotion.is_none() {
                    self.move_orderer.store_killer(*mv, ply);
                    self.move_orderer.update_history(*mv, depth as usize);
                }

                node_type = NodeType::LowerBound;
                break;
            }

            if score > alpha {
                alpha = score;
                node_type = NodeType::Exact;
            }
        }

        // Store in TT
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

        self.move_lists[ply] = moves;
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

        if !in_check {
            let stand_pat = self.evaluator.evaluate(board);

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

        let mut moves = Vec::with_capacity(if in_check { AVG_LEGAL_MOVES } else { 16 });

        if in_check {
            movegen::legal(board, &mut moves);

            if moves.is_empty() {
                return mated_in(ply as u32);
            }
        } else {
            movegen::captures(board, &mut moves);
        }

        let side = board.side_to_move();
        let occupied = board.occupied();
        let pieces = board.pieces();
        self.move_orderer
            .order_moves_with_see(&mut moves, None, ply, side, occupied, pieces);

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

impl<E: Evaluator + Send> Searcher for MtdfSearcher<E> {
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
        self.move_orderer.clear_repetitions();
        self.tt.new_search();

        let start_time = self.start_time.unwrap();
        let max_depth = limits.depth.unwrap_or(MAX_PLY as u8).min(MAX_PLY as u8);

        let mut best_move: Option<Move> = None;
        let mut best_score: Score = NEG_MATE_SCORE;
        let mut pv = Vec::with_capacity(max_depth as usize);

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

        // First guess for MTD(f) - use 0 or previous best score
        let mut first_guess: Score = 0;

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

            // MTD(f) search
            let score = self.mtdf(board, first_guess, depth);

            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            first_guess = score;
            best_score = score;

            let pv_len = self.pv_length[0];
            if pv_len > 0 {
                best_move = Some(self.pv_table[0][0]);
                pv.clear();
                for i in 0..pv_len {
                    pv.push(self.pv_table[0][i]);
                }
            }

            self.info.score = score;
            self.info.time_elapsed = start_time.elapsed();
            mem::swap(&mut self.info.pv, &mut pv);
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
        self.tt.clear();
        self.move_orderer.clear();
    }

    fn resize_tt(&mut self, size_mb: usize) {
        self.tt.resize(size_mb);
    }

    fn hashfull(&self) -> u16 {
        self.tt.hashfull()
    }

    fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    fn algorithm_name(&self) -> &'static str {
        "MTD(f)"
    }
}

impl<E: Evaluator + Default> Default for MtdfSearcher<E> {
    fn default() -> Self {
        Self::new(E::default(), 16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::SimpleEvaluator;

    #[test]
    fn test_mtdf_search_basic() {
        let evaluator = SimpleEvaluator::new();
        let mut searcher = MtdfSearcher::new(evaluator, 1);

        let mut board = Board::starting_position().unwrap();

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_mtdf_mate_in_one() {
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = MtdfSearcher::new(evaluator, 1);

        let limits = SearchLimits::depth(3);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        let best = result.best_move.unwrap();
        assert_eq!(best.to.to_string(), "a8");
    }
}
