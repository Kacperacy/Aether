use super::node::MctsNode;
use crate::eval::Evaluator;
use crate::search::searcher::{SearchCallback, Searcher};
use crate::search::{SearchInfo, SearchLimits, SearchResult};
use aether_core::{NEG_MATE_SCORE, Score};
use board::Board;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Exploration constant for UCB1 (typically sqrt(2) ≈ 1.414)
const EXPLORATION_CONSTANT: f64 = 1.414;

/// Nodes to search before checking time
const NODES_PER_TIME_CHECK: u64 = 1024;

/// Time buffer to stop early and account for overhead (in milliseconds)
const TIME_BUFFER_MS: u64 = 15;

/// Monte Carlo Tree Search implementation
pub struct MctsSearcher<E: Evaluator> {
    evaluator: E,
    info: SearchInfo,
    stop_flag: Arc<AtomicBool>,
    start_time: Option<Instant>,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
    nodes_limit: Option<u64>,
}

impl<E: Evaluator> MctsSearcher<E> {
    pub fn new(evaluator: E) -> Self {
        Self {
            evaluator,
            info: SearchInfo::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            start_time: None,
            soft_limit: None,
            hard_limit: None,
            nodes_limit: None,
        }
    }

    /// Run one iteration of MCTS using recursive selection
    fn run_iteration(&mut self, node: &mut MctsNode, board: &mut Board) -> f64 {
        self.info.nodes += 1;

        // Check time at every node and signal stop
        if self.should_stop() {
            self.stop_flag.store(true, Ordering::Release);
            return 0.0;
        }

        // Progressive widening: limit children based on visits
        let is_root = node.mv.is_none();
        let max_children = if is_root {
            35 // Most positions have ~35 legal moves
        } else {
            15 + (node.visits / 50) as usize // Start with more, grow faster
        };
        let should_expand = !node.untried_moves.is_empty() && node.children.len() < max_children;

        // If node has untried moves and we haven't reached max children, expand
        if should_expand {
            // Select move to expand based on static evaluation (best move first)
            let (move_idx, _prior) = self.select_best_untried_move_with_prior(node, board);
            let mv = node.untried_moves[move_idx];
            let _ = board.make_move(&mv);

            let mut child_moves = Vec::new();
            movegen::legal(board, &mut child_moves);

            let _ = node.expand(move_idx, child_moves);
            let child_idx = node.children.len() - 1;

            // Evaluate the new position
            let value = self.evaluate_position(board);

            let _ = board.unmake_move(&mv);

            // Update the new child
            node.children[child_idx].backpropagate(value);

            // Return negated value for parent
            let parent_value = -value;
            node.backpropagate(parent_value);
            return parent_value;
        }

        // If no children (terminal node), evaluate
        if node.children.is_empty() {
            let value = self.evaluate_position(board);
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

    /// Evaluate a position using the evaluator
    /// Returns value from the perspective of the side to move
    fn evaluate_position(&self, board: &Board) -> f64 {
        let mut moves = Vec::new();
        movegen::legal(board, &mut moves);

        if moves.is_empty() {
            if board.is_in_check(board.side_to_move()) {
                return -1.0; // Checkmate - losing for side to move
            } else {
                return 0.0; // Stalemate
            }
        }

        if board.is_fifty_move_draw()
            || board.is_threefold_repetition()
            || board.is_insufficient_material()
        {
            return 0.0;
        }

        // Evaluator already returns score from side-to-move perspective
        let eval = self.evaluator.evaluate(board);
        self.score_to_win_probability(eval)
    }

    #[inline]
    fn score_to_win_probability(&self, score: Score) -> f64 {
        let win_prob = 1.0 / (1.0 + 10_f64.powf(-score as f64 / 400.0));
        2.0 * win_prob - 1.0
    }

    #[inline]
    fn value_to_score(&self, value: f64) -> Score {
        let win_prob = (value + 1.0) / 2.0;
        let win_prob = win_prob.clamp(0.001, 0.999);
        let score = -400.0 * ((1.0 / win_prob) - 1.0).log10();
        score as Score
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

    /// Select the best untried move based on static evaluation
    /// Returns (move_index, prior_probability)
    fn select_best_untried_move_with_prior(&self, node: &MctsNode, board: &mut Board) -> (usize, f64) {
        if node.untried_moves.len() == 1 {
            return (0, 1.0);
        }

        // Score all moves
        let scores: Vec<i32> = node
            .untried_moves
            .iter()
            .map(|mv| self.move_ordering_score(mv, board))
            .collect();

        // Find best move
        let (best_idx, best_score) = scores
            .iter()
            .enumerate()
            .max_by_key(|(_, s)| **s)
            .map(|(i, &s)| (i, s))
            .unwrap();

        // Convert score to prior using softmax-like transformation
        // Higher scores = higher prior
        let min_score = *scores.iter().min().unwrap();
        let adjusted_best = (best_score - min_score) as f64;
        let sum_exp: f64 = scores
            .iter()
            .map(|&s| ((s - min_score) as f64 / 100.0).exp())
            .sum();
        let prior = (adjusted_best / 100.0).exp() / sum_exp;

        (best_idx, prior.max(0.01)) // Minimum prior of 1%
    }

    /// Score a move for ordering purposes (higher = expand first)
    #[inline]
    fn move_ordering_score(&self, mv: &aether_core::Move, board: &Board) -> i32 {
        let mut score = 0i32;

        // Captures are very good - MVV-LVA style
        if let Some(captured) = mv.capture {
            let victim_value = aether_core::PIECE_VALUES[captured as usize];
            let attacker_value = aether_core::PIECE_VALUES[mv.piece as usize];
            score += 10000 + victim_value - attacker_value / 100;
        }

        // Promotions are very good
        if mv.promotion.is_some() {
            score += 9000;
        }

        let to_file = mv.to.file() as i32;
        let to_rank = mv.to.rank() as i32;
        let from_rank = mv.from.rank() as i32;

        // Central squares are very good for knights and bishops
        // Center distance from e4/d4/e5/d5 cluster
        let center_file_dist = (to_file - 3).abs().min((to_file - 4).abs());
        let center_rank_dist = (to_rank - 3).abs().min((to_rank - 4).abs());
        let center_dist = center_file_dist + center_rank_dist;

        match mv.piece {
            aether_core::Piece::Knight => {
                // Knights: strongly prefer center, HEAVILY penalize rim
                score += (6 - center_dist) * 30;
                // Rim squares are terrible for knights - big penalty
                if to_file == 0 || to_file == 7 || to_rank == 0 || to_rank == 7 {
                    score -= 200; // Much bigger penalty
                }
            }
            aether_core::Piece::Bishop => {
                // Bishops: prefer central diagonals
                score += (5 - center_dist) * 20;
                // Rim is also bad for bishops
                if to_file == 0 || to_file == 7 {
                    score -= 50;
                }
            }
            aether_core::Piece::Pawn => {
                // Central pawns (d, e files) are much more valuable
                let is_center_file = to_file == 3 || to_file == 4;
                if is_center_file {
                    // Double pawn push to center is excellent
                    let is_double_push = mv.flags.is_double_pawn_push;
                    if is_double_push {
                        score += 80; // d5/e5 type moves
                    } else {
                        score += 50; // d6/e6 type moves
                    }
                }
                // Flank pawn moves in opening are usually bad
                if to_file == 0 || to_file == 7 {
                    score -= 50;
                }
                if to_file == 1 || to_file == 6 {
                    score -= 25;
                }
            }
            _ => {}
        }

        // Developing moves from back rank (knights and bishops)
        let is_back_rank = if board.side_to_move() == aether_core::Color::White {
            from_rank <= 1
        } else {
            from_rank >= 6
        };
        if is_back_rank {
            match mv.piece {
                aether_core::Piece::Knight | aether_core::Piece::Bishop => {
                    score += 80; // Strong bonus for development
                }
                _ => {}
            }
        }

        // Castling is good
        if mv.flags.is_castle {
            score += 70;
        }

        score
    }
}

impl<E: Evaluator + Send> Searcher for MctsSearcher<E> {
    fn search(
        &mut self,
        board: &mut Board,
        limits: &SearchLimits,
        on_info: SearchCallback<'_>,
    ) -> SearchResult {
        self.stop_flag.store(false, Ordering::Release);
        self.info = SearchInfo::new();
        self.start_time = Some(Instant::now());
        // Subtract time buffer to stop early and account for overhead
        self.soft_limit = limits.time.map(|t| {
            t.saturating_sub(Duration::from_millis(TIME_BUFFER_MS))
        });
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
            let only_move = legal_moves[0];
            board.make_move(&only_move).expect("legal move should not fail");
            let eval = self.evaluator.evaluate(board);
            board.unmake_move(&only_move).expect("unmake should not fail");

            self.info.depth = 1;
            self.info.time_elapsed = start_time.elapsed();
            self.info.calculate_nps();

            return SearchResult {
                best_move: Some(only_move),
                score: -eval,
                pv: vec![only_move],
                info: self.info.clone(),
            };
        }

        // Save first move as fallback before moving legal_moves
        let fallback_move = legal_moves[0];
        let mut root = MctsNode::root(legal_moves);

        let mut last_report_time = start_time;
        let report_interval = Duration::from_millis(1000);
        let mut iteration = 0u64;

        // Minimum iterations before checking time limits
        const MIN_ITERATIONS: u64 = 1;

        loop {
            // Check stop conditions
            if iteration >= MIN_ITERATIONS && self.should_stop() {
                break;
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
            // Fallback: if no iterations completed, return first untried move
            (Some(root.untried_moves[0]), 0)
        } else {
            // Ultimate fallback
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
        "MCTS"
    }
}

impl<E: Evaluator + Default> Default for MctsSearcher<E> {
    fn default() -> Self {
        Self::new(E::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::SimpleEvaluator;

    #[test]
    fn test_mcts_search_basic() {
        let evaluator = SimpleEvaluator::new();
        let mut searcher = MctsSearcher::new(evaluator);

        let mut board = Board::starting_position().unwrap();

        let limits = SearchLimits::nodes(1000);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        assert!(result.best_move.is_some());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_mcts_mate_in_one() {
        // Position: Ra8# is the only mate
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = MctsSearcher::new(evaluator);

        let limits = SearchLimits::nodes(50000);
        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        let best = result.best_move.unwrap();
        assert_eq!(best.to_string(), "a1a8", "MCTS should find Ra8#");
    }

    #[test]
    fn test_mcts_with_time_limit() {
        use std::time::Duration;

        let mut board = Board::starting_position().unwrap();
        let evaluator = SimpleEvaluator::new();
        let mut searcher = MctsSearcher::new(evaluator);

        // Very short time limit - 50ms
        let limits = SearchLimits {
            time: Some(Duration::from_millis(50)),
            hard_time: Some(Duration::from_millis(100)),
            ..Default::default()
        };

        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        // Should return a move within reasonable time
        assert!(result.best_move.is_some());
        assert!(result.info.nodes > 0);
    }

    #[test]
    fn test_mcts_iterations_in_400ms() {
        use std::time::Duration;

        let mut board = Board::starting_position().unwrap();
        let evaluator = SimpleEvaluator::new();
        let mut searcher = MctsSearcher::new(evaluator);

        // Simulate 10+0.1 time control - about 400ms per move
        let limits = SearchLimits {
            time: Some(Duration::from_millis(400)),
            hard_time: Some(Duration::from_millis(1000)),
            ..Default::default()
        };

        let result = searcher.search(&mut board, &limits, &mut |_, _, _| {});

        eprintln!("MCTS in 400ms:");
        eprintln!("  Nodes: {}", result.info.nodes);
        eprintln!("  Time: {:?}", result.info.time_elapsed);
        eprintln!("  Best move: {:?}", result.best_move);
        eprintln!("  Score: {}", result.score);

        assert!(result.best_move.is_some());
        assert!(result.info.nodes > 100, "Should do more than 100 iterations");
    }

    #[test]
    fn test_mcts_response_to_e4() {
        use std::time::Duration;

        // Position after 1.e4
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let mut board: Board = fen.parse().unwrap();

        let evaluator = SimpleEvaluator::new();
        let mut searcher = MctsSearcher::new(evaluator);

        // Debug: print move ordering scores
        let mut legal_moves = Vec::new();
        movegen::legal(&board, &mut legal_moves);
        let mut scored_moves: Vec<_> = legal_moves
            .iter()
            .map(|mv| (mv.to_string(), searcher.move_ordering_score(mv, &board)))
            .collect();
        scored_moves.sort_by_key(|(_, score)| -score);
        eprintln!("Move ordering scores (top 10):");
        for (mv, score) in scored_moves.iter().take(10) {
            eprintln!("  {}: {}", mv, score);
        }

        let limits = SearchLimits {
            time: Some(Duration::from_millis(400)),
            hard_time: Some(Duration::from_millis(1000)),
            ..Default::default()
        };

        // Capture root node info for debugging
        let mut last_info = SearchInfo::new();
        let result = searcher.search(&mut board, &limits, &mut |info, _, _| {
            last_info = info.clone();
        });

        eprintln!("MCTS response to 1.e4:");
        eprintln!("  Best move: {}", result.best_move.unwrap());
        eprintln!("  Score: {}", result.score);
        eprintln!("  Nodes: {}", result.info.nodes);
        eprintln!("  Depth (log2 visits): {}", last_info.depth);

        // Black should play something reasonable (e5, c5, e6, d5, etc.)
        let mv = result.best_move.unwrap().to_string();
        let reasonable_responses = ["e7e5", "c7c5", "e7e6", "d7d5", "d7d6", "g8f6", "b8c6"];
        assert!(
            reasonable_responses.contains(&mv.as_str()),
            "MCTS played {} which is not a common response to 1.e4",
            mv
        );
    }
}
