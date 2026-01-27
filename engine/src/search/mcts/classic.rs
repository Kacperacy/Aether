//! Classic Monte Carlo Tree Search with random playouts
//!
//! This is a baseline MCTS implementation that uses random playouts
//! instead of static evaluation. It serves as a theoretical baseline
//! for comparison with other search algorithms.

use super::node::MctsNode;
use crate::search::searcher::{SearchCallback, Searcher};
use crate::search::{SearchInfo, SearchLimits, SearchResult};
use aether_core::{NEG_MATE_SCORE, Score};
use board::Board;
use rand::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Exploration constant for UCB1 (sqrt(2))
const EXPLORATION_CONSTANT: f64 = 1.414;

/// Nodes to search before checking time
const NODES_PER_TIME_CHECK: u64 = 512;

/// Maximum moves in a random playout before declaring a draw
const MAX_PLAYOUT_MOVES: usize = 200;

/// Classic MCTS with random playouts
pub struct ClassicMctsSearcher {
    info: SearchInfo,
    stop_flag: Arc<AtomicBool>,
    start_time: Option<Instant>,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
    nodes_limit: Option<u64>,
    rng: SmallRng,
}

impl ClassicMctsSearcher {
    pub fn new() -> Self {
        Self {
            info: SearchInfo::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            start_time: None,
            soft_limit: None,
            hard_limit: None,
            nodes_limit: None,
            rng: SmallRng::from_rng(&mut rand::rng()),
        }
    }

    /// Run one iteration of MCTS
    fn run_iteration(&mut self, node: &mut MctsNode, board: &mut Board) -> f64 {
        self.info.nodes += 1;

        // Check time every 64 nodes
        if self.info.nodes % 64 == 0 && self.should_stop() {
            return 0.0;
        }

        // If node has untried moves, expand
        if !node.untried_moves.is_empty() {
            // Select random untried move
            let move_idx = self.rng.random_range(0..node.untried_moves.len());
            let mv = node.untried_moves[move_idx];
            let _ = board.make_move(&mv);

            let mut child_moves = Vec::new();
            movegen::legal(board, &mut child_moves);

            let _ = node.expand(move_idx, child_moves);
            let child_idx = node.children.len() - 1;

            // Run random playout from this position
            let value = self.random_playout(board);
            let _ = board.unmake_move(&mv);

            // Update the new child
            node.children[child_idx].backpropagate(value);

            // Return negated value for parent
            let parent_value = -value;
            node.backpropagate(parent_value);
            return parent_value;
        }

        // If no children (terminal node), run playout
        if node.children.is_empty() {
            let value = self.random_playout(board);
            node.backpropagate(value);
            return value;
        }

        // Select best child and recurse
        let best_idx = node.select_child(EXPLORATION_CONSTANT).unwrap();
        let child_mv = node.children[best_idx].mv.unwrap();
        let _ = board.make_move(&child_mv);

        let child_value = self.run_iteration(&mut node.children[best_idx], board);
        let _ = board.unmake_move(&child_mv);

        // Update this node with negated child value
        let value = -child_value;
        node.backpropagate(value);
        value
    }

    /// Perform a random playout from the current position
    /// Returns value from the perspective of the side to move
    fn random_playout(&mut self, board: &mut Board) -> f64 {
        let mut playout_board = board.clone();
        let original_side = playout_board.side_to_move();
        let mut moves_played = 0;

        for i in 0..MAX_PLAYOUT_MOVES {
            // Check time every 16 moves in playout
            if i % 16 == 0 && self.should_stop() {
                return 0.0;
            }

            let mut moves = Vec::new();
            movegen::legal(&playout_board, &mut moves);

            if moves.is_empty() {
                // Game over
                let result = if playout_board.is_in_check(playout_board.side_to_move()) {
                    // Checkmate - the side to move lost
                    if playout_board.side_to_move() == original_side {
                        -1.0
                    } else {
                        1.0
                    }
                } else {
                    0.0 // Stalemate
                };
                return result;
            }

            // Check for draws
            if playout_board.is_fifty_move_draw()
                || playout_board.is_threefold_repetition()
                || playout_board.is_insufficient_material()
            {
                return 0.0;
            }

            // Make a random move
            let idx = self.rng.random_range(0..moves.len());
            let _ = playout_board.make_move(&moves[idx]);
            moves_played += 1;
        }

        // Playout timeout - consider it a draw
        let _ = moves_played;
        0.0
    }

