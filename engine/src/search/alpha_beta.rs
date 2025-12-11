use crate::eval::Evaluator;
use crate::search::move_ordering::MoveOrderer;
use crate::search::{
    NodeType, SearchInfo, SearchLimits, SearchResult, TTEntry, TranspositionTable,
};
use aether_core::{MATE_SCORE, Move, NEG_MATE_SCORE, PAWN_VALUE, QUEEN_VALUE, Score, mated_in};
use board::{BoardOps, BoardQuery};
use movegen::{Generator, MoveGen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const MAX_PLY: usize = 128;
const NODE_CHECK_INTERVAL: u64 = 4096;

pub struct AlphaBetaSearcher<E: Evaluator> {
    evaluator: E,
    generator: Generator,
    tt: TranspositionTable,
    move_orderer: MoveOrderer,
    info: SearchInfo,
    // pv_table: Vec<Vec<Move>>,
    stop_flag: Arc<AtomicBool>,
    start_time: Option<Instant>,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
}

impl<E: Evaluator> AlphaBetaSearcher<E> {
    /// Creates a new AlphaBetaSearcher with the given evaluator.
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
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    pub fn clear_tt(&mut self) {
        self.tt.clear();
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
        self.stop_flag.store(false, Ordering::SeqCst);
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

        // Iterative deepening
        for depth in 1..=max_depth {
            if let Some(limit) = self.soft_limit {
                if start_time.elapsed() >= limit {
                    break;
                }
            }

            if self.stop_flag.load(Ordering::SeqCst) {
                break;
            }

            self.info.depth = depth;
            let mut current_pv = Vec::new();

            let score = self.alpha_beta(
                board,
                depth,
                0,
                NEG_MATE_SCORE,
                MATE_SCORE,
                &mut current_pv,
                true,
            );

            if self.stop_flag.load(Ordering::SeqCst) {
                break;
            }

            best_score = score;
            if !current_pv.is_empty() {
                best_move = Some(current_pv[0]);
                pv = current_pv;
            }

            self.info.score = score;
            self.info.time_elapsed = start_time.elapsed();
            self.info.pv = pv.clone();
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
        self.info.nodes += 1;

        if self.info.nodes % NODE_CHECK_INTERVAL == 0 {
            if self.should_stop() {
                self.stop_flag.store(true, Ordering::SeqCst);
                return 0;
            }
        }

        if self.stop_flag.load(Ordering::SeqCst) {
            return 0;
        }

        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        let zobrist_key = board.zobrist_hash_raw();
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = self.tt.probe(zobrist_key) {
            tt_move = entry.best_move;

            if entry.depth >= depth && !is_pv_node {
                let tt_score = TTEntry::score_from_tt(entry.score, ply);

                match entry.node_type {
                    NodeType::Exact => return tt_score,
                    NodeType::LowerBound => {
                        if tt_score >= beta {
                            return beta;
                        }
                    }
                    NodeType::UpperBound => {
                        if tt_score <= alpha {
                            return alpha;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiescence(board, ply, alpha, beta);
        }

        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        if moves.is_empty() {
            return if board.is_in_check(board.side_to_move()) {
                return mated_in(ply as u32);
            } else {
                0 // Stalemate
            };
        }

        self.move_orderer.order_moves_with_tt(&mut moves, tt_move);

        let mut best_score = NEG_MATE_SCORE;
        let mut best_move: Option<Move> = None;
        let mut local_pv: Vec<Move> = Vec::new();
        let mut node_type = NodeType::UpperBound;

        for mv in moves {
            board.make_move(&mv).ok();

            let mut child_pv: Vec<Move> = Vec::new();
            let score = -self.alpha_beta(
                board,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                &mut child_pv,
                is_pv_node,
            );

            board.unmake_move(&mv).ok();

            if score > best_score {
                best_score = score;
                best_move = Some(mv);

                local_pv.clear();
                local_pv.push(mv);
                local_pv.extend_from_slice(&child_pv);
            }

            if score >= beta {
                node_type = NodeType::LowerBound;

                let tt_score = TTEntry::score_to_tt(beta, ply);
                let entry = TTEntry::new(
                    zobrist_key,
                    best_move,
                    tt_score,
                    depth,
                    node_type,
                    self.tt.generation(),
                );
                self.tt.store(entry);

                return beta;
            }

            if score > alpha {
                alpha = score;
                node_type = NodeType::Exact;
            }
        }

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

        *pv = local_pv;
        best_score
    }

    /// Quiescence search - search captures until a quiet position is reached
    fn quiescence<T: BoardOps + BoardQuery>(
        &mut self,
        board: &mut T,
        ply: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.info.nodes += 1;

        if self.info.nodes % NODE_CHECK_INTERVAL == 0 {
            if self.should_stop() {
                self.stop_flag.store(true, Ordering::SeqCst);
                return 0;
            }
        }

        if self.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        let stand_pat = self.evaluator.evaluate(board);

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Delta pruning: if even the best possible capture (queen)
        // can't improve alpha, skip search
        if stand_pat + QUEEN_VALUE + 200 < alpha {
            return alpha;
        }

        let mut moves = Vec::new();
        self.generator.captures(board, &mut moves);

        self.move_orderer.order_moves(&mut moves);

        for mv in moves {
            // SSE pruning: skip captures that are unlikely to be good
            // (Simple version: skip if captured piece is significantly less valuable)
            if let Some(captured) = mv.capture {
                if mv.piece.value() > captured.value() + PAWN_VALUE {
                    continue;
                }
            }

            board.make_move(&mv).ok();
            let score = -self.quiescence(board, ply + 1, -beta, -alpha);
            board.unmake_move(&mv).ok();

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

    fn should_stop(&self) -> bool {
        if self.stop_flag.load(Ordering::SeqCst) {
            return true;
        }

        if let Some(start) = self.start_time {
            if let Some(limit) = self.hard_limit {
                if start.elapsed() >= limit {
                    return true;
                }
            }
        }

        false
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
}
