use crate::eval::Evaluator;
use crate::search::{SearchInfo, SearchLimits, SearchResult};
use aether_core::{MATE_SCORE, Move, NEG_MATE_SCORE, Score, mated_in};
use board::BoardOps;
use movegen::{Generator, MoveGen};
use std::time::{Duration, Instant};

const MAX_PLY: usize = 128;
const NODE_CHECK_INTERVAL: u64 = 4096;

pub struct AlphaBetaSearcher<E: Evaluator> {
    evaluator: E,
    generator: Generator,
    info: SearchInfo,
    stop_flag: bool,
    pv_table: Vec<Vec<Move>>,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
}

impl<E: Evaluator> AlphaBetaSearcher<E> {
    /// Creates a new AlphaBetaSearcher with the given evaluator.
    pub fn new(evaluator: E) -> Self {
        Self {
            evaluator,
            generator: Generator,
            info: SearchInfo::new(),
            stop_flag: false,
            pv_table: vec![Vec::new(); MAX_PLY],
            start_time: None,
            time_limit: None,
        }
    }

    fn iterative_deepening<T: BoardOps>(
        &mut self,
        board: &mut T,
        limits: &SearchLimits,
    ) -> SearchResult {
        self.stop_flag = false;
        self.info = SearchInfo::new();

        let mut best_move = None;
        let mut best_score = NEG_MATE_SCORE;

        let max_depth = limits.depth.unwrap_or(MAX_PLY as u8);

        for depth in 1..=max_depth {
            if self.should_stop(limits) {
                break;
            }

            for pv in &mut self.pv_table {
                pv.clear();
            }

            let score =
                self.alpha_beta(board, depth, 0, NEG_MATE_SCORE, MATE_SCORE, &mut Vec::new());

            if self.should_stop(limits) {
                break;
            }

            best_score = score;
            if !self.pv_table[0].is_empty() {
                best_move = Some(self.pv_table[0][0]);
            }

            // self.send_uci_info(depth, score, &self.pv_table[0]);
        }

        SearchResult {
            best_move,
            score: best_score,
            pv: self.pv_table[0].clone(),
            info: self.info.clone(),
        }
    }

    fn alpha_beta<T: BoardOps>(
        &mut self,
        board: &mut T,
        depth: u8,
        ply: usize,
        alpha: Score,
        beta: Score,
        pv: &mut Vec<Move>,
    ) -> Score {
        self.info.nodes += 1;
        if self.info.nodes % NODE_CHECK_INTERVAL == 0 {
            if self.should_stop_internal() {
                self.stop_flag = true;
                return 0;
            }
        }

        if depth == 0 {
            return self.evaluator.evaluate(board);
        }

        if ply >= MAX_PLY {
            return self.evaluator.evaluate(board);
        }

        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        if moves.is_empty() {
            return self.detect_checkmate_or_stalemate(board, ply);
        }

        let (best_score, best_move, local_pv) =
            self.search_moves_at_node(board, moves, depth, ply, alpha, beta);

        best_score
    }

    fn search_moves_at_node<T: BoardOps>(
        &mut self,
        board: &mut T,
        moves: Vec<Move>,
        depth: u8,
        ply: usize,
        mut alpha: Score,
        beta: Score,
    ) -> (Score, Option<Move>, Vec<Move>) {
        let mut best_score = NEG_MATE_SCORE;
        let mut best_move: Option<Move> = None;
        let mut local_pv = Vec::new();

        for mv in moves {
            board.make_move(&mv).unwrap();

            let mut child_pv = Vec::new();
            let score = -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha, &mut child_pv);

            board.unmake_move(&mv).unwrap();

            if score > best_score {
                best_score = score;
                best_move = Some(mv);

                local_pv.clear();
                local_pv.push(mv);
                local_pv.extend_from_slice(&child_pv);
            }

            if score > alpha {
                alpha = score;
            }

            if alpha >= beta {
                break; // Beta cutoff
            }
        }

        (best_score, best_move, local_pv)
    }

    fn should_stop(&self, limits: &SearchLimits) -> bool {
        if self.stop_flag {
            return true;
        }

        if let (Some(max_time), Some(start)) = (limits.time, self.start_time) {
            if start.elapsed() >= max_time {
                return true;
            }
        }

        false
    }

    fn should_stop_internal(&self) -> bool {
        if self.stop_flag {
            return true;
        }

        if self.info.nodes % NODE_CHECK_INTERVAL == 0 {
            if let (Some(max_time), Some(start)) = (self.time_limit, self.start_time) {
                if start.elapsed() >= max_time {
                    return true;
                }
            }
        }

        false
    }

    fn detect_checkmate_or_stalemate<T: BoardOps>(&self, board: &mut T, ply: usize) -> Score {
        if board.is_in_check(board.side_to_move()) {
            mated_in(ply as u32)
        } else {
            0 // Stalemate
        }
    }
}