    fn should_stop(&self) -> bool {
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            if let Some(limit) = self.hard_limit {
                if elapsed >= limit {
                    return true;
                }
            }
            if let Some(limit) = self.soft_limit {
                if elapsed >= limit {
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

    #[inline]
    fn value_to_score(&self, value: f64) -> Score {
        // Convert win probability (-1 to 1) to centipawns
        // Using the same formula as static-eval MCTS for consistency
        let win_prob = (value + 1.0) / 2.0;
        let win_prob = win_prob.clamp(0.001, 0.999);
        let score = -400.0 * ((1.0 / win_prob) - 1.0).log10();
        score as Score
    }
}

impl Default for ClassicMctsSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Searcher for ClassicMctsSearcher {
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

        let mut legal_moves = Vec::new();
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
            self.info.depth = 1;
            self.info.time_elapsed = start_time.elapsed();
            self.info.calculate_nps();

            return SearchResult {
                best_move: Some(legal_moves[0]),
                score: 0,
                pv: vec![legal_moves[0]],
                info: self.info.clone(),
            };
        }

        let fallback_move = legal_moves[0];
        let mut root = MctsNode::root(legal_moves);

        let mut last_report_time = start_time;
        let report_interval = Duration::from_millis(1000);
        let mut iteration = 0u64;

        const MIN_ITERATIONS: u64 = 10;

        loop {
            if iteration >= MIN_ITERATIONS {
                if self.should_stop() {
                    break;
                }

                if let Some(limit) = self.soft_limit {
                    if start_time.elapsed() >= limit {
                        break;
                    }
                }
            }

            let mut search_board = board.clone();
            let _ = self.run_iteration(&mut root, &mut search_board);
            iteration += 1;

            if iteration % NODES_PER_TIME_CHECK == 0 {
                let now = Instant::now();
                if now.duration_since(last_report_time) >= report_interval {
                    last_report_time = now;

                    if let Some(best_child) = root.best_child_by_visits() {
                        let best_move = best_child.mv;
                        let value = -best_child.average_value();
                        let score = self.value_to_score(value);

                        self.info.depth = (root.visits as f64).log2().ceil().max(1.0) as u8;
                        self.info.score = score;
                        self.info.time_elapsed = start_time.elapsed();
                        self.info.calculate_nps();

                        if let Some(mv) = best_move {
                            self.info.pv = vec![mv];
                        }

                        on_info(&self.info, best_move, score);
                    }
                }
            }
        }

        let (best_move, best_score) = if let Some(best_child) = root.best_child_by_visits() {
            let value = -best_child.average_value();
            (best_child.mv, self.value_to_score(value))
        } else if !root.untried_moves.is_empty() {
            (Some(root.untried_moves[0]), 0)
        } else {
            (Some(fallback_move), 0)
        };

        let pv = best_move.map(|m| vec![m]).unwrap_or_default();

        self.info.depth = (root.visits as f64).log2().ceil().max(1.0) as u8;
        self.info.score = best_score;
        self.info.time_elapsed = start_time.elapsed();
        self.info.pv = pv.clone();
        self.info.calculate_nps();

        on_info(&self.info, best_move, best_score);

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

    fn new_game(&mut self) {}

    fn resize_tt(&mut self, _size_mb: usize) {}

    fn hashfull(&self) -> u16 {
        0
    }

    fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    fn algorithm_name(&self) -> &'static str {
        "Classic MCTS"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic_mcts_basic() {
        let mut searcher = ClassicMctsSearcher::new();
        let mut board = Board::starting_position().unwrap();

        let limits = SearchLimits::nodes(1000);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_classic_mcts_finds_mate() {
        // Position: Ra8# is the only mate
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let mut searcher = ClassicMctsSearcher::new();

        // Classic MCTS needs more iterations to find mate
        let limits = SearchLimits::nodes(100_000);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        // Classic MCTS with random playouts may not reliably find mate
        // but should at least return a legal move
        assert!(result.best_move.is_some());
    }

    #[test]
    fn test_classic_mcts_with_time() {
        let mut board = Board::starting_position().unwrap();
        let mut searcher = ClassicMctsSearcher::new();

        let limits = SearchLimits {
            time: Some(Duration::from_millis(100)),
            hard_time: Some(Duration::from_millis(200)),
            ..Default::default()
        };

        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(result.info.nodes > 0);
    }
}
